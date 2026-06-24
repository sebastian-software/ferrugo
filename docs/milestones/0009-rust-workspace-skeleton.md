# 0009: Rust Workspace Skeleton

Status: todo
Phase: 0
Size: small
Depends on: 0003

## Goal

Create the smallest Rust workspace needed for the Phase 0 CLI/library probe.

## Scope

- Create a Cargo workspace.
- Add a thumbnail facade crate.
- Add a CLI crate.
- Set basic formatting and lint expectations.

## Non-Goals

- Create the full future renderer crate graph.
- Implement a Rust-native parser.
- Add Node-API bindings.

## Deliverables

- Minimal Cargo workspace.
- Library crate for thumbnail API types and traits.
- CLI crate shell.

## Acceptance Criteria

- `cargo check` passes.
- Public types avoid PDFium-specific naming.
- The workspace shape does not imply full renderer parity work.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.

## Completion Notes

Empty until done.

