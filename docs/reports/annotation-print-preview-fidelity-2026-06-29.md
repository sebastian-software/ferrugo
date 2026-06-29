# Annotation Print Preview Fidelity Report

Date: 2026-06-29
Milestone: 0193

## Summary

The native renderer now applies PDF annotation visibility flags for static
screen and print-preview thumbnails. Existing appearance streams remain
authoritative, bounded markup/widget synthesis remains available for common
appearance-free annotations, and unsupported FreeText synthesis is classified
with the public `annotation.appearance` bucket.

## Preview Policy

`ThumbnailOptions.annotation_mode` controls annotation visibility:

- `Screen`: render annotations unless `/F` includes `Invisible`, `Hidden`, or
  `NoView`.
- `Print`: render only annotations whose `/F` includes `Print`; `Invisible`
  and `Hidden` still suppress output, while `NoView` does not suppress print
  output.

The CLI exposes this for direct rendering through:

```sh
cargo run -p ferrugo-cli -- render-native \
  fixtures/generated/annotation-print-preview-flags.pdf \
  --output target/annotation-print-preview.png \
  --annotation-mode print
```

## Coverage

`fixtures/annotation-print-preview-manifest.tsv` contains:

- `flags`: printable, non-printable, hidden, no-view, and no-view-printable
  appearance-stream annotations.
- `appearance-stream`: normal stamp/highlight appearance streams.
- `synthesized-markup`: highlight, underline, square, circle, and text-note
  synthesis.
- `nonvisual-link`: appearance-free link annotations stay visually inert.
- `unsupported-synthesis`: appearance-free FreeText returns
  `annotation.appearance`.

## Validation

Focused tests:

```sh
cargo test -p ferrugo-native annotation -- --nocapture
cargo test -p ferrugo-native freetext -- --nocapture
cargo test -p ferrugo-cli annotation_mode -- --nocapture
```

Results: annotation tests 11 passed, FreeText boundary test 1 passed, CLI
annotation-mode parser test 1 passed.

Supported native fallback gate:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/annotation-print-preview-manifest.tsv \
  --include-family flags \
  --include-family appearance-stream \
  --include-family synthesized-markup \
  --include-family nonvisual-link \
  --fail-on-fallback
```

Result: 7 total, 7 native rendered, 0 fallback required.

Unsupported boundary gate:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/annotation-print-preview-manifest.tsv \
  --include-family unsupported-synthesis
```

Result: 1 total, 1 fallback required, categorized as
`annotation.appearance`.

Visual comparison for supported families:

```sh
cargo run -p ferrugo-cli -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/annotation-print-preview-manifest.tsv \
  --include-family flags \
  --include-family appearance-stream \
  --include-family synthesized-markup \
  --include-family nonvisual-link \
  --max-mae 8 \
  --max-p95 32 \
  --max-changed-ratio 0.15
```

Result: 7 total, 2 exact, 5 accepted drift, 0 blockers, 0 native errors,
0 reference errors.

The preview-mode fixture is validated by direct pixel assertions for both
`AnnotationMode::Screen` and `AnnotationMode::Print`, because reference
renderers do not expose a stable cross-tool print-preview flag mode through the
same CLI path.
