# 0108: Advanced Shading Mesh Tessellation

Status: done
Phase: 19
Size: medium
Depends on: 0107

## Goal

Improve native rendering for smooth gradients and mesh shadings found in
presentations, charts, and design-heavy business PDFs.

## Scope

- Implement bounded tessellation for common mesh shading types.
- Reuse gradient sampling buffers across shading patches.
- Add quality knobs tied to thumbnail dimensions.
- Add fixtures for axial, radial, and mesh gradient documents.

## Non-Goals

- Match PDFium at arbitrary zoom levels.
- Add GPU rendering.
- Allow shading tessellation to exceed page memory budgets.

## Deliverables

- Mesh shading tessellation path.
- Performance and visual-diff report.
- Shading quality and budget documentation.

## Acceptance Criteria

- Common gradient PDFs render natively with acceptable drift.
- Tessellation work scales with output resolution, not source complexity alone.
- Oversized shadings are budgeted and reported deterministically.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run shading fixture comparisons.
- Run renderer benchmarks for gradient-heavy PDFs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Commit `f4c6a5b`: added native Type 4 free-form Gouraud mesh shading support
  for bounded 8-bit coordinate/component meshes with explicit flag-0 triangle
  records.
- Added `fixtures/generated/type4-mesh-shading.pdf` and manifest coverage for a
  report-family mesh fixture.
- Added byte and triangle budgets:
  `DisplayListOptions::max_mesh_shading_bytes` and
  `DisplayListOptions::max_mesh_shading_triangles`.
- Preserved the existing unsupported mesh fixture as a typed fallback boundary
  for mesh features outside this slice.
- Report:
  `docs/reports/mesh-shading-tessellation-2026-06-25.md`.

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
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/shading-0108-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/shading-0108-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/shading-0108-visual-diff.json`
