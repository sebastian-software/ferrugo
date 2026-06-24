# 0119: Cross-Platform Rendering Determinism Gate

Status: todo
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

Empty until done.
