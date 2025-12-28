use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use bisere::*;
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

// Test data structure for biSere
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct UserData {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

// Test data structure for Serde-based formats
#[derive(Serialize, Deserialize, Clone)]
struct UserDataSerde {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}

// Helper to serialize with biSere
fn bisere_serialize(data: &UserData) -> Vec<u8> {
    let mut serializer = BinarySerializer::new();
    let offset_table_size = 4 * std::mem::size_of::<OffsetEntry>() as u32;
    let data_size = std::mem::size_of::<UserData>() as u32;
    let var_size = 0;
    
    let header = FormatHeader::new(offset_table_size, data_size, var_size);
    serializer.write_header(header);
    
    let mut offset = 0u32;
    let entries = vec![
        OffsetEntry { field_id: 1, offset, field_type: FieldType::Uint64 as u16, size: 8 },
        OffsetEntry { field_id: 2, offset: { offset += 8; offset }, field_type: FieldType::Uint32 as u16, size: 4 },
        OffsetEntry { field_id: 3, offset: { offset += 4; offset }, field_type: FieldType::Float64 as u16, size: 8 },
        OffsetEntry { field_id: 4, offset: { offset += 8; offset }, field_type: FieldType::Uint8 as u16, size: 1 },
    ];
    serializer.write_offset_table(&entries);
    serializer.write_data(bytemuck::bytes_of(data));
    serializer.write_var_data(&[]);
    serializer.into_buffer()
}

fn bisere_deserialize(buffer: &[u8]) -> (u64, u32, f64, u8) {
    let view = BinaryView::view(buffer).unwrap();
    let id = *view.get_field::<u64>(1).unwrap();
    let age = *view.get_field::<u32>(2).unwrap();
    let score = *view.get_field::<f64>(3).unwrap();
    let active = *view.get_field::<u8>(4).unwrap();
    (id, age, score, active)
}

fn bincode_serialize(data: &UserDataSerde) -> Vec<u8> {
    bincode::serialize(data).unwrap()
}

fn bincode_deserialize(buffer: &[u8]) -> UserDataSerde {
    bincode::deserialize(buffer).unwrap()
}

fn postcard_serialize(data: &UserDataSerde) -> Vec<u8> {
    postcard::to_allocvec(data).unwrap()
}

fn postcard_deserialize(buffer: &[u8]) -> UserDataSerde {
    postcard::from_bytes(buffer).unwrap()
}

fn messagepack_serialize(data: &UserDataSerde) -> Vec<u8> {
    rmp_serde::to_vec(data).unwrap()
}

fn messagepack_deserialize(buffer: &[u8]) -> UserDataSerde {
    rmp_serde::from_slice(buffer).unwrap()
}

fn serde_json_serialize(data: &UserDataSerde) -> Vec<u8> {
    serde_json::to_vec(data).unwrap()
}

fn serde_json_deserialize(buffer: &[u8]) -> UserDataSerde {
    serde_json::from_slice(buffer).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let test_data = UserData {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1,
    };
    
    let test_data_serde = UserDataSerde {
        id: 12345,
        age: 30,
        score: 95.5,
        active: 1,
    };

    // Serialization benchmarks
    let mut group = c.benchmark_group("serialize");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("bisere", |b| {
        b.iter(|| bisere_serialize(black_box(&test_data)))
    });
    
    group.bench_function("bincode", |b| {
        b.iter(|| bincode_serialize(black_box(&test_data_serde)))
    });
    
    group.bench_function("postcard", |b| {
        b.iter(|| postcard_serialize(black_box(&test_data_serde)))
    });
    
    group.bench_function("messagepack", |b| {
        b.iter(|| messagepack_serialize(black_box(&test_data_serde)))
    });
    
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json_serialize(black_box(&test_data_serde)))
    });
    
    group.finish();

    // Deserialization benchmarks
    let bisere_buf = bisere_serialize(&test_data);
    let bincode_buf = bincode_serialize(&test_data_serde);
    let postcard_buf = postcard_serialize(&test_data_serde);
    let msgpack_buf = messagepack_serialize(&test_data_serde);
    let json_buf = serde_json_serialize(&test_data_serde);

    let mut group = c.benchmark_group("deserialize");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("bisere", |b| {
        b.iter(|| bisere_deserialize(black_box(&bisere_buf)))
    });
    
    group.bench_function("bincode", |b| {
        b.iter(|| bincode_deserialize(black_box(&bincode_buf)))
    });
    
    group.bench_function("postcard", |b| {
        b.iter(|| postcard_deserialize(black_box(&postcard_buf)))
    });
    
    group.bench_function("messagepack", |b| {
        b.iter(|| messagepack_deserialize(black_box(&msgpack_buf)))
    });
    
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json_deserialize(black_box(&json_buf)))
    });
    
    group.finish();

    // Round-trip benchmarks
    let mut group = c.benchmark_group("roundtrip");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("bisere", |b| {
        b.iter(|| {
            let buf = bisere_serialize(black_box(&test_data));
            bisere_deserialize(black_box(&buf))
        })
    });
    
    group.bench_function("bincode", |b| {
        b.iter(|| {
            let buf = bincode_serialize(black_box(&test_data_serde));
            bincode_deserialize(black_box(&buf))
        })
    });
    
    group.bench_function("postcard", |b| {
        b.iter(|| {
            let buf = postcard_serialize(black_box(&test_data_serde));
            postcard_deserialize(black_box(&buf))
        })
    });
    
    group.bench_function("messagepack", |b| {
        b.iter(|| {
            let buf = messagepack_serialize(black_box(&test_data_serde));
            messagepack_deserialize(black_box(&buf))
        })
    });
    
    group.finish();

    // Field access benchmarks (zero-copy advantage)
    let bisere_buf = bisere_serialize(&test_data);
    let mut group = c.benchmark_group("field_access");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("bisere_zero_copy", |b| {
        let view = BinaryView::view(&bisere_buf).unwrap();
        b.iter(|| {
            black_box(*view.get_field::<u64>(1).unwrap());
            black_box(*view.get_field::<u32>(2).unwrap());
            black_box(*view.get_field::<f64>(3).unwrap());
        })
    });
    
    group.bench_function("bincode_full_deserialize", |b| {
        b.iter(|| {
            let data: UserDataSerde = bincode_deserialize(black_box(&bincode_buf));
            black_box(data.id);
            black_box(data.age);
            black_box(data.score);
        })
    });
    
    group.bench_function("postcard_full_deserialize", |b| {
        b.iter(|| {
            let data: UserDataSerde = postcard_deserialize(black_box(&postcard_buf));
            black_box(data.id);
            black_box(data.age);
            black_box(data.score);
        })
    });
    
    group.finish();

    // In-place modification benchmark
    let mut bisere_buf = bisere_serialize(&test_data);
    let mut group = c.benchmark_group("inplace_modification");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("bisere_inplace", |b| {
        b.iter(|| {
            let mut view = BinaryViewMut::view_mut(black_box(&mut bisere_buf)).unwrap();
            let new_age = 31u32;
            view.modify_field(2, &new_age).unwrap();
        })
    });
    
    group.bench_function("bincode_re_serialize", |b| {
        b.iter(|| {
            let mut data: UserDataSerde = bincode_deserialize(black_box(&bincode_buf));
            data.age = 31;
            black_box(bincode_serialize(&data));
        })
    });
    
    group.bench_function("postcard_re_serialize", |b| {
        b.iter(|| {
            let mut data: UserDataSerde = postcard_deserialize(black_box(&postcard_buf));
            data.age = 31;
            black_box(postcard_serialize(&data));
        })
    });
    
    group.finish();

    // Buffer size comparison
    println!("\n=== Buffer Size Comparison ===");
    println!("biSere:      {} bytes", bisere_buf.len());
    println!("bincode:     {} bytes", bincode_buf.len());
    println!("postcard:    {} bytes", postcard_buf.len());
    println!("messagepack: {} bytes", msgpack_buf.len());
    println!("serde_json:  {} bytes", json_buf.len());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

