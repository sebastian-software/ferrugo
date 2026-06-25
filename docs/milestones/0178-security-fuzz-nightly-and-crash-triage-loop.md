# 0178: Security Fuzz Nightly And Crash Triage Loop

Status: todo
Phase: 33
Size: medium
Depends on: 0177

## Goal

Turn parser and renderer fuzzing into a repeatable maintenance loop with clear
triage rules for crashes, panics, timeouts, and excessive allocation.

## Scope

- Define nightly or local fuzz targets for syntax, object loading, content
  streams, image decoding, and raster paths.
- Add crash artifact minimization and triage documentation.
- Classify findings by security risk, correctness risk, and unsupported input.
- Keep fuzz findings tied to typed errors and bounded resource behavior.

## Non-Goals

- Treat every malformed input as renderable.
- Add unsafe optimizations in fuzz-touched paths without invariants.
- Publish private crash corpora without review.

## Deliverables

- Fuzz target matrix.
- Crash triage workflow.
- Resource-exhaustion regression tests for resolved findings.

## Acceptance Criteria

- Fuzz targets cover major parser and render boundaries.
- Panics, crashes, and uncontrolled allocations have triage paths.
- Resolved findings become regression tests or documented unsupported cases.

## Validation

- Run fuzz smoke suite.
- Run native-only `cargo test`.
- Run adversarial corpus classification.
- Run memory budget tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
