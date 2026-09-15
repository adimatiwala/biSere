use bisere::*;
use bisere::format::MAGIC;
use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
struct TestData {
    id: u64,
    age: u32,
    score: f64,
    active: u8, // Using u8 instead of bool since bool is not Pod
}

fn create_test_buffer() -> Vec<u8> {
    let data = TestData {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1, // 1 for true
    };
    
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<TestData>() as u32;
    let var_size = 256;
    
    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);
    
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
            offset: {
                offset += 8;
                offset
            },
            field_type: FieldType::Uint32 as u16,
            size: 4,
        },
        OffsetEntry {
            field_id: 3,
            offset: {
                offset += 4;
                offset
            },
            field_type: FieldType::Float64 as u16,
            size: 8,
        },
        OffsetEntry {
            field_id: 4,
            offset: {
                offset += 8;
                offset
            },
            field_type: FieldType::Uint8 as u16, // Using Uint8 instead of Bool
            size: 1,
        },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&data));
    serializer.write_var_data(&vec![0u8; var_size as usize]);
    
    serializer.into_buffer()
}

#[test]
fn test_roundtrip() {
    let buffer = create_test_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    
    // Whether a multi-byte field's real address is aligned depends on
    // the allocator's placement of the Vec (Miri deliberately does not
    // over-align it), not just its offset within the buffer — so this
    // test uses get_field_unaligned for every field that isn't 1 byte.
    let id: u64 = view.get_field_unaligned(1).unwrap();
    let age: u32 = view.get_field_unaligned(2).unwrap();
    let score: f64 = view.get_field_unaligned(3).unwrap();
    let active: &u8 = view.get_field(4).unwrap();

    assert_eq!(id, 12345);
    assert_eq!(age, 30);
    assert_eq!(score, 95.5);
    assert_eq!(*active, 1);
}

#[test]
fn test_zero_copy() {
    let buffer = create_test_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Uses the u8 field (active): align 1 is satisfied by any address,
    // so this reliably exercises get_field's zero-copy reference without
    // depending on where the allocator happens to place the Vec (a u64
    // or u32 field here would only be aligned by luck; see the note in
    // test_roundtrip).
    let active_ptr: &u8 = view.get_field(4).unwrap();

    // Verify that the pointer points into the original buffer
    let buffer_ptr = buffer.as_ptr() as usize;
    let active_ptr_addr = active_ptr as *const u8 as usize;

    assert!(active_ptr_addr >= buffer_ptr);
    assert!(active_ptr_addr < buffer_ptr + buffer.len());

    // Verify the value matches
    assert_eq!(*active_ptr, 1);
}

#[test]
fn test_modify_fixed() {
    let mut buffer = create_test_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    
    // Modify u64
    let new_id = 99999u64;
    view_mut.modify_field(1, &new_id).unwrap();
    
    // Modify u32
    let new_age = 35u32;
    view_mut.modify_field(2, &new_age).unwrap();
    
    // Modify f64
    let new_score = 88.8f64;
    view_mut.modify_field(3, &new_score).unwrap();
    
    // Modify bool (as u8)
    let new_active = 0u8; // 0 for false
    view_mut.modify_field(4, &new_active).unwrap();
    
    // Verify modifications
    let view = BinaryView::view(&buffer).unwrap();
    // Whether a multi-byte field's real address is aligned depends on
    // the allocator's placement of the Vec, not just its file offset;
    // use get_field_unaligned for every field wider than 1 byte.
    assert_eq!(view.get_field_unaligned::<u64>(1).unwrap(), 99999);
    assert_eq!(view.get_field_unaligned::<u32>(2).unwrap(), 35);
    assert_eq!(view.get_field_unaligned::<f64>(3).unwrap(), 88.8);
    assert_eq!(*view.get_field::<u8>(4).unwrap(), 0);
}

#[test]
fn test_modify_string() {
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
    
    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    
    // Modify string
    view_mut.modify_string(10, "World").unwrap();
    
    // Verify
    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_string(10).unwrap(), "World");
}

#[test]
fn test_modify_blob() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        256,
    );
    serializer.write_header(header);
    
    let entries = vec![OffsetEntry {
        field_id: 20,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: 256,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 256]);
    
    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    
    // Modify blob
    let new_blob = b"Test blob data";
    view_mut.modify_blob(20, new_blob).unwrap();
    
    // Verify - blob may have trailing zeros, so check it starts with our data
    let view = BinaryView::view(&buffer).unwrap();
    let retrieved = view.get_blob(20).unwrap();
    assert!(retrieved.len() >= new_blob.len());
    assert_eq!(&retrieved[..new_blob.len()], new_blob);
}

#[test]
fn test_error_invalid_magic() {
    let mut buffer = vec![0u8; 100];
    // Set invalid magic
    buffer[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    
    match BinaryView::view(&buffer) {
        Err(SerializationError::InvalidMagic { expected, found }) => {
            assert_eq!(expected, MAGIC);
            assert_eq!(found, 0xDEADBEEF);
        }
        _ => panic!("Expected InvalidMagic error"),
    }
}

#[test]
fn test_error_field_not_found() {
    let buffer = create_test_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    
    match view.get_field::<u32>(999) {
        Err(SerializationError::FieldNotFound { field_id }) => {
            assert_eq!(field_id, 999);
        }
        _ => panic!("Expected FieldNotFound error"),
    }
}

#[test]
fn test_error_buffer_too_small() {
    let buffer = vec![0u8; 10]; // Too small for header
    
    match BinaryView::view(&buffer) {
        Err(SerializationError::BufferTooSmall { needed, have }) => {
            assert!(needed > have);
        }
        _ => panic!("Expected BufferTooSmall error"),
    }
}

#[test]
fn test_error_bounds_checking() {
    // Test InvalidOffset - create buffer with invalid offset entry
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        10, // Small var section
    );
    serializer.write_header(header);
    
    let entries = vec![OffsetEntry {
        field_id: 1,
        offset: 1000, // Invalid offset beyond buffer
        field_type: FieldType::Uint32 as u16,
        size: 4,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 10]);
    
    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    
    // This should fail with InvalidOffset
    match view.get_field::<u32>(1) {
        Err(SerializationError::InvalidOffset { .. }) => {}
        _ => panic!("Expected InvalidOffset error"),
    }
    
    // Test FieldSizeMismatch
    let mut buffer2 = create_test_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer2).unwrap();
    
    // Try to modify with wrong size
    let wrong_value = 0u16; // Should be u32
    match view_mut.modify_field(2, &wrong_value) {
        Err(SerializationError::FieldSizeMismatch { expected, got }) => {
            assert_eq!(expected, 4);
            assert_eq!(got, 2);
        }
        _ => panic!("Expected FieldSizeMismatch error"),
    }
    
    // Test string size mismatch
    let mut serializer3 = BinarySerializer::new();
    let header3 = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        10, // Small var section
    );
    serializer3.write_header(header3);
    
    let entries3 = vec![OffsetEntry {
        field_id: 10,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 10, // Only 10 bytes
    }];
    serializer3.write_offset_table(&entries3);
    serializer3.write_data(&[]);
    serializer3.write_var_data(&vec![0u8; 10]);
    
    let mut buffer3 = serializer3.into_buffer();
    let mut view_mut3 = BinaryViewMut::view_mut(&mut buffer3).unwrap();
    
    // Try to write string that's too long
    match view_mut3.modify_string(10, "This string is way too long to fit") {
        Err(SerializationError::FieldSizeMismatch { expected, got }) => {
            assert!(got > expected);
        }
        _ => panic!("Expected FieldSizeMismatch error"),
    }
}

// ========== Comprehensive Additional Tests ==========

#[test]
fn test_all_integer_types() {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, Pod, Zeroable)]
    struct AllInts {
        i8_val: i8,
        i16_val: i16,
        i32_val: i32,
        i64_val: i64,
        u8_val: u8,
        u16_val: u16,
        u32_val: u32,
        u64_val: u64,
    }

    let data = AllInts {
        i8_val: -128,
        i16_val: -32768,
        i32_val: -2147483648,
        i64_val: -9223372036854775808,
        u8_val: 255,
        u16_val: 65535,
        u32_val: 4294967295,
        u64_val: 18446744073709551615,
    };

    let mut serializer = BinarySerializer::new();
    let offset_table_size = 8 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<AllInts>() as u32;
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    serializer.write_header(header);

    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry { field_id: 1, offset, field_type: FieldType::Int8 as u16, size: 1 },
        OffsetEntry { field_id: 2, offset: { offset += 1; offset }, field_type: FieldType::Int16 as u16, size: 2 },
        OffsetEntry { field_id: 3, offset: { offset += 2; offset }, field_type: FieldType::Int32 as u16, size: 4 },
        OffsetEntry { field_id: 4, offset: { offset += 4; offset }, field_type: FieldType::Int64 as u16, size: 8 },
        OffsetEntry { field_id: 5, offset: { offset += 8; offset }, field_type: FieldType::Uint8 as u16, size: 1 },
        OffsetEntry { field_id: 6, offset: { offset += 1; offset }, field_type: FieldType::Uint16 as u16, size: 2 },
        OffsetEntry { field_id: 7, offset: { offset += 2; offset }, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 8, offset: { offset += 4; offset }, field_type: FieldType::Uint64 as u16, size: 8 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&data));
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Some of these fields land at addresses unaligned for their type in
    // this packed layout; get_field correctly rejects those, so this test
    // uses get_field_unaligned throughout to exercise both accessors'
    // correctness without hand-verifying alignment per field.
    assert_eq!(view.get_field_unaligned::<i8>(1).unwrap(), -128);
    assert_eq!(view.get_field_unaligned::<i16>(2).unwrap(), -32768);
    assert_eq!(view.get_field_unaligned::<i32>(3).unwrap(), -2147483648);
    assert_eq!(view.get_field_unaligned::<i64>(4).unwrap(), -9223372036854775808);
    assert_eq!(view.get_field_unaligned::<u8>(5).unwrap(), 255);
    assert_eq!(view.get_field_unaligned::<u16>(6).unwrap(), 65535);
    assert_eq!(view.get_field_unaligned::<u32>(7).unwrap(), 4294967295);
    assert_eq!(view.get_field_unaligned::<u64>(8).unwrap(), 18446744073709551615);
}

#[test]
fn test_all_float_types() {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, Pod, Zeroable)]
    struct AllFloats {
        f32_val: f32,
        f64_val: f64,
    }

    let data = AllFloats {
        f32_val: 3.14159,
        f64_val: 2.718281828459045,
    };

    let mut serializer = BinarySerializer::new();
    let offset_table_size = 2 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<AllFloats>() as u32;
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    serializer.write_header(header);

    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry { field_id: 1, offset, field_type: FieldType::Float32 as u16, size: 4 },
        OffsetEntry { field_id: 2, offset: { offset += 4; offset }, field_type: FieldType::Float64 as u16, size: 8 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&data));
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Whether a field's real address is aligned depends on the
    // allocator's placement of the Vec, not just its file offset; use
    // get_field_unaligned for both fields here.
    let f32_val = view.get_field_unaligned::<f32>(1).unwrap();
    let f64_val = view.get_field_unaligned::<f64>(2).unwrap();
    assert!((f32_val - 3.14159).abs() < 0.0001);
    assert!((f64_val - 2.718281828459045).abs() < 0.0000001);
}

#[test]
fn test_edge_case_values() {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, Pod, Zeroable)]
    struct EdgeCases {
        zero_u64: u64,
        max_u64: u64,
        min_i64: i64,
        zero_f64: f64,
        neg_f64: f64,
    }

    let data = EdgeCases {
        zero_u64: 0,
        max_u64: u64::MAX,
        min_i64: i64::MIN,
        zero_f64: 0.0,
        neg_f64: -123.456,
    };

    let mut serializer = BinarySerializer::new();
    let offset_table_size = 5 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<EdgeCases>() as u32;
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    serializer.write_header(header);

    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry { field_id: 1, offset, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 2, offset: { offset += 8; offset }, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 3, offset: { offset += 8; offset }, field_type: FieldType::Int64 as u16, size: 8 },
        OffsetEntry { field_id: 4, offset: { offset += 8; offset }, field_type: FieldType::Float64 as u16, size: 8 },
        OffsetEntry { field_id: 5, offset: { offset += 8; offset }, field_type: FieldType::Float64 as u16, size: 8 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&data));
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // The 5-entry offset table shifts the data section to an address not
    // a multiple of 8, so every 8-byte field here is unaligned.
    assert_eq!(view.get_field_unaligned::<u64>(1).unwrap(), 0);
    assert_eq!(view.get_field_unaligned::<u64>(2).unwrap(), u64::MAX);
    assert_eq!(view.get_field_unaligned::<i64>(3).unwrap(), i64::MIN);
    assert_eq!(view.get_field_unaligned::<f64>(4).unwrap(), 0.0);
    assert!((view.get_field_unaligned::<f64>(5).unwrap() - (-123.456)).abs() < 0.0001);
}

#[test]
fn test_multiple_strings() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        3 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        512,
    );
    serializer.write_header(header);

    let entries = vec![
        OffsetEntry { field_id: 10, offset: 0, field_type: FieldType::String as u16, size: 100 },
        OffsetEntry { field_id: 20, offset: 100, field_type: FieldType::String as u16, size: 200 },
        OffsetEntry { field_id: 30, offset: 300, field_type: FieldType::String as u16, size: 212 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);

    let mut var_data = vec![0u8; 512];
    var_data[0..5].copy_from_slice(b"First");
    var_data[100..106].copy_from_slice(b"Second");
    var_data[300..305].copy_from_slice(b"Third");
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    assert_eq!(view.get_string(10).unwrap(), "First");
    assert_eq!(view.get_string(20).unwrap(), "Second");
    assert_eq!(view.get_string(30).unwrap(), "Third");
}

#[test]
fn test_multiple_blobs() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        3 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        512,
    );
    serializer.write_header(header);

    let entries = vec![
        OffsetEntry { field_id: 11, offset: 0, field_type: FieldType::Blob as u16, size: 50 },
        OffsetEntry { field_id: 22, offset: 50, field_type: FieldType::Blob as u16, size: 100 },
        OffsetEntry { field_id: 33, offset: 150, field_type: FieldType::Blob as u16, size: 362 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 512]);

    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();

    let blob1 = b"Blob 1 data";
    let blob2 = b"Blob 2 data longer";
    let blob3 = b"Blob 3 data even longer";

    view_mut.modify_blob(11, blob1).unwrap();
    view_mut.modify_blob(22, blob2).unwrap();
    view_mut.modify_blob(33, blob3).unwrap();

    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(&view.get_blob(11).unwrap()[..blob1.len()], blob1);
    assert_eq!(&view.get_blob(22).unwrap()[..blob2.len()], blob2);
    assert_eq!(&view.get_blob(33).unwrap()[..blob3.len()], blob3);
}

#[test]
fn test_empty_string() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        100,
    );
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 10,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 100,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 100]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_string(10).unwrap(), "");
}

#[test]
fn test_empty_blob() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        100,
    );
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 20,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: 100,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 100]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    let blob = view.get_blob(20).unwrap();
    assert_eq!(blob.len(), 100);
    assert!(blob.iter().all(|&b| b == 0));
}

#[test]
fn test_unicode_string() {
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

    let unicode_str = "Hello 世界 🌍";
    let mut var_data = vec![0u8; 256];
    var_data[0..unicode_str.len()].copy_from_slice(unicode_str.as_bytes());
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_string(10).unwrap(), unicode_str);
}

#[test]
fn test_non_sequential_field_ids() {
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = 4 + 8 + 4 + 8; // u32 + u64 + u32 + u64
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    serializer.write_header(header);

    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry { field_id: 100, offset, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 50, offset: { offset += 4; offset }, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 200, offset: { offset += 8; offset }, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 1, offset: { offset += 4; offset }, field_type: FieldType::Uint64 as u16, size: 8 },
    ];
    serializer.write_offset_table(&entries);

    let mut data = vec![0u8; data_size as usize];
    data[0..4].copy_from_slice(&100u32.to_le_bytes());
    data[4..12].copy_from_slice(&200u64.to_le_bytes());
    data[12..16].copy_from_slice(&300u32.to_le_bytes());
    data[16..24].copy_from_slice(&400u64.to_le_bytes());
    serializer.write_data(&data);
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Real alignment depends on the allocator's placement of the Vec,
    // not just file offset; see the note in test_roundtrip.
    assert_eq!(view.get_field_unaligned::<u32>(100).unwrap(), 100);
    assert_eq!(view.get_field_unaligned::<u64>(50).unwrap(), 200);
    assert_eq!(view.get_field_unaligned::<u32>(200).unwrap(), 300);
    assert_eq!(view.get_field_unaligned::<u64>(1).unwrap(), 400);
}

#[test]
fn test_multiple_modifications() {
    let mut buffer = create_test_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();

    // Modify all fields multiple times
    for i in 0..10 {
        let new_age = 20 + i as u32;
        view_mut.modify_field(2, &new_age).unwrap();
    }

    let view = BinaryView::view(&buffer).unwrap();
    // See the allocator-alignment note in test_roundtrip.
    assert_eq!(view.get_field_unaligned::<u32>(2).unwrap(), 29);
}

#[test]
fn test_large_buffer() {
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 1 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = 0;
    let var_size = 65535; // Max u16 value
    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 1,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: var_size as u16,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; var_size as usize]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    let blob = view.get_blob(1).unwrap();
    assert_eq!(blob.len(), var_size as usize);
}

#[test]
fn test_many_fields() {
    const NUM_FIELDS: usize = 50;
    let mut serializer = BinarySerializer::new();
    let offset_table_size = (NUM_FIELDS * std::mem::size_of::<OffsetEntry>()) as u32;
    let data_size = (NUM_FIELDS * 4) as u32; // Each field is u32
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    serializer.write_header(header);

    let mut offset = 0u32;
    let mut entries = Vec::new();
    for i in 0..NUM_FIELDS {
        entries.push(OffsetEntry {
            field_id: i as u32,
            offset,
            field_type: FieldType::Uint32 as u16,
            size: 4,
        });
        offset += 4;
    }
    serializer.write_offset_table(&entries);

    let mut data = vec![0u8; data_size as usize];
    for i in 0..NUM_FIELDS {
        let value = (i * 100) as u32;
        data[i * 4..(i + 1) * 4].copy_from_slice(&value.to_le_bytes());
    }
    serializer.write_data(&data);
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Real alignment depends on the allocator's placement of the Vec,
    // not just file offset; see the note in test_roundtrip.
    for i in 0..NUM_FIELDS {
        let value = view.get_field_unaligned::<u32>(i as u32).unwrap();
        assert_eq!(value, (i * 100) as u32);
    }
}

#[test]
fn test_string_boundary_conditions() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        10,
    );
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 10,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 10, // Exactly 10 bytes (9 chars + null)
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);

    let mut var_data = vec![0u8; 10];
    var_data[0..9].copy_from_slice(b"123456789");
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_string(10).unwrap(), "123456789");
}

#[test]
fn test_error_unsupported_version() {
    let mut buffer = vec![0u8; 100];
    // Set correct magic but wrong version
    buffer[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    buffer[4..8].copy_from_slice(&999u32.to_le_bytes()); // Invalid version

    match BinaryView::view(&buffer) {
        Err(SerializationError::UnsupportedVersion { version }) => {
            assert_eq!(version, 999);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_error_wrong_field_type() {
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
        field_type: FieldType::Blob as u16, // Wrong type - should be String
        size: 256,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&vec![0u8; 256]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // Try to get as string when it's a blob
    match view.get_string(10) {
        Err(SerializationError::FieldSizeMismatch { .. }) => {}
        _ => panic!("Expected FieldSizeMismatch error for wrong type"),
    }
}

#[test]
fn test_modify_string_to_empty() {
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

    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    view_mut.modify_string(10, "").unwrap();

    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_string(10).unwrap(), "");
}

#[test]
fn test_find_entry() {
    let buffer = create_test_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    assert!(view.find_entry(1).is_some());
    assert!(view.find_entry(2).is_some());
    assert!(view.find_entry(3).is_some());
    assert!(view.find_entry(4).is_some());
    assert!(view.find_entry(999).is_none());
}

#[test]
fn test_view_rejects_offset_table_size_not_multiple_of_entry_size() {
    // offset_table_size = 13 is not a multiple of size_of::<OffsetEntry>()
    // (12). Every existing length check (header size, total_size) passes
    // for a buffer sized to match, and then bytemuck::cast_slice panics
    // on the leftover byte instead of BinaryView::view returning Err.
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(13, 0, 0);
    serializer.write_header(header);
    serializer.write_data(&[0u8; 13]); // pad so buffer.len() >= total_size
    let buffer = serializer.into_buffer();

    match BinaryView::view(&buffer) {
        Err(_) => {} // any error is fine; a panic is not
        Ok(_) => panic!("expected an error for a malformed offset table size"),
    }
}

#[test]
fn test_header_total_size_does_not_overflow_u32_arithmetic() {
    // header_size + offset_table_size + data_size + var_size must not be
    // summed as u32 before widening to usize: u32::MAX-scale fields can
    // wrap silently in release builds, making total_size() (and the
    // buffer.len() < total_size guard in view()) pass when it should
    // not.
    let header = FormatHeader {
        magic: bisere::format::MAGIC,
        version: bisere::format::VERSION,
        header_size: bisere::format::HEADER_SIZE as u32,
        offset_table_size: u32::MAX - 10,
        data_size: u32::MAX - 10,
        var_size: u32::MAX - 10,
        checksum: 0,
        reserved: [0; 6],
    };
    // On 64-bit usize this must reflect the true (huge) sum, not a
    // wrapped-around small number that a short buffer could satisfy.
    assert!(header.total_size() > u32::MAX as usize);
}

#[test]
fn test_reserve_does_not_change_serialized_output() {
    // BinarySerializer::reserve is a pure pre-allocation hint (like
    // Vec::reserve): calling it before the write_* sequence must produce
    // byte-for-byte identical output to not calling it.
    let data = TestData { id: 1, age: 2, score: 3.0, active: 1 };
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<TestData>() as u32;
    let header = FormatHeader::new(offset_table_size, data_size, 0);
    let entries = [
        OffsetEntry { field_id: 1, offset: 0, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 2, offset: 8, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 3, offset: 12, field_type: FieldType::Float64 as u16, size: 8 },
        OffsetEntry { field_id: 4, offset: 20, field_type: FieldType::Uint8 as u16, size: 1 },
    ];

    let mut without_reserve = BinarySerializer::new();
    without_reserve.write_header(header);
    without_reserve.write_offset_table(&entries);
    without_reserve.write_data(bytemuck::bytes_of(&data));
    without_reserve.write_var_data(&[]);

    let mut with_reserve = BinarySerializer::new();
    with_reserve.reserve(header.total_size());
    with_reserve.write_header(header);
    with_reserve.write_offset_table(&entries);
    with_reserve.write_data(bytemuck::bytes_of(&data));
    with_reserve.write_var_data(&[]);

    assert_eq!(without_reserve.into_buffer(), with_reserve.into_buffer());
}

#[test]
fn test_view_unchecked_reads_a_wellformed_buffer_correctly() {
    let buffer = create_test_buffer();
    let view = BinaryView::view_unchecked(&buffer).unwrap();

    // get_field_unaligned throughout: whether a field's real address is
    // aligned depends on the allocator's placement of the Vec, not just
    // its file offset — see the note in test_roundtrip.
    let id: u64 = view.get_field_unaligned(1).unwrap();
    assert_eq!(id, 12345);
    let score: f64 = view.get_field_unaligned(3).unwrap();
    assert_eq!(score, 95.5);
}

#[test]
fn test_view_unchecked_still_rejects_malformed_offset_table_size() {
    // view_unchecked skips magic/version/total_size validation, but must
    // keep the offset_table_size % entry_size check — otherwise a
    // malformed header reaches bytemuck::cast_slice and panics instead of
    // returning an error, reopening the bug fixed in the soundness pass.
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(13, 0, 0);
    serializer.write_header(header);
    serializer.write_data(&[0u8; 13]);
    let buffer = serializer.into_buffer();

    match BinaryView::view_unchecked(&buffer) {
        Err(SerializationError::InvalidOffsetTableSize { .. }) => {}
        Err(e) => panic!("expected InvalidOffsetTableSize, got a different error: {}", e),
        Ok(_) => panic!("expected InvalidOffsetTableSize, got Ok"),
    }
}

#[test]
fn test_view_unchecked_rejects_buffer_too_small_for_header() {
    let buffer = vec![0u8; 10];
    match BinaryView::view_unchecked(&buffer) {
        Err(SerializationError::BufferTooSmall { needed, have }) => {
            assert_eq!(needed, 80);
            assert_eq!(have, 10);
        }
        Err(e) => panic!("expected BufferTooSmall, got a different error: {}", e),
        Ok(_) => panic!("expected BufferTooSmall, got Ok"),
    }
}

#[test]
fn test_modify_string_zeros_trailing_bytes_on_shrink() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(1 * std::mem::size_of::<OffsetEntry>() as u32, 0, 10);
    serializer.write_header(header);
    let entries = vec![OffsetEntry { field_id: 10, offset: 0, field_type: FieldType::String as u16, size: 10 }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(b"WorldWide\0"); // 9 bytes + NUL, fills the field

    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    view_mut.modify_string(10, "Hi").unwrap(); // shrink to 2 bytes

    // Every byte past "Hi" in the field's 10-byte span must be zero, not
    // leftover bytes from "WorldWide" — this is the property the
    // fill-then-copy vs copy-then-fill reordering must preserve.
    let var_start = header.var_section_offset();
    assert_eq!(&buffer[var_start..var_start + 2], b"Hi");
    assert!(buffer[var_start + 2..var_start + 10].iter().all(|&b| b == 0));
}

#[test]
fn test_modify_blob_zeros_trailing_bytes_on_shrink() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(1 * std::mem::size_of::<OffsetEntry>() as u32, 0, 10);
    serializer.write_header(header);
    let entries = vec![OffsetEntry { field_id: 20, offset: 0, field_type: FieldType::Blob as u16, size: 10 }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&[0xAAu8; 10]); // field fully "dirty"

    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();
    view_mut.modify_blob(20, &[1, 2, 3]).unwrap(); // shrink to 3 bytes

    let var_start = header.var_section_offset();
    assert_eq!(&buffer[var_start..var_start + 3], &[1, 2, 3]);
    assert!(buffer[var_start + 3..var_start + 10].iter().all(|&b| b == 0));
}

#[test]
fn test_serialize_to_buffer_matches_manual_serializer() {
    // serialize_to_buffer is the one-call equivalent of writing header,
    // offset table, data, and var sections through BinarySerializer in
    // order; benches/serialization_bench.rs already depends on it
    // existing with this signature.
    let header = FormatHeader::new(
        4 * std::mem::size_of::<OffsetEntry>() as u32,
        std::mem::size_of::<TestData>() as u32,
        0,
    );
    let entries = [
        OffsetEntry { field_id: 1, offset: 0, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 2, offset: 8, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 3, offset: 12, field_type: FieldType::Float64 as u16, size: 8 },
        OffsetEntry { field_id: 4, offset: 20, field_type: FieldType::Uint8 as u16, size: 1 },
    ];
    let data = TestData { id: 12345, age: 30, score: 95.5, active: 1 };

    let via_helper = bisere::serialize_to_buffer(&header, &entries, bytemuck::bytes_of(&data), &[]);

    let mut manual = BinarySerializer::new();
    manual.write_header(header);
    manual.write_offset_table(&entries);
    manual.write_data(bytemuck::bytes_of(&data));
    manual.write_var_data(&[]);
    let via_manual = manual.into_buffer();

    assert_eq!(via_helper, via_manual);

    // And it must actually be readable back through BinaryView.
    let view = BinaryView::view(&via_helper).unwrap();
    assert_eq!(view.get_field_unaligned::<u64>(1).unwrap(), 12345);
}

#[test]
fn test_get_string_does_not_read_past_entry_size() {
    // Two adjacent String fields, neither null-terminated within its own
    // declared size. get_string must stop at entry.size, not keep
    // scanning into the next field's bytes looking for a NUL.
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(2 * std::mem::size_of::<OffsetEntry>() as u32, 0, 10);
    serializer.write_header(header);

    let entries = vec![
        OffsetEntry { field_id: 1, offset: 0, field_type: FieldType::String as u16, size: 5 },
        OffsetEntry { field_id: 2, offset: 5, field_type: FieldType::String as u16, size: 5 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    // Neither 5-byte region contains a NUL anywhere.
    serializer.write_var_data(b"AAAAABBBBB");

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    assert_eq!(view.get_string(1).unwrap(), "AAAAA");
}

#[test]
fn test_get_string_out_of_bounds_offset_returns_error_not_panic() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(1 * std::mem::size_of::<OffsetEntry>() as u32, 0, 4);
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 1,
        offset: 1000, // pushes string_offset past the buffer entirely
        field_type: FieldType::String as u16,
        size: 4,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&[0u8; 4]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    assert!(view.get_string(1).is_err());
}

#[test]
fn test_get_string_invalid_utf8_returns_invalid_utf8_error() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(1 * std::mem::size_of::<OffsetEntry>() as u32, 0, 4);
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 1,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 4,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&[0xFF, 0xFE, 0x00, 0x00]); // invalid UTF-8, NUL-terminated

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    match view.get_string(1) {
        Err(SerializationError::InvalidUtf8 { field_id }) => assert_eq!(field_id, 1),
        other => panic!("expected InvalidUtf8, got {:?}", other),
    }
}

#[test]
fn test_modify_field_rejects_variable_length_field_type() {
    // A field declared as String must not be writable through
    // modify_field: that would overwrite the var-section bytes as raw
    // POD data, bypassing the size/NUL-terminator rules modify_string
    // enforces.
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(1 * std::mem::size_of::<OffsetEntry>() as u32, 0, 8);
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 1,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 8,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);
    serializer.write_var_data(&[0u8; 8]);

    let mut buffer = serializer.into_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();

    let value = 0xFFFFFFFFFFFFFFFFu64; // matches entry.size (8), wrong field_type
    assert!(view_mut.modify_field(1, &value).is_err());
}

#[test]
fn test_get_field_rejects_unaligned_field() {
    let buffer = create_test_buffer();
    let view = BinaryView::view(&buffer).unwrap();

    // score (field_id 3) is an f64 that lands at an address not a
    // multiple of 8 in this layout. get_field must reject it instead of
    // creating a misaligned &f64 (which is UB and aborts the process on
    // current rustc). get_field_unaligned is the correct escape hatch.
    match view.get_field::<f64>(3) {
        Err(SerializationError::UnalignedField { field_id, .. }) => {
            assert_eq!(field_id, 3);
        }
        other => panic!("expected UnalignedField error, got {:?}", other),
    }

    let score = view.get_field_unaligned::<f64>(3).unwrap();
    assert_eq!(score, 95.5);
}

#[test]
fn test_view_mut_no_aliasing_ub() {
    // BinaryViewMut must not hold overlapping &mut references into the
    // same buffer (header/offset_table as separate &mut fields aliasing
    // `buffer` is UB). Exercising find_entry (reads the table) and
    // modify_field (writes through buffer) in the same view is exactly
    // the access pattern that trips Stacked/Tree Borrows under Miri if
    // the references overlap. Run under `cargo +nightly miri test` to
    // check.
    let mut buffer = create_test_buffer();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer).unwrap();

    assert!(view_mut.find_entry(1).is_some());
    let new_age = 40u32;
    view_mut.modify_field(2, &new_age).unwrap();
    assert!(view_mut.find_entry(2).is_some());

    // get_field_unaligned rather than get_field: this test is about
    // aliasing in BinaryViewMut, not about get_field's alignment check
    // (test_get_field_rejects_unaligned_field covers that) — and whether
    // this field is aligned depends on the allocator's placement of the
    // Vec, which Miri deliberately does not over-align.
    let view = BinaryView::view(&buffer).unwrap();
    assert_eq!(view.get_field_unaligned::<u32>(2).unwrap(), 40);
}

#[test]
fn test_buffer_methods() {
    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(0, 0, 0);
    serializer.write_header(header);
    
    let buffer_ref = serializer.buffer();
    assert!(buffer_ref.len() >= 80); // At least header size
    
    let buffer = serializer.into_buffer();
    assert!(buffer.len() >= 80);
}
