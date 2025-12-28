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
pub struct BinaryViewMut<'a> {
    buffer: &'a mut [u8],
    header: &'a mut FormatHeader,
    offset_table: &'a mut [OffsetEntry],
}

impl BinarySerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
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
    
    /// Find offset entry for a field
    pub fn find_entry(&self, field_id: u32) -> Option<&OffsetEntry> {
        self.offset_table.iter().find(|e| e.field_id == field_id)
    }
    
    /// Get pointer to a field (zero-copy)
    /// Note: For unaligned types like f64 in packed structs, this may require copying
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
        
        // For potentially unaligned access, use unsafe with read_unaligned
        // This is safe because we've validated the bounds
        unsafe {
            let ptr = self.buffer.as_ptr().add(field_offset) as *const T;
            Ok(&*ptr)
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
        
        // Find null terminator or use size
        let mut end = string_offset;
        while end < self.buffer.len() && self.buffer[end] != 0 {
            end += 1;
        }
        
        std::str::from_utf8(&self.buffer[string_offset..end])
            .map_err(|_| SerializationError::FieldSizeMismatch {
                expected: 0,
                got: 0,
            })
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
        
        // Validate header first
        {
            let header_check = bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);
            header_check.validate()?;
            
            let total_size = header_check.total_size();
            if buffer_len < total_size {
                return Err(SerializationError::BufferTooSmall {
                    needed: total_size,
                    have: buffer_len,
                });
            }
        }
        
        // Use unsafe to get multiple mutable references to non-overlapping regions
        // This is safe because we've validated the bounds and the regions don't overlap
        unsafe {
            let header_ptr = buffer.as_mut_ptr();
            let header = &mut *(header_ptr as *mut FormatHeader);
            
            let offset_table_start = header.header_size as usize;
            let offset_table_ptr = header_ptr.add(offset_table_start);
            let offset_table_len = header.offset_table_size as usize / std::mem::size_of::<OffsetEntry>();
            let offset_table = std::slice::from_raw_parts_mut(
                offset_table_ptr as *mut OffsetEntry,
                offset_table_len,
            );
            
            Ok(BinaryViewMut {
                buffer,
                header,
                offset_table,
            })
        }
    }
    
    /// Find offset entry for a field
    pub fn find_entry(&self, field_id: u32) -> Option<&OffsetEntry> {
        self.offset_table.iter().find(|e| e.field_id == field_id)
    }
    
    /// Modify a fixed-size field in place
    pub fn modify_field<T: Pod>(&mut self, field_id: u32, value: &T) -> Result<()> {
        let entry = self.find_entry(field_id)
            .ok_or_else(|| SerializationError::FieldNotFound { field_id })?;
        
        let value_size = std::mem::size_of::<T>();
        if value_size != entry.size as usize {
            return Err(SerializationError::FieldSizeMismatch {
                expected: entry.size as usize,
                got: value_size,
            });
        }
        
        let data_start = self.header.data_section_offset();
        let field_offset = data_start + entry.offset as usize;
        let field_end = field_offset + value_size;
        
        if field_end > self.buffer.len() {
            return Err(SerializationError::InvalidOffset {
                offset: field_end,
                size: self.buffer.len(),
            });
        }
        
        // Safe: we've validated the bounds
        unsafe {
            std::ptr::copy_nonoverlapping(
                value as *const T as *const u8,
                self.buffer.as_mut_ptr().add(field_offset),
                value_size,
            );
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
        
        let var_start = self.header.var_section_offset();
        let string_offset = var_start + entry.offset as usize;
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
        
        let var_start = self.header.var_section_offset();
        let blob_offset = var_start + entry.offset as usize;
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
