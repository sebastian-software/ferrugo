# 0176: WASM Viewer Integration Performance Gate

Status: todo
Phase: 33
Size: medium
Depends on: 0175

## Goal

Validate WASM viewer integration as a secondary deployment profile, after the
server-side Rust-native renderer has the relevant correctness and resource
behavior for typical documents.

## Scope

- Define WASM package size, initialization, first-page, and thumbnail latency
  budgets.
- Identify renderer dependencies that block or bloat WASM builds.
- Add a small viewer-oriented smoke harness for supported documents.
- Document unsupported APIs or features in WASM mode.
- Keep WASM findings as profile-specific follow-up unless they expose a shared
  renderer correctness or safety defect.

## Non-Goals

- Build a full production web viewer.
- Add PDFium, native plugins, or browser-only dependencies.
- Let WASM packaging or latency concerns block server-side PDFium replacement
  gates by themselves.

## Deliverables

- WASM viewer performance report.
- Package size and latency budget checks.
- Follow-up backlog for WASM-specific improvements.

## Acceptance Criteria

- WASM build is PDFium-free and has documented feature flags.
- Typical sample documents render within agreed viewer budgets.
- Unsupported WASM features are explicit.
- Shared renderer correctness failures are promoted to the main backlog; purely
  browser-profile limitations remain secondary.

## Validation

- Run WASM build check where available.
- Run native-only `cargo test`.
- Run viewer smoke harness.
- Measure package size and first-page latency.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
