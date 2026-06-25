# 0220: PDFium-Free 1.4 Readiness Gate

Status: todo
Phase: 41
Size: medium
Depends on: 0219

## Goal

Make the PDFium-free 1.4 release decision using cross-producer typical-document
coverage, server scheduler tuning, constrained server evidence, and a clear
unsupported-feature SLA. WASM and mobile low-memory results inform compatibility
backlog decisions but are not primary release blockers by themselves.

## Scope

- Run the complete native-only 1.4 validation matrix across supported document
  families and primary server deployment profiles.
- Compare 1.4 coverage, fidelity, memory, throughput, unsupported categories,
  and consumer-facing diagnostics against the 1.3 baseline.
- Verify PDFium is absent from supported runtime, package, CI, and deployment
  paths.
- Decide release, stabilize, or defer based on measured evidence.

## Non-Goals

- Claim complete PDF specification support.
- Hide unsupported behavior behind non-public diagnostics.
- Retain PDFium comparison tooling without a fresh explicit decision.

## Deliverables

- PDFium-free 1.4 readiness report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.4 backlog.

## Acceptance Criteria

- Cross-producer typical-document families pass native-only release gates.
- Supported desktop and server profiles meet documented release budgets.
- WASM, mobile, embedded, and low-memory profile failures are classified as
  compatibility backlog unless they expose shared renderer correctness, safety,
  or unbounded resource defects.
- Consumer-facing unsupported behavior is stable and documented.
- No supported path requires PDFium.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.4 supported corpus gate.
- Run independent visual oracle validation.
- Run benchmark, memory, server, and package profile checks.
- Run low-end and WASM profile checks as secondary compatibility signals.
- Run security and fuzz smoke suite.
- Run repository scan for unsupported PDFium runtime references.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
