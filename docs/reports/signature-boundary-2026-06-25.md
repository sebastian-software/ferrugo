# Digital Signature Boundary Report

Date: 2026-06-25.
Milestone: 0112.

## Summary

Milestone 0112 keeps digital signatures in the thumbnail renderer as a visual
and metadata boundary. Visible signature widgets render through the existing
static AcroForm appearance path. Metadata now exposes presence-only signature
signals, but the renderer does not validate certificates, hash byte ranges, or
interpret signature contents.

## Implementation

- Added `DocumentStructure::has_signature_fields`.
- Added `DocumentStructure::has_signature_byte_range`.
- Added bounded metadata scanning for AcroForm signature fields.
- Added generated fixture
  `fixtures/generated/digital-signature-appearance.pdf`.
- Updated CLI metadata JSON to include the new presence-only fields.

The metadata scan walks the AcroForm `/Fields` array up to the existing
metadata-budget style limit and checks only dictionary structure. It does not
decode `/Contents`, verify `/ByteRange`, inspect certificates, or mutate the
document.

## Evidence

Benchmark artifact: `target/signature-0112-benchmark.json`

- Total: 98 fixtures.
- Native rendered: 91.
- Fallback required: 6.
- Errors: 1 encrypted fixture.
- Budget failures: 7 existing fallback/error cases.

Supported-family gate artifact: `target/signature-0112-supported-gate.json`

- Total: 43.
- Native rendered: 43.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

PDFium visual comparison artifact: `target/signature-0112-visual-diff.json`

- Total: 98.
- Exact: 31.
- Accepted drift: 18.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1 encrypted fixture.

New fixture result:

| Fixture | Family | Status | MAE | Changed Ratio | p95 | Max Delta | Notes |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| `digital-signature-appearance.pdf` | `form` | accepted drift | 0.137 | 0.011944 | 0 | 60 | Native and PDFium both render 3,000 non-white pixels. |

The remaining visual blockers are existing corpus gaps in text/font rendering,
form synthesis, page geometry, CMYK/spot-color conversion, transparency alpha,
and other renderer areas. The new signature fixture is not a blocker.

## Validation Commands

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-native signature -- --nocapture`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/signature-0112-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/signature-0112-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/signature-0112-visual-diff.json`

## Follow-Ups

- Keep cryptographic signature validation out of the native thumbnail path.
- Let later e-signature corpus milestones expand visible signature, stamp, and
  audit-page coverage.
- If callers need real validation, expose it as a separate API with explicit
  security and certificate-chain semantics.
