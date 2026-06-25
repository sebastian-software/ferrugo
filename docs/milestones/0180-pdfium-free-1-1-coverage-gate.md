# 0180: PDFium-Free 1.1 Coverage Gate

Status: todo
Phase: 33
Size: medium
Depends on: 0179

## Goal

Make the next PDFium-free release decision using expanded typical-document
coverage, native-only validation, and explicit unsupported boundaries.

## Scope

- Run the full native-only validation matrix across the expanded corpus.
- Compare 1.1 coverage, performance, memory, and unsupported categories against
  the 1.0 readiness baseline.
- Decide whether the renderer is ready for 1.1 release, stabilization, or a
  targeted deferral.
- Produce the next implementation backlog from measured gaps.

## Non-Goals

- Claim complete PDF specification support.
- Reintroduce PDFium runtime fallback to pass the gate.
- Ignore documented unsupported cases that affect typical documents.

## Deliverables

- PDFium-free 1.1 coverage report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.1 backlog.

## Acceptance Criteria

- Expanded typical-document families pass native-only gates at documented
  thresholds.
- Performance and memory budgets are measured and acceptable for supported
  workflows.
- Unsupported boundaries remain typed, documented, and visible to consumers.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run visual comparison with the selected PDFium-free oracle strategy.
- Run benchmark and memory profiles.
- Run package dry-runs.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
