# 0132: WASM Renderer Packaging And Size Gate

Status: todo
Phase: 24
Size: medium
Depends on: 0131

## Goal

Evaluate and package the Rust-native renderer for WebAssembly consumers without
requiring PDFium or native dynamic libraries.

## Scope

- Add a WASM build profile for native thumbnail rendering.
- Measure package size, initialization cost, and unsupported dependencies.
- Define which CLI/library features are excluded from WASM.
- Add smoke fixtures that can run in a browser or WASM test harness.

## Non-Goals

- Ship a complete browser viewer.
- Support PDFium fallback in WASM.
- Optimize every dependency for minimum size in the first pass.

## Deliverables

- WASM packaging notes and build gate.
- Size and startup report.
- Browser or WASM smoke-test fixture path.

## Acceptance Criteria

- Native renderer builds for the selected WASM target.
- Package size and startup costs are recorded and bounded.
- Unsupported features fail at compile time or with documented errors.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run WASM target check.
- Run WASM smoke fixtures where supported locally.
- Run package size measurement.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
