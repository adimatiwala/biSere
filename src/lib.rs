pub mod error;
pub mod format;
pub mod serializer;

pub use error::{Result, SerializationError};
pub use format::{FieldType, FormatHeader, OffsetEntry};
pub use serializer::{serialize_to_buffer, BinarySerializer, BinaryView, BinaryViewMut};
