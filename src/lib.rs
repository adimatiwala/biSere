pub mod error;
pub mod format;
pub mod serializer;

pub use error::{Result, SerializationError};
pub use format::{FieldType, FormatHeader, OffsetEntry};
pub use serializer::{BinarySerializer, BinaryView, BinaryViewMut};
