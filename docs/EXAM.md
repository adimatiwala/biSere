# biSere — 25-Question Exam

Mix of short answer, multiple choice, and fill-in-the-blank. Based on the biSere project (format, design, benchmarks, improvements, production).

---

## Questions

**1. (Fill in the blank)** The biSere format magic number is the 32-bit value `0x42495345`, which spells the ASCII string **________** in big-endian.

**2. (MCQ)** How many bytes is the biSere `FormatHeader`?  
A) 48  
B) 64  
C) 80  
D) 128  

**3. (Short answer)** Name the four sections of the biSere binary format, in order.

**4. (Fill in the blank)** In an `OffsetEntry`, the `offset` field is the offset from the start of the **________** section, not the start of the buffer.

**5. (MCQ)** Which trait does biSere rely on for safe zero-copy transmutation of fixed-size types?  
A) `Serialize`  
B) `Pod` (bytemuck)  
C) `Copy`  
D) `AsBytes`  

**6. (Short answer)** What are the three core components of biSere’s architecture (types/structs)?

**7. (Fill in the blank)** Variable-length strings in biSere are stored as **________** UTF-8 in the variable data section.

**8. (MCQ)** For small offset tables (e.g. ≤16 entries) with dense 1-based field IDs, how does `BinaryView` find an entry by `field_id`?  
A) Binary search  
B) Linear search  
C) Direct index: `offset_table[field_id - 1]`  
D) HashMap built at view creation  

**9. (Short answer)** Why did `BinaryViewMut` originally risk undefined behavior, and how was it fixed?

**10. (MCQ)** When a field lies at an unaligned offset (e.g. `f64` in a packed struct at offset 12), what should the user use instead of `get_field` to avoid UB?  
A) `get_string`  
B) `get_field_unaligned`  
C) `view_unchecked`  
D) `read_unaligned` manually  

**11. (Fill in the blank)** The header’s **________** field is reserved for an optional integrity check but is currently unused (0).

**12. (Short answer)** In the benchmarks, why is biSere’s serialized buffer much larger (e.g. 149 bytes) than postcard (12 bytes) for the same logical data?

**13. (MCQ)** In the “varying sizes” benchmark (batch serializing N structs), why does biSere perform worse than bincode/postcard?  
A) Slower per-byte write speed  
B) One header and offset table per buffer, so N× overhead  
C) No SIMD  
D) Larger dependency set  

**14. (Fill in the blank)** The fastest deserialize path in the benchmark uses a layout-specific path with four **________** reads, not the generic `BinaryView`.

**15. (Short answer)** Give one design limitation of biSere regarding schema evolution or cross-version compatibility.

**16. (MCQ)** Which error variant should be used when the bytes in a string field are not valid UTF-8?  
A) `FieldSizeMismatch`  
B) `InvalidUtf8`  
C) `InvalidOffset`  
D) `FieldNotFound`  

**17. (Short answer)** What does `view_unchecked` do differently from `view()`, and when is it appropriate to use?

**18. (MCQ)** What is the `FieldType` enum value for a 64-bit unsigned integer?  
A) 4  
B) 6  
C) 8  
D) 10  

**19. (Fill in the blank)** To get the best serialization performance, the user should call **________** with `header.total_size()` before writing the header and data.

**20. (Short answer)** Name two things that “productionizing” biSere would require (from the productionization roadmap).

**21. (MCQ)** Which of these does biSere *not* support in its wire format?  
A) Nested structs with their own offset tables  
B) Fixed-size integers and floats  
C) Variable-length strings and blobs  
D) Multiple fields with different types in one buffer  

**22. (Fill in the blank)** biSere is **________**-only: it does not define a cross-language schema or wire format for other languages.

**23. (Short answer)** Why might someone choose rkyv over biSere (or vice versa) for zero-copy + in-place use cases?

**24. (MCQ)** In the benchmark “round-trip” (serialize then deserialize), which library is documented as 1st in the recent results?  
A) bincode  
B) postcard  
C) biSere  
D) serde_json  

**25. (Short answer)** What is one improvement suggested for the serialization API (to avoid manual header/offset table construction)?

---

## Answer key

| # | Answer |
|---|--------|
| **1** | **BISE** (or "BISE" in ASCII). |
| **2** | **C) 80** (4+4+4+4+4+4+8+48 = 80). |
| **3** | (1) Format Header, (2) Offset Table, (3) Fixed Data Section, (4) Variable Data Section. (Order matters.) |
| **4** | **data** (or “fixed data” / “data section”). |
| **5** | **B) Pod (bytemuck)**. |
| **6** | **BinarySerializer**, **BinaryView**, **BinaryViewMut**. |
| **7** | **null-terminated** (or “null-terminated”). |
| **8** | **C) Direct index: offset_table[field_id - 1]** (when dense 1-based). (D is also used when table is larger and not dense.) |
| **9** | It stored `buffer`, `header`, and `offset_table` as separate `&mut` references, but header and table are subregions of buffer → overlapping mutable references (UB). Fixed by storing only `buffer` and deriving header/table when needed. |
| **10** | **B) get_field_unaligned**. |
| **11** | **checksum**. |
| **12** | biSere has a fixed 80-byte header plus a full offset table (e.g. 48 bytes for 4 entries) plus the data; postcard (and similar) use a minimal encoding with no per-buffer header/table. |
| **13** | **B) One header and offset table per buffer, so N× overhead.** |
| **14** | **read_unaligned** (or “ptr::read_unaligned”). |
| **15** | Any one of: no IDL/schema; no versioned schema or optional/added fields; reader and writer must agree on exact layout; no safe evolution across versions. |
| **16** | **B) InvalidUtf8**. |
| **17** | `view_unchecked` skips format validation (magic, version, sizes). Use only when the buffer is trusted (e.g. you just serialized it); otherwise use `view()` for untrusted input. |
| **18** | **C) 8** (Uint64 = 8 in FieldType). |
| **19** | **reserve** (i.e. `serializer.reserve(header.total_size())`). |
| **20** | Any two of: document all `unsafe` with # Safety; add WrongFieldType error; fuzz/malformed-input tests; builder API for BinarySerializer; field iteration; public API audit and semver; CI (test, clippy, fmt); format spec; safety/alignment docs. |
| **21** | **A) Nested structs with their own offset tables** (flat layout only). |
| **22** | **Rust** (or “Rust-only”). |
| **23** | rkyv: more mature, richer type system (e.g. collections), schema via derive. biSere: simpler format, no external schema, bytemuck/POD-focused, good for fixed layout and minimal deps. (Accept equivalent comparison.) |
| **24** | **C) biSere**. |
| **25** | **Builder API** for BinarySerializer (builder that takes field list and computes header + offset table so users don’t hand-roll sizes/offsets). |

---

## Scoring suggestion

- Fill-in-the-blank: 1 point each (7 questions → 7 points).
- MCQ: 1 point each (10 questions → 10 points).
- Short answer: 2 points each (8 questions → 16 points).  
Total: 33 points (or scale to 100).

Allow partial credit for short answers (e.g. one good point = 1 point).
