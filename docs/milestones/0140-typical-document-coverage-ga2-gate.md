# 0140: Typical Document Coverage GA2 Gate

Status: done
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

- Added `docs/reports/native-renderer-ga2-coverage-2026-06-26.md`.
- Core supported-family gate passed: 67/67 `browser-print`, `office-export`,
  and `form` fixtures rendered natively with zero fallback and zero errors.
- Full current corpus coverage: 155 total, 146 native rendered, 8 typed
  fallbacks, 1 encrypted error.
- PDFium visual oracle remains a broad GA blocker: 32 exact, 23 accepted drift,
  91 blockers, 8 native errors, and 1 both-error encrypted row.
- Native-only packaging evidence remains PDFium-free; full CLI package
  preparation is still release-order blocked until internal crates are
  available from the registry.
- Recommendation: proceed to runtime PDFium deletion/quarantine work for normal
  native-only paths while keeping PDFium comparison tooling for maintainer
  visual-oracle gates.
