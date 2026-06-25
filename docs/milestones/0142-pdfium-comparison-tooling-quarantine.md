# 0142: PDFium Comparison Tooling Quarantine

Status: done
Phase: 26
Size: medium
Depends on: 0141

## Goal

Move remaining PDFium usage into a quarantined maintainer tooling boundary so
normal consumers cannot accidentally depend on it.

## Scope

- Split comparison binaries, scripts, and feature flags from runtime crates.
- Add naming and docs that distinguish oracle comparison from production render.
- Verify package manifests do not expose PDFium by default.
- Add CI checks that fail on new runtime PDFium references.

## Non-Goals

- Rebuild visual comparison from scratch in this milestone.
- Remove the ability to compare against PDFium for debugging.
- Add new PDFium capabilities.

## Deliverables

- Quarantined comparison-tool layout.
- Reference scan or lint for forbidden runtime PDFium imports.
- Maintainer documentation for running oracle comparisons.

## Acceptance Criteria

- Runtime crates remain PDFium-free by default.
- Any PDFium reference has an explicit maintainer-tooling justification.
- A regression check catches accidental runtime reintroduction.

## Validation

- Run native-only `cargo check`.
- Run all feature `cargo check`.
- Run native-only `cargo test`.
- Run the forbidden-reference check.
- Run one maintainer comparison job with the opt-in feature.

## Completion Notes

- Added `scripts/check_pdfium_quarantine.sh` to guard the native-only CLI graph
  and runtime crates against accidental PDFium reintroduction.
- Kept PDFium commands available only as explicit maintainer tooling behind the
  `pdfium` feature.
- Guarded the private `render-worker` command with the internal
  `PDFRUST_PDFIUM_RENDER_WORKER` marker; direct CLI invocation now fails with a
  usage error.
- Verified one opt-in PDFium `visual-diff` oracle job still runs and reports
  `accepted_drift` with no blockers.
- Report: `docs/reports/pdfium-comparison-tooling-quarantine-2026-06-26.md`.
