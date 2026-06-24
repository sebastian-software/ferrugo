# 0056: Encryption And Permissions Policy

Status: todo
Phase: 7
Size: small
Depends on: 0055

## Goal

Define and implement safe handling for encrypted PDFs at the native backend
boundary.

## Scope

- Detect encrypted documents from trailer and catalog metadata.
- Return stable unsupported or authentication-required errors.
- Document whether owner/user password workflows are in scope later.
- Ensure encrypted payloads are not partially interpreted as plain objects.

## Non-Goals

- Implement decryption.
- Bypass permissions.
- Store passwords or credentials.

## Deliverables

- Encryption detection path.
- Typed facade errors for encrypted documents.
- Tests for encrypted-document detection fixtures.

## Acceptance Criteria

- Encrypted PDFs fail before unsafe partial rendering.
- The caller can distinguish encrypted from malformed and unsupported files.
- Future decryption work has a documented decision point.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run encrypted fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
