# 0210: Native Renderer 1.3 Hardening Gate

Status: done
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

- Generated the 1.3 hardening scorecard at
  `target/hardening-0210-scorecard/scorecard.json`.
- Re-ran fuzz, native-only release, WASM, workspace check/test, and all-features
  clippy gates after the form, annotation, color, and codec slices.
- Added a repeat-render benchmark over supported real-world-style families at
  `target/hardening-0210-repeat.json`.
- Produced `docs/reports/native-renderer-1-3-hardening-2026-06-29.md`.
- Result: no panics, untyped failures, server batch budget failures, package
  failures, fuzz-smoke failures, or WASM-smoke failures in the hardening run.
- Remaining 1.3 risk is explicit and typed: 12 fallback rows plus one encrypted
  policy error in the scorecard corpus. These are follow-up work, not hidden
  PDFium runtime dependencies.

Validation run:

- `bash scripts/generate_coverage_scorecard.sh target/hardening-0210-scorecard`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --include-family invoice --include-family statement --include-family scanned-packet --include-family form --include-family browser-export --include-family office-export --include-family report --include-family malformed-recovery --repetitions 3 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/hardening-0210-repeat.json`
- `bash scripts/check_fuzz_smoke.sh`
- `bash scripts/check_native_only_release.sh`
- `bash scripts/check_wasm_smoke.sh`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
