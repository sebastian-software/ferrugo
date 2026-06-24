# Phase 0 Report

Status: completed with local PDFium build and live render measured.
Date: 2026-06-24.

Phase 0 created the measurement spine for PDF thumbnail generation: pinned
PDFium inputs, generated fixtures, a backend-neutral Rust API, a serialized
PDFium backend shell, a PNG CLI, error taxonomy, and baseline metadata.

## Summary

The Rust side is operational and validated locally. The pinned PDFium source
build also completed locally, and the PDFium backend rendered the generated
text fixture through the release CLI.

## Completed Artifacts

- License and attribution policy: `LICENSE-MIT`, `LICENSE-APACHE`,
  `docs/policies/attribution.md`.
- PDFium checkout and build inputs:
  - `docs/build/pdfium-checkout.md`
  - `docs/build/pdfium-gn-args.md`
  - PDFium revision `573758fe2dd928279cd52b5a4bc955a6938aab39`
- Measurement protocol: `docs/measurements/pdfium-build-baseline.md`.
- Fixture policy and seed PDFs:
  - `docs/fixtures.md`
  - `fixtures/generated/*.pdf`
  - `scripts/generate_fixtures.py`
- Rust workspace:
  - `crates/pdfrust-thumbnail`
  - `crates/pdfrust-pdfium`
  - `crates/pdfrust-cli`
- Error taxonomy: `docs/errors.md`.
- Baseline metadata format: `docs/baselines.md`,
  `baselines/examples/*.json`.

## Measurements

PDFium revision `573758fe2dd928279cd52b5a4bc955a6938aab39` was built locally.

- Static complete library: `out/pdfrust-thumb/obj/libpdfium.a`, 264M.
- Runtime component dylib: `out/pdfrust-dylib/libpdfium.dylib`, 5.4M plus
  colocated `@rpath` dylib dependencies.
- Runtime path:
  `/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib`.
- Smoke probe: `initialized=true`, `last_error=0`.

Release CLI render measurements for `fixtures/generated/text-page.pdf`:

| max edge | dimensions | time | max RSS |
| --- | --- | --- | --- |
| 256 | 256x137 | 0.04s real, 0.01s user, 0.02s sys | 24,313,856 bytes |
| 512 | 300x160 | 0.03s real, 0.01s user, 0.02s sys | 24,674,304 bytes |
| 1024 | 300x160 | 0.03s real, 0.01s user, 0.02s sys | 24,625,152 bytes |

`max-edge` 512 and 1024 are identical because the generated fixture page is
300x160 pixels at PDFium's default scale and the renderer does not upscale.

Available local validation:

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Fixture regeneration through `python3 scripts/generate_fixtures.py`
- JSON validation for baseline examples

## Error Behavior

The facade exposes stable classes:

- `encrypted`
- `malformed`
- `unsupported`
- `timeout`
- `internal`

PDFium error-code mapping is implemented for known `FPDF_GetLastError` codes.
Timeout remains a facade class and CLI option; enforcement around a live backend
still needs worker or process isolation.

## Risks And Blockers

- The component dylib depends on colocated `@rpath` dylibs, so packaging still
  needs a distribution decision.
- Timeout behavior needs either backend cancellation, worker isolation, or
  process isolation before it is robust enough for hostile PDFs.

## Decision

Continue with the current Rust facade and PDFium-backed measurement harness, but
gate product-facing reliability on timeout and isolation:

1. Use the current Rust facade, fixtures, CLI, and baseline format as the test
   spine.
2. Use process isolation for product-facing timeout enforcement before Node-API
   or npm packaging; see `docs/decisions/0002-timeout-and-process-isolation.md`.
3. Keep Rust-native renderer work as the long-term direction, but use the
   PDFium-backed baseline to define expected thumbnail behavior first.
