# 0084: Visual Diff Dashboard And Review Workflow

Status: todo
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

Empty until done.
