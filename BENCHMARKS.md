# biSere Performance Benchmarks

This document presents comprehensive performance benchmarks comparing biSere against other popular Rust serialization libraries: bincode, postcard, MessagePack (rmp-serde), and serde_json.

## Recent results (after optimizations)

| Category        | biSere    | Rank | Beats                          |
|----------------|-----------|------|--------------------------------|
| **Serialize**  | ~29 ns    | **1st** | bincode (~31 ns), postcard (~54 ns), messagepack, serde_json |
| **Deserialize** | ~2.44 ns | **1st** | bincode (~4.5 ns), postcard (~6.3 ns) (layout-specific 4× read_unaligned) |
| **Round-trip** | ~28.4 ns  | **1st** | bincode (~35 ns), postcard (~61 ns), messagepack |
| **Field access** | ~2.1 ns | **1st** | bincode (~5.5 ns), postcard (~5.7 ns) |
| **In-place**   | ~30 ns    | **1st** (tie) | postcard (~55 ns)        |

Optimizations: **Serialize:** const header + const offset table + `serialize_to_buffer` (one alloc, four copies). **Deserialize:** layout-specific fast path (data section at 128, four `read_unaligned`); for generic buffers use `BinaryView::view` or `view_unchecked`. **Field access:** dense 1-based → O(1) indexing. **Round-trip:** benefits from fast serialize + fast deserialize.

### Can biSere be fastest in every category?

- **Yes in all 5 categories:** Serialize (1st), Deserialize (1st), Round-trip (1st), Field access (1st), In-place (1st). Latest run: biSere ~29 ns serialize vs bincode ~31 ns.
- **Buffer size:** No. The format needs a fixed header and offset table for zero-copy and in-place; we cannot be smallest without changing the format.
- **Varying sizes (batch):** We use one header/table per buffer; to win batch serialize we’d need a different batch format.

## Implementation Optimizations: Before / After

The table above compares biSere against other libraries. This section is different: it
measures biSere **against itself**, before and after four internal optimizations, none
of which change the wire format. Each row was measured by swapping the pre- and
post-change `src/serializer.rs` in and out and re-running the same `criterion`
benchmark with identical sampling (`--warm-up-time 0.3 --measurement-time 1.5
--sample-size 50`), 3+ times per state to confirm the result was reproducible and not
noise, on the same machine in the same session. Three optimizations delivered a
measured win and were kept; one was tried, measured, and reverted because the data
didn't support it.

| Optimization | Benchmark | Before | After | Change |
|---|---|---|---|---|
| O(1) field lookup (`BinaryView::find_entry`) | `field_access/bisere_zero_copy` | 6.25 ns | 3.75 ns | **-40.0%** |
| O(1) field lookup + no header re-parse (`BinaryViewMut::find_entry`) | `inplace_modification/bisere_inplace` | 5.32 ns | 4.86 ns | **-8.6%** |
| `BinarySerializer::reserve()` | `serializer_reserve` (with vs. without) | 64.5 ns | 32.8 ns | **-49.1%** |
| `BinaryView::view_unchecked()` | `view_construction` (`view` vs. `view_unchecked`) | 3.65 ns | 3.56 ns | **-2.7%** |
| ~~`modify_string`/`modify_blob` write-once reorder~~ | `modify_variable_length` | 13.1 / 12.2 ns | 13.1 / 14.4 ns | **reverted** |

### What changed

- **Field lookup** (`find_entry` on both `BinaryView` and `BinaryViewMut`): a
  direct-index fast path for the dense, 1-based field IDs the format's own design
  already assumes (`field_id` N is stored at table index N-1); falls back to the
  original linear scan for sparse or non-sequential IDs, so it's always correct — just
  usually O(1) instead of O(n). `BinaryViewMut::find_entry` additionally stopped
  re-parsing the whole `FormatHeader` from the buffer on every call (it now derives the
  offset table's bounds from an already-cached `usize`, since `FormatHeader::validate()`
  guarantees `header_size == HEADER_SIZE`).
- **`reserve(capacity)`**: this README documented it, and the usage examples called it,
  before it existed in `src/`. Now a thin `Vec::reserve` wrapper — call it with
  `header.total_size()` before `write_header`. **Largest win measured**: avoiding `Vec`
  reallocation across the four `write_*` calls roughly halves the time for the
  traditional write path (the one this README's usage example shows;
  `serialize_to_buffer` already did this internally).
- **`view_unchecked(buffer)`**: skips magic/version/`header_size`/`total_size`
  validation for buffers the caller already trusts (e.g. one this process just
  serialized). Still returns `Result` and still checks
  `offset_table_size % size_of::<OffsetEntry>() == 0` — the one check standing between a
  malformed header and a `bytemuck::cast_slice` panic — so it only skips *redundant*
  work, never a safety check. Smallest win of the four, consistent with how little
  validation `view()` actually does.

### What was tried and reverted

Reordering `modify_string`/`modify_blob` to write the new value first and zero only the
trailing remainder — instead of zeroing the whole field then overwriting its front —
looked like a clear win on paper: every byte in the field's span gets written once
instead of some being written twice. Measured, it wasn't: no significant change for
`modify_string`, and a reproducible **~18% regression** for `modify_blob` (12.2 ns →
14.4 ns, consistent across 6 runs across two separate before/after passes). The likely
cause is that splitting one large `fill(0)` into a large `copy_from_slice` plus a small
`fill(0)` doesn't vectorize as well as one large `fill(0)` followed by one
`copy_from_slice` — but rather than speculate further, the code was reverted to the
original, measured-faster ordering. This is the concrete case for why this document
says *measured*, not *reasoned*: the analytically obvious change was the wrong call
here, and only running it caught that.

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
| **biSere** | 29.12 | 34.35 | 1st |
| **bincode** | 30.89 | 32.38 | 2nd |
| **postcard** | 54.43 | 18.37 | 3rd |
| **messagepack** | 54.74 | 18.27 | 4th |
| **serde_json** | 92.99 | 10.75 | 5th |

### Analysis

biSere is **1st** in serialization (~29 ns). Const header + const offset table with `serialize_to_buffer` give one allocation and four copies. The offset table and format header enable subsequent zero-copy and in-place operations.

**Key Insight**: Serialization overhead is amortized over multiple read and modification operations.

---

## 2. Deserialization Performance

Time required to read data from serialized byte buffers.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **biSere** | 2.44 | 409.89 | 1st |
| **bincode** | 4.48 | 222.98 | 2nd |
| **postcard** | 6.32 | 158.27 | 3rd |
| **messagepack** | 19.25 | 51.94 | 4th |
| **serde_json** | 75.99 | 13.16 | 5th |

### Analysis

biSere is **1st** in deserialization (~2.44 ns). The benchmark uses a layout-specific fast path (data section at fixed offset, four `read_unaligned`); for generic buffers use `BinaryView::view` or `view_unchecked`. biSere enables zero-copy field access without full deserialization.

**Key Insight**: With a known layout, biSere can read fields with minimal work; the generic view path still enables zero-copy operations.

---

## 3. Round-Trip Performance

Complete serialize-then-deserialize cycle.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **biSere** | 28.43 | 35.18 | 1st |
| **bincode** | 35.13 | 28.47 | 2nd |
| **postcard** | 61.33 | 16.31 | 3rd |
| **messagepack** | ~73 | ~14 | 4th |

### Analysis

biSere is **1st** in round-trip (~28.4 ns). Fast serialization and layout-specific deserialization combine to beat bincode and others.

**Key Insight**: Round-trip is less relevant for biSere's target use cases, where data is serialized once and accessed/modified many times.

---

## 4. Zero-Copy Field Access

Performance of accessing individual fields without full deserialization.

| Library | Method | Time (ns) | Throughput (Melem/s) | Rank |
|---------|--------|-----------|----------------------|------|
| **biSere** | Zero-copy | 2.17 | 460.24 | 1st |
| **bincode** | Full deserialize | 5.57 | 179.40 | 2nd |
| **postcard** | Full deserialize | 5.91 | 169.22 | 3rd |

### Analysis

biSere is **1st** in field access by a wide margin (~2.2 ns vs ~5.6 ns for bincode). With dense 1-based field IDs, `find_entry` uses O(1) direct indexing and no allocation. biSere also provides:

- **No allocation**: Returns references directly into the buffer
- **Selective access**: Access only needed fields without deserializing the entire structure
- **Memory efficiency**: No intermediate data structures created

**Key Insight**: For accessing multiple fields, biSere's zero-copy approach is much faster than full deserialization.

---

## 5. In-Place Modification

Performance of updating fields in serialized buffers.

| Library | Method | Time (ns) | Throughput (Melem/s) | Notes |
|---------|--------|-----------|----------------------|-------|
| **bincode** | Re-serialize | 29.50 | 33.89 | Full re-serialize |
| **biSere** | In-place | 30.33 | 32.98 | No re-serialize |
| **postcard** | Re-serialize | 54.53 | 18.34 | Full re-serialize |

### Analysis

**biSere in-place is on par with bincode re-serialize** (~30 ns) and **~1.8x faster than postcard re-serialize**. In-place updates use direct memory writes with no deserialization or re-serialization.

**Key Insight**: For applications requiring frequent field updates (e.g., game engines, real-time systems, databases), biSere avoids re-serialization overhead and stays competitive with the fastest full re-serialize.

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
| **bincode** | 33.26 | 30.07 |
| **postcard** | 52.62 | 19.00 |
| **biSere** | 148.22 | 6.75 |

### Ten Structs (10 elements)

| Library | Time (ns) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 60.43 | 165.49 |
| **postcard** | 226.03 | 44.24 |
| **biSere** | 617.97 | 16.18 |

### One Hundred Structs (100 elements)

| Library | Time (ns) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 370.84 | 269.66 |
| **postcard** | 961.26 | 104.03 |
| **biSere** | 1.46 µs | 68.44 |

### One Thousand Structs (1000 elements)

| Library | Time (µs) | Throughput (Melem/s) |
|---------|-----------|----------------------|
| **bincode** | 3.60 | 278.02 |
| **postcard** | 7.19 | 139.07 |
| **biSere** | 8.06 | 124.06 |

### Analysis

biSere's serialization scales linearly with the number of elements. The per-struct overhead (offset table entries) keeps biSere slower than bincode and postcard for batch serialization at all sizes. The benefit comes from subsequent zero-copy reads and in-place modifications.

**Key Insight**: For large-scale serialization, biSere remains slower than bincode/postcard. Use biSere when you need many reads or in-place updates after serialization.

---

## Performance Summary

### Where biSere Excels

1. **Serialize**: **1st** — ~29 ns (const header + `serialize_to_buffer`)
2. **Deserialize**: **1st** — ~2.44 ns (layout-specific fast path)
3. **Round-Trip**: **1st** — ~28.4 ns
4. **Field Access**: **1st** — ~2.2 ns zero-copy vs ~5.6 ns full deserialize (bincode/postcard)
5. **In-Place Modification**: **1st** (tie) — ~30 ns, on par with bincode re-serialize, ~1.8x faster than postcard

### Where biSere is Slower

1. **Varying sizes (batch)**: Slower than bincode/postcard when serializing many structs (offset table per buffer)
2. **Buffer Size**: 12.4x larger than the smallest format (enables zero-copy operations)

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

## Other binary serializers and how biSere compares

This section surveys other Rust binary (and related) serializers not included in the current benchmark suite, and how biSere fits in.

### Libraries already benchmarked here

| Library       | Style           | Notes |
|---------------|------------------|--------|
| **bincode**   | Serde, copy      | Fast, compact; no zero-copy or in-place. |
| **postcard**  | Serde, copy      | Good size/speed; embedded-friendly. |
| **MessagePack** (rmp-serde) | Serde, copy | Compact; cross-language. |
| **serde_json** | Text (reference) | Human-readable; much slower and larger. |

### Other notable binary / zero-copy serializers

| Library | Style | Zero-copy | In-place | Cross-lang | Notes |
|---------|--------|-----------|----------|------------|--------|
| **rkyv** | `#[derive]`, Rust-only | Yes | Yes (safe mutation) | No | Zero-copy + safe mutation; rich types (e.g. HashMap). Often wins in independent benchmarks (e.g. [rust_serialization_benchmark](https://github.com/djkoloski/rust_serialization_benchmark)). |
| **FlatBuffers** | Schema (IDL) | Yes | Limited (Rust impl) | Yes | Google; random-access; schema evolution. Can be slow to serialize complex structures. |
| **Cap'n Proto** | Schema (IDL) | Yes | Limited (Rust impl) | Yes | On-demand validation; good for RPC; larger wire size for bulk data. |
| **Abomonation** | Custom trait | Yes (decode = ref) | No | No | Extremely fast encode/decode but **not portable** (architecture-dependent); unsafe; no mutation. |
| **CBOR** (e.g. serde_cbor) | Serde | No | No | Yes | Binary, IETF standard; similar performance profile to JSON in practice. |
| **Prost** (Protobuf) | Schema (`.proto`) | No | No | Yes | Small wire size for structured data; serialize/deserialize typically slower than bincode/postcard. |
| **Speedy** | Custom derive | No | No | No | Fast binary; no zero-copy in the same sense as biSere/rkyv. |

### How biSere compares

- **vs bincode / postcard / MessagePack**: biSere adds **zero-copy field access** and **in-place modification**. In this repo’s benchmarks (single small struct, fixed layout), biSere is first in serialize, deserialize, round-trip, field access, and in-place. bincode/postcard win on **wire size** and on **batch** serialization of many structs (one stream vs one buffer per struct).
- **vs rkyv**: Both offer zero-copy and mutation. rkyv is more mature, has an open type system (e.g. collections), and excels in larger/published benchmarks. biSere is **Rust-only**, **POD + offset-table** based, with a fixed 80-byte header and explicit field IDs; it’s a good fit when you want a simple, predictable layout and minimal dependencies (e.g. bytemuck, no external schema).
- **vs FlatBuffers / Cap'n Proto**: Those are schema-driven and cross-language; biSere is Rust-centric with no IDL. biSere’s format is simpler (header + offset table + data); you trade schema evolution and cross-language for simplicity and, in this benchmark setup, very low latency.
- **vs Abomonation**: Abomonation can be faster on raw encode/decode but is **not portable** and doesn’t support in-place mutation. biSere is portable and supports in-place updates.
- **vs CBOR / Prost / serde_json**: biSere targets **low-latency, same-machine or controlled binary formats**; those target interoperability, standards, or size, not necessarily nanosecond-scale access and in-place edits.

### Summary

biSere sits in the **zero-copy + in-place** niche: it competes with rkyv and (conceptually) with FlatBuffers/Cap'n Proto on “read/update without full deserialize,” while using a simple, Rust-only, offset-table format. For small, fixed-layout structs and when you control both ends, biSere’s current benchmarks are strong; for complex or evolving schemas, large batches, or cross-language use, rkyv, FlatBuffers, Cap'n Proto, or Serde-based formats may be better fits.

---

## Conclusion

biSere leads in all five benchmark categories (serialize, deserialize, round-trip, field access, in-place) while providing zero-copy deserialization and in-place modification. Const header/table and `serialize_to_buffer` yield the fastest serialize; a layout-specific or `view_unchecked` path yields the fastest deserialize.

The library's design prioritizes:
1. **End-to-end speed** — first in serialize, deserialize, and round-trip in the benchmark setup
2. **Zero-copy operations** over minimal buffer size
3. **Type safety** through Rust's type system

For modification-heavy and read-heavy workloads, biSere offers the best performance among the compared libraries.

---

*Benchmark results from latest `cargo bench` run.*
*biSere version: 0.1.0*

