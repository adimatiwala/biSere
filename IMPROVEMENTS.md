# biSere — Suggested Improvements

Improvements are grouped by priority: **Critical (safety)**, **High (API/UX)**, **Medium (performance/docs)**, and **Future (features)**.

**Done:** Critical items 1–3 below have been implemented (BinaryViewMut UB fix, alignment-safe `get_field` + `get_field_unaligned`, `InvalidUtf8` + `UnalignedField` errors). Also implemented since: `BinarySerializer::reserve()`, `BinaryView::view_unchecked()`, `serialize_to_buffer()` (one-allocation serialize), and the field-lookup fast path in item 7 below — all measured, see [BENCHMARKS.md](BENCHMARKS.md#implementation-optimizations-before--after).

---

## Where the design lacks

A concise summary of design gaps and trade-offs.

| Area | Limitation | Consequence |
|------|------------|-------------|
| **Wire size** | Fixed 80-byte header + full offset table per buffer | 12.4× larger than smallest (postcard) for tiny structs; poor for many small messages. |
| **Batch / streaming** | One header + table per serialized value | Batch serializing N structs is N× overhead; no “stream of records” format. bincode/postcard win on varying-sizes benchmark. |
| **Schema / evolution** | No IDL, no versioned schema, no “optional field” or “added field” semantics | Can’t evolve format safely across versions; reader and writer must agree on layout. |
| **Serialization API** | Manual header + offset table + write order | Error-prone; no builder or derive. Easy to mis-specify sizes or offsets (see improvement #4). |
| **Introspection** | No way to list fields without knowing `field_id`s | Hard to debug or generic tools; no “iterate all fields” (see #5). |
| **Variable-length growth** | Var section size fixed at write time | Can’t grow a string/blob in-place beyond reserved size; must re-serialize to enlarge. |
| **Nested / recursive** | Flat layout only | No nested structs or sub-buffers with their own tables. |
| **Portability** | No endianness tag or conversion | Same-endian only; not suitable for cross-platform wire format without extra handling. |
| **Integrity** | `checksum` in header unused | No corruption detection. |
| **Rich types** | POD + strings/blobs only | No maps, sequences, or enums in the format; no Serde/reflection integration. |
| **Ecosystem** | Rust-only, custom format | No codegen from schema, no cross-language support, no standard tooling. |

**Summary:** The design optimizes for **single-buffer, same-process, fixed-layout, zero-copy + in-place** use. It lacks schema evolution, small wire size, batch/streaming, and cross-platform portability—by design or because those are left to future work.

---

## Critical (safety)

### 1. Fix overlapping mutable references in `BinaryViewMut`

**Issue:** `BinaryViewMut` holds `buffer: &'a mut [u8]`, `header: &'a mut FormatHeader`, and `offset_table: &'a mut [OffsetEntry]`. The header and offset table are *subregions* of `buffer`, so this creates overlapping mutable references, which is **undefined behavior** in Rust.

**Fix:** Store only `buffer: &'a mut [u8]` and derive header/offset table when needed (e.g. via raw pointers or re-parsing in each method). Do not store `&mut FormatHeader` or `&mut [OffsetEntry]` that alias `buffer`.

### 2. Alignment-safe `get_field`

**Issue:** `get_field<T>()` does `unsafe { Ok(&*ptr) }`. For unaligned types (e.g. `f64` in a packed struct at offset 12), dereferencing the pointer is UB.

**Fix:** Either:
- Use `ptr::read_unaligned` and return `T` by value (loses zero-copy for that call), or
- Add a `get_field_unaligned` that returns a copy, and document that `get_field` requires the field to be aligned (and optionally add a runtime alignment check).

### 3. Dedicated UTF-8 error for string fields

**Issue:** Invalid UTF-8 in `get_string` is mapped to `FieldSizeMismatch { expected: 0, got: 0 }`, which is misleading.

**Fix:** Add `SerializationError::InvalidUtf8` (e.g. with optional byte offset or slice) and use it in `get_string` when `from_utf8` fails.

---

## High (API and ergonomics)

### 4. Builder API for `BinarySerializer`

**Issue:** Building a buffer requires manual header math, offset table construction, and ordering of `write_*` calls. Easy to get wrong (e.g. wrong `offset_table_size`, wrong offsets).

**Partially done:** `BinarySerializer::reserve(capacity)` now exists — call it with `header.total_size()` before `write_header` to pre-allocate one buffer instead of growing it through reallocation across the four `write_*` calls. Measured **~49% faster** (64.5 ns → 32.8 ns, criterion) for the traditional write sequence this section's builder example is meant to replace — the largest win of anything changed this pass. This doesn't remove the manual header/offset-table math the issue above describes; a real builder is still open.

**Fix:** Add a builder, e.g.:

```rust
BinarySerializer::builder()
    .fixed_field(1, FieldType::Uint64, 8)?
    .fixed_field(2, FieldType::Uint32, 4)?
    .var_string(10, 256)?
    .build_and_serialize(|data_region, var_region| { ... })
```

Builder computes header and offset table and ensures consistency.

### 5. Field iteration API

**Issue:** No way to enumerate fields except by knowing `field_id`s.

**Fix:** Expose `BinaryView::entries() -> impl Iterator<Item = &OffsetEntry>` (and optionally `fields()` that yields `(field_id, type, value)`-like access). Helps introspection and debugging.

### 6. Type mismatch error variant

**Issue:** Requesting a string for a blob (or wrong fixed type) yields `FieldSizeMismatch`, which is overloaded.

**Fix:** Add `SerializationError::WrongFieldType { field_id, expected, got }` and use it in `get_string`/`get_blob` and in `modify_*` when `entry.field_type` doesn’t match.

---

## Medium (performance and docs)

### 7. Faster field lookup ✅ Done

**Issue:** `find_entry` was O(n) over the offset table on every `get_field`, `get_string`, `get_blob`, `modify_field`, `modify_string`, and `modify_blob` call.

**Done:** Direct-index fast path on both `BinaryView::find_entry` and `BinaryViewMut::find_entry`, for the dense, 1-based field IDs the format's own design already assumes (`field_id` N stored at table index N-1) — falls back to the original linear scan for sparse or non-sequential IDs, so correctness doesn't depend on ID layout, only speed does. `BinaryViewMut::find_entry` also stopped re-parsing the whole `FormatHeader` from the buffer on every call (`data_section_offset`/`var_section_offset` are cached as plain `usize` in `view_mut()`, and the offset table's start is the `HEADER_SIZE` constant now that `validate()` guarantees `header_size == HEADER_SIZE`).

*A `HashMap<u32, usize>` cache (built once per `view()`/`view_mut()` call) was considered instead and rejected: it adds a heap allocation to every view construction — including the common "read one or two fields from a buffer just serialized" case — to optimize a narrower sparse/large-table/many-lookups pattern. Direct indexing is O(1) with zero allocation for the documented common case (see `docs/EXAM.md` Q8) and never worse than the prior linear scan otherwise.*

**Measured** (criterion, same struct and machine as [BENCHMARKS.md](BENCHMARKS.md)):

| Benchmark | Before | After | Change |
|---|---|---|---|
| `field_access/bisere_zero_copy` (`BinaryView`, 3 field reads) | 6.25 ns | 3.75 ns | -40% |
| `inplace_modification/bisere_inplace` (`BinaryViewMut`, 1 field write) | 5.32 ns | 4.86 ns | -9% |

### 8. Document safety and alignment

**Issue:** Unsafe blocks need clear invariants; alignment requirements for `get_field` are not obvious.

**Fix:**  
- Add `# Safety` to every `unsafe` block (invariants that justify safety).  
- In module or type docs, state: “`get_field` returns a reference into the buffer; for packed or unaligned layouts, prefer a copy or an alignment-safe API.”

### 9. Checksum (optional)

**Issue:** Header has a `checksum` field but it’s unused.

**Fix:** Add optional checksum (e.g. CRC or xxHash) over header + offset table + data + var section; validate in `view`/`view_mut` when non-zero. Keep backward compatible (checksum 0 = disabled).

---

## Future (features)

- **Nested structures:** Recursive or nested “sub-buffers” with their own offset tables.
- **Endianness:** Configurable or tagged endianness for multi-platform.
- **Schema versioning / migration:** Version in header + rules to upgrade old buffers.
- **Derive macro:** `#[derive(BiSere)]` to generate serialization and offset table from a struct.
- **Dynamic string/blob resize:** Allow growing variable-length fields (e.g. by appending and updating size in header or offset table).
- **Memory-mapped files:** `BinaryView` over `mmap` (or similar) for large files.

---

## Summary table

| Area        | Improvement                          | Effort  | Impact   |
|------------|--------------------------------------|--------|----------|
| Safety     | Fix `BinaryViewMut` overlapping refs  | Medium | Critical |
| Safety     | Alignment-safe `get_field`           | Small  | High     |
| Safety     | `InvalidUtf8` (and type mismatch)   | Small  | Medium   |
| API        | Builder for serializer               | Medium | High     |
| API        | Field iteration                      | Small  | Medium   |
| Perf       | O(log n) or O(1) field lookup        | Medium | Medium   |
| Docs       | Safety and alignment docs            | Small  | High     |
| Features   | Checksum, endianness, derive, etc.   | Large  | Variable |

Implementing the three critical safety items (1–3) is recommended first; then builder and docs (4, 8).
