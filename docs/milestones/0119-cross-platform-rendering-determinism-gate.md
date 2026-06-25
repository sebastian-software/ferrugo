# 0119: Cross-Platform Rendering Determinism Gate

Status: done
Phase: 21
Size: medium
Depends on: 0118

## Goal

Measure and control native rendering differences across supported platforms and
CPU architectures.

## Scope

- Define deterministic output tolerances for Linux, macOS, and supported CPUs.
- Record platform metadata in benchmark and visual-diff reports.
- Add fixtures that stress fonts, color conversion, and raster edge cases.
- Document accepted drift and hard blockers.

## Non-Goals

- Require byte-identical output for every anti-aliased edge.
- Support untested platforms as first-class targets.
- Hide platform-specific unsupported categories.

## Deliverables

- Cross-platform visual-diff gate.
- Platform metadata in reports.
- Determinism policy and blocker list.

## Acceptance Criteria

- Supported platforms produce output within documented tolerances.
- Platform-specific drift is visible in reports.
- Determinism failures block native-only release candidates.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run visual-diff corpus gates on supported platforms.
- Run renderer benchmarks with platform metadata.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added target platform metadata to native/PDFium benchmark reports and
  PDFium visual-diff reports.
- Added `docs/policies/cross-platform-determinism.md` to define artifact
  requirements, default tolerances, and hard blockers for native-only release
  candidates.
- Updated benchmark and visual-diff policy docs to describe the platform
  metadata contract.
- Local macOS/aarch64 supported-family native gate passed with 46/46 native
  renders, 0 fallbacks, and 0 errors.
- Local macOS/aarch64 benchmark reported 106 fixtures, 99 native renders, 6
  fallbacks, 1 error, and 7 budget failures; supported families had no budget
  failures.
- Local PDFium visual-diff reported 106 fixtures, 35 exact, 22 accepted drift,
  42 blockers, 6 native errors, 0 PDFium errors, and 1 both-error case.
- Cross-platform readiness remains gated on additional Linux/target artifacts;
  missing platform artifacts are now documented release-candidate blockers.
- Report: `docs/reports/cross-platform-determinism-gate-2026-06-25.md`.
- Implementation commit: `fb149f6 feat: add platform metadata to renderer reports`.
