# Scanned-page direct image fast path

Issue: #51

## Summary

This change adds a conservative scanned-page classifier and a direct-image
thumbnail route for pages shaped as:

- optional full-page white background before the image;
- exactly one opaque full-page DeviceGray or DeviceRGB image;
- optional invisible OCR text;
- no annotations, clips, transparency groups, masks, rotated/skewed image
  transforms, or multiple image XObjects.

The fast path writes RGBA output directly from decoded image samples. It
precomputes target-column sample indices and reuses expanded rows and pixels
when nearest-neighbor scaling maps multiple target rows or columns to the same
source sample. Rejected pages continue through the existing generic native
pipeline and report the fallback reason in trace and benchmark diagnostics.

## Trace counters

`trace-native` and `benchmark-native` now include
`scanned_page_fast_path_summary`:

- `classifier_calls`
- `candidate_pages`
- `direct_image_calls`
- fallback counters for annotations, ordered content, forms/transparency, path
  content, visible text, image count, masks/non-opaque images, transformed
  images, non-full-page images, and direct-route errors.

## Performance

Command shape:

```sh
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/mobile-ocr-overlay-scan.pdf --max-edge 320 --iterations 500 --max-ms 10000 --max-output-bytes 1048576 --output target/issue51-mobile-ocr-*.json
```

Results on this machine:

| Fixture | Branch | main | Route |
| --- | ---: | ---: | --- |
| `mobile-ocr-overlay-scan.pdf` run 1 | 0.085 ms | 0.153 ms | `direct_image_calls=1` |
| `mobile-ocr-overlay-scan.pdf` run 2 | 0.086 ms | 0.155 ms | `direct_image_calls=1` |

That is approximately 1.80x on run 1 and 1.80x on run 2.

The smaller `scanned-page.pdf` control also routes direct, but is too small to
be the headline performance proof:

| Fixture | Branch | main | Route |
| --- | ---: | ---: | --- |
| `scanned-page.pdf` | 0.058 ms | 0.100-0.107 ms | `direct_image_calls=1` |

## Fidelity

Branch direct output for `mobile-ocr-overlay-scan.pdf` is byte-identical to
`main` generic native output:

```text
3c87bdd0effed60411744e5c8e78097e809a0b27a3f3a39562f84ddef8d5e128
```

Poppler visual diff for the same fixture is exact:

```text
status=exact changed_pixels=0 changed_ratio=0 mean_abs_error=0
```

Synthetic unit coverage also compares direct-image output against the generic
image raster path byte-for-byte.

## Conservative fallbacks

The classifier intentionally leaves these fixtures on the generic route:

- `scanner-large-image-budget.pdf`: extra path content,
  `fallback_path_content=1`, `direct_image_calls=0`.
- `mobile-mixed-compression-scan.pdf`: multiple image XObjects,
  `fallback_image_count=1`.
- rotated or cropped mobile scans with extra path/geometry content:
  `fallback_path_content=1`.

`scanned-page.pdf` still reports a Poppler visual-diff blocker in the existing
`images-color` subsystem, so it is used here only as a direct-route counter and
small-page performance control rather than fidelity evidence.

## Validation

```sh
cargo fmt --check
cargo test -p ferrugo-native scanned_page_fast_path --no-default-features -- --nocapture
cargo test -p ferrugo-native direct_scanned_page_thumbnail_should_match_generic_image_raster --no-default-features -- --nocapture
cargo test -p ferrugo-native render_with_trace_should_report_scanned_page_direct_image_call --no-default-features -- --nocapture
cargo test -p ferrugo trace_native_config_should_bound_event_count --no-default-features -- --nocapture
cargo test -p ferrugo benchmark_native_should_report_fill_route_summary --no-default-features -- --nocapture
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated/mobile-ocr-overlay-scan.pdf --max-edge 320 --output target/issue51-mobile-ocr-poppler.json
```
