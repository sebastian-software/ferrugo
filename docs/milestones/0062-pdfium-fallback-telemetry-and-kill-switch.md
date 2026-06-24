# 0062: PDFium Fallback Telemetry And Kill Switch

Status: todo
Phase: 9
Size: small
Depends on: 0061

## Goal

Measure and control every remaining PDFium fallback so replacement progress is
visible and reversible.

## Scope

- Emit structured fallback reasons from automatic backend selection.
- Add a strict native-only mode for CI and release validation.
- Add a fallback deny-list or kill switch for targeted experiments.
- Track fallback counts by unsupported feature category.

## Non-Goals

- Upload telemetry to a remote service.
- Change renderer output quality.
- Remove fallback paths before evidence supports it.

## Deliverables

- Fallback reason taxonomy.
- Native-only validation mode.
- Documentation for interpreting fallback diagnostics.

## Acceptance Criteria

- Every automatic PDFium fallback has a typed reason.
- CI can fail when native-only rendering regresses on supported categories.
- Local corpus runs summarize fallback volume by category.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run native-only fixture validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
