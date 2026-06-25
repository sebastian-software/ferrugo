# 0156: Native Renderer API And Semver Policy

Status: todo
Phase: 29
Size: small
Depends on: 0155

## Goal

Define the public Rust-native renderer API and semver policy after PDFium is no
longer part of normal runtime behavior.

## Scope

- Audit public API types for backend leakage and unstable implementation detail.
- Define semver commitments for rendering options, diagnostics, and errors.
- Mark internal renderer modules and maintainer tools clearly.
- Document migration guidance from PDFium-backed APIs.

## Non-Goals

- Freeze all internal implementation details.
- Add language bindings.
- Promise full PDF specification support.

## Deliverables

- API and semver policy document.
- Public API cleanup backlog.
- Migration guidance for native-only consumers.

## Acceptance Criteria

- Public API boundaries are stable, documented, and PDFium-free.
- Diagnostics and error enums have a documented compatibility policy.
- Future internal renderer changes can proceed without breaking consumers.

## Validation

- Run public API docs build if available.
- Run native-only `cargo test`.
- Run package dry-runs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
