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
}

pub type Result<T> = std::result::Result<T, SerializationError>;
