# XFA Fallback Policy Report

Date: 2026-06-25.
Milestone: 0111.

## Summary

Milestone 0111 defines the native renderer boundary for XFA and dynamic-form
PDFs. The native path now detects `/AcroForm /XFA` before page rendering. XFA
hybrids with static AcroForm fields continue through the existing static
appearance paths, while XFA-only dynamic forms return the unsupported bucket
`form.xfa-dynamic`.

No XFA packet is decoded to build layout or field values, and no JavaScript or
XFA runtime behavior is introduced.

## Implementation

- Added private native unsupported bucket `form.xfa-dynamic`.
- Added early render policy detection for catalog `/AcroForm /XFA`.
- Allowed XFA hybrids when `/Fields` is present and non-empty.
- Rejected XFA documents without static fields before rendering.
- Added generated fixtures:
  - `fixtures/generated/xfa-static-appearance.pdf`
  - `fixtures/generated/xfa-dynamic-no-static-appearance.pdf`

The static fixture includes an AcroForm widget appearance and matching static
page content. This keeps the local no-XFA PDFium oracle visually useful while
still proving that the native XFA policy does not block static hybrid output.

## Evidence

Benchmark artifact: `target/xfa-0111-benchmark.json`

- Total: 97 fixtures.
- Native rendered: 90.
- Fallback required: 6.
- Errors: 1 encrypted fixture.
- Budget failures: 7 existing fallback/error cases, including the expected
  dynamic XFA unsupported boundary.

Supported-family gate artifact: `target/xfa-0111-supported-gate.json`

- Total: 42.
- Native rendered: 42.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

PDFium visual comparison artifact: `target/xfa-0111-visual-diff.json`

- Total: 97.
- Exact: 31.
- Accepted drift: 17.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1 encrypted fixture.

XFA fixture results:

| Fixture | Family | Status | MAE | Changed Ratio | p95 | Max Delta | Notes |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| `xfa-static-appearance.pdf` | `form` | accepted drift | 0.011 | 0.000179 | 0 | 64 | Native and PDFium both render 1,200 non-white pixels. |
| `xfa-dynamic-no-static-appearance.pdf` | `mixed-layout` | native error | n/a | n/a | n/a | n/a | Expected `unsupported` with bucket `form.xfa-dynamic`. |

The remaining visual blockers are existing corpus gaps in text/font rendering,
form synthesis, page geometry, CMYK/spot-color conversion, transparency alpha,
and other renderer areas. The static XFA fixture is not a blocker.

## Validation Commands

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-native xfa -- --nocapture`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/xfa-0111-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/xfa-0111-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/xfa-0111-visual-diff.json`

## Follow-Ups

- Keep XFA runtime execution out of the native thumbnail renderer.
- Revisit hybrid detection only if real corpus files show static fields hidden
  behind non-standard AcroForm structures.
- Use later government-form and business-form corpus gates to decide whether
  additional static XFA classification is worth supporting.
