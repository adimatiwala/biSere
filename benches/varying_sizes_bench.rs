use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use bisere::*;
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TestStruct {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

#[derive(Serialize, Deserialize, Clone)]
struct TestStructSerde {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

fn bisere_serialize_many(data: &[TestStruct]) -> Vec<u8> {
    let mut serializer = BinarySerializer::new();
    let num_fields = 4;
    let offset_table_size = (data.len() * num_fields * std::mem::size_of::<OffsetEntry>()) as u32;
    let data_size = (data.len() * std::mem::size_of::<TestStruct>()) as u32;
    let var_size = 0;
    
    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);
    
    // Create offset entries for each struct
    let mut entries = Vec::new();
    for (idx, _) in data.iter().enumerate() {
        let base_offset = (idx * std::mem::size_of::<TestStruct>()) as u32;
        let mut offset = base_offset;
        entries.push(OffsetEntry { 
            field_id: (idx * 4 + 1) as u32, 
            offset, 
            field_type: FieldType::Uint64 as u16, 
            size: 8 
        });
        entries.push(OffsetEntry { 
            field_id: (idx * 4 + 2) as u32, 
            offset: { offset += 8; offset }, 
            field_type: FieldType::Uint32 as u16, 
            size: 4 
        });
        entries.push(OffsetEntry { 
            field_id: (idx * 4 + 3) as u32, 
            offset: { offset += 4; offset }, 
            field_type: FieldType::Float64 as u16, 
            size: 8 
        });
        entries.push(OffsetEntry { 
            field_id: (idx * 4 + 4) as u32, 
            offset: { offset += 8; offset }, 
            field_type: FieldType::Uint8 as u16, 
            size: 1 
        });
    }
    
    serializer.write_offset_table(&entries);
    
    // Serialize all structs
    let mut all_data = Vec::new();
    for item in data {
        all_data.extend_from_slice(bytemuck::bytes_of(item));
    }
    serializer.write_data(&all_data);
    serializer.write_var_data(&[]);
    serializer.into_buffer()
}

fn bincode_serialize_many(data: &[TestStructSerde]) -> Vec<u8> {
    bincode::serialize(data).unwrap()
}

fn postcard_serialize_many(data: &[TestStructSerde]) -> Vec<u8> {
    postcard::to_allocvec(data).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    // Test with different data sizes
    let sizes = vec![1, 10, 100, 1000];
    
    let mut group = c.benchmark_group("serialize_varying_sizes");
    
    for size in sizes {
        let data_vec: Vec<TestStruct> = (0..size).map(|i| TestStruct {
            id: i as u64,
            age: (i % 100) as u32,
            score: (i as f64) * 0.1,
            active: (i % 2) as u8,
        }).collect();
        
        let data_vec_serde: Vec<TestStructSerde> = data_vec.iter().map(|d| TestStructSerde {
            id: d.id,
            age: d.age,
            score: d.score,
            active: d.active,
        }).collect();
        
        group.throughput(Throughput::Elements(size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("bisere", size),
            &data_vec,
            |b, data| {
                b.iter(|| bisere_serialize_many(black_box(data)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("bincode", size),
            &data_vec_serde,
            |b, data| {
                b.iter(|| bincode_serialize_many(black_box(data)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("postcard", size),
            &data_vec_serde,
            |b, data| {
                b.iter(|| postcard_serialize_many(black_box(data)))
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

