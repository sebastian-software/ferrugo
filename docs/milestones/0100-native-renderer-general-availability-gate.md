# 0100: Native Renderer General Availability Gate

Status: todo
Phase: 17
Size: medium
Depends on: 0099

## Goal

Decide whether the Rust renderer can be declared generally available for the
targeted typical-document surface without PDFium as a normal dependency.

## Scope

- Run native-only tests, corpus gates, benchmarks, fuzz smoke checks, and package
  validation.
- Compare supported-category output against the latest PDFium-enabled baseline.
- Review remaining unsupported categories, fallback policy, and rollback plan.
- Produce the GA decision and post-GA maintenance backlog.

## Non-Goals

- Claim full PDF specification coverage.
- Remove maintainer-only PDFium comparison infrastructure.
- Ship without documented unsupported and degraded categories.

## Deliverables

- Native renderer GA report.
- Go/no-go decision for PDFium-free normal operation.
- Post-GA maintenance and deletion backlog.

## Acceptance Criteria

- GA decision is based on measured fidelity, performance, memory, safety, and
  packaging evidence.
- Normal supported-document operation does not require PDFium.
- Remaining PDFium usage is maintainer-only, emergency-only, or explicitly
  scoped to unsupported categories.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run renderer benchmark suite.
- Run fuzz smoke targets.
- Run package validation.
- Run PDFium-enabled comparison baseline.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
