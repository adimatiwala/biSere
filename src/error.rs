use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Invalid magic number: expected {expected:#x}, found {found:#x}")]
    InvalidMagic { expected: u32, found: u32 },
    
    #[error("Unsupported format version: {version}")]
    UnsupportedVersion { version: u32 },
    
    #[error("Field not found: {field_id}")]
    FieldNotFound { field_id: u32 },
    
    #[error("Field size mismatch: expected {expected}, got {got}")]
    FieldSizeMismatch { expected: usize, got: usize },
    
    #[error("Buffer too small: need {needed} bytes, have {have}")]
    BufferTooSmall { needed: usize, have: usize },
    
    #[error("Invalid offset: {offset} exceeds buffer size {size}")]
    InvalidOffset { offset: usize, size: usize },

    #[error("Field {field_id} at offset {offset} is not aligned to {align} bytes; use get_field_unaligned")]
    UnalignedField {
        field_id: u32,
        offset: usize,
        align: usize,
    },

    #[error("Invalid header size: expected {expected}, found {found}")]
    InvalidHeaderSize { expected: usize, found: usize },

    #[error("Invalid offset table size: {size} is not a multiple of the entry size ({entry_size})")]
    InvalidOffsetTableSize { size: usize, entry_size: usize },

    #[error("Field {field_id} contains invalid UTF-8")]
    InvalidUtf8 { field_id: u32 },

    #[error("Field {field_id} has type {field_type}, which is not usable as a fixed-size field")]
    WrongFieldType { field_id: u32, field_type: u16 },
}

pub type Result<T> = std::result::Result<T, SerializationError>;
