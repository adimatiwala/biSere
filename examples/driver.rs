use bisere::*;
use bytemuck::{Pod, Zeroable};

// Test data structures
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
struct UserData {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         biSere Test Driver                                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut tests_passed = 0;
    let mut tests_failed = 0;

    // Test 1: Basic serialization and deserialization
    println!("┌─ Test 1: Basic Serialization/Deserialization ─────────────┐");
    match test_basic_serialization() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 2: Zero-copy verification
    println!("┌─ Test 2: Zero-Copy Verification ──────────────────────────┐");
    match test_zero_copy() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 3: In-place modification
    println!("┌─ Test 3: In-Place Modification ──────────────────────────┐");
    match test_inplace_modification() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 4: Multiple field types
    println!("┌─ Test 4: Multiple Field Types ───────────────────────────┐");
    match test_multiple_types() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 5: String fields
    println!("┌─ Test 5: String Fields ───────────────────────────────────┐");
    match test_string_fields() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 6: Blob fields
    println!("┌─ Test 6: Blob Fields ─────────────────────────────────────┐");
    match test_blob_fields() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 7: Mixed fixed and variable fields
    println!("┌─ Test 7: Mixed Fixed and Variable Fields ─────────────────┐");
    match test_mixed_fields() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 8: Error handling
    println!("┌─ Test 8: Error Handling ──────────────────────────────────┐");
    match test_error_handling() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 9: Format validation
    println!("┌─ Test 9: Format Validation ──────────────────────────────┐");
    match test_format_validation() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 10: Buffer size and layout
    println!("┌─ Test 10: Buffer Size and Layout ────────────────────────┐");
    match test_buffer_layout() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 11: All integer types
    println!("┌─ Test 11: All Integer Types ──────────────────────────────┐");
    match test_all_integer_types() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 12: Edge case values
    println!("┌─ Test 12: Edge Case Values ───────────────────────────────┐");
    match test_edge_case_values() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 13: Multiple strings
    println!("┌─ Test 13: Multiple Strings ───────────────────────────────┐");
    match test_multiple_strings() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 14: Multiple blobs
    println!("┌─ Test 14: Multiple Blobs ────────────────────────────────┐");
    match test_multiple_blobs() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 15: Empty strings and blobs
    println!("┌─ Test 15: Empty Strings and Blobs ────────────────────────┐");
    match test_empty_strings_blobs() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 16: Unicode strings
    println!("┌─ Test 16: Unicode Strings ────────────────────────────────┐");
    match test_unicode_strings() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 17: Non-sequential field IDs
    println!("┌─ Test 17: Non-Sequential Field IDs ────────────────────────┐");
    match test_non_sequential_field_ids() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 18: Multiple modifications
    println!("┌─ Test 18: Multiple Modifications ──────────────────────────┐");
    match test_multiple_modifications() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 19: Many fields stress test
    println!("┌─ Test 19: Many Fields Stress Test ────────────────────────┐");
    match test_many_fields() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 20: String boundary conditions
    println!("┌─ Test 20: String Boundary Conditions ──────────────────────┐");
    match test_string_boundary_conditions() {
        Ok(_) => {
            println!("│ ✓ PASSED");
            tests_passed += 1;
        }
        Err(e) => {
            println!("│ ✗ FAILED: {}", e);
            tests_failed += 1;
        }
    }
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Summary
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    Test Summary                            ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Tests Passed: {:2}                                          ║", tests_passed);
    println!("║ Tests Failed: {:2}                                          ║", tests_failed);
    println!("║ Total Tests: {:2}                                          ║", tests_passed + tests_failed);
    println!("╚════════════════════════════════════════════════════════════╝");

    if tests_failed > 0 {
        std::process::exit(1);
    }
}

fn test_basic_serialization() -> Result<()> {
    let user = UserData {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1,
    };

    // Serialize
    let buffer = serialize_user_data(&user)?;
    println!("│ Serialized {} bytes", buffer.len());

    // Deserialize
    let view = BinaryView::view(&buffer)?;
    let id: &u64 = view.get_field(1)?;
    let age: &u32 = view.get_field(2)?;
    let score: &f64 = view.get_field(3)?;
    let active: &u8 = view.get_field(4)?;

    // Copy values to avoid alignment issues with packed structs
    let id_val = *id;
    let age_val = *age;
    let score_val = *score;
    let active_val = *active;
    let user_id = user.id;
    let user_age = user.age;
    let user_score = user.score;
    let user_active = user.active;

    // Verify
    assert_eq!(id_val, user_id, "ID mismatch");
    assert_eq!(age_val, user_age, "Age mismatch");
    assert_eq!(score_val, user_score, "Score mismatch");
    assert_eq!(active_val, user_active, "Active mismatch");

    println!("│ Deserialized: ID={}, Age={}, Score={}, Active={}", id_val, age_val, score_val, active_val != 0);
    Ok(())
}

fn test_zero_copy() -> Result<()> {
    let user = UserData {
        id: 99999,
        age: 42,
        score: 88.8,
        active: 0,
    };

    let buffer = serialize_user_data(&user)?;
    let view = BinaryView::view(&buffer)?;
    let id_ptr: &u64 = view.get_field(1)?;

    // Verify pointer is within buffer
    let buffer_ptr = buffer.as_ptr() as usize;
    let id_ptr_addr = id_ptr as *const u64 as usize;

    assert!(
        id_ptr_addr >= buffer_ptr && id_ptr_addr < buffer_ptr + buffer.len(),
        "Pointer not within buffer bounds"
    );

    // Verify value (copy to avoid alignment issues)
    let id_ptr_val = *id_ptr;
    let user_id = user.id;
    assert_eq!(id_ptr_val, user_id, "Zero-copy value mismatch");

    println!("│ Zero-copy verified: pointer at offset {} from buffer start", 
             id_ptr_addr - buffer_ptr);
    Ok(())
}

fn test_inplace_modification() -> Result<()> {
    let user = UserData {
        id: 11111,
        age: 25,
        score: 75.0,
        active: 1,
    };

    let mut buffer = serialize_user_data(&user)?;
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer)?;

    // Modify multiple fields
    let new_age = 26u32;
    let new_score = 80.0f64;
    let new_active = 0u8;

    view_mut.modify_field(2, &new_age)?;
    view_mut.modify_field(3, &new_score)?;
    view_mut.modify_field(4, &new_active)?;

    // Verify modifications
    let view = BinaryView::view(&buffer)?;
    assert_eq!(*view.get_field::<u32>(2)?, new_age, "Age modification failed");
    assert_eq!(*view.get_field::<f64>(3)?, new_score, "Score modification failed");
    assert_eq!(*view.get_field::<u8>(4)?, new_active, "Active modification failed");

    println!("│ Modified: Age={}, Score={}, Active={}", new_age, new_score, new_active != 0);
    Ok(())
}

fn test_multiple_types() -> Result<()> {
    let point = Point3D { x: 1.5, y: 2.5, z: 3.5 };

    let mut serializer = BinarySerializer::new();
    let offset_table_size = 3 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<Point3D>() as u32;
    let var_size = 0;

    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);

    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry {
            field_id: 1,
            offset,
            field_type: FieldType::Float32 as u16,
            size: 4,
        },
        OffsetEntry {
            field_id: 2,
            offset: { offset += 4; offset },
            field_type: FieldType::Float32 as u16,
            size: 4,
        },
        OffsetEntry {
            field_id: 3,
            offset: { offset += 4; offset },
            field_type: FieldType::Float32 as u16,
            size: 4,
        },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&point));
    serializer.write_var_data(&[]);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer)?;

    let x: &f32 = view.get_field(1)?;
    let y: &f32 = view.get_field(2)?;
    let z: &f32 = view.get_field(3)?;

    // Copy values to avoid alignment issues
    let x_val = *x;
    let y_val = *y;
    let z_val = *z;
    let point_x = point.x;
    let point_y = point.y;
    let point_z = point.z;

    assert_eq!(x_val, point_x, "X mismatch");
    assert_eq!(y_val, point_y, "Y mismatch");
    assert_eq!(z_val, point_z, "Z mismatch");

    println!("│ Point3D: x={}, y={}, z={}", x, y, z);
    Ok(())
}

fn test_string_fields() -> Result<()> {
    let test_string = "Hello, biSere!";
    let max_size = 256;

    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        max_size,
    );
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 10,
        offset: 0,
        field_type: FieldType::String as u16,
        size: max_size as u16,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);

    let mut var_data = vec![0u8; max_size as usize];
    var_data[0..test_string.len()].copy_from_slice(test_string.as_bytes());
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer)?;
    let retrieved = view.get_string(10)?;

    assert_eq!(retrieved, test_string, "String mismatch");

    // Test modification
    let mut buffer_mut = buffer.clone();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
    let new_string = "Modified!";
    view_mut.modify_string(10, new_string)?;

    let view2 = BinaryView::view(&buffer_mut)?;
    let modified = view2.get_string(10)?;
    assert_eq!(modified, new_string, "String modification failed");

    println!("│ Original: '{}'", test_string);
    println!("│ Modified: '{}'", modified);
    Ok(())
}

fn test_blob_fields() -> Result<()> {
    let blob_data = b"Binary data\x00\x01\x02\x03";
    let max_size = 256;

    let mut serializer = BinarySerializer::new();
    let header = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        max_size,
    );
    serializer.write_header(header);

    let entries = vec![OffsetEntry {
        field_id: 20,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: max_size as u16,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);

    let mut var_data = vec![0u8; max_size as usize];
    var_data[0..blob_data.len()].copy_from_slice(blob_data);
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer)?;
    let retrieved = view.get_blob(20)?;

    assert_eq!(&retrieved[..blob_data.len()], blob_data, "Blob mismatch");

    // Test modification
    let mut buffer_mut = buffer.clone();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
    let new_blob = b"New blob data";
    view_mut.modify_blob(20, new_blob)?;

    let view2 = BinaryView::view(&buffer_mut)?;
    let modified = view2.get_blob(20)?;
    assert_eq!(&modified[..new_blob.len()], new_blob, "Blob modification failed");

    println!("│ Original blob length: {}", blob_data.len());
    println!("│ Modified blob length: {}", new_blob.len());
    Ok(())
}

fn test_mixed_fields() -> Result<()> {
    let user = UserData {
        id: 55555,
        age: 35,
        score: 92.3,
        active: 1,
    };
    let name = "John Doe";
    let max_var_size = 256;

    let mut serializer = BinarySerializer::new();
    let offset_table_size = 5 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<UserData>() as u32;

    let header = FormatHeader::new(offset_table_size, data_size, max_var_size);
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
            offset: { offset += 8; offset },
            field_type: FieldType::Uint32 as u16,
            size: 4,
        },
        OffsetEntry {
            field_id: 3,
            offset: { offset += 4; offset },
            field_type: FieldType::Float64 as u16,
            size: 8,
        },
        OffsetEntry {
            field_id: 4,
            offset: { offset += 8; offset },
            field_type: FieldType::Uint8 as u16,
            size: 1,
        },
        OffsetEntry {
            field_id: 10,
            offset: 0,
            field_type: FieldType::String as u16,
            size: max_var_size as u16,
        },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(&user));

    let mut var_data = vec![0u8; max_var_size as usize];
    var_data[0..name.len()].copy_from_slice(name.as_bytes());
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer)?;

    let id: &u64 = view.get_field(1)?;
    let age: &u32 = view.get_field(2)?;
    let score: &f64 = view.get_field(3)?;
    let active: &u8 = view.get_field(4)?;
    let name_str = view.get_string(10)?;

    // Copy values to avoid alignment issues
    let id_val = *id;
    let age_val = *age;
    let score_val = *score;
    let active_val = *active;
    let user_id = user.id;
    let user_age = user.age;
    let user_score = user.score;
    let user_active = user.active;

    assert_eq!(id_val, user_id);
    assert_eq!(age_val, user_age);
    assert_eq!(score_val, user_score);
    assert_eq!(active_val, user_active);
    assert_eq!(name_str, name);

    println!("│ Mixed fields: ID={}, Age={}, Score={}, Active={}, Name='{}'", 
             id, age, score, *active != 0, name_str);
    Ok(())
}

fn test_error_handling() -> Result<()> {
    // Test invalid magic
    let invalid_buffer = vec![0u8; 100];
    match BinaryView::view(&invalid_buffer) {
        Err(SerializationError::InvalidMagic { .. }) => {
            println!("│ ✓ InvalidMagic error caught");
        }
        _ => return Err(SerializationError::InvalidMagic { expected: 0, found: 0 }),
    }

    // Test field not found
    let buffer = serialize_user_data(&UserData { id: 1, age: 1, score: 1.0, active: 1 })?;
    let view = BinaryView::view(&buffer)?;
    match view.get_field::<u32>(999) {
        Err(SerializationError::FieldNotFound { .. }) => {
            println!("│ ✓ FieldNotFound error caught");
        }
        _ => return Err(SerializationError::FieldNotFound { field_id: 999 }),
    }

    // Test buffer too small
    let small_buffer = vec![0u8; 10];
    match BinaryView::view(&small_buffer) {
        Err(SerializationError::BufferTooSmall { .. }) => {
            println!("│ ✓ BufferTooSmall error caught");
        }
        _ => return Err(SerializationError::BufferTooSmall { needed: 0, have: 0 }),
    }

    // Test field size mismatch
    let mut buffer_mut = serialize_user_data(&UserData { id: 1, age: 1, score: 1.0, active: 1 })?;
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
    let wrong_value = 0u16; // Should be u32
    match view_mut.modify_field(2, &wrong_value) {
        Err(SerializationError::FieldSizeMismatch { .. }) => {
            println!("│ ✓ FieldSizeMismatch error caught");
        }
        _ => return Err(SerializationError::FieldSizeMismatch { expected: 0, got: 0 }),
    }

    Ok(())
}

fn test_format_validation() -> Result<()> {
    let user = UserData {
        id: 77777,
        age: 28,
        score: 85.5,
        active: 1,
    };

    let buffer = serialize_user_data(&user)?;
    let _view = BinaryView::view(&buffer)?;

    // Read header directly from buffer (since header field is private)
    use bisere::format::{FormatHeader, HEADER_SIZE, MAGIC, VERSION};
    let header = bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);

    // Copy header fields to avoid alignment issues with packed struct
    let magic = header.magic;
    let version = header.version;
    let header_size_val = header.header_size;
    let offset_table_size_val = header.offset_table_size;
    let total_size = header.total_size();

    // Verify header fields
    assert_eq!(magic, MAGIC, "Magic number mismatch");
    assert_eq!(version, VERSION, "Version mismatch");
    assert_eq!(header_size_val, HEADER_SIZE as u32, "Header size mismatch");

    println!("│ Magic: {:#x}", magic);
    println!("│ Version: {}", version);
    println!("│ Total size: {} bytes", total_size);
    println!("│ Offset table size: {} bytes ({} entries)", 
             offset_table_size_val,
             offset_table_size_val as usize / std::mem::size_of::<OffsetEntry>());

    Ok(())
}

fn test_buffer_layout() -> Result<()> {
    let user = UserData {
        id: 88888,
        age: 40,
        score: 99.9,
        active: 1,
    };

    let buffer = serialize_user_data(&user)?;
    let _view = BinaryView::view(&buffer)?;

    // Read header directly from buffer (since header field is private)
    use bisere::format::{FormatHeader, HEADER_SIZE};
    let header = bytemuck::from_bytes::<FormatHeader>(&buffer[0..HEADER_SIZE]);

    // Copy header fields to avoid alignment issues with packed struct
    let header_size = header.header_size as usize;
    let offset_table_size = header.offset_table_size as usize;
    let data_size = header.data_size as usize;
    let var_size = header.var_size as usize;
    let expected_size = header.total_size();

    assert_eq!(buffer.len(), expected_size, "Buffer size mismatch");

    println!("│ Header size: {} bytes", header_size);
    println!("│ Offset table size: {} bytes", offset_table_size);
    println!("│ Data section size: {} bytes", data_size);
    println!("│ Variable section size: {} bytes", var_size);
    println!("│ Total: {} bytes", expected_size);

    // Verify section offsets
    let data_offset = header.data_section_offset();
    let var_offset = header.var_section_offset();

    assert_eq!(data_offset, header_size + offset_table_size, "Data offset incorrect");
    assert_eq!(var_offset, data_offset + data_size, "Var offset incorrect");

    println!("│ Data section offset: {}", data_offset);
    println!("│ Variable section offset: {}", var_offset);

    Ok(())
}

fn test_all_integer_types() -> Result<()> {
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
    let view = BinaryView::view(&buffer)?;

    let i8_val = *view.get_field::<i8>(1)?;
    let i16_val = *view.get_field::<i16>(2)?;
    let i32_val = *view.get_field::<i32>(3)?;
    let i64_val = *view.get_field::<i64>(4)?;
    let u8_val = *view.get_field::<u8>(5)?;
    let u16_val = *view.get_field::<u16>(6)?;
    let u32_val = *view.get_field::<u32>(7)?;
    let u64_val = *view.get_field::<u64>(8)?;

    assert_eq!(i8_val, -128);
    assert_eq!(i16_val, -32768);
    assert_eq!(i32_val, -2147483648);
    assert_eq!(i64_val, -9223372036854775808);
    assert_eq!(u8_val, 255);
    assert_eq!(u16_val, 65535);
    assert_eq!(u32_val, 4294967295);
    assert_eq!(u64_val, 18446744073709551615);

    println!("│ All integer types verified: i8={}, i16={}, i32={}, i64={}, u8={}, u16={}, u32={}, u64={}", 
             i8_val, i16_val, i32_val, i64_val, u8_val, u16_val, u32_val, u64_val);
    Ok(())
}

fn test_edge_case_values() -> Result<()> {
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
    let view = BinaryView::view(&buffer)?;

    let zero_u64 = *view.get_field::<u64>(1)?;
    let max_u64 = *view.get_field::<u64>(2)?;
    let min_i64 = *view.get_field::<i64>(3)?;
    let zero_f64 = *view.get_field::<f64>(4)?;
    let neg_f64 = *view.get_field::<f64>(5)?;

    assert_eq!(zero_u64, 0);
    assert_eq!(max_u64, u64::MAX);
    assert_eq!(min_i64, i64::MIN);
    assert_eq!(zero_f64, 0.0);
    assert!((neg_f64 - (-123.456)).abs() < 0.0001);

    println!("│ Edge cases: zero={}, max={}, min={}, zero_f64={}, neg_f64={}", 
             zero_u64, max_u64, min_i64, zero_f64, neg_f64);
    Ok(())
}

fn test_multiple_strings() -> Result<()> {
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
    let view = BinaryView::view(&buffer)?;

    let s1 = view.get_string(10)?;
    let s2 = view.get_string(20)?;
    let s3 = view.get_string(30)?;

    assert_eq!(s1, "First");
    assert_eq!(s2, "Second");
    assert_eq!(s3, "Third");

    println!("│ Multiple strings: '{}', '{}', '{}'", s1, s2, s3);
    Ok(())
}

fn test_multiple_blobs() -> Result<()> {
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
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer)?;

    let blob1 = b"Blob 1 data";
    let blob2 = b"Blob 2 data longer";
    let blob3 = b"Blob 3 data even longer";

    view_mut.modify_blob(11, blob1)?;
    view_mut.modify_blob(22, blob2)?;
    view_mut.modify_blob(33, blob3)?;

    let view = BinaryView::view(&buffer)?;
    assert_eq!(&view.get_blob(11)?[..blob1.len()], blob1);
    assert_eq!(&view.get_blob(22)?[..blob2.len()], blob2);
    assert_eq!(&view.get_blob(33)?[..blob3.len()], blob3);

    println!("│ Multiple blobs: {} bytes, {} bytes, {} bytes", 
             blob1.len(), blob2.len(), blob3.len());
    Ok(())
}

fn test_empty_strings_blobs() -> Result<()> {
    // Test empty string
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
    let view = BinaryView::view(&buffer)?;
    assert_eq!(view.get_string(10)?, "");

    // Test empty blob
    let mut serializer2 = BinarySerializer::new();
    let header2 = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        100,
    );
    serializer2.write_header(header2);

    let entries2 = vec![OffsetEntry {
        field_id: 20,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: 100,
    }];
    serializer2.write_offset_table(&entries2);
    serializer2.write_data(&[]);
    serializer2.write_var_data(&vec![0u8; 100]);

    let buffer2 = serializer2.into_buffer();
    let view2 = BinaryView::view(&buffer2)?;
    let blob = view2.get_blob(20)?;
    assert_eq!(blob.len(), 100);
    assert!(blob.iter().all(|&b| b == 0));

    println!("│ Empty string: '{}' (length {})", view.get_string(10)?, view.get_string(10)?.len());
    println!("│ Empty blob: {} bytes (all zeros)", blob.len());
    Ok(())
}

fn test_unicode_strings() -> Result<()> {
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
    let view = BinaryView::view(&buffer)?;
    let retrieved = view.get_string(10)?;
    assert_eq!(retrieved, unicode_str);

    println!("│ Unicode string: '{}' ({} bytes)", retrieved, retrieved.len());
    Ok(())
}

fn test_non_sequential_field_ids() -> Result<()> {
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = 4 + 8 + 4 + 8;
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
    let view = BinaryView::view(&buffer)?;

    let v100 = *view.get_field::<u32>(100)?;
    let v50 = *view.get_field::<u64>(50)?;
    let v200 = *view.get_field::<u32>(200)?;
    let v1 = *view.get_field::<u64>(1)?;

    assert_eq!(v100, 100);
    assert_eq!(v50, 200);
    assert_eq!(v200, 300);
    assert_eq!(v1, 400);

    println!("│ Non-sequential IDs: field 100={}, field 50={}, field 200={}, field 1={}", 
             v100, v50, v200, v1);
    Ok(())
}

fn test_multiple_modifications() -> Result<()> {
    let mut buffer = serialize_user_data(&UserData { id: 1, age: 20, score: 50.0, active: 1 })?;
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer)?;

    // Modify all fields multiple times
    for i in 0..10 {
        let new_age = 20 + i as u32;
        view_mut.modify_field(2, &new_age)?;
    }

    let view = BinaryView::view(&buffer)?;
    let final_age = *view.get_field::<u32>(2)?;
    assert_eq!(final_age, 29);

    println!("│ Multiple modifications: final age after 10 changes = {}", final_age);
    Ok(())
}

fn test_many_fields() -> Result<()> {
    const NUM_FIELDS: usize = 50;
    let mut serializer = BinarySerializer::new();
    let offset_table_size = (NUM_FIELDS * std::mem::size_of::<OffsetEntry>()) as u32;
    let data_size = (NUM_FIELDS * 4) as u32;
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
    let view = BinaryView::view(&buffer)?;

    let mut all_correct = true;
    for i in 0..NUM_FIELDS {
        let value = *view.get_field::<u32>(i as u32)?;
        if value != (i * 100) as u32 {
            all_correct = false;
            break;
        }
    }
    assert!(all_correct);

    println!("│ Many fields: {} fields, all values correct", NUM_FIELDS);
    Ok(())
}

fn test_string_boundary_conditions() -> Result<()> {
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
        size: 10,
    }];
    serializer.write_offset_table(&entries);
    serializer.write_data(&[]);

    let mut var_data = vec![0u8; 10];
    var_data[0..9].copy_from_slice(b"123456789");
    serializer.write_var_data(&var_data);

    let buffer = serializer.into_buffer();
    let view = BinaryView::view(&buffer)?;
    let s = view.get_string(10)?;
    assert_eq!(s, "123456789");

    println!("│ Boundary condition: string of length {} in {} byte field", s.len(), 10);
    Ok(())
}

// Helper function to serialize UserData
fn serialize_user_data(user: &UserData) -> Result<Vec<u8>> {
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<UserData>() as u32;
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
            offset: { offset += 8; offset },
            field_type: FieldType::Uint32 as u16,
            size: 4,
        },
        OffsetEntry {
            field_id: 3,
            offset: { offset += 4; offset },
            field_type: FieldType::Float64 as u16,
            size: 8,
        },
        OffsetEntry {
            field_id: 4,
            offset: { offset += 8; offset },
            field_type: FieldType::Uint8 as u16,
            size: 1,
        },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(user));
    serializer.write_var_data(&vec![0u8; var_size as usize]);

    Ok(serializer.into_buffer())
}

