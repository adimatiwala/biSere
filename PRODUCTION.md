# Productionization roadmap

What it takes to make biSere production-ready. Phases are ordered by dependency and risk.

---

## Phase 1: Safety and correctness (must-have)

These are required before trusting the library with untrusted or long-lived data.

| Item | Status | Action |
|------|--------|--------|
| **BinaryViewMut overlapping refs** | Done | Already fixed (only `buffer` stored). |
| **Alignment-safe get_field** | Done | `get_field_unaligned`, alignment check, `UnalignedField` error exist. |
| **InvalidUtf8 / UnalignedField errors** | Done | Error variants and use in `get_string` / `get_field` exist. |
| **Document all `unsafe`** | Partial | `view_unchecked` and the `get_field` vs `get_field_unaligned` distinction are documented (doc comments explain when each applies and why). Not every `unsafe` block in `src/` has a full `# Safety` invariant comment yet. |
| **WrongFieldType error** | Todo | Add `SerializationError::WrongFieldType { field_id, expected, got }`; use in get_string/get_blob and modify_* instead of overloading `FieldSizeMismatch`. |
| **Fuzz / malformed input** | Todo | Fuzz `view()` and accessors on random bytes; add tests with truncated, oversized, and corrupted buffers to ensure no panics or UB. |

**Exit criteria:** No known UB or unsoundness; all `unsafe` documented; malformed input handled without panic.

---

## Phase 2: API stability and ergonomics (should-have)

Makes the library hard to misuse and gives a stable contract for semver.

| Item | Action |
|------|--------|
| **Builder for BinarySerializer** | Partial: `reserve(capacity)` exists (~49% faster than the unreserved write sequence, measured — see [BENCHMARKS.md](BENCHMARKS.md#implementation-optimizations-before--after)). A real builder that computes header + offset table from a field list, so users don't hand-roll sizes/offsets, is still open. |
| **Field iteration** | `BinaryView::entries()` (and optionally a safe `fields()`-style API) so callers can enumerate without knowing field IDs. |
| **Public API audit** | `serialize_to_buffer` and `view_unchecked` now exist and are documented (the latter measured ~3% faster than full `view()` validation, and deliberately keeps the one bounds check that prevents a panic on a malformed `offset_table_size`). Still need to decide what else is part of the stable API and mark the rest `pub(crate)` or unstable. |
| **Semver and changelog** | Publish 0.1.x with “pre-1.0” caveats; maintain CHANGELOG.md; bump minor for new features, patch for fixes. |
| **MSRV** | Pin minimum supported Rust version in README and CI; test on that version. |

**Exit criteria:** Users can build buffers without manual header math; public API is clearly scoped and documented; releases are versioned and logged.

---

## Phase 3: Testing and CI (should-have)

| Item | Action |
|------|--------|
| **CI** | GitHub Actions (or similar): `cargo test`, `cargo clippy`, `cargo fmt -- --check` on push/PR. |
| **Property / round-trip tests** | Round-trip all supported types; assert size and byte-equality where relevant. |
| **Benchmark in CI** | Optional: run benches and fail on large regressions (e.g. threshold vs baseline). |
| **Doc tests** | Ensure README and key doc comments compile and run (`cargo test --doc`). |

**Exit criteria:** Every PR is tested and linted; regressions are caught automatically.

---

## Phase 4: Documentation and format contract (should-have)

| Item | Action |
|------|--------|
| **Format spec** | Single source of truth for wire format (header layout, offset table, section order, version semantics). README or FORMAT.md; link from crate docs. |
| **Safety and alignment** | Module-level doc: when to use `get_field` vs `get_field_unaligned`; that `view_unchecked` skips validation. |
| **Examples** | At least one “happy path” example (serialize → view → read/modify) that’s in the crate and linked from README. |
| **Production disclaimer** | Either remove “educational/experimental” once Phase 1–2 are done, or clearly state what’s production-ready (e.g. “format and view API stable; builder still experimental”). |

**Exit criteria:** A new user can understand the format, safety rules, and basic usage from docs alone.

---

## Phase 5: Optional production hardening (nice-to-have)

| Item | Effort | Notes |
|------|--------|--------|
| **Optional checksum** | Small | Use header `checksum`; 0 = disabled. Validate in `view`/`view_mut` when non-zero. |
| **Endianness** | Medium | Document “same-endian only” or add optional byte-swap for cross-platform. |
| **Derive macro** | Large | `#[derive(BiSere)]` for structs to generate header + table + accessors; reduces manual errors. |
| **Schema / versioning** | Large | Version in header + migration path for added/removed fields. |

---

## Minimal “production-ready” set

- **Phase 1** complete (safety, documented unsafe, WrongFieldType, fuzz/malformed tests).
- **Phase 2** at least: builder + public API audit + semver/changelog.
- **Phase 3** CI with test + clippy + fmt.
- **Phase 4** format spec + safety/alignment docs + one clear example.

That gives: no known soundness issues, a harder-to-misuse API, stable public surface, and clear docs. Optional checksum and “same-endian” documentation can follow without blocking 1.0.

---

## Checklist summary

```
[ ] All unsafe documented with # Safety
[ ] WrongFieldType error and use in API
[ ] Fuzz or malformed-input tests
[ ] Builder for BinarySerializer
[ ] Field iteration (entries / fields)
[ ] Public API audit and semver policy
[ ] CHANGELOG.md and release process
[ ] CI: test, clippy, fmt
[ ] Format spec (README or FORMAT.md)
[ ] Safety and alignment docs
[ ] At least one end-to-end example
[ ] (Optional) Checksum; endianness doc or support
```
