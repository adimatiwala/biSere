use bytemuck::{Pod, Zeroable};
use crate::error::{Result, SerializationError};

pub const MAGIC: u32 = 0x42495345; // "BISE" in ASCII
pub const VERSION: u32 = 1;
// FormatHeader size: 4 (magic) + 4 (version) + 4 (header_size) + 4 (offset_table_size) 
// + 4 (data_size) + 4 (var_size) + 8 (checksum) + 48 (reserved[6]) = 80 bytes
pub const HEADER_SIZE: usize = 80;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FormatHeader {
    pub magic: u32,              // Format identifier
    pub version: u32,             // Format version
    pub header_size: u32,        // Size of header
    pub offset_table_size: u32,  // Size of offset table in bytes
    pub data_size: u32,          // Size of fixed data section
    pub var_size: u32,           // Size of variable-length section
    pub checksum: u64,           // Optional integrity check
    pub reserved: [u64; 6],      // Reserved for future use
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OffsetEntry {
    pub field_id: u32,    // Unique field identifier
    pub offset: u32,      // Offset from start of data section
    pub field_type: u16,  // Field type
    pub size: u16,        // Field size (fixed) or max size (variable)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FieldType {
    Int8 = 1,
    Int16 = 2,
    Int32 = 3,
    Int64 = 4,
    Uint8 = 5,
    Uint16 = 6,
    Uint32 = 7,
    Uint64 = 8,
    Float32 = 9,
    Float64 = 10,
    Bool = 11,
    String = 12,    // Variable length
    Blob = 13,      // Variable length binary
}

impl FormatHeader {
    pub fn new(offset_table_size: u32, data_size: u32, var_size: u32) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            header_size: HEADER_SIZE as u32,
            offset_table_size,
            data_size,
            var_size,
            checksum: 0, // Can be computed later
            reserved: [0; 6],
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.magic != MAGIC {
            return Err(SerializationError::InvalidMagic {
                expected: MAGIC,
                found: self.magic,
            });
        }
        
        if self.version != VERSION {
            return Err(SerializationError::UnsupportedVersion {
                version: self.version,
            });
        }
        
        Ok(())
    }
    
    pub fn total_size(&self) -> usize {
        (self.header_size + self.offset_table_size + self.data_size + self.var_size) as usize
    }
    
    pub fn data_section_offset(&self) -> usize {
        (self.header_size + self.offset_table_size) as usize
    }
    
    pub fn var_section_offset(&self) -> usize {
        self.data_section_offset() + self.data_size as usize
    }
}
