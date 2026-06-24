# 0086: Type3 Font And CharProc Rendering

Status: done
Phase: 15
Size: medium
Depends on: 0085

## Goal

Render common Type3 fonts by executing glyph CharProcs through the existing
display-list and raster paths.

## Scope

- Parse Type3 font dictionaries, glyph widths, font matrices, and encodings.
- Execute CharProc content streams with bounded recursion and graphics state.
- Reuse path, image, and color operations already supported by page rendering.
- Add fixtures for barcode-like, symbol, and simple vector glyphs.

## Non-Goals

- Support unbounded recursive glyph programs.
- Implement every PDF text feature in the same milestone.
- Add unsafe font execution paths.

## Deliverables

- Type3 glyph rendering path.
- CharProc recursion and operation budgets.
- Differential fixtures and support matrix updates.

## Acceptance Criteria

- Common vector Type3 glyphs render through the native backend.
- Recursive or malformed CharProcs fail with deterministic errors.
- Glyph rendering reuses existing rasterization without duplicate engines.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run Type3 visual comparisons.
- Run malformed Type3 budget tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed in 0086 implementation slice.

- Added Type3 font metadata loading for `/FontMatrix`, `/FontBBox`,
  `/FirstChar`, `/LastChar`, `/Widths`, `/Encoding` differences, and referenced
  `/CharProcs` streams.
- Routed Type3 text rasterization through the existing content/path display-list
  and raster pipeline instead of adding a duplicate glyph renderer.
- Stored Type3 metadata behind `Arc` so cloned text display items do not carry
  large CharProc payloads by value.
- Added deterministic fixtures for vector, symbol-like, and barcode-like Type3
  glyph programs.
- Added malformed/budget coverage through a Type3 CharProc byte-limit test.
- Added native fixture regression tests for the generated Type3 PDFs.
- Published the validation report at
  `docs/reports/type3-font-coverage-2026-06-25.md`.
