# ICC Profile Cache And Transform Optimization

Date: 2026-06-25
Milestone: 0106

## Summary

Milestone 0106 adds a bounded native ICCBased image color-space path. The
renderer now validates ICC profile streams, maps supported channel counts to
the existing native DeviceGray, DeviceRGB, or DeviceCMYK sample paths, and
caches validated transform metadata by stable profile identity.

This is still a thumbnail-renderer approximation. The current path does not
run a full color-management engine or build device-link transforms. It keeps
the render path deterministic, bounded, and native-only for common ICCBased
image fixtures while recording the remaining CMYK visual-parity gap.

## Implementation

- Added `IccTransformCache` with caller-owned reuse, entry-budget eviction, and
  cache hit/miss/eviction metrics.
- Added profile identity hashing based on decoded profile bytes, byte length,
  and channel count.
- Added ICC profile and transform workspace budgets to `DisplayListOptions`:
  - `max_icc_profile_bytes`: 1 MiB default.
  - `max_icc_transform_workspace_bytes`: 64 KiB default.
  - `max_icc_transform_cache_entries`: 32 entries default.
- Added ICC budget fields to native memory diagnostics.
- Added generated ICCBased image fixtures:
  - `fixtures/generated/icc-rgb-image.pdf`
  - `fixtures/generated/icc-gray-image.pdf`
  - `fixtures/generated/icc-cmyk-image.pdf`

## Evidence

Supported-family native-only gate:

- Total: 41
- Native rendered: 41
- Fallback required: 0
- Errors: 0
- Browser-print: 6/6 native rendered
- Form: 12/12 native rendered
- Office-export: 23/23 native rendered
- Artifact: `target/icc-cache-0106-supported-gate.json`

Native benchmark:

- Total fixtures: 91
- Native rendered: 85
- Fallback required: 5
- Errors: 1
- Budget failures: 6
- Scan family: 13/16 native rendered, 3 expected fallback-required codec
  fixtures, max render time 47.130 ms.
- Artifact: `target/icc-cache-0106-benchmark.json`

New ICC fixture benchmark results:

| Fixture | Native status | Mean ms | Output bytes | Budget violations |
| --- | --- | ---: | ---: | --- |
| `icc-rgb-image.pdf` | native_rendered | 0.892 | 57600 | none |
| `icc-gray-image.pdf` | native_rendered | 0.889 | 57600 | none |
| `icc-cmyk-image.pdf` | native_rendered | 1.128 | 57600 | none |

PDFium visual comparison:

- Artifact: `target/icc-cache-0106-visual-diff.json`
- `icc-rgb-image.pdf`: exact match.
- `icc-gray-image.pdf`: exact match.
- `icc-cmyk-image.pdf`: blocker, MAE 20.407, changed ratio 0.444444, p95
  channel delta 90.

The CMYK fixture confirms native execution and budget compliance, but not
visual parity. Native currently routes ICCBased CMYK through the existing
DeviceCMYK thumbnail approximation; full ICC CMYK transform fidelity remains a
future color-management slice.

## Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-render`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/icc-cache-0106-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/icc-cache-0106-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/icc-cache-0106-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Follow-Ups

- Replace DeviceCMYK fallback approximation with a real bounded ICC transform
  path when the renderer takes on color-management fidelity beyond thumbnails.
- Decide whether a longer-lived document/render-session cache should wrap
  `IccTransformCache` once multi-page document scheduling grows stateful
  renderer sessions.
- Add real-world ICC profiles to the private corpus after the privacy review
  loop lands.
