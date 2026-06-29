# Native Text Extraction Search Boundary 2026-06-29

Milestone: 0186.

## Summary

Added the first backend-neutral native text extraction API. The API exposes
page text runs, decoded Unicode, per-glyph positions, visibility state, and
bounded extraction limits without coupling searchability to visual raster
fidelity.

This is a search-boundary slice, not a full semantic reading-order or selection
highlight implementation.

## API

`ferrugo-thumbnail` now exposes:

- `TextExtractionBackend`
- `TextExtractionOptions`
- `PageText`
- `TextRun`
- `PositionedGlyph`
- `TextPoint`

Native extraction uses the existing text display-list path, so it reuses the
same ToUnicode, Encoding Differences, Identity-H/V, spacing, and text rendering
mode handling that the renderer already depends on.

## Fixture Coverage

Added `fixtures/text-extraction-search-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `visible-text` | 1 | Simple visible standard-base text baseline. |
| `office-search` | 1 | Typical office/table search baseline. |
| `browser-search` | 1 | Browser article print search baseline. |
| `ocr-layer` | 1 | Invisible OCR text layer baseline. |
| `tagged-search` | 1 | Tagged invoice search boundary baseline. |

This manifest is a focused search fixture set. The first implementation tests
the API directly against representative visible and invisible text fixtures;
broader corpus scoring can build on the manifest without changing the API.

## Validation Evidence

Focused native tests:

| Test | Coverage |
| --- | --- |
| `native_backend_should_extract_visible_text_runs` | Extracts `ferrugo thumbnail fixture` with visible text and positioned glyphs. |
| `native_backend_should_extract_invisible_ocr_text_runs` | Extracts two invisible OCR text runs and preserves `visible = false`. |
| `native_backend_should_bound_extracted_text_glyphs` | Confirms glyph extraction truncates when `max_glyphs` is reached. |

The bounded glyph test is the initial memory profile for this API: extraction
returns a truncated result instead of growing the output without limit.

## Boundary Notes

- Supported: content-stream order text runs for pages that the native renderer
  can parse.
- Supported: Unicode text mapped by the existing font/CMap path.
- Supported: per-glyph origin positions in page coordinate space.
- Supported: invisible OCR text layers as searchable but non-painting text.
- Out of scope: semantic document understanding, producer repair,
  exact tagged reading order, UI search highlighting, and text selection quads.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs crates/ferrugo-thumbnail/src/lib.rs fixtures/text-extraction-search-manifest.tsv docs/backend/native.md docs/corpus-taxonomy.md docs/milestones/README.md docs/milestones/0186-native-text-extraction-and-search-parity-gate.md docs/reports/native-text-extraction-search-boundary-2026-06-29.md
cargo test -p ferrugo-native extract -- --nocapture
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
