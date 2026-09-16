# biSere Performance Benchmarks

This document presents comprehensive performance benchmarks comparing biSere against other popular Rust serialization libraries: bincode, postcard, MessagePack (rmp-serde), and serde_json.

## Recent results

| Category        | biSere    | Rank | vs. fastest binary alternative |
|----------------|-----------|------|--------------------------------|
| **Serialize**  | ~26 ns    | 2nd | bincode ~21 ns (bincode faster) |
| **Deserialize** (via `BinaryView`, the public API) | ~8.6 ns | **Last** of the binary formats | bincode ~4.5 ns, postcard ~6.5 ns (both faster) |
| **Round-trip** | ~52 ns  | 2nd | bincode ~26 ns (bincode faster) |
| **Field access** (zero-copy read) | ~3.9 ns | **1st** | beats bincode ~6.0 ns, postcard ~5.7 ns |
| **In-place modification** | ~4.9 ns | **1st**, by a wide margin | beats bincode's re-serialize ~23.8 ns (~4.8×) |

**biSere wins 2 of these 5 categories** — field access and in-place modification, the two
that measure what its architecture is actually for (read/patch a buffer without a full
deserialize cycle). It is not the fastest at serialize, general-API deserialize, or
round-trip; bincode is faster at all three. A `bisere_deserialize_layout_specific`
benchmark exists that beats everyone at ~2.5 ns by skipping `BinaryView` and the offset
table entirely for a compile-time-known fixed layout — that's a real, usable technique
for a hot path where you control both ends and know the exact layout ahead of time, but
it is not what `BinaryView::view()` does, and it's kept as a separate, clearly-labeled
row rather than reported as "biSere's deserialize performance." See the
[Deserialization Performance](#2-deserialization-performance) section below for why
that distinction matters.

### Can biSere be fastest in every category?

No, and that's fine — it isn't the goal. The two categories biSere wins (field access,
in-place modification) are the two the architecture is built for. Serialize,
deserialize, round-trip, buffer size, and batch serialization are all slower than at
least one alternative — the direct cost of the fixed header + offset table format.

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
| **bincode** | 21.0 | 47.7 | 1st |
| **biSere** | 25.9 | 38.9 | 2nd |
| **messagepack** | 46.9 | 21.3 | 3rd |
| **postcard** | 58.4 | 17.3 | 4th |
| **serde_json** | 87.3 | 11.4 | 5th |

### Analysis

biSere is **2nd**, behind bincode. `serialize_to_buffer` (const header + const offset
table, one allocation, four copies) is efficient, but bincode's serde-based path is
still faster for this small a struct — consistently ~20% faster across repeated runs.

**Key Insight**: Serialization isn't where biSere's advantage lives. Its offset table
and format header exist to enable the zero-copy and in-place operations measured in
sections 4 and 5, which is where the actual win is.

---

## 2. Deserialization Performance

Time required to read data from serialized byte buffers.

The benchmark measures two distinct biSere paths, reported as separate rows:
`bisere` calls `BinaryView::view(buffer)` followed by four `get_field_unaligned`
calls — what a caller without prior knowledge of the layout does to deserialize a
buffer through the public API. `bisere_layout_specific` skips `BinaryView` and the
offset table entirely, reading four fields via `ptr::read_unaligned` at a
compile-time-known fixed offset — a legitimate technique when the caller controls both
ends and knows the exact layout ahead of time, but a different thing from deserializing
through the format.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **biSere** (`bisere_layout_specific`, known layout, bypasses `BinaryView`) | 2.5 | 400 | — *(not a fair comparison; see below)* |
| **bincode** | 4.5 | 222 | 1st |
| **postcard** | 6.5 | 154 | 2nd |
| **biSere** (`bisere`, via `BinaryView::view()`, the public API) | 8.6 | 116 | **3rd — last of the binary formats** |
| **messagepack** | 15.9 | 63 | 4th |
| **serde_json** | 78.0 | 12.8 | 5th |

### Analysis

Measured honestly, biSere's general-purpose deserialize path is **the slowest of the
three binary formats** — slower than both bincode and postcard. This isn't a bug to
fix; it's the direct cost of `view()`'s validation (magic, version, header size, offset
table size, total size) plus an offset-table lookup per field, none of which bincode or
postcard's direct serde deserialization pays. `view_unchecked()` (see the
["Implementation Optimizations"](#implementation-optimizations-before--after) section
above) shaves a small amount off the validation, not enough to close a 2-4 ns gap.

The layout-specific row is real and useful for a caller who controls both ends and
knows the exact fixed layout at compile time — it's ~1.8× faster than bincode. But it
isn't "biSere," any more than a `bincode` caller who memcpy'd known struct offsets out
of a buffer would be "bincode's" performance. It bypasses the thing that makes biSere a
format (the offset table, field IDs, validation) rather than a raw memory layout
convention.

**Key Insight**: Don't choose biSere for deserialize speed — choose bincode or postcard
for that. Choose biSere when you need to read individual fields selectively without
deserializing the whole struct (section 4) or modify fields in place without a
deserialize/serialize round trip (section 5); those are where the real advantage is.

---

## 3. Round-Trip Performance

Complete serialize-then-deserialize cycle.

| Library | Time (ns) | Throughput (Melem/s) | Rank |
|---------|-----------|----------------------|------|
| **bincode** | 26.1 | 38.3 | 1st |
| **biSere** | 52.0 | 19.7 | 2nd |
| **postcard** | 62.9 | 15.9 | 3rd |
| **messagepack** | 63.7 | 15.7 | 4th |

This uses `bisere_deserialize` (the `BinaryView::view()` path), so it reflects the same
serialize + deserialize costs as sections 1 and 2 combined.

### Analysis

biSere is **2nd**, behind bincode by roughly 2×. Its serialize is already slower than
bincode's (section 1), and its deserialize is the slowest of the three binary formats
(section 2), so round-trip compounds both.

**Key Insight**: Round-trip isn't biSere's use case — it measures serialize-once,
read-once, discard, which is exactly the pattern where a fixed header and offset table
buy nothing. biSere's advantage shows up when a buffer is read or modified many times
after one serialize (sections 4 and 5).

---

## 4. Zero-Copy Field Access

Performance of accessing individual fields without full deserialization.

| Library | Method | Time (ns) | Throughput (Melem/s) | Rank |
|---------|--------|-----------|----------------------|------|
| **biSere** | Zero-copy | 3.9 | 254 | 1st |
| **postcard** | Full deserialize | 5.7 | 176 | 2nd |
| **bincode** | Full deserialize | 6.0 | 166 | 3rd |

### Analysis

biSere is **1st** in field access (~3.9 ns vs ~5.7-6.0 ns for bincode/postcard). With dense 1-based field IDs, `find_entry` uses O(1) direct indexing and no allocation. biSere also provides:

- **No allocation**: Returns references directly into the buffer
- **Selective access**: Access only needed fields without deserializing the entire structure
- **Memory efficiency**: No intermediate data structures created

**Key Insight**: For accessing multiple fields, biSere's zero-copy approach is much faster than full deserialization.

---

## 5. In-Place Modification

Performance of updating fields in serialized buffers.

| Library | Method | Time (ns) | Throughput (Melem/s) | Notes |
|---------|--------|-----------|----------------------|-------|
| **biSere** | In-place | 4.9 | 202 | No re-serialize |
| **bincode** | Re-serialize | 23.8 | 42.0 | Full re-serialize |
| **postcard** | Re-serialize | 61.9 | 16.1 | Full re-serialize |

### Analysis

**biSere in-place is ~4.8× faster than bincode's re-serialize** and **~12.6× faster than postcard's**. In-place updates use a direct memory write (`ptr::write_unaligned`) with no deserialization or re-serialization at all — this is the clearest, widest margin in the whole comparison, and it's the operation biSere's format exists for.

**Key Insight**: For applications requiring frequent field updates (e.g., game engines, real-time systems, databases), this is where biSere's fixed header and offset table pay for themselves — every other category in this document is a cost that gets amortized across however many in-place modifications a buffer sees after being serialized once.

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

1. **In-Place Modification**: **1st**, by a wide margin — ~4.9 ns vs ~23.8 ns for bincode's re-serialize (~4.8×), ~61.9 ns for postcard's (~12.6×)
2. **Field Access**: **1st** — ~3.9 ns zero-copy vs ~5.7-6.0 ns full deserialize (postcard/bincode)

### Where biSere is Slower

1. **Serialize**: 2nd — bincode is ~20% faster (~21 ns vs ~26 ns)
2. **Deserialize** (via the public `BinaryView::view()` API): last of the binary formats — bincode (~4.5 ns) and postcard (~6.5 ns) are both faster than biSere (~8.6 ns). A layout-specific bypass exists that beats everyone at ~2.5 ns, but it isn't a deserialize through the format — see [section 2](#2-deserialization-performance)
3. **Round-Trip**: 2nd — bincode is ~2× faster (~26 ns vs ~52 ns), compounding the serialize and deserialize gaps above
4. **Varying sizes (batch)**: slower than bincode/postcard when serializing many structs (offset table per buffer)
5. **Buffer Size**: 12.4× larger than the smallest format (postcard) for this struct

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

- **vs bincode / postcard / MessagePack**: biSere adds **zero-copy field access** and **in-place modification** — the two categories it wins in this repo's benchmarks (single small struct, fixed layout). bincode is faster at serialize, deserialize, and round-trip; bincode/postcard also win on **wire size** and on **batch** serialization of many structs (one stream vs one buffer per struct).
- **vs rkyv**: Both offer zero-copy and mutation. rkyv is more mature, has an open type system (e.g. collections), and excels in larger/published benchmarks. biSere is **Rust-only**, **POD + offset-table** based, with a fixed 80-byte header and explicit field IDs; it’s a good fit when you want a simple, predictable layout and minimal dependencies (e.g. bytemuck, no external schema).
- **vs FlatBuffers / Cap'n Proto**: Those are schema-driven and cross-language; biSere is Rust-centric with no IDL. biSere’s format is simpler (header + offset table + data); you trade schema evolution and cross-language for simplicity and, in this benchmark setup, very low latency.
- **vs Abomonation**: Abomonation can be faster on raw encode/decode but is **not portable** and doesn’t support in-place mutation. biSere is portable and supports in-place updates.
- **vs CBOR / Prost / serde_json**: biSere targets **low-latency, same-machine or controlled binary formats**; those target interoperability, standards, or size, not necessarily nanosecond-scale access and in-place edits.

### Summary

biSere sits in the **zero-copy + in-place** niche: it competes with rkyv and (conceptually) with FlatBuffers/Cap'n Proto on “read/update without full deserialize,” while using a simple, Rust-only, offset-table format. For small, fixed-layout structs and when you control both ends, biSere’s current benchmarks are strong; for complex or evolving schemas, large batches, or cross-language use, rkyv, FlatBuffers, Cap'n Proto, or Serde-based formats may be better fits.

---

## Conclusion

biSere wins 2 of the 5 benchmark categories here — field access and in-place
modification — by wide margins (1.5-2× and 4.8-12.6× respectively). It loses serialize,
deserialize, and round-trip to bincode, and loses buffer size and batch serialization to
both bincode and postcard.

The library's design prioritizes:
1. **Zero-copy field access and in-place modification** — the operations a fixed header
   and offset table are built to support, and the two places that investment pays off
2. **Predictable, explicit layout** (POD types, field IDs, `bytemuck`) over minimal wire
   size or serialize/deserialize throughput
3. **Type safety** through Rust's type system

Choose biSere for workloads that serialize once and then read or modify fields many
times afterward. For write-once/read-once, read-many-without-modification, batch
serialization of many records, or minimal wire size, bincode or postcard are faster.

---

*Benchmark results from latest `cargo bench` run.*
*biSere version: 0.1.0*

