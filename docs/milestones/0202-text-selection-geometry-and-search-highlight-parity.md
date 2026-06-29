# 0202: Text Selection Geometry And Search Highlight Parity

Status: done
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

- Added page-space `TextQuad` geometry to `PositionedGlyph`.
- Added native quad derivation from glyph origins, text matrix, graphics CTM,
  font size, and writing mode without growing the shared display item enum.
- Extended `fixtures/text-extraction-search-manifest.tsv` with
  `rotated-geometry`, `vertical-geometry`, `ligature-geometry`, and
  `mixed-script-geometry` families.
- Added bounded repeated-query coverage for text geometry extraction.
- Focused text-geometry support gate: 9/9 native, 0 fallback, 0 errors.
- Focused text-geometry benchmark gate: 9/9 native, 0 budget failures.
- Produced
  `docs/reports/text-selection-geometry-search-highlight-2026-06-29.md`.
- Validation:
  - `cargo test -p ferrugo-render text_display_list_should -- --nocapture`
  - `cargo test -p ferrugo-native extract -- --nocapture`
  - `cargo test -p ferrugo-native geometry -- --nocapture`
  - `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/text-extraction-search-manifest.tsv --include-family visible-text --include-family office-search --include-family browser-search --include-family ocr-layer --include-family tagged-search --include-family rotated-geometry --include-family vertical-geometry --include-family ligature-geometry --include-family mixed-script-geometry --fail-on-fallback --max-edge 160 --output target/text-geometry-0202-support.json`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/text-extraction-search-manifest.tsv --include-family visible-text --include-family office-search --include-family browser-search --include-family ocr-layer --include-family tagged-search --include-family rotated-geometry --include-family vertical-geometry --include-family ligature-geometry --include-family mixed-script-geometry --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/text-geometry-0202-benchmark.json`
  - `cargo fmt --check`
  - `git diff --check -- crates/ferrugo-thumbnail/src/lib.rs crates/ferrugo-render/src/lib.rs crates/ferrugo-native/src/lib.rs fixtures/text-extraction-search-manifest.tsv docs/backend/native.md docs/corpus-taxonomy.md docs/milestones/0202-text-selection-geometry-and-search-highlight-parity.md docs/milestones/README.md docs/reports/text-selection-geometry-search-highlight-2026-06-29.md`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
