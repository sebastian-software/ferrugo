# 0109: Transparency Isolation Knockout And Luminosity Masks

Status: done
Phase: 19
Size: medium
Depends on: 0108

## Goal

Close remaining transparency gaps for isolated groups, knockout groups, and
luminosity soft masks in typical office and design PDFs.

## Scope

- Implement isolated and knockout group compositing rules.
- Support alpha and luminosity soft mask conversion.
- Keep intermediate surfaces bounded by page and group dimensions.
- Add fixtures with shadows, overlays, watermarks, and masked images.

## Non-Goals

- Implement print-production transparency flattening.
- Allocate full-resolution group surfaces when thumbnails can be clipped.
- Hide unsupported blend interactions.

## Deliverables

- Transparency group compositing updates.
- Surface allocation metrics.
- Visual comparison report for transparency fixtures.

## Acceptance Criteria

- Common transparency groups render without PDFium fallback.
- Intermediate surfaces are clipped and memory-bounded.
- Unsupported transparency cases return typed reasons with fixture evidence.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run transparency visual comparisons.
- Run memory-budget stress fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Commit `d814920`: preserved alpha on transparency group intermediate surfaces
  and applied caller graphics-state alpha/blend mode during final group
  compositing.
- Group rasterization now runs the full nested display list on the bounded
  intermediate surface instead of path-only rendering.
- Added `fixtures/generated/transparency-knockout-group.pdf` to cover inherited
  ExtGState alpha inside an isolated `/K true` transparency group.
- The `/K true` fixture follows the local PDFium oracle: overlap remains normal
  semi-transparent group composition with low-amplitude drift, not a divergent
  hard-knockout interpretation.
- Report:
  `docs/reports/transparency-group-alpha-2026-06-25.md`.

Validation:

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-render`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/transparency-0109-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/transparency-0109-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/transparency-0109-visual-diff.json`
