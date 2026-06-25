# 0140: Typical Document Coverage GA2 Gate

Status: todo
Phase: 25
Size: medium
Depends on: 0139

## Goal

Run a second general-availability coverage gate for the Rust-native renderer
after document-family, performance, packaging, and hardening expansions.

## Scope

- Re-run the full supported corpus with family-level pass/fail summaries.
- Compare native-only behavior against the retained maintainer PDFium baseline.
- Verify native-only packaging, diagnostics, memory profiles, and server/WASM
  readiness evidence.
- Produce the next deletion or stabilization backlog.

## Non-Goals

- Claim complete PDF specification coverage.
- Reintroduce PDFium as a runtime dependency.
- Hide unsupported categories behind ambiguous success states.

## Deliverables

- GA2 native renderer coverage report.
- Family-level support matrix and blocker backlog.
- Recommendation for the next stabilization or deletion cycle.

## Acceptance Criteria

- Typical supported document families render natively within documented gates.
- Unsupported categories are explicit, typed, and not runtime PDFium-dependent.
- Remaining work is split into small follow-up milestones or deletion tasks.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus visual comparisons.
- Run renderer benchmark suite and memory profiles.
- Run native-only package validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
