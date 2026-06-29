# Spot Color Approximation

Date: 2026-06-25
Milestone: 0105

## Summary

Milestone 0105 adds a bounded native approximation path for `/Separation` and
`/DeviceN` color spaces used by print-oriented PDFs. The renderer now resolves
page `/ColorSpace` resources for path and Form XObject content, evaluates Type
2 tint transforms, converts supported alternate DeviceGray, DeviceRGB, and
DeviceCMYK spaces to RGB, and records spot-color approximation metadata on the
captured `DeviceColor`.

This is thumbnail output, not press proofing. The native path intentionally
does not expose separations, overprint simulation, or unbounded sampled/function
evaluation.

## Implementation

- Added `ColorSpaceResources` for page `/ColorSpace` resource dictionaries.
- Added `/Separation` and `/DeviceN` parsing with supported alternate spaces:
  `/DeviceGray`, `/DeviceRGB`, and `/DeviceCMYK`.
- Added bounded Type 2 tint-transform evaluation with at most four output
  components.
- Added `DeviceColor::Spot` with `SpotColorApproximation` metadata for
  diagnostics and reports.
- Added `cs`/`CS` and `sc`/`scn`/`SC`/`SCN` support for spot-color fill and
  stroke paths.
- Added generated fixtures:
  - `fixtures/generated/separation-spot-color.pdf`
  - `fixtures/generated/devicen-spot-color.pdf`

## Evidence

Supported-family native-only gate:

- Total: 41
- Native rendered: 41
- Fallback required: 0
- Errors: 0
- Browser-print: 6/6 native rendered
- Form: 12/12 native rendered
- Office-export: 23/23 native rendered
- Artifact: `target/spot-color-0105-supported-gate.json`

PDFium visual comparison:

- Total: 88
- Exact: 28
- Accepted drift: 6
- Blockers: 48
- Native errors: 5
- PDFium errors: 0
- Both errors: 1
- Artifact: `target/spot-color-0105-visual-diff.json`

New spot-color fixtures render natively without fallback or native errors.
They remain visual blockers in the PDFium comparison because this milestone
targets predictable RGB thumbnail approximation, not proofing parity.

| Fixture | Status | MAE | Changed Ratio | p95 | Notes |
| --- | --- | ---: | ---: | ---: | --- |
| `devicen-spot-color.pdf` | blocker | 24.391 | 0.319444 | 127 | Native paints the Type 2 DeviceN approximation; PDFium rendered the synthetic fixture white. |
| `separation-spot-color.pdf` | blocker | 8.638 | 0.428333 | 33 | Native and PDFium both paint the fixture, but RGB approximation differs. |

## Validation

- `cargo fmt --check`
- `cargo check -p ferrugo-render -p ferrugo-native`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-render`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/spot-color-0105-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 1.0 --max-p95 8 --max-changed-ratio 0.02 --output target/spot-color-0105-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Follow-Ups

- Add Type 0 sampled function support for spot tint transforms if real corpus
  fixtures need it.
- Add function-resource indirection once page color-space functions appear in
  non-synthetic documents.
- Revisit visual thresholds only after 0110 overprint simulation and later
  print-oriented corpus gates.
