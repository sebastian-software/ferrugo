# Glyph Cache Reuse Follow-up

Date: 2026-07-03
Issue: #27

## Summary

This follow-up keeps the existing fallback glyph bitmap size-tier policy,
records fixture-level hit-rate evidence, documents the outline reuse boundary,
and relaxes one Type3 rendered-glyph reuse gate with byte-parity coverage.

## Fallback Glyph Bitmap Size Tiers

The fallback glyph bitmap cache key already stores a size-tiered `cell_eighths`
value through `quantize_glyph_cell`:

- cells below `24.0` keep eighth-cell precision;
- cells from `24.0` up to `48.0` keep quarter-cell precision;
- cells at `48.0` and above round to whole-cell precision.

The focused invariant remains covered by:

```bash
cargo test -p ferrugo-render glyph_bitmap_cache_should_reuse_quantized_size_tiers -- --nocapture
```

That test exercises three size tiers and verifies `3` hits, `3` misses, and
`3` retained entries for near-equivalent cell sizes.

## Text-heavy Trace Evidence

Artifacts:

- `target/issue-27-trace-text-page.json`
- `target/issue-27-trace-business-invoice-dense.json`
- `target/issue-27-trace-office-table.json`

Commands:

```bash
cargo run -p ferrugo --no-default-features -- trace-native fixtures/generated/text-page.pdf --max-edge 160 --output target/issue-27-trace-text-page.json
cargo run -p ferrugo --no-default-features -- trace-native fixtures/generated/business-invoice-dense.pdf --max-edge 160 --output target/issue-27-trace-business-invoice-dense.json
cargo run -p ferrugo --no-default-features -- trace-native fixtures/generated/office-table.pdf --max-edge 160 --output target/issue-27-trace-office-table.json
```

| Fixture | Entries | Hits | Misses | Hit rate | Bytes | Evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `text-page.pdf` | 15 | 8 | 15 | 34.8% | 5696 | 0 |
| `business-invoice-dense.pdf` | 38 | 70 | 38 | 64.8% | 16384 | 0 |
| `office-table.pdf` | 21 | 14 | 21 | 40.0% | 9664 | 0 |
| Total | 74 | 92 | 74 | 55.4% | 31744 | 0 |

These traces are request-local native render traces. They contain cache
counters and timing attribution, but no document bytes, text samples, glyph
samples, or decoded font bytes.

## Glyph Outline Reuse Boundary

`GlyphOutlineCache` and outline extraction exist for supported embedded font
programs, but the native text raster path still routes visible text through
Type3 CharProc paths or fallback glyph bitmaps. There is no production
outline-backed text rasterizer to attach outline reuse to yet.

Outline reuse is therefore intentionally deferred until an outline-backed text
render path exists. Reusing extracted outlines before that path would add cache
surface area without removing current raster work or producing measurable page
output evidence.

## Type3 Render Cache Gate Evaluation

The Type3 rendered-glyph cache keeps the conservative gates for path semantics:

- fill-only Type3 path items;
- no fill pattern or state fill pattern;
- normal blend mode;
- fully opaque fill and stroke alpha;
- no fill or stroke overprint;
- no pending clip path.

This slice relaxes the `options.scissor.is_none()` gate only. Scissored raster
options are already part of `Type3GlyphRenderKey`, so scissored renders use a
separate cache identity from non-scissored renders and from differently
scissored bands.

Byte parity is covered by:

```bash
cargo test -p ferrugo-render type3_glyph_render_cache_should_reuse_scissored_surfaces_with_byte_parity -- --nocapture
```

The regression renders repeated Type3 glyphs through the uncached path and the
cached path under `RasterScissor::new(0, 12, 40, 32)`, verifies identical RGBA
bytes, and verifies the rendered-glyph cache summary: `1` miss, `1` hit, `1`
insert, and `0` evictions.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render type3_glyph_render_cache_should_reuse_scissored_surfaces_with_byte_parity -- --nocapture
cargo test -p ferrugo-render glyph_bitmap_cache_should_reuse_quantized_size_tiers -- --nocapture
```
