# 0208: Color Managed Print Preview Extended Gate

Status: todo
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

Empty until done.
