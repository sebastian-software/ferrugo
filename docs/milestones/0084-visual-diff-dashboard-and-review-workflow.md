# 0084: Visual Diff Dashboard And Review Workflow

Status: done
Phase: 14
Size: medium
Depends on: 0083

## Goal

Make native-versus-PDFium visual differences reviewable at corpus scale without
turning every small pixel delta into manual work.

## Scope

- Generate compact per-fixture diff artifacts.
- Group failures by category, severity, and changed renderer subsystem.
- Add thresholds for exact match, acceptable antialiasing drift, and blocker
  differences.
- Document the human review workflow for milestone gates.

## Non-Goals

- Build a hosted service.
- Require PDFium for native-only smoke validation.
- Hide known blocker differences behind loose thresholds.

## Deliverables

- Local visual diff report generator.
- Diff threshold policy.
- Review workflow documentation.

## Acceptance Criteria

- A developer can identify blocker diffs from one report.
- Accepted visual drift is documented by category.
- Regression artifacts are small enough for local iteration.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run native/PDFium visual diff generation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Commit `faed727`: added `ferrugo-cli visual-diff` behind the `pdfium`
  feature.
- Added per-fixture visual metrics, exact/accepted-drift/blocker
  classification, family grouping, and renderer-subsystem grouping.
- Added focused tests for visual-diff metric classification and subsystem
  assignment.
- Added threshold and review workflow documentation in
  `docs/policies/visual-diff-thresholds.md`.
- Recorded the local review evidence in
  `docs/reports/visual-diff-dashboard-2026-06-24.md`.

Validation completed:

```text
cargo fmt --check
cargo check
cargo test
cargo test -p ferrugo-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --max-edge 120 --output target/0084-visual-diff.json
```
