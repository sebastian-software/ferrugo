# 0097: Fuzzing And Adversarial PDF Hardening

Status: todo
Phase: 16
Size: medium
Depends on: 0096

## Goal

Increase confidence that native parsing and rendering fail safely on malformed
or adversarial PDFs.

## Scope

- Add fuzz targets for primitive parsing, xref loading, stream decoding, content
  tokenization, and page rendering setup.
- Add regression fixtures for discovered crashes, panics, and excessive work.
- Enforce recursion, allocation, stream, and operator budgets.
- Document how to run smoke fuzzing locally.

## Non-Goals

- Make fuzzing mandatory for every local test run.
- Treat all malformed PDFs as recoverable.
- Hide panics behind broad catch-all errors without fixing root causes.

## Deliverables

- Fuzz target suite.
- Crash regression corpus.
- Adversarial PDF hardening report.

## Acceptance Criteria

- Fuzz smoke runs find no panics in targeted parser and render setup paths.
- Known adversarial inputs fail with stable errors.
- Resource exhaustion limits are covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run fuzz smoke targets.
- Run malformed corpus checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
