# Overprint Simulation Report

Date: 2026-06-25.
Milestone: 0110.

## Summary

Milestone 0110 adds a pragmatic native overprint approximation boundary for
print-oriented PDFs. The renderer now accepts ExtGState `/OP`, `/op`, and
`/OPM` values, validates overprint mode 0 or 1, and preserves the requested
flags on display-list graphics state instead of returning an unsupported
overprint fallback.

The current renderer still paints RGB thumbnail output through the existing
color approximation path. This keeps common overprint documents visible and
diagnosable, while avoiding a false claim of press-proof separations or full
CMYK knockout behavior.

## Implementation

- Added `fill_overprint`, `stroke_overprint`, and `overprint_mode` to
  `GraphicsState` and `ExtGraphicsState`.
- Parsed ExtGState `/op`, `/OP`, and `/OPM` with typed validation.
- Removed the hard fallback for enabled overprint flags.
- Preserved overprint flags when applying ExtGState resources to display-list
  path items.
- Added a generated overprint spot-color fixture:
  `fixtures/generated/overprint-spot-approximation.pdf`.

The fixture uses a `/Separation` spot color with a DeviceRGB alternate. An
earlier CMYK alternate would have mixed this milestone with the known 0105
CMYK spot-color visual gap, so the committed fixture isolates overprint-state
acceptance from the broader prepress color-parity work.

## Evidence

Benchmark artifact: `target/overprint-0110-benchmark.json`

- Total: 95 fixtures.
- Native rendered: 89.
- Fallback required: 5.
- Errors: 1 encrypted fixture.
- Budget failures: 6 existing fallback/error cases.

Supported-family gate artifact: `target/overprint-0110-supported-gate.json`

- Total: 41.
- Native rendered: 41.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

PDFium visual comparison artifact: `target/overprint-0110-visual-diff.json`

- Total: 95.
- Exact: 31.
- Accepted drift: 16.
- Blockers: 42.
- Native errors: 5.
- PDFium errors: 0.
- Both errors: 1 encrypted fixture.

New fixture result:

| Fixture | Status | MAE | Changed Ratio | p95 | Max Delta | Notes |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `overprint-spot-approximation.pdf` | accepted drift | 0.042 | 0.000278 | 0 | 242 | Native and PDFium both render 14,400 non-white pixels; only four pixels differ. |

The remaining visual blockers are existing corpus gaps in text/font rendering,
form synthesis, page geometry, CMYK/spot-color conversion, transparency alpha,
and other renderer areas. The new overprint fixture is not a blocker.

## Validation Commands

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p pdfrust-render`
- `cargo test -p pdfrust-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/overprint-0110-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/overprint-0110-supported-gate.json`
- `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium/out/pdfrust-dylib:/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/overprint-0110-visual-diff.json`

## Follow-Ups

- Add true device-separation overprint compositing only after real print corpus
  evidence justifies the complexity.
- Keep CMYK spot-color visual parity tracked under the color/prepress
  milestones instead of folding it into this approximation slice.
- Use the preserved display-list overprint flags for future diagnostics and
  policy decisions.
