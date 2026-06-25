# 0202: Text Selection Geometry And Search Highlight Parity

Status: todo
Phase: 38
Size: medium
Depends on: 0201

## Goal

Improve Rust-native text geometry so search hits, selection boxes, and copied
text positions match rendered glyphs for common office, browser, report, and
long-form PDFs.

## Scope

- Audit glyph run geometry against page transforms, text matrices, writing
  modes, and font subset metrics.
- Add selection and search-highlight fixtures for rotated, scaled, vertical,
  ligature, and mixed-script text.
- Expose typed geometry drift diagnostics without retaining document text beyond
  the validation artifact.
- Add memory-bounded caches for repeated text geometry queries.

## Non-Goals

- Build a full document editing model.
- Guarantee perfect extraction for every malformed encoding.
- Add OCR for pages without a text layer.

## Deliverables

- Text geometry regression corpus.
- Search-highlight and selection parity report.
- Geometry drift diagnostics and cache budget notes.

## Acceptance Criteria

- Common text runs produce stable selection boxes within documented tolerance.
- Search highlights align with rendered glyph bounds for supported fonts.
- Repeated geometry queries stay within the memory budget.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run text geometry corpus comparisons.
- Run search-highlight snapshot tests.
- Run repeated-query cache benchmark.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
