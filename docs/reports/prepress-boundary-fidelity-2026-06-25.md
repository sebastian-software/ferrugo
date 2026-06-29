# Prepress Boundary Fidelity 2026-06-25

Milestone: 0128.

## Decision

Print-production boundary thumbnails now have a focused native gate. The
native renderer renders all eight prepress manifest rows without PDFium
fallback, errors, or benchmark budget failures.

This milestone defines the boundary rather than claiming print-proofing
support. CropBox selects the visible thumbnail box, BleedBox and TrimBox are
context only, OutputIntents are accepted as metadata/context, and spot
colors/overprint remain visual thumbnail approximations.

PDFium remains a maintainer-only visual oracle. The prepress oracle run used
explicit thumbnail-oriented thresholds because printer marks, process-color
blocks, and overprint approximations have higher expected antialiasing and
color drift than the default strict review gate.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `prepress-trim-bleed-marks.pdf` | trim/bleed | CropBox, BleedBox, TrimBox, visible trim and bleed marks |
| `prepress-output-intent-page-boxes.pdf` | output intent | catalog OutputIntent, CropBox, BleedBox, TrimBox, colored blocks |
| `prepress-registration-color-bars.pdf` | registration | registration targets and process color bars |
| `prepress-spot-overprint-boundary.pdf` | spot/overprint | Separation color space and overprint approximation boundary |

`fixtures/prepress-boundary-manifest.tsv` combines these with existing
CropBox, OutputIntent, stroke mark, and overprint/spot-color baselines.

## Boundary Policy

Policy: `docs/policies/prepress-boundary.md`

- CropBox is the selected visible page box for thumbnails when present;
  MediaBox is used otherwise.
- BleedBox and TrimBox are accepted as context but do not select the thumbnail
  boundary yet.
- OutputIntents are accepted as metadata/context and do not imply
  color-managed proofing.
- Separation, DeviceN, spot colors, and overprint remain thumbnail
  approximations.
- Printer marks render as normal vector content when they are inside the
  selected visible page box.

## Native Gate Evidence

Artifact: `target/prepress-0128-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `trim-bleed` | 2 | 2 | 0 | 0 |
| `output-intent` | 2 | 2 | 0 | 0 |
| `registration` | 2 | 2 | 0 | 0 |
| `spot-overprint` | 2 | 2 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

The native regression test also checks visible non-background pixel counts so
printer marks, color bars, and spot/overprint proxy content cannot silently
collapse to empty output.

## Page Box Evidence

The native metadata test inspects `prepress-output-intent-page-boxes.pdf` and
verifies one page with a first-page size of `300.0 x 220.0`, derived from the
declared CropBox. The underlying MediaBox is larger, while BleedBox and TrimBox
remain boundary context.

## Benchmark Evidence

Artifact: `target/prepress-0128-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `trim-bleed` | 2 | 2 | 77.325 | 103.265 | 0 |
| `output-intent` | 2 | 2 | 30.472 | 54.598 | 0 |
| `registration` | 2 | 2 | 15.291 | 28.761 | 0 |
| `spot-overprint` | 2 | 2 | 34.167 | 47.031 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/prepress-0128-visual-diff.json`

Thresholds: `--max-mae 6.5 --max-p95 42 --max-changed-ratio 0.13`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `trim-bleed` | 2 | 1 | 1 | 0 | 0 | 0 |
| `output-intent` | 2 | 1 | 1 | 0 | 0 | 0 |
| `registration` | 2 | 0 | 2 | 0 | 0 | 0 |
| `spot-overprint` | 2 | 0 | 2 | 0 | 0 | 0 |
| **Total** | **8** | **2** | **6** | **0** | **0** | **0** |

The broader thresholds are local to this prepress thumbnail boundary. They
capture expected drift from antialiasing, page-box clipping of printer marks,
and color approximation. They do not redefine global visual-diff defaults and
do not claim proofing-level color or overprint parity.

## Follow-Up Backlog

- Add explicit page-box selection modes if consumers need MediaBox, CropBox,
  BleedBox, TrimBox, or ArtBox previews.
- Add sanitized producer-derived print-shop PDFs when redistribution is
  cleared.
- Improve spot-color and overprint diagnostics so approximation decisions are
  visible in debug artifacts.
- Keep proofing and preflight validation out of the thumbnail renderer unless
  a future API explicitly scopes that responsibility.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/prepress-boundary-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p ferrugo-native prepress -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/prepress-boundary-manifest.tsv --include-family trim-bleed --include-family output-intent --include-family registration --include-family spot-overprint --fail-on-fallback --max-edge 160 --output target/prepress-0128-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/prepress-boundary-manifest.tsv --include-family trim-bleed --include-family output-intent --include-family registration --include-family spot-overprint --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/prepress-0128-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/prepress-boundary-manifest.tsv --include-family trim-bleed --include-family output-intent --include-family registration --include-family spot-overprint --max-edge 160 --max-mae 6.5 --max-p95 42 --max-changed-ratio 0.13 --output target/prepress-0128-visual-diff.json
```
