# 0099: PDFium Fallback Removal Drill

Status: todo
Phase: 17
Size: medium
Depends on: 0098

## Goal

Run a controlled drill that disables PDFium fallback for supported document
categories and measures the remaining operational risk.

## Scope

- Add a native-only validation mode that treats accidental PDFium use as a
  failure.
- Run supported corpus categories through the native-only path.
- Record unsupported categories and required user-facing errors.
- Identify any fallback paths that can be deleted immediately.

## Non-Goals

- Delete comparison tooling.
- Pretend unsupported categories are supported.
- Remove emergency fallback without a rollback path.

## Deliverables

- PDFium fallback removal drill report.
- Native-only supported-category gate.
- Deletion candidates for fallback code and configuration.

## Acceptance Criteria

- Supported categories pass without invoking PDFium.
- Remaining fallback paths are justified by documented unsupported categories.
- The drill produces a clear delete, defer, or keep decision per fallback path.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run native-only supported corpus gate.
- Run PDFium-enabled comparison smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
