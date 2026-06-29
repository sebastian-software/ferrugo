# 0085: Page Geometry Boxes Rotation And User Units

Status: done
Phase: 15
Size: medium
Depends on: 0084

## Goal

Match common PDF page geometry behavior across media boxes, crop boxes,
rotation, and user units.

## Scope

- Implement effective page box selection for rendering.
- Handle page rotation and non-default user units consistently.
- Add fixtures for rotated office exports, cropped scans, and unusual page
  sizes.
- Keep raster dimensions bounded by thumbnail options and memory policy.

## Non-Goals

- Add print imposition support.
- Render outside the selected page box.
- Accept unbounded page dimensions.

## Deliverables

- Page geometry implementation updates.
- Geometry fixture set.
- Native/PDFium comparison report.

## Acceptance Criteria

- Rendered thumbnail dimensions match the selected page geometry.
- Rotation and crop behavior match PDFium for typical documents.
- Path, image, and text placement remain stable after geometry transforms.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run geometry-focused corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Commit `fbfb703`: added object-layer parsing for `/Rotate` and `/UserUnit`,
  native geometry wiring, and generated geometry fixtures.
- Added fixtures:
  - `fixtures/generated/rotated-office-export.pdf`
  - `fixtures/generated/cropped-scan-page.pdf`
  - `fixtures/generated/user-unit-page.pdf`
- Recorded native/PDFium comparison evidence in
  `docs/reports/page-geometry-coverage-2026-06-24.md`.

Validation completed:

```text
cargo fmt --check
cargo check
cargo test
cargo test -p ferrugo-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/0085-geometry-visual-diff.json
```

Note: full `git diff --check` still reports pre-existing trailing whitespace in
the unstaged `.gitignore` change. The 0085 touched-file diff check passed.
