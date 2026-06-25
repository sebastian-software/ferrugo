# 0220: PDFium-Free 1.4 Readiness Gate

Status: todo
Phase: 41
Size: medium
Depends on: 0219

## Goal

Make the PDFium-free 1.4 release decision using cross-producer typical-document
coverage, low-end reliability evidence, server and WASM scheduler tuning, and a
clear unsupported-feature SLA.

## Scope

- Run the complete native-only 1.4 validation matrix across supported document
  families and deployment profiles.
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
- Supported desktop, server, WASM, and low-end profiles meet documented budgets.
- Consumer-facing unsupported behavior is stable and documented.
- No supported path requires PDFium.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.4 supported corpus gate.
- Run independent visual oracle validation.
- Run benchmark, memory, low-end, server, WASM, and package profile checks.
- Run security and fuzz smoke suite.
- Run repository scan for unsupported PDFium runtime references.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
