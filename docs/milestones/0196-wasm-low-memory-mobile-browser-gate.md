# 0196: WASM Low Memory Mobile Browser Gate

Status: todo
Phase: 37
Size: medium
Depends on: 0195

## Goal

Validate the native renderer in a low-memory WASM browser profile suitable for
mobile preview and client-side thumbnail workflows.

## Scope

- Define browser memory, binary size, and startup budgets for WASM.
- Run representative corpus slices in a browser automation harness.
- Audit allocations that are expensive under WASM.
- Document unsupported browser-only constraints.

## Non-Goals

- Build a complete PDF viewer UI.
- Require all server-side features in WASM.
- Ship PDFium or native dynamic libraries to the browser.

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

Empty until done.
