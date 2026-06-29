# Scanned Document Coverage

Date: 2026-06-24.
Milestone: 0066.

## Scope

The committed scan corpus is generated and license-safe. It covers small codec
and color-space fixtures plus a page-sized scan-like image fixture:

- unfiltered RGB Image XObject;
- DeviceCMYK and Indexed color spaces;
- DCT/JPEG and Flate PNG predictor decoding;
- soft-mask alpha compositing;
- page-sized DeviceGray image placement.

## Corpus Summary

Command:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/scan-summary.json
```

Result:

| Family | Total | Native rendered | Native pass rate | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `scan` | 7 | 7 | 1.000 | 0 | 0 |

## PDFium Differential Smoke

Native and direct PDFium rendering were run at `--max-edge 200` using the local
PDFium dylib at
`/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`.

| Fixture | Native | PDFium | Native size | PDFium size |
| --- | --- | --- | --- | --- |
| `cmyk-image.pdf` | ok | ok | 120x120 | 120x120 |
| `dct-image.pdf` | ok | ok | 120x120 | 120x120 |
| `image-xobject.pdf` | ok | ok | 120x120 | 120x120 |
| `indexed-image.pdf` | ok | ok | 120x120 | 120x120 |
| `predictor-image.pdf` | ok | ok | 120x120 | 120x120 |
| `scanned-page.pdf` | ok | ok | 160x200 | 160x200 |
| `soft-mask-image.pdf` | ok | ok | 120x120 | 120x120 |

## Memory Diagnostics

`compare-metadata` on `scanned-page.pdf` matched PDFium page metadata and
reported the native budget snapshot:

| Budget | Value |
| --- | ---: |
| `max_page_pixels` | 16777216 |
| `max_image_bytes` | 33554432 |
| `max_font_program_bytes` | 16777216 |
| `max_cmap_bytes` | 1048576 |
| `max_text_run_bytes` | 65536 |
| `max_display_items` | 8192 |

Oversized decoded image data already maps to `renderer.memory-budget` through
`ImageBytesOverflow`; future large-scan slices should add reduced adversarial
fixtures for CCITT, JPX, JBIG2, and very large image dimensions without
committing large binary samples.
