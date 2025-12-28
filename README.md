# biSere

**biSere** (Binary Serialization) is a high-performance, zero-copy binary serialization library for Rust. It provides efficient serialization with support for in-place modification of serialized data, making it ideal for scenarios where you need to update data without full deserialization.

## Project Status

**Disclaimer**: This project was developed as a side project for educational and experimental purposes. While the library demonstrates promising performance characteristics in specific use cases (particularly in-place modification), performance optimizations and additional features are still under development. Please be aware of this:

- Performance may not match production-grade serialization libraries in all scenarios
- Additional features and optimizations are planned (see Future Improvements section)
- The library is suitable for experimentation and specific use cases, but may require further development for production use

## Overview

biSere is designed around three core principles:

1. **Zero-Copy Deserialization**: Access serialized data directly without copying
2. **In-Place Modification**: Update fields in serialized buffers without re-serialization
3. **Type Safety**: Leverage Rust's type system with `bytemuck::Pod` for safe transmutation

## Key Features

- Zero-copy field access via `BinaryView`
- In-place field modification via `BinaryViewMut`
- Support for fixed-size types (integers, floats, booleans)
- Support for variable-length types (strings, blobs)
- Offset table for efficient field lookup
- Format validation (magic number, version checking)
- Comprehensive error handling
- Memory-safe with bounds checking

## Binary Format Specification

The biSere format consists of four sections:

```
┌─────────────────────────────────────────┐
│         Format Header (80 bytes)        │
│  - Magic: 0x42495345 ("BISE")          │
│  - Version: 1                            │
│  - Header size, offset table size        │
│  - Data section size, var section size   │
│  - Checksum (optional, currently 0)      │
│  - Reserved space (48 bytes)              │
├─────────────────────────────────────────┤
│      Offset Table (variable size)        │
│  - Array of OffsetEntry structures       │
│  - Each entry: field_id, offset, type    │
├─────────────────────────────────────────┤
│      Fixed Data Section                  │
│  - Fixed-size fields (POD types)          │
│  - Contiguous layout                      │
├─────────────────────────────────────────┤
│      Variable Data Section               │
│  - Strings (null-terminated)              │
│  - Blobs (binary data)                    │
└─────────────────────────────────────────┘
```

### Format Header Structure

```rust
struct FormatHeader {
    magic: u32,              // 0x42495345 ("BISE")
    version: u32,            // Format version (currently 1)
    header_size: u32,        // Always 80
    offset_table_size: u32,  // Size in bytes
    data_size: u32,          // Fixed data section size
    var_size: u32,           // Variable data section size
    checksum: u64,          // Optional integrity check (currently unused)
    reserved: [u64; 6],      // Reserved for future use
}
```

### Offset Entry Structure

```rust
struct OffsetEntry {
    field_id: u32,    // Unique identifier for the field
    offset: u32,      // Offset from start of data section
    field_type: u16,  // FieldType enum value
    size: u16,        // Size in bytes (fixed) or max size (variable)
}
```

### Supported Field Types

| Type ID | Type Name | Size | Description |
|---------|-----------|------|-------------|
| 1 | Int8 | 1 | Signed 8-bit integer |
| 2 | Int16 | 2 | Signed 16-bit integer |
| 3 | Int32 | 4 | Signed 32-bit integer |
| 4 | Int64 | 8 | Signed 64-bit integer |
| 5 | Uint8 | 1 | Unsigned 8-bit integer |
| 6 | Uint16 | 2 | Unsigned 16-bit integer |
| 7 | Uint32 | 4 | Unsigned 32-bit integer |
| 8 | Uint64 | 8 | Unsigned 64-bit integer |
| 9 | Float32 | 4 | 32-bit floating point |
| 10 | Float64 | 8 | 64-bit floating point |
| 11 | Bool | 1 | Boolean (stored as u8) |
| 12 | String | variable | Null-terminated UTF-8 string |
| 13 | Blob | variable | Binary data |

## Architecture

### Core Components

1. **`BinarySerializer`**: Builds serialized buffers
   - Writes header, offset table, and data sections
   - Manages buffer construction

2. **`BinaryView<'a>`**: Zero-copy read-only view
   - Validates format and provides field access
   - Returns references directly into the buffer

3. **`BinaryViewMut<'a>`**: Mutable view for in-place modification
   - Allows updating fields without re-serialization
   - Maintains format integrity

### Design Decisions

- **Offset Table**: Enables O(n) field lookup by field_id, allowing flexible field ordering
- **Separate Sections**: Fixed and variable data are separated for efficient access patterns
- **Packed Structs**: Uses `#[repr(C, packed)]` for compact, predictable layout
- **bytemuck Integration**: Leverages `Pod` trait for safe zero-copy operations

## Usage

### Basic Example

```rust
use bisere::*;
use bytemuck::{Pod, Zeroable};

// Define your data structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UserData {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

fn main() -> Result<()> {
    // 1. Create data
    let user = UserData {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1,
    };
    
    // 2. Serialize
    let mut serializer = BinarySerializer::new();
    
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<UserData>() as u32;
    let var_size = 256;
    
    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);
    
    // Build offset table
    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry {
            field_id: 1,
            offset,
            field_type: FieldType::Uint64 as u16,
            size: 8,
        },
        OffsetEntry {
            field_id: 2,
            offset: { offset += 8; offset },
            field_type: FieldType::Uint32 as u16,
            size: 4,
        },
        // ... more entries
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&user));
    serializer.write_var_data(&vec![0u8; var_size as usize]);
    
    let buffer = serializer.into_buffer();
    
    // 3. Deserialize (zero-copy)
    let view = BinaryView::view(&buffer)?;
    let id: &u64 = view.get_field(1)?;
    let age: &u32 = view.get_field(2)?;
    
    println!("ID: {}, Age: {}", id, age);
    
    // 4. In-place modification
    let mut buffer_mut = buffer.clone();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
    let new_age = 31u32;
    view_mut.modify_field(2, &new_age)?;
    
    Ok(())
}
```

### String Fields

```rust
// Serialize string
let mut serializer = BinarySerializer::new();
let header = FormatHeader::new(
    1 * std::mem::size_of::<OffsetEntry>() as u32,
    0,
    256,
);
serializer.write_header(header);

let entries = vec![OffsetEntry {
    field_id: 10,
    offset: 0,
    field_type: FieldType::String as u16,
    size: 256,
}];
serializer.write_offset_table(&entries);
serializer.write_data(&[]);

let mut var_data = vec![0u8; 256];
var_data[0..5].copy_from_slice(b"Hello");
serializer.write_var_data(&var_data);

let buffer = serializer.into_buffer();

// Read string (zero-copy)
let view = BinaryView::view(&buffer)?;
let name = view.get_string(10)?;
println!("Name: {}", name);

// Modify string in-place
let mut buffer_mut = buffer.clone();
let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
view_mut.modify_string(10, "World")?;
```

### Blob Fields

```rust
// Similar to strings, but for binary data
let blob_data = b"binary data";
let view = BinaryView::view(&buffer)?;
let blob = view.get_blob(20)?;

// Modify blob
let mut view_mut = BinaryViewMut::view_mut(&mut buffer)?;
view_mut.modify_blob(20, b"new binary data")?;
```

## API Reference

### BinarySerializer

- `new() -> Self`: Create a new serializer
- `write_header(header: FormatHeader)`: Write format header
- `write_offset_table(entries: &[OffsetEntry])`: Write offset table
- `write_data(data: &[u8])`: Write fixed-size data section
- `write_var_data(data: &[u8])`: Write variable-length data section
- `into_buffer() -> Vec<u8>`: Consume serializer and return buffer
- `buffer() -> &[u8]`: Get reference to current buffer

### BinaryView

- `view(buffer: &[u8]) -> Result<Self>`: Create view from buffer
- `find_entry(field_id: u32) -> Option<&OffsetEntry>`: Find offset entry
- `get_field<T: Pod>(field_id: u32) -> Result<&T>`: Get field reference (zero-copy)
- `get_string(field_id: u32) -> Result<&str>`: Get string field (zero-copy)
- `get_blob(field_id: u32) -> Result<&[u8]>`: Get blob field (zero-copy)

### BinaryViewMut

- `view_mut(buffer: &mut [u8]) -> Result<Self>`: Create mutable view
- `find_entry(field_id: u32) -> Option<&OffsetEntry>`: Find offset entry
- `modify_field<T: Pod>(field_id: u32, value: &T) -> Result<()>`: Modify fixed-size field
- `modify_string(field_id: u32, value: &str) -> Result<()>`: Modify string field
- `modify_blob(field_id: u32, value: &[u8]) -> Result<()>`: Modify blob field

## Error Handling

The library provides comprehensive error handling via `SerializationError`:

- `InvalidMagic`: Wrong magic number in header
- `UnsupportedVersion`: Format version mismatch
- `FieldNotFound`: Requested field_id doesn't exist
- `FieldSizeMismatch`: Type/size mismatch
- `BufferTooSmall`: Buffer insufficient for operation
- `InvalidOffset`: Offset exceeds buffer bounds

## Performance Characteristics

- **Zero-Copy**: Field access returns references directly into the buffer
- **O(n) Field Lookup**: Linear search through offset table (n = number of fields)
- **In-Place Updates**: No re-serialization needed for modifications
- **Memory Efficient**: Packed structs minimize overhead
- **Bounds Checking**: All operations validate bounds for safety

### Performance Considerations

- Field lookup is linear in the number of fields. For many fields, consider:
  - Sorting offset table by field_id and using binary search
  - Using a hash map for field_id → OffsetEntry mapping
- String modification requires the new value to fit in existing space
- Blob modification similarly constrained by pre-allocated size

## Limitations and Known Issues

1. **Alignment**: For unaligned types (e.g., `f64` in packed structs), direct pointer dereference may cause alignment issues. The current implementation uses unsafe pointer access which may require copying for proper alignment.

2. **Checksum**: The checksum field in the header is currently unused (always 0). Future versions may implement integrity checking.

3. **Field Lookup**: Linear search through offset table. For large numbers of fields, consider optimizing the lookup strategy.

4. **String/Blob Size**: Variable-length fields cannot grow beyond their pre-allocated size during modification.

5. **UTF-8 Validation**: String errors currently map to `FieldSizeMismatch` with zeros, which could be improved.

## Dependencies

- `bytemuck`: Safe transmutation for zero-copy operations
- `thiserror`: Error handling utilities

## Development

### Running Tests

```bash
cargo test
```

### Running Examples

The project includes two example programs:

**Basic Usage Example:**
```bash
cargo run --example usage
```
Demonstrates basic serialization, deserialization, string handling, and error cases.

**Comprehensive Test Driver:**
```bash
cargo run --example driver
```
A comprehensive test driver that exercises all major functionality of the library:

- Basic serialization/deserialization
- Zero-copy verification
- In-place modification
- Multiple field types (integers, floats, etc.)
- String fields
- Blob fields
- Mixed fixed and variable fields
- Error handling validation
- Format validation
- Buffer layout verification

The driver provides formatted output showing the status of each test and a summary at the end. All tests must pass for the driver to exit successfully.

### Benchmarks

The project includes comprehensive benchmarks comparing biSere against other serialization libraries. For detailed benchmark results and analysis, see [BENCHMARKS.md](BENCHMARKS.md).

Run benchmarks with:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench serialization_bench
cargo bench --bench varying_sizes_bench
```

**Benchmark Suites:**

1. **`serialization_bench`**: Comprehensive comparison against:
   - bincode
   - postcard
   - MessagePack (rmp-serde)
   - serde_json
   
   Tests serialization, deserialization, round-trip, zero-copy field access, and in-place modification.

2. **`varying_sizes_bench`**: Performance with varying data sizes (1, 10, 100, 1000 structs).

**Expected Results:**
- biSere demonstrates superior performance in deserialization operations (zero-copy) and field access
- biSere demonstrates superior performance in in-place modification operations (no re-serialization required)
- biSere may exhibit slower performance during initial serialization due to offset table setup overhead

See `benches/README.md` for detailed benchmark documentation and `BENCHMARKS.md` for comprehensive benchmark results and analysis.

## License

MIT


## Future Improvements

- [ ] Implement checksum computation and validation
- [ ] Optimize field lookup (binary search or hash map)
- [ ] Add alignment-safe field access
- [ ] Add builder API for easier serialization
- [ ] Support for nested structures
- [ ] Improved error messages for UTF-8 validation failures
- [ ] Dynamic string/blob resizing
- [ ] Field iteration API
- [ ] Serialization from structs (derive macro)
- [ ] Endianness handling
- [ ] Compression support
- [ ] Streaming serialization
- [ ] Schema versioning and migration
- [ ] Memory-mapped file support

