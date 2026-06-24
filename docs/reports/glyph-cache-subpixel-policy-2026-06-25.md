# Glyph Cache Subpixel Policy 2026-06-25

Milestone: 0087.

## Implemented Slice

- Added `GlyphBitmapCache` for the built-in ASCII fallback text rasterizer.
- Bounded the cache to `DEFAULT_GLYPH_BITMAP_CACHE_LIMIT` entries per
  rasterization pass.
- Keyed cached fallback glyph bitmaps by normalized character, quantized cell
  size, and mask-only paint policy.
- Kept color outside the cache key because cached values are geometry masks;
  the rasterizer applies the active paint color at draw time.
- Preserved user-space subpixel glyph origins until final device-pixel coverage.

## Cache Policy

The cache stores prepared fallback glyph rectangles, not painted pixels. This
keeps memory bounded and prevents color-specific duplicate entries. Entries are
evicted oldest-first when the pass-local cache reaches its configured limit.

The size component is quantized to micro user-space units. This avoids `f64`
cache keys while keeping same-size glyph reuse stable and preventing accidental
mixing of visibly different font sizes.

## Subpixel Policy

The native renderer preserves glyph origins in user space. It does not round
text positions during display-list construction. Pixel coverage is resolved only
when transformed glyph rectangles are filled on the raster device.

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/glyph-cache-benchmark-0087.json
```

Summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 58 | 56 | 1 | 1 | 3 |

Text-heavy family details:

| Family | Total | Native rendered | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| `office-export` | 14 | 14 | `29.341` | `92.505` | `844160` |

The three budget failures were not introduced by this slice: the existing
encrypted fixture, optional-content fallback, and vector-stress render-time
budget remain the limiting cases.

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/glyph-cache-visual-diff-0087.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 58 | 22 | 8 | 26 | 1 | 0 | 1 |

Text-font subsystem summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 12 | 0 | 3 | 9 | 0 | 0 | 0 |

The visual-diff summary matches the 0086 post-Type3 state, confirming the cache
is output-neutral for the generated corpus.

## Validation

```text
cargo fmt --check
cargo check --workspace --no-default-features
cargo test -p pdfrust-render glyph_bitmap_cache
cargo test -p pdfrust-render text_display_list_should_preserve_subpixel_glyph_origins
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0087 touched files and passed.

## Remaining Limits

- The cache currently accelerates the built-in ASCII fallback glyph path.
- True embedded-outline glyph raster caching remains future work once the native
  renderer stops relying on fallback ASCII bitmaps for common text.
