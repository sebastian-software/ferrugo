# 0110: Overprint Simulation For Print-Oriented PDFs

Status: done
Phase: 19
Size: medium
Depends on: 0109

## Goal

Provide a pragmatic native overprint simulation path for print-oriented PDFs
whose thumbnails should remain visually useful.

## Scope

- Interpret overprint graphics state flags.
- Simulate common overprint behavior in RGB output.
- Record approximation status in diagnostics.
- Add fixtures with separations, spot colors, and knockout interactions.

## Non-Goals

- Guarantee press-proof color accuracy.
- Implement full prepress validation.
- Make overprint simulation the default without measurable evidence.

## Deliverables

- Overprint simulation implementation or guarded policy fallback.
- Approximation diagnostics.
- Visual comparison report for print-oriented PDFs.

## Acceptance Criteria

- Typical overprint documents produce useful thumbnails or explicit fallback.
- Approximation status is visible to callers.
- Simulation does not regress non-overprint documents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run overprint fixture comparisons.
- Run color regression corpus checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Commit `f16569b` accepts `/OP`, `/op`, and `/OPM` in ExtGState resources as a
  bounded RGB-thumbnail approximation instead of treating enabled overprint as
  a hard native fallback.
- Graphics-state overprint flags are preserved on display-list path items so
  diagnostics and future renderer policy can distinguish approximated overprint
  content from direct RGB/spot-color content.
- Added `fixtures/generated/overprint-spot-approximation.pdf`, generated from
  `scripts/generate_fixtures.py` and registered in `fixtures/corpus-manifest.tsv`.
- The fixture uses a `/Separation` spot color with a DeviceRGB alternate. This
  keeps the milestone focused on overprint-state acceptance and avoids
  duplicating the existing CMYK spot-color visual-parity gap from 0105.
- Corpus benchmark artifact: `target/overprint-0110-benchmark.json`.
- Supported-family gate artifact: `target/overprint-0110-supported-gate.json`.
- PDFium visual-diff artifact: `target/overprint-0110-visual-diff.json`.
- Report: `docs/reports/overprint-simulation-2026-06-25.md`.
