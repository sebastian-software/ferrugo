# 0138: Transparency And Blend Conformance Corpus

Status: done
Phase: 25
Size: medium
Depends on: 0137

## Goal

Create a focused conformance corpus for transparency groups, soft masks, alpha
constants, and blend modes used by typical generated PDFs.

## Scope

- Add fixtures for isolated groups, knockout boundaries, luminosity masks, and
  common blend modes.
- Compare native output against PDFium with documented thumbnail tolerances.
- Classify hard blockers separately from accepted anti-aliasing drift.
- Profile intermediate surface allocations and compositing cost.

## Non-Goals

- Claim full print-proof transparency conformance.
- Implement unsupported blend modes without evidence of common need.
- Ignore memory budgets for nested groups.

## Deliverables

- Transparency conformance fixture family.
- Blend and mask support matrix update.
- Compositing benchmark and memory report.

## Acceptance Criteria

- Common transparency cases render natively or fail with typed reasons.
- Nested groups stay inside intermediate-surface budgets.
- Visual-diff policy distinguishes tolerable drift from compositing errors.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run transparency conformance visual comparisons.
- Run compositing memory benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Added `fixtures/transparency-conformance-manifest.tsv`.
- Added generated conformance fixtures for isolated alpha groups, blend-mode
  arrays, unsupported Overlay blend mode, and ExtGState luminosity soft masks.
- Registered the new fixtures in `fixtures/corpus-manifest.tsv`.
- Added native backend regression tests for supported group/blend fixtures and
  typed `graphics.transparency` fallback tests for unsupported boundaries.
- Added ExtGState soft-mask classification so `/SMask /None` remains accepted
  and non-`None` ExtGState soft masks are no longer silently ignored.
- Wrote `docs/reports/transparency-blend-conformance-2026-06-26.md` and updated
  the native backend transparency/blend matrix.
- Validation:
  `cargo fmt --check`;
  `cargo test -p pdfrust-render ext_graphics_state_resources -- --nocapture`;
  `cargo test -p pdfrust-native transparency -- --nocapture`;
  `cargo test -p pdfrust-native blend_mode -- --nocapture`;
  transparency fallback gates;
  transparency benchmark;
  PDFium visual diff;
  `cargo check --workspace`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo test --workspace`;
  `cargo test --workspace --no-default-features`.

The existing `transparency-alpha.pdf` stroke-edge max-delta visual blocker
remains documented as a follow-up. ExtGState luminosity masks and Overlay blend
mode are typed unsupported boundaries for this milestone.
