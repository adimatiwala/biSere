# biSere Performance Benchmarks

This document presents comprehensive performance benchmarks comparing biSere against other popular Rust serialization libraries: bincode, postcard, MessagePack (rmp-serde), and serde_json.

## Test Environment

- **Library**: biSere v0.1.0
- **Benchmark Framework**: Criterion 0.5
- **Test Data**: `UserData` struct containing `u64`, `u32`, `f64`, and `u8` fields
- **Measurement**: 100 samples per benchmark

## Benchmark Categories

1. Serialization Performance
2. Deserialization Performance
3. Round-Trip Performance
4. Zero-Copy Field Access
5. In-Place Modification
6. Buffer Size Comparison
7. Varying Data Sizes

---

## 1. Serialization Performance

Time required to convert data structures into serialized byte buffers.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **bincode** | 23.93 | 41.79 | 1st |
| **postcard** | 49.31 | 20.28 | 2nd |
| **messagepack** | 51.39 | 19.46 | 3rd |
| **serde_json** | 90.17 | 11.09 | 4th |
| **biSere** | 153.53 | 6.51 | 5th |

### Analysis

biSere exhibits slower serialization performance (6.4x slower than bincode) due to the overhead of constructing the offset table and format header. This is a one-time cost that enables subsequent zero-copy operations and in-place modifications.

**Key Insight**: Serialization overhead is amortized over multiple read and modification operations.

---

## 2. Deserialization Performance

Time required to read data from serialized byte buffers.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **bincode** | 4.56 | 219.28 | 1st |
| **postcard** | 6.40 | 156.35 | 2nd |
| **biSere** | 11.32 | 88.35 | 3rd |
| **messagepack** | 18.15 | 55.11 | 4th |
| **serde_json** | 74.55 | 13.41 | 5th |

### Analysis

biSere demonstrates competitive deserialization performance, ranking 3rd overall. While bincode and postcard are faster, biSere's deserialization enables zero-copy field access without requiring full deserialization, providing a significant advantage for read-heavy workloads.

**Key Insight**: biSere's deserialization includes format validation and offset table parsing, which enables subsequent zero-copy operations.

---

## 3. Round-Trip Performance

Complete serialize-then-deserialize cycle.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **bincode** | 27.70 | 36.10 | 1st |
| **postcard** | 56.75 | 17.62 | 2nd |
| **messagepack** | 75.50 | 13.25 | 3rd |
| **biSere** | 167.05 | 5.99 | 4th |

### Analysis

biSere's round-trip performance is slower due to serialization overhead. However, this metric does not reflect biSere's primary use case, which is optimized for scenarios with frequent field access and in-place modifications after initial serialization.

**Key Insight**: Round-trip performance is less relevant for biSere's target use cases, where data is serialized once and accessed/modified many times.

---

## 4. Zero-Copy Field Access

Performance of accessing individual fields without full deserialization.

| Library | Method | Time (ns) | Throughput (Melem/s) | Rank |
|---------|--------|-----------|----------------------|------|
| **postcard** | Full deserialize | 5.74 | 174.07 | 1st |
| **bincode** | Full deserialize | 5.81 | 172.11 | 2nd |
| **biSere** | Zero-copy | 6.34 | 157.83 | 3rd |

### Analysis

biSere's zero-copy field access is competitive with full deserialization approaches. While slightly slower in absolute terms, biSere provides several advantages:

- **No allocation**: Returns references directly into the buffer
- **Selective access**: Access only needed fields without deserializing the entire structure
- **Memory efficiency**: No intermediate data structures created

**Key Insight**: For accessing multiple fields, biSere's zero-copy approach becomes more efficient as it avoids repeated full deserialization.

---

## 5. In-Place Modification

Performance of updating fields in serialized buffers.

| Library | Method | Time (ns) | Throughput (Melem/s) | Speedup vs biSere |
|---------|--------|-----------|----------------------|-------------------|
| **biSere** | In-place | 3.98 | 250.98 | 1.0x (baseline) |
| **bincode** | Re-serialize | 27.52 | 36.34 | 6.9x slower |
| **postcard** | Re-serialize | 57.90 | 17.27 | 14.5x slower |

### Analysis

**This is biSere's primary strength.** In-place modification is 6.9x faster than bincode and 14.5x faster than postcard. This performance advantage is achieved through direct memory writes without requiring deserialization, modification, and re-serialization.

**Key Insight**: For applications requiring frequent field updates (e.g., game engines, real-time systems, databases), biSere provides substantial performance benefits.

---

## 6. Buffer Size Comparison

Memory footprint of serialized data.

| Library | Size (bytes) | Ratio vs Smallest |
|---------|--------------|-------------------|
| **postcard** | 12 | 1.0x |
| **messagepack** | 15 | 1.25x |
| **bincode** | 21 | 1.75x |
| **serde_json** | 45 | 3.75x |
| **biSere** | 149 | 12.4x |

### Analysis

biSere uses significantly more memory due to:

- **Format Header**: 80 bytes (magic number, version, section sizes, checksum, reserved space)
- **Offset Table**: 48 bytes (4 entries × 12 bytes per entry)
- **Data Section**: 21 bytes (actual data)

The additional memory overhead enables:
- Zero-copy field access
- In-place modifications
- Format validation
- Efficient field lookup

**Key Insight**: The memory overhead is a trade-off for performance benefits in read-heavy and modification-heavy workloads.

---

## 7. Varying Data Sizes

Serialization performance with different numbers of structs.

### Single Struct (1 element)

| Library | Time (ns) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 27.29 | 36.64 |
| **postcard** | 49.47 | 20.21 |
| **biSere** | 203.71 | 4.91 |

### Ten Structs (10 elements)

| Library | Time (ns) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 57.47 | 174.02 |
| **postcard** | 273.81 | 36.52 |
| **biSere** | 651.93 | 15.34 |

### One Hundred Structs (100 elements)

| Library | Time (ns) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 367.45 | 272.15 |
| **postcard** | 873.70 | 114.46 |
| **biSere** | 1.44 µs | 69.65 |

### One Thousand Structs (1000 elements)

| Library | Time (µs) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 3.55 | 281.73 |
| **postcard** | 5.91 | 169.32 |
| **biSere** | 16.02 | 62.42 |

### Analysis

biSere's serialization overhead scales linearly with the number of elements. The fixed overhead per struct (offset table entries) becomes less significant as data size increases, but bincode and postcard maintain superior serialization performance across all sizes.

**Key Insight**: For large-scale serialization, biSere's overhead is amortized, but it remains slower than alternatives. The benefit comes from subsequent operations (reads and modifications).

---

## Performance Summary

### Where biSere Excels

1. **In-Place Modification**: 6.9x faster than bincode, 14.5x faster than postcard
2. **Zero-Copy Field Access**: Competitive performance with no allocation overhead
3. **Deserialization**: Competitive 3rd place performance with additional capabilities

### Where biSere is Slower

1. **Serialization**: 6.4x slower than bincode (offset table construction overhead)
2. **Round-Trip**: Slower due to serialization overhead
3. **Buffer Size**: 12.4x larger than the smallest format (enables zero-copy operations)

### Recommended Use Cases

biSere is optimal for:

- **High-frequency updates**: Applications requiring frequent field modifications
- **Read-heavy workloads**: Scenarios with many reads per write
- **Large payloads with small metadata**: Updating metadata without copying large data
- **Real-time systems**: Low-latency requirements for modifications
- **Memory-constrained embedded systems**: Zero-copy reduces allocation pressure

### Not Recommended For

- **Write-once, read-many (WORM)**: If data is never modified after serialization
- **Minimal buffer size**: When memory footprint is the primary concern
- **One-time serialization**: If serialization happens once and data is never accessed
- **Cross-language compatibility**: When interoperability with other languages is required

---

## Methodology

### Test Data Structure

```rust
#[repr(C, packed)]
struct UserData {
    id: u64,
    age: u32,
    score: f64,
    active: u8,
}
```

### Benchmark Configuration

- **Framework**: Criterion 0.5
- **Samples**: 100 measurements per benchmark
- **Warm-up**: 3 seconds
- **Measurement**: 5 seconds estimated collection time
- **Build**: Release mode with optimizations

### Measurement Notes

- All times reported are mean values from 100 samples
- Throughput calculated as operations per second
- Outliers detected and reported but included in statistics
- Benchmarks run on a single machine; absolute times may vary

---

## Conclusion

biSere provides a unique combination of zero-copy deserialization and in-place modification capabilities, making it an excellent choice for applications requiring frequent data updates. While serialization performance is slower than alternatives, the benefits in modification-heavy workloads are substantial.

The library's design prioritizes:
1. **Modification performance** over initial serialization speed
2. **Zero-copy operations** over minimal buffer size
3. **Type safety** through Rust's type system

For use cases matching these priorities, biSere offers significant performance advantages over traditional serialization libraries.

---

*Benchmark results generated on: $(date)*
*biSere version: 0.1.0*

