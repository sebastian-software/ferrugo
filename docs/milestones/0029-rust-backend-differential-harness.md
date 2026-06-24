# 0029: Rust Backend Differential Harness

Status: todo
Phase: 1
Size: medium
Depends on: 0028

## Goal

Create the test harness that compares Rust-native document behavior against the
PDFium oracle.

## Scope

- Add a Rust-native backend placeholder behind the thumbnail facade.
- Add a comparison command or test helper for fixture metadata.
- Compare page count, page size, error class, and later pixel output.
- Store comparison results in the existing baseline format.

## Non-Goals

- Render pixels with the Rust backend.
- Require PDFium in normal unit tests.
- Support the full real-world corpus in CI.

## Deliverables

- Rust-native backend adapter skeleton.
- Differential comparison harness.
- Baseline examples for metadata-only comparisons.

## Acceptance Criteria

- Generated fixtures can be compared against PDFium for page metadata.
- Mismatches produce actionable diagnostics.
- The harness can run without committing large rendered artifacts.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run one live comparison against the local PDFium dylib.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
