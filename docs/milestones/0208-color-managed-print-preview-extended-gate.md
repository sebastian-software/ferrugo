# 0208: Color Managed Print Preview Extended Gate

Status: done
Phase: 39
Size: medium
Depends on: 0207

## Goal

Extend color-managed print-preview confidence for common business, design,
government, and print-shop PDFs without depending on PDFium runtime rendering.

## Scope

- Expand fixtures for ICC output intents, CMYK, spot color approximations,
  overprint simulation, transparency, and print-preview annotation states.
- Measure color transform cache behavior and memory usage across repeated pages.
- Document known differences between screen preview and print-oriented output.
- Add diagnostics for unsupported or approximated color behavior.

## Non-Goals

- Guarantee press-proof color accuracy for every device profile.
- Implement a full prepress workflow.
- Use PDFium as a runtime print-preview renderer.

## Deliverables

- Extended color-managed print-preview corpus.
- Color fidelity and approximation report.
- Color transform cache budget update.

## Acceptance Criteria

- Common print-preview documents render within documented color tolerances.
- Spot color and overprint approximations are explicit.
- Repeated color transforms stay within cache and memory budgets.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run print-preview visual comparisons.
- Run color transform cache benchmark.
- Run approximation diagnostics snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `fixtures/color-managed-print-preview-manifest.tsv` as the 0208 gate
  for OutputIntent metadata, ICCBased images, DeviceCMYK/process color,
  registration bars, spot-color approximations, overprint approximation,
  prepress page boxes, and print-visible annotations.
- Documented the gate in `docs/corpus-taxonomy.md`.
- Added
  `native_backend_should_render_color_managed_print_preview_gate` to exercise
  the combined server-side print-preview path with `AnnotationMode::Print`.
- Produced `docs/reports/color-managed-print-preview-extended-2026-06-29.md`.

Validation run:

- `cargo fmt --check`
- `cargo test -p pdfrust-native color_managed_print_preview -- --nocapture`
- `cargo test -p pdfrust-render icc_transform_cache -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --fail-on-fallback --max-edge 180 --output target/color-print-0208-supported.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/color-print-0208-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --output target/color-print-0208-operators.json`
- `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --max-edge 120 --max-mae 10 --max-p95 72 --max-changed-ratio 0.25 --output target/color-print-0208-poppler.json`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
