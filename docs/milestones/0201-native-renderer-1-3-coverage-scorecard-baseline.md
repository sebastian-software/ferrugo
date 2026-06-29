# 0201: Native Renderer 1.3 Coverage Scorecard Baseline

Status: done
Phase: 38
Size: medium
Depends on: 0200

## Goal

Create the 1.3 coverage scorecard that makes the next Rust-native renderer
work measurable across typical document families, unsupported boundaries, and
runtime profiles.

## Scope

- Consolidate native coverage from office, browser, scanner, report, form, and
  print-oriented corpus families.
- Track pass, partial, unsupported, timeout, memory-budget, and visual-drift
  outcomes separately.
- Weight gaps by typical-document impact instead of raw PDF feature count.
- Produce a ranked 1.3 implementation queue that excludes runtime PDFium.

## Non-Goals

- Reopen PDFium as a supported runtime renderer.
- Claim specification-complete rendering.
- Replace detailed per-feature reports with a single aggregate number.

## Deliverables

- Native renderer 1.3 coverage scorecard.
- Weighted gap list by document family and feature category.
- Updated validation thresholds for the 1.3 milestone group.

## Acceptance Criteria

- Every supported and near-supported corpus family has an explicit score.
- Unsupported cases are typed and linked to follow-up milestones.
- Runtime PDFium usage remains outside the supported path.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus coverage scan.
- Run unsupported-category snapshot checks.
- Run benchmark and memory summary generation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `scripts/generate_coverage_scorecard.sh`.
- Produced `docs/reports/native-renderer-1-3-coverage-scorecard-2026-06-29.md`.
- Baseline weighted score: `94.04`.
- Baseline family blocker: `presentation` scores `86.09`, below the proposed
  `88.00` per-family threshold.
- Top weighted gaps: `image.filter`, `graphics.optional-content`,
  `graphics.transparency`, `graphics.color-management`, and
  `graphics.pattern-shading`.
- Runtime PDFium remains excluded from supported 1.3 coverage.
- Validation:
  - `scripts/generate_coverage_scorecard.sh target/coverage-scorecard-0201`
  - `cargo check --workspace --no-default-features`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
