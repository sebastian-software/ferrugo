# 0196: WASM Low Memory Mobile Browser Gate

Status: done
Phase: 37
Size: medium
Depends on: 0195

## Goal

Validate the native renderer in a low-memory WASM browser profile as a
secondary compatibility signal for mobile preview and client-side thumbnail
workflows.

## Scope

- Define browser memory, binary size, and startup budgets for WASM.
- Run representative corpus slices in a browser automation harness.
- Audit allocations that are expensive under WASM.
- Document unsupported browser-only constraints.
- Promote only shared renderer correctness, safety, or unbounded allocation
  defects into the server-side release backlog.

## Non-Goals

- Build a complete PDF viewer UI.
- Require all server-side features in WASM.
- Ship PDFium or native dynamic libraries to the browser.
- Block server-side PDFium-free releases solely on mobile browser profile
  limitations.

## Deliverables

- WASM low-memory validation report.
- Browser fixture smoke suite.
- Size and allocation follow-up backlog.

## Acceptance Criteria

- Common preview workflows run under documented mobile memory budgets.
- WASM binary size remains within release thresholds.
- Browser failures are typed and reproducible.

## Validation

- Run native-only `cargo test`.
- Run WASM build checks.
- Run browser rendering smoke tests.
- Run binary size and memory reports.

## Completion Notes

Completed 2026-06-29.

- Added a low-memory WASM preview suite covering plain text, browser print,
  mobile scan, AcroForm, and dense invoice fixtures.
- Extended the WASM smoke API and harness to report fixture count and total
  RGBA output bytes.
- Added release thresholds for artifact size, compile time, instantiate time,
  smoke render time, minimum fixture count, and total output bytes.
- Documented browser-only constraints and follow-up optimization backlog in
  `docs/reports/wasm-low-memory-mobile-browser-2026-06-29.md`.

Validation:

- `cargo fmt --check`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-wasm-smoke --no-default-features -- --nocapture`
- `FERRUGO_WASM_REPORT=target/wasm-0196-mobile-smoke.json bash scripts/check_wasm_smoke.sh`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --no-default-features`
