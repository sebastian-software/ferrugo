# Text Selection Geometry And Search Highlight Parity

Date: 2026-06-29
Milestone: 0202

## Summary

Milestone 0202 adds a stable native text-geometry layer for search highlights
and selection overlays. `PositionedGlyph` now carries a `TextQuad` in the same
page-space coordinate system as the existing glyph origin, so callers can build
highlight and selection UI without reaching into renderer internals.

This is still a renderer geometry slice, not a full document editing or semantic
reading-order model.

## API Changes

`ferrugo-thumbnail` now exposes:

- `TextQuad`
- `PositionedGlyph::quad`

The quad contains:

- `origin`: baseline origin point.
- `advance`: baseline advance point.
- `advance_ascent`: advance point offset by the run ascent vector.
- `origin_ascent`: origin point offset by the run ascent vector.

The native backend derives these points from the same display-list text state
used for rendering. That keeps search geometry aligned with current text matrix,
graphics CTM, writing mode, spacing, and glyph advance behavior.

## Regression Corpus

`fixtures/text-extraction-search-manifest.tsv` now includes focused geometry
families in addition to the earlier extraction/search set.

| Family | Fixture | Purpose |
| --- | --- | --- |
| `visible-text` | `text-page.pdf` | Simple horizontal text geometry. |
| `office-search` | `office-table.pdf` | Typical office/table search baseline. |
| `browser-search` | `browser-chromium-article-print.pdf` | Browser article text baseline. |
| `ocr-layer` | `ocr-invisible-text-layer.pdf` | Searchable invisible OCR text with non-painting quads. |
| `tagged-search` | `tagged-invoice-reading-order.pdf` | Tagged-PDF search boundary baseline. |
| `rotated-geometry` | `rotated-office-export.pdf` | Transformed page/text geometry. |
| `vertical-geometry` | `vertical-cjk-text.pdf` | Vertical text advance geometry. |
| `ligature-geometry` | `opentype-ligature-text.pdf` | Mapped glyph chunk geometry for ligatures. |
| `mixed-script-geometry` | `combining-mark-text.pdf` | Combining-mark text geometry. |

Focused support gate:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 9 | 9 | 0 | 0 |

Focused benchmark gate:

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 9 | 9 | 0 | 0 | 0 |

## Geometry And Memory Budget

The renderer display list keeps the existing glyph-origin vector. The public
backend derives advance and ascent points from neighboring glyph origins plus
the text matrix, graphics CTM, font size, and writing mode. That avoids growing
the shared `DisplayItem` enum while preserving the existing
`TextExtractionOptions` bounds:

- `max_runs`
- `max_glyphs`

Repeated extraction does not retain a process-global geometry cache. The
caller-visible geometry result is bounded by the same per-query limits as the
existing text extraction API, and truncated results set `truncated = true`.

## Validation Evidence

Focused tests:

| Test | Coverage |
| --- | --- |
| `text_display_list_should_parse_generated_text_fixture` | Horizontal glyph advance and ascent points. |
| `text_display_list_should_apply_tm_and_ctm_to_origin` | Text matrix and graphics CTM scaling for geometry. |
| `text_display_list_should_advance_identity_v_text_vertically` | Vertical writing-mode advance geometry. |
| `native_backend_should_extract_visible_text_runs` | Public `PositionedGlyph::quad` consistency. |
| `native_backend_should_bound_repeated_text_geometry_queries` | Repeated bounded geometry extraction with tight run/glyph limits. |
| `native_backend_should_extract_vertical_text_geometry_quads` | Public vertical text quad advance direction. |

## Boundaries

- Supported: page-space quads for decoded glyph chunks in native text runs.
- Supported: invisible OCR text quads remain searchable while not painting.
- Supported: vertical writing-mode advance direction from current renderer
  semantics.
- Deferred: exact semantic reading order, OCR generation, text editing, and
  producer-specific repair.
- Deferred: UI search query matching and highlight merging across run breaks.

## Validation

Commands run:

```sh
cargo test -p ferrugo-render text_display_list_should -- --nocapture
cargo test -p ferrugo-native extract -- --nocapture
cargo test -p ferrugo-native geometry -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/text-extraction-search-manifest.tsv --include-family visible-text --include-family office-search --include-family browser-search --include-family ocr-layer --include-family tagged-search --include-family rotated-geometry --include-family vertical-geometry --include-family ligature-geometry --include-family mixed-script-geometry --fail-on-fallback --max-edge 160 --output target/text-geometry-0202-support.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/text-extraction-search-manifest.tsv --include-family visible-text --include-family office-search --include-family browser-search --include-family ocr-layer --include-family tagged-search --include-family rotated-geometry --include-family vertical-geometry --include-family ligature-geometry --include-family mixed-script-geometry --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/text-geometry-0202-benchmark.json
cargo fmt --check
git diff --check -- crates/ferrugo-thumbnail/src/lib.rs crates/ferrugo-render/src/lib.rs crates/ferrugo-native/src/lib.rs fixtures/text-extraction-search-manifest.tsv docs/backend/native.md docs/corpus-taxonomy.md docs/reports/text-selection-geometry-search-highlight-2026-06-29.md
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
