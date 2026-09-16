use crate::error::{Result, SerializationError};
use crate::format::{FieldType, FormatHeader, OffsetEntry, HEADER_SIZE};
use bytemuck::Pod;

/// High-performance binary serializer with in-place modification support
pub struct BinarySerializer {
    buffer: Vec<u8>,
}

/// Zero-copy view into a serialized buffer
pub struct BinaryView<'a> {
    buffer: &'a [u8],
    header: &'a FormatHeader,
    offset_table: &'a [OffsetEntry],
}

/// Mutable view for in-place modification
///
/// Stores only `buffer`; the header and offset table are subregions of
/// it, so holding separate `&mut FormatHeader` / `&mut [OffsetEntry]`
/// fields alongside `buffer` would be overlapping mutable references —
/// undefined behavior, regardless of whether they're ever used together.
/// Header and entries are derived per call instead (both are `Copy`, so
/// this costs a cheap copy, not a re-parse of the whole buffer).
pub struct BinaryViewMut<'a> {
    buffer: &'a mut [u8],
    data_offset: usize,
    var_offset: usize,
}

impl BinarySerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Pre-allocate buffer capacity. Call with `header.total_size()`
    /// before `write_header` so the four `write_*` calls that follow
    /// fill one allocation instead of growing the buffer through
    /// reallocation.
    pub fn reserve(&mut self, capacity: usize) {
        self.buffer.reserve(capacity);
    }

    pub fn write_header(&mut self, header: FormatHeader) {
        let header_bytes = bytemuck::bytes_of(&header);
        self.buffer.extend_from_slice(header_bytes);
    }
    
    pub fn write_offset_table(&mut self, entries: &[OffsetEntry]) {
        let table_bytes = bytemuck::cast_slice(entries);
        self.buffer.extend_from_slice(table_bytes);
    }
    
    pub fn write_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }
    
    pub fn write_var_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }
    
    pub fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }
    
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

impl<'a> BinaryView<'a> {
    /// Create a view into an existing buffer (zero-copy)
    pub fn view(buffer: &'a [u8]) -> Result<Self> {
        if buffer.len() < HEADER_SIZE {
            return Err(SerializationError::BufferTooSmall {
                needed: HEADER_SIZE,
                have: buffer.len(),
            });
        }
        
        let header = bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);
        header.validate()?;
        
        let total_size = header.total_size();
        if buffer.len() < total_size {
            return Err(SerializationError::BufferTooSmall {
                needed: total_size,
                have: buffer.len(),
            });
        }
        
        let offset_table_start = header.header_size as usize;
        let offset_table_end = offset_table_start + header.offset_table_size as usize;
        let offset_table = bytemuck::cast_slice::<u8, OffsetEntry>(
            &buffer[offset_table_start..offset_table_end]
        );
        
        Ok(BinaryView {
            buffer,
            header,
            offset_table,
        })
    }
    
    /// Create a view while skipping magic/version/header_size validation
    /// and the full `total_size` bounds computation — for buffers this
    /// process just serialized (or otherwise trusts), where re-checking
    /// identity fields that were already correct by construction is pure
    /// overhead.
    ///
    /// Still returns `Result` rather than panicking: a buffer too short
    /// to hold a header returns `BufferTooSmall`, and `offset_table_size`
    /// not a multiple of `size_of::<OffsetEntry>()` returns
    /// `InvalidOffsetTableSize` — that check in particular is kept
    /// because skipping it is what let a malformed header reach
    /// `bytemuck::cast_slice` and panic (the bug fixed in the soundness
    /// pass), so dropping it here would just reopen it. Every accessor
    /// called on the result still does its own bounds/alignment checks,
    /// so this only skips *redundant* validation, never a safety check —
    /// it's not a validation bypass for untrusted input, and a buffer
    /// with a wrong magic/version will simply be misread, not cause UB.
    pub fn view_unchecked(buffer: &'a [u8]) -> Result<Self> {
        if buffer.len() < HEADER_SIZE {
            return Err(SerializationError::BufferTooSmall {
                needed: HEADER_SIZE,
                have: buffer.len(),
            });
        }

        let header = bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);

        let entry_size = std::mem::size_of::<OffsetEntry>();
        if header.offset_table_size as usize % entry_size != 0 {
            return Err(SerializationError::InvalidOffsetTableSize {
                size: header.offset_table_size as usize,
                entry_size,
            });
        }

        let offset_table_start = header.header_size as usize;
        let offset_table_end = offset_table_start.saturating_add(header.offset_table_size as usize);
        if offset_table_start > offset_table_end || offset_table_end > buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: offset_table_end,
                size: buffer.len(),
            });
        }

        let offset_table = bytemuck::cast_slice::<u8, OffsetEntry>(
            &buffer[offset_table_start..offset_table_end]
        );

        Ok(BinaryView {
            buffer,
            header,
            offset_table,
        })
    }

    /// Find offset entry for a field.
    ///
    /// Fast path: for the dense, 1-based field IDs the format's own
    /// design assumes (`field_id` N is stored at index N-1), this is a
    /// direct index instead of a scan. Falls back to a linear scan for
    /// sparse or non-sequential IDs, so this is always correct — the
    /// fast path is purely an optimization, never a behavior change.
    pub fn find_entry(&self, field_id: u32) -> Option<&OffsetEntry> {
        if field_id >= 1 {
            if let Some(entry) = self.offset_table.get((field_id - 1) as usize) {
                if entry.field_id == field_id {
                    return Some(entry);
                }
            }
        }
        self.offset_table.iter().find(|e| e.field_id == field_id)
    }

    /// Get a reference to a field (zero-copy).
    ///
    /// Returns `UnalignedField` if the field's address does not satisfy
    /// `T`'s alignment requirement — creating a reference to a misaligned
    /// value is undefined behavior, so this is checked rather than risked.
    /// Use [`get_field_unaligned`](Self::get_field_unaligned) for fields
    /// that are not guaranteed to be aligned (e.g. an `f64` following
    /// odd-sized fields in a packed layout).
    pub fn get_field<T: Pod>(&self, field_id: u32) -> Result<&T> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        let data_start = self.header.data_section_offset();
        let field_offset = data_start + entry.offset as usize;
        let field_end = field_offset + std::mem::size_of::<T>();

        if field_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: field_end,
                size: self.buffer.len(),
            });
        }

        let align = std::mem::align_of::<T>();
        let field_addr = self.buffer.as_ptr() as usize + field_offset;
        if field_addr % align != 0 {
            return Err(SerializationError::UnalignedField {
                field_id,
                offset: field_offset,
                align,
            });
        }

        // Safety: bounds were checked above, and we just verified the
        // address is properly aligned for `T`, so this reference is valid.
        unsafe {
            let ptr = self.buffer.as_ptr().add(field_offset) as *const T;
            Ok(&*ptr)
        }
    }

    /// Get a copy of a field's value without requiring alignment.
    ///
    /// Use this instead of [`get_field`](Self::get_field) when the field
    /// may not be aligned for `T` (`get_field` will return
    /// `UnalignedField` in that case). Reads via `ptr::read_unaligned`,
    /// so it never constructs an invalid reference.
    pub fn get_field_unaligned<T: Pod>(&self, field_id: u32) -> Result<T> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        let data_start = self.header.data_section_offset();
        let field_offset = data_start + entry.offset as usize;
        let field_end = field_offset + std::mem::size_of::<T>();

        if field_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: field_end,
                size: self.buffer.len(),
            });
        }

        // Safety: bounds were checked above; read_unaligned does not
        // require the source address to be aligned for `T`.
        unsafe {
            let ptr = self.buffer.as_ptr().add(field_offset) as *const T;
            Ok(std::ptr::read_unaligned(ptr))
        }
    }
    
    /// Get string field (zero-copy)
    pub fn get_string(&self, field_id: u32) -> Result<&str> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        if entry.field_type != FieldType::String as u16 {
            return Err(SerializationError::FieldSizeMismatch {
                expected: FieldType::String as usize,
                got: entry.field_type as usize,
            });
        }

        let var_start = self.header.var_section_offset();
        let string_offset = var_start + entry.offset as usize;

        if string_offset > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: string_offset,
                size: self.buffer.len(),
            });
        }

        // Find a null terminator, but never past this field's declared
        // size — otherwise an unterminated string would read into
        // whatever comes after it in the var section.
        let declared_end = string_offset.saturating_add(entry.size as usize);
        let scan_end = declared_end.min(self.buffer.len());

        let mut end = string_offset;
        while end < scan_end && self.buffer[end] != 0 {
            end += 1;
        }

        std::str::from_utf8(&self.buffer[string_offset..end])
            .map_err(|_| SerializationError::InvalidUtf8 { field_id })
    }
    
    /// Get blob field (zero-copy)
    pub fn get_blob(&self, field_id: u32) -> Result<&[u8]> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;
        
        if entry.field_type != FieldType::Blob as u16 {
            return Err(SerializationError::FieldSizeMismatch {
                expected: FieldType::Blob as usize,
                got: entry.field_type as usize,
            });
        }
        
        let var_start = self.header.var_section_offset();
        let blob_offset = var_start + entry.offset as usize;
        let blob_end = blob_offset + entry.size as usize;
        
        if blob_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: blob_end,
                size: self.buffer.len(),
            });
        }
        
        Ok(&self.buffer[blob_offset..blob_end])
    }
}

impl<'a> BinaryViewMut<'a> {
    /// Get mutable view for in-place modification
    pub fn view_mut(buffer: &'a mut [u8]) -> Result<Self> {
        let buffer_len = buffer.len();
        if buffer_len < HEADER_SIZE {
            return Err(SerializationError::BufferTooSmall {
                needed: HEADER_SIZE,
                have: buffer_len,
            });
        }

        // FormatHeader is Copy, so this reads the header by value — no
        // reference into `buffer` survives the call, leaving buffer free
        // to be mutably reborrowed later without any aliasing.
        let header = *bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);
        header.validate()?;

        let total_size = header.total_size();
        if buffer_len < total_size {
            return Err(SerializationError::BufferTooSmall {
                needed: total_size,
                have: buffer_len,
            });
        }

        let data_offset = header.data_section_offset();
        let var_offset = header.var_section_offset();

        Ok(BinaryViewMut {
            buffer,
            data_offset,
            var_offset,
        })
    }

    /// Find offset entry for a field, by value.
    ///
    /// Returns a copy rather than a reference into the offset table:
    /// `BinaryViewMut` holds only `buffer` (see the struct doc comment),
    /// so an entry can't be borrowed from it without aliasing whatever
    /// `&mut` a subsequent `modify_*` call needs.
    ///
    /// Doesn't re-read the header from the buffer: `validate()` (run in
    /// `view_mut`) already guarantees `header.header_size == HEADER_SIZE`,
    /// so the offset table always starts at that constant, and the entry
    /// count is derived from the already-cached `data_offset`
    /// (`data_offset - HEADER_SIZE` is `offset_table_size` by
    /// definition) instead of re-parsing the header on every call.
    ///
    /// Also has the same dense-1-based-field-ID direct-index fast path as
    /// `BinaryView::find_entry` — see its doc comment.
    pub fn find_entry(&self, field_id: u32) -> Option<OffsetEntry> {
        let entry_size = std::mem::size_of::<OffsetEntry>();
        let count = (self.data_offset - HEADER_SIZE) / entry_size;
        let table = bytemuck::cast_slice::<u8, OffsetEntry>(
            &self.buffer[HEADER_SIZE..HEADER_SIZE + count * entry_size]
        );

        if field_id >= 1 {
            if let Some(entry) = table.get((field_id - 1) as usize) {
                if entry.field_id == field_id {
                    return Some(*entry);
                }
            }
        }
        table.iter().find(|e| e.field_id == field_id).copied()
    }

    /// Modify a fixed-size field in place
    pub fn modify_field<T: Pod>(&mut self, field_id: u32, value: &T) -> Result<()> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        // String/Blob fields live in the var section under different
        // rules (NUL-termination, in-place-clear before write); writing
        // raw POD bytes into one via this path would corrupt those
        // invariants, so only fixed-size field types are allowed here.
        if entry.field_type == FieldType::String as u16 || entry.field_type == FieldType::Blob as u16 {
            return Err(SerializationError::WrongFieldType {
                field_id,
                field_type: entry.field_type,
            });
        }

        let value_size = std::mem::size_of::<T>();
        if value_size != entry.size as usize {
            return Err(SerializationError::FieldSizeMismatch {
                expected: entry.size as usize,
                got: value_size,
            });
        }

        let field_offset = self.data_offset + entry.offset as usize;
        let field_end = field_offset + value_size;

        if field_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: field_end,
                size: self.buffer.len(),
            });
        }

        // Safety: bounds were checked above. write_unaligned (unlike
        // copy_nonoverlapping) does not require the destination to be
        // aligned for `T`, which packed layouts do not guarantee.
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(field_offset) as *mut T;
            std::ptr::write_unaligned(dst, *value);
        }

        Ok(())
    }

    /// Modify a string field in place (must fit in existing space)
    pub fn modify_string(&mut self, field_id: u32, value: &str) -> Result<()> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        if entry.field_type != FieldType::String as u16 {
            return Err(SerializationError::FieldSizeMismatch {
                expected: FieldType::String as usize,
                got: entry.field_type as usize,
            });
        }

        let value_bytes = value.as_bytes();
        if value_bytes.len() + 1 > entry.size as usize {
            return Err(SerializationError::FieldSizeMismatch {
                expected: entry.size as usize,
                got: value_bytes.len() + 1,
            });
        }

        let string_offset = self.var_offset + entry.offset as usize;
        let string_end = string_offset + entry.size as usize;

        if string_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: string_end,
                size: self.buffer.len(),
            });
        }

        // Clear existing string
        self.buffer[string_offset..string_end].fill(0);

        // Write new string
        self.buffer[string_offset..string_offset + value_bytes.len()]
            .copy_from_slice(value_bytes);

        Ok(())
    }

    /// Modify a blob field in place
    pub fn modify_blob(&mut self, field_id: u32, value: &[u8]) -> Result<()> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;

        if entry.field_type != FieldType::Blob as u16 {
            return Err(SerializationError::FieldSizeMismatch {
                expected: FieldType::Blob as usize,
                got: entry.field_type as usize,
            });
        }

        if value.len() > entry.size as usize {
            return Err(SerializationError::FieldSizeMismatch {
                expected: entry.size as usize,
                got: value.len(),
            });
        }

        let blob_offset = self.var_offset + entry.offset as usize;
        let blob_end = blob_offset + entry.size as usize;

        if blob_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: blob_end,
                size: self.buffer.len(),
            });
        }

        // Clear existing blob
        self.buffer[blob_offset..blob_end].fill(0);

        // Write new blob
        self.buffer[blob_offset..blob_offset + value.len()]
            .copy_from_slice(value);

        Ok(())
    }
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize header, offset table, fixed data, and var data into one
/// buffer in a single allocation.
///
/// Equivalent to constructing a [`BinarySerializer`] and calling
/// `write_header`, `write_offset_table`, `write_data`, and
/// `write_var_data` in order, but sizes the buffer up front
/// (`header.total_size()`) instead of growing it as each section is
/// appended.
pub fn serialize_to_buffer(
    header: &FormatHeader,
    entries: &[OffsetEntry],
    data: &[u8],
    var_data: &[u8],
) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(header.total_size());
    buffer.extend_from_slice(bytemuck::bytes_of(header));
    buffer.extend_from_slice(bytemuck::cast_slice(entries));
    buffer.extend_from_slice(data);
    buffer.extend_from_slice(var_data);
    buffer
}
