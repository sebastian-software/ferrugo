# 0103: OpenType Layout Feature Coverage For PDFs

Status: todo
Phase: 18
Size: medium
Depends on: 0102

## Goal

Cover the OpenType layout behavior needed by typical PDFs that depend on
ligatures, marks, kerning, and shaped text output.

## Scope

- Map PDF text state into the shaping inputs needed by the native renderer.
- Support common GPOS and GSUB paths used by embedded fonts.
- Keep shaping buffers reusable to avoid per-glyph allocation churn.
- Add fixtures for ligatures, combining marks, Arabic text, and office exports.

## Non-Goals

- Add text editing or selection behavior.
- Support every script-specific shaping edge case in one slice.
- Replace earlier CMap and ToUnicode extraction work.

## Deliverables

- OpenType layout coverage implementation.
- Shaping fixture set and visual comparison report.
- Documentation of unsupported layout features.

## Acceptance Criteria

- Common ligatures and mark positioning render in native output.
- Repeated page renders reuse buffers and avoid unbounded allocations.
- Unsupported shaping cases produce explicit typed fallback reasons.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run shaped-text visual comparisons.
- Run allocation-sensitive text rendering smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
