# 0210: Native Renderer 1.3 Hardening Gate

Status: todo
Phase: 39
Size: medium
Depends on: 0209

## Goal

Harden the 1.3 Rust-native renderer after form, annotation, color, and codec
work by validating security, crash resistance, deterministic output, and
profile-specific budgets.

## Scope

- Run the 1.3 corpus through native-only security, fuzz, memory, determinism,
  and packaging gates.
- Triage new crashes, panics, timeouts, and memory-budget failures.
- Freeze release-blocking unsupported categories for 1.3.
- Produce a measured hardening report and release risk list.

## Non-Goals

- Add large new rendering features during the hardening gate.
- Treat fuzz-only adversarial cases as equal to common-document blockers.
- Hide instability behind automatic PDFium fallback.

## Deliverables

- Native renderer 1.3 hardening report.
- Crash, timeout, and memory-budget triage list.
- Release-blocking risk recommendation.

## Acceptance Criteria

- Supported 1.3 corpus runs without panics or untyped failures.
- Determinism and memory gates pass for release profiles.
- Remaining risks are explicit and mapped to follow-up work.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.3 corpus gate.
- Run fuzz and malformed-PDF smoke suites.
- Run deterministic render comparison.
- Run benchmark, memory, WASM, and package profile checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
