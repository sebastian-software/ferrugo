# 0086: Type3 Font And CharProc Rendering

Status: todo
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

Empty until done.
