use bisere::*;
use bytemuck::{Pod, Zeroable};

// Example data structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UserData {
    id: u64,
    age: u32,
    score: f64,
    active: u8, // Using u8 instead of bool since bool is not Pod
}

fn main() -> Result<()> {
    println!("=== Binary Serialization Format Example ===\n");
    
    // 1. Create sample data
    let user = UserData {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1, // 1 for true
    };
    
    // Copy fields to avoid unaligned reference issues with packed structs
    let id = user.id;
    let age = user.age;
    let score = user.score;
    let active = user.active != 0;
    println!("Original data: ID={}, Age={}, Score={}, Active={}", 
             id, age, score, active);
    
    // 2. Serialize
    let mut serializer = BinarySerializer::new();
    
    // Build header
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32; // 4 fields
    let data_size = std::mem::size_of::<UserData>() as u32;
    let var_size = 256; // Space for variable-length data
    
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
    
    // Write data
    let user_bytes = bytemuck::bytes_of(&user);
    serializer.write_data(user_bytes);
    
    // Write variable section (empty for now)
    serializer.write_var_data(&vec![0u8; var_size as usize]);
    
    let buffer = serializer.into_buffer();
    println!("\nSerialized {} bytes", buffer.len());
    
    // 3. Deserialize (zero-copy)
    let view = BinaryView::view(&buffer)?;
    
    let id: &u64 = view.get_field(1)?;
    let age: &u32 = view.get_field(2)?;
    let score: &f64 = view.get_field(3)?;
    let active: &u8 = view.get_field(4)?;
    
    println!("\nDeserialized (zero-copy) - ID: {}, Age: {}, Score: {}, Active: {}", 
             id, age, score, *active != 0);
    
    // 4. In-place modification
    let mut buffer_mut = buffer.clone();
    let mut view_mut = BinaryViewMut::view_mut(&mut buffer_mut)?;
    
    // Modify age
    let new_age = 31u32;
    view_mut.modify_field(2, &new_age)?;
    println!("\nModified age in-place to {}", new_age);
    
    // Verify modification
    let view2 = BinaryView::view(&buffer_mut)?;
    let modified_age: &u32 = view2.get_field(2)?;
    println!("Verified modified age: {}", modified_age);
    
    // 5. String example
    println!("\n=== String Field Example ===");
    let mut serializer2 = BinarySerializer::new();
    let header2 = FormatHeader::new(
        1 * std::mem::size_of::<OffsetEntry>() as u32,
        0,
        256,
    );
    serializer2.write_header(header2);
    
    let string_entries = vec![OffsetEntry {
        field_id: 10,
        offset: 0,
        field_type: FieldType::String as u16,
        size: 256,
    }];
    serializer2.write_offset_table(&string_entries);
    serializer2.write_data(&[]);
    
    let mut var_data = vec![0u8; 256];
    var_data[0..5].copy_from_slice(b"Hello");
    serializer2.write_var_data(&var_data);
    
    let buffer3 = serializer2.into_buffer();
    let view3 = BinaryView::view(&buffer3)?;
    let name = view3.get_string(10)?;
    println!("String field: '{}'", name);
    
    // Modify string
    let mut buffer3_mut = buffer3.clone();
    let mut view3_mut = BinaryViewMut::view_mut(&mut buffer3_mut)?;
    view3_mut.modify_string(10, "World")?;
    
    let view3_updated = BinaryView::view(&buffer3_mut)?;
    let name_updated = view3_updated.get_string(10)?;
    println!("Modified string: '{}'", name_updated);
    
    // 6. Error handling examples
    println!("\n=== Error Handling Examples ===");
    
    // Test invalid magic
    let invalid_buffer = vec![0u8; 100];
    match BinaryView::view(&invalid_buffer) {
        Err(SerializationError::InvalidMagic { expected, found }) => {
            println!("Caught InvalidMagic error: expected {:#x}, found {:#x}", expected, found);
        }
        _ => println!("Unexpected result"),
    }
    
    // Test field not found
    match view.get_field::<u32>(999) {
        Err(SerializationError::FieldNotFound { field_id }) => {
            println!("Caught FieldNotFound error for field_id: {}", field_id);
        }
        _ => println!("Unexpected result"),
    }
    
    // Test buffer too small
    let small_buffer = vec![0u8; 10];
    match BinaryView::view(&small_buffer) {
        Err(SerializationError::BufferTooSmall { needed, have }) => {
            println!("Caught BufferTooSmall error: needed {}, have {}", needed, have);
        }
        _ => println!("Unexpected result"),
    }
    
    println!("\n=== Example Complete ===");
    Ok(())
}
