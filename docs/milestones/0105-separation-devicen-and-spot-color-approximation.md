# 0105: Separation DeviceN And Spot Color Approximation

Status: todo
Phase: 19
Size: medium
Depends on: 0104

## Goal

Render common print-oriented PDFs that use Separation, DeviceN, and spot-color
alternate spaces with predictable RGB thumbnail output.

## Scope

- Parse Separation and DeviceN color spaces.
- Apply alternate tint transforms with bounded sampled evaluation.
- Record when spot-color output is an approximation.
- Add fixtures from print reports, presentations, and marketing exports.

## Non-Goals

- Produce press-ready separations.
- Implement full color proofing workflows.
- Support unbounded function sampling.

## Deliverables

- Native spot-color approximation path.
- Color-space policy updates.
- Visual comparison report for print-oriented fixtures.

## Acceptance Criteria

- Typical spot-color PDFs render without PDFium fallback.
- Approximation status is visible in diagnostics and reports.
- Function evaluation is bounded by memory and time budgets.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run spot-color fixture comparisons.
- Run color-function budget tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
