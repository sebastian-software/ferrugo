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
| Total page image bytes | 134,217,728 | All resident decoded image resources for one page |
| Embedded font bytes | 16,777,216 | One decoded embedded font program |
| ToUnicode CMap bytes | 1,048,576 | One decoded CMap stream |
| ToUnicode entries | 4,096 | One parsed CMap |
| Text run bytes | 65,536 | One decoded text run |
| Display items | 8,192 | One display list |
| Path segments | 16,384 | One current path |
| Flattened segments | 65,536 | One rasterization pass |
| Transparency group pixels | 16,777,216 | One intermediate transparency raster |
| Glyph outline cache entries | 4,096 | One glyph outline cache |
| Text raster scratch retained atoms | 4,096 | One rasterization pass, retained only between bounded text runs |
| Temporary spooling bytes | 0 | Spooling is disabled by default |

## Cache Behavior

- Embedded font programs are cached by object reference during resource
  resolution so repeated font descriptors share decoded bytes within one
  resource load.
- Image resources store decoded samples behind shared reference-counted buffers,
  so repeated image placements do not duplicate samples. A page-level decoded
  image budget rejects many individually valid images before they can become an
  unbounded resident set.
- Glyph outlines use a bounded cache. When `max_cache_entries` is reached, the
  oldest entry is evicted before storing a new outline. A value of `0` disables
  outline caching without disabling outline extraction.
- Text fallback rasterization reuses a pass-local scratch vector for expanded
  glyph and combining-mark atoms. If one unusually large text run grows the
  scratch capacity beyond the retained-atom limit, the next small text run
  releases that oversized capacity instead of carrying it for the rest of the
  page.
- Temporary spooling is disabled by default. Future opt-in spooling must define
  storage location, byte ceiling, cleanup behavior, and privacy implications
  before writing document-derived intermediates.

## Diagnostics

`ferrugo-cli compare-metadata` includes a `rust_native_memory` JSON block with
the Rust-native default budget snapshot, including the page-level image budget
and disabled spooling policy. This makes local differential runs explainable
when a fixture fails due to a budget limit.

## Non-Goals

- Global process memory limits.
- Resident-set-size reporting.
- Silent best-effort downgrade after budget exhaustion.

## Low-End Reliability Profiles

Milestone 0217 records low-end reliability profiles in
`fixtures/low-end-reliability-profile-matrix.tsv`. These profiles are explicit
checks around existing renderer limits rather than new hidden defaults:

| Profile | Primary limit | Gate |
| --- | --- | --- |
| Low-memory typical workflows | `--native-profile low-memory`, `--max-edge 120` | 20/20 cross-producer supported rows render natively. |
| Low-memory repeated renders | `--native-profile low-memory`, isolated render cache policy | 20/20 repeated rows render with 0 budget failures. |
| Server-constrained batch | `--max-workers 2`, `--max-in-flight-pixels 51200`, `--native-profile low-memory` | 50/50 page jobs render with 0 budget failures. |
| WASM smoke | 4 MiB artifact, 250 ms compile/smoke gates | Secondary browser-profile signal only. |
| Reduced-canvas deterministic render | `--max-edge 96` | Two single-render outputs are byte-identical. |

The low-memory profile remains thumbnail-oriented. Low-end profile failures
should be promoted to the server backlog only when they reveal shared renderer
correctness, safety, or unbounded-resource behavior.
