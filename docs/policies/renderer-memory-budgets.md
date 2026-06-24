# Renderer Memory Budgets

Status: accepted.
Date: 2026-06-24.

The Rust-native renderer uses explicit local budgets before broader corpus runs
reduce reliance on the PDFium fallback. These budgets are deterministic checks
inside parser, display-list, and raster paths; they are not operating-system
memory accounting.

## Default Budgets

| Budget | Default | Scope |
| --- | ---: | --- |
| Page raster pixels | 16,777,216 | One page RGBA raster before allocation |
| Image XObject bytes | 33,554,432 | One decoded image resource |
| Embedded font bytes | 16,777,216 | One decoded embedded font program |
| ToUnicode CMap bytes | 1,048,576 | One decoded CMap stream |
| ToUnicode entries | 4,096 | One parsed CMap |
| Text run bytes | 65,536 | One decoded text run |
| Display items | 8,192 | One display list |
| Path segments | 16,384 | One current path |
| Flattened segments | 65,536 | One rasterization pass |
| Transparency group pixels | 16,777,216 | One intermediate transparency raster |
| Glyph outline cache entries | 4,096 | One glyph outline cache |

## Cache Behavior

- Embedded font programs are cached by object reference during resource
  resolution so repeated font descriptors share decoded bytes within one
  resource load.
- Image resources store decoded samples behind shared reference-counted buffers,
  so repeated image placements do not duplicate samples.
- Glyph outlines use a bounded cache. When `max_cache_entries` is reached, the
  oldest entry is evicted before storing a new outline. A value of `0` disables
  outline caching without disabling outline extraction.

## Diagnostics

`pdfrust-cli compare-metadata` includes a `rust_native_memory` JSON block with
the Rust-native default budget snapshot. This makes local differential runs
explainable when a fixture fails due to a budget limit.

## Non-Goals

- Global process memory limits.
- Resident-set-size reporting.
- Silent best-effort downgrade after budget exhaustion.
