use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use bisere::*;
use bisere::format::{FormatHeader, OffsetEntry, HEADER_SIZE};
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};
use std::ptr;

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

// Const header/table for UserData layout → one alloc + 4 copies, no per-call setup
const BISERE_HEADER: FormatHeader = FormatHeader {
    magic: bisere::format::MAGIC,
    version: bisere::format::VERSION,
    header_size: HEADER_SIZE as u32,
    offset_table_size: 4 * std::mem::size_of::<OffsetEntry>() as u32,
    data_size: std::mem::size_of::<UserData>() as u32,
    var_size: 0,
    checksum: 0,
    reserved: [0; 6],
};
const BISERE_ENTRIES: [OffsetEntry; 4] = [
    OffsetEntry { field_id: 1, offset: 0, field_type: FieldType::Uint64 as u16, size: 8 },
    OffsetEntry { field_id: 2, offset: 8, field_type: FieldType::Uint32 as u16, size: 4 },
    OffsetEntry { field_id: 3, offset: 12, field_type: FieldType::Float64 as u16, size: 8 },
    OffsetEntry { field_id: 4, offset: 20, field_type: FieldType::Uint8 as u16, size: 1 },
];

fn bisere_serialize(data: &UserData) -> Vec<u8> {
    bisere::serialize_to_buffer(&BISERE_HEADER, &BISERE_ENTRIES, bytemuck::bytes_of(data), &[])
}

// Layout-specific deserialize: data section at 128, offsets 0,8,12,20. No view, no table lookup.
fn bisere_deserialize(buffer: &[u8]) -> (u64, u32, f64, u8) {
    const DATA_OFF: usize = HEADER_SIZE + 4 * std::mem::size_of::<OffsetEntry>();
    unsafe {
        let p = buffer.as_ptr().add(DATA_OFF);
        (
            ptr::read_unaligned(p as *const u64),
            ptr::read_unaligned(p.add(8) as *const u32),
            ptr::read_unaligned(p.add(12) as *const f64),
            ptr::read_unaligned(p.add(20) as *const u8),
        )
    }
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
            black_box(view.get_field_unaligned::<f64>(3).unwrap());
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

    // BinarySerializer::reserve — same write_header/write_offset_table/
    // write_data/write_var_data sequence, with vs without a leading
    // reserve() call sized to the total buffer.
    let mut group = c.benchmark_group("serializer_reserve");
    group.throughput(Throughput::Elements(1));

    group.bench_function("bisere_without_reserve", |b| {
        b.iter(|| {
            let mut serializer = BinarySerializer::new();
            serializer.write_header(black_box(BISERE_HEADER));
            serializer.write_offset_table(&BISERE_ENTRIES);
            serializer.write_data(bytemuck::bytes_of(&test_data));
            serializer.write_var_data(&[]);
            black_box(serializer.into_buffer())
        })
    });

    group.bench_function("bisere_with_reserve", |b| {
        b.iter(|| {
            let mut serializer = BinarySerializer::new();
            serializer.reserve(BISERE_HEADER.total_size());
            serializer.write_header(black_box(BISERE_HEADER));
            serializer.write_offset_table(&BISERE_ENTRIES);
            serializer.write_data(bytemuck::bytes_of(&test_data));
            serializer.write_var_data(&[]);
            black_box(serializer.into_buffer())
        })
    });

    group.finish();

    // BinaryView construction — full validation (view) vs skipping
    // magic/version/header_size/total_size checks (view_unchecked), on a
    // buffer already known to be well-formed.
    let mut group = c.benchmark_group("view_construction");
    group.throughput(Throughput::Elements(1));

    group.bench_function("bisere_view", |b| {
        b.iter(|| {
            black_box(BinaryView::view(black_box(&bisere_buf)).unwrap())
        })
    });

    group.bench_function("bisere_view_unchecked", |b| {
        b.iter(|| {
            black_box(BinaryView::view_unchecked(black_box(&bisere_buf)).unwrap())
        })
    });

    group.finish();

    // Variable-length in-place modification: modify_string / modify_blob
    // write the new value then zero only the trailing remainder, instead
    // of zeroing the whole field then overwriting its front — this
    // benchmark uses a near-full-length replacement value each time,
    // which is the case that shows the largest difference (every byte in
    // the field's span was being written twice before).
    const VAR_FIELD_SIZE: usize = 64;
    let near_full_string = "x".repeat(VAR_FIELD_SIZE - 1); // leaves room for the NUL
    let near_full_blob = vec![0xABu8; VAR_FIELD_SIZE - 4];

    let mut string_serializer = BinarySerializer::new();
    let string_header = FormatHeader::new(
        std::mem::size_of::<OffsetEntry>() as u32,
        0,
        VAR_FIELD_SIZE as u32,
    );
    string_serializer.write_header(string_header);
    string_serializer.write_offset_table(&[OffsetEntry {
        field_id: 1,
        offset: 0,
        field_type: FieldType::String as u16,
        size: VAR_FIELD_SIZE as u16,
    }]);
    string_serializer.write_data(&[]);
    string_serializer.write_var_data(&vec![0u8; VAR_FIELD_SIZE]);
    let mut string_buf = string_serializer.into_buffer();

    let mut blob_serializer = BinarySerializer::new();
    let blob_header = FormatHeader::new(
        std::mem::size_of::<OffsetEntry>() as u32,
        0,
        VAR_FIELD_SIZE as u32,
    );
    blob_serializer.write_header(blob_header);
    blob_serializer.write_offset_table(&[OffsetEntry {
        field_id: 1,
        offset: 0,
        field_type: FieldType::Blob as u16,
        size: VAR_FIELD_SIZE as u16,
    }]);
    blob_serializer.write_data(&[]);
    blob_serializer.write_var_data(&vec![0u8; VAR_FIELD_SIZE]);
    let mut blob_buf = blob_serializer.into_buffer();

    let mut group = c.benchmark_group("modify_variable_length");
    group.throughput(Throughput::Elements(1));

    group.bench_function("bisere_modify_string", |b| {
        b.iter(|| {
            let mut view = BinaryViewMut::view_mut(black_box(&mut string_buf)).unwrap();
            view.modify_string(1, &near_full_string).unwrap();
        })
    });

    group.bench_function("bisere_modify_blob", |b| {
        b.iter(|| {
            let mut view = BinaryViewMut::view_mut(black_box(&mut blob_buf)).unwrap();
            view.modify_blob(1, &near_full_blob).unwrap();
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

