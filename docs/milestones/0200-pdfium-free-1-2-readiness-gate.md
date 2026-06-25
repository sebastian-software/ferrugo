# 0200: PDFium-Free 1.2 Readiness Gate

Status: todo
Phase: 37
Size: medium
Depends on: 0199

## Goal

Make the PDFium-free 1.2 release decision using expanded document-family
coverage, native-only packaging, performance budgets, and explicit unsupported
boundaries.

## Scope

- Run the complete native-only validation matrix for the 1.2 corpus.
- Compare coverage, fidelity, memory, startup, and throughput against 1.1.
- Verify runtime PDFium remains absent from supported packages.
- Decide release, stabilize, or defer based on measured evidence.

## Non-Goals

- Claim complete PDF specification support.
- Reopen PDFium runtime fallback to pass the gate.
- Ship without documented unsupported boundaries.

## Deliverables

- PDFium-free 1.2 readiness report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.2 backlog.

## Acceptance Criteria

- Typical-document families pass native-only release gates.
- Packaging and deployment profiles remain PDFium-free.
- Performance, memory, and fidelity budgets are documented and acceptable.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run visual validation using the PDFium-free oracle strategy.
- Run benchmark, memory, server, and package profile checks.
- Run WASM and low-memory profile checks as non-blocking compatibility signals
  unless they expose shared renderer correctness, safety, or unbounded resource
  defects.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
