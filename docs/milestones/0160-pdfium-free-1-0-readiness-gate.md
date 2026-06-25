# 0160: PDFium-Free 1.0 Readiness Gate

Status: todo
Phase: 29
Size: medium
Depends on: 0159

## Goal

Make the release decision for a PDFium-free Rust-native renderer that covers a
large share of typical documents with explicit unsupported boundaries.

## Scope

- Run the full native-only validation matrix.
- Summarize document-family coverage, known unsupported categories, and risks.
- Verify packaging, API, memory, security, and performance evidence.
- Produce the 1.0 release, stabilization, or defer recommendation.

## Non-Goals

- Claim complete PDF specification support.
- Hide known unsupported cases behind broad marketing language.
- Reintroduce PDFium as a runtime dependency to pass the gate.

## Deliverables

- PDFium-free 1.0 readiness report.
- Release/defer decision with evidence.
- Final blocker or stabilization backlog.

## Acceptance Criteria

- Supported document families pass native-only gates with documented thresholds.
- Runtime PDFium dependency remains absent.
- Remaining unsupported behavior is explicit, typed, and acceptable for the
  release decision.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus visual comparison.
- Run benchmark and memory profiles.
- Run package dry-runs.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
