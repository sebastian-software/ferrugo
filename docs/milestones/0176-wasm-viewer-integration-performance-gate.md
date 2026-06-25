# 0176: WASM Viewer Integration Performance Gate

Status: todo
Phase: 33
Size: medium
Depends on: 0175

## Goal

Validate that the Rust-native renderer can support browser or WASM viewer
integration for typical documents within size, memory, and latency budgets.

## Scope

- Define WASM package size, initialization, first-page, and thumbnail latency
  budgets.
- Identify renderer dependencies that block or bloat WASM builds.
- Add a small viewer-oriented smoke harness for supported documents.
- Document unsupported APIs or features in WASM mode.

## Non-Goals

- Build a full production web viewer.
- Add PDFium, native plugins, or browser-only dependencies.
- Optimize every renderer path for WASM before measuring.

## Deliverables

- WASM viewer performance report.
- Package size and latency budget checks.
- Follow-up backlog for WASM-specific improvements.

## Acceptance Criteria

- WASM build is PDFium-free and has documented feature flags.
- Typical sample documents render within agreed viewer budgets.
- Unsupported WASM features are explicit.

## Validation

- Run WASM build check where available.
- Run native-only `cargo test`.
- Run viewer smoke harness.
- Measure package size and first-page latency.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
