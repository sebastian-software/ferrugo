# 0205: PDFium-Free 1.3 Typical Document Gate

Status: todo
Phase: 38
Size: medium
Depends on: 0204

## Goal

Make the PDFium-free 1.3 decision using the expanded typical-document corpus,
native renderer scorecard, and memory/performance evidence from phase 38.

## Scope

- Run the 1.3 native-only validation matrix across supported typical-document
  families.
- Compare 1.3 coverage, fidelity, unsupported categories, memory, and throughput
  against the 1.2 readiness baseline.
- Decide release, stabilize, or defer based on measured evidence.
- Produce the next implementation backlog without relying on runtime PDFium.

## Non-Goals

- Claim complete PDF specification support.
- Hide unsupported categories behind aggregate pass rates.
- Reintroduce PDFium runtime fallback to pass the gate.

## Deliverables

- PDFium-free 1.3 typical-document report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.3 backlog.

## Acceptance Criteria

- Typical document families pass native-only gates at documented thresholds.
- Performance and memory budgets are met for supported profiles.
- Unsupported boundaries remain typed, visible, and documented for consumers.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.3 supported corpus gate.
- Run visual validation with the PDFium-free oracle strategy.
- Run benchmark, memory, WASM, and package profile checks.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
