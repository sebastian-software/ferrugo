# 0058: Renderer Cache And Memory Budgets

Status: in-progress
Phase: 8
Size: medium
Depends on: 0057

## Goal

Put renderer-wide memory and cache controls in place before larger corpus runs
and PDFium fallback reduction.

## Scope

- Define per-document, per-page, image, glyph, and temporary-buffer budgets.
- Add cache eviction for decoded streams, images, fonts, and glyph outlines.
- Expose memory diagnostics in differential and thumbnail runs.
- Add tests for budget exhaustion and cache reuse.

## Non-Goals

- Global process memory accounting.
- Perfect operating-system memory reporting.
- Premature micro-optimization without measurements.

## Deliverables

- Renderer budget configuration.
- Cache accounting and eviction hooks.
- Budget-focused tests and benchmark notes.

## Acceptance Criteria

- Large or adversarial PDFs fail with budget errors instead of exhausting
  memory.
- Common documents benefit from cache reuse without unbounded growth.
- Memory diagnostics are visible in local comparison output.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run targeted large-fixture and adversarial-fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- First slice added `PageTransformOptions` with
  `DEFAULT_PAGE_RASTER_PIXELS_LIMIT`, enforcing a page-raster pixel budget
  before allocation and returning a typed `PageRasterPixelsOverflow` error.
- Second slice exposed the Rust-native default memory budget snapshot through
  `NativeBackend::memory_diagnostics()` and the `compare-metadata`
  `rust_native_memory` JSON block.
- Third slice changed the glyph-outline cache from fail-when-full behavior to
  bounded oldest-entry eviction and documented `max_cache_entries == 0` as an
  uncached lookup mode.
