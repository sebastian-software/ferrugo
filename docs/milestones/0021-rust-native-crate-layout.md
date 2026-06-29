# 0021: Rust Native Crate Layout

Status: done
Phase: 1
Size: small
Depends on: 0020

## Goal

Create the Rust-native renderer crate layout without implementing parser or
renderer behavior yet.

## Scope

- Add crates for syntax, object model, content interpretation, rendering, and
  the Rust backend adapter.
- Keep public thumbnail API types in `ferrugo-thumbnail`.
- Start implementation crates with `#![forbid(unsafe_code)]`.
- Define crate ownership boundaries and dependency direction.
- Add empty smoke tests so the workspace shape is validated by Cargo.

## Non-Goals

- Parse PDF syntax.
- Render pixels.
- Remove the PDFium backend.
- Add Node-API bindings.

## Deliverables

- Rust-native crates in the workspace.
- Short architecture note for crate responsibilities.
- Minimal compile-only tests.

## Acceptance Criteria

- The workspace builds with the new crates.
- PDFium-specific types do not enter Rust-native public crate APIs.
- Dependency direction is acyclic and documented.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Added Rust-native workspace crates:
  `ferrugo-syntax`, `ferrugo-object`, `ferrugo-content`, `ferrugo-render`, and
  `ferrugo-native`.
- Added `#![forbid(unsafe_code)]` to each Rust-native implementation crate.
- Added compile-time smoke tests for crate roles and dependency direction.
- Added `docs/architecture/rust-native-crates.md` to document ownership
  boundaries.
- Kept the PDFium backend isolated from the Rust-native crate APIs.
