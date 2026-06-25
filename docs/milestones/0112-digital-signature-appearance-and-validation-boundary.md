# 0112: Digital Signature Appearance And Validation Boundary

Status: in-progress
Phase: 20
Size: small
Depends on: 0111

## Goal

Render visible signature appearances while clearly separating thumbnail rendering
from cryptographic signature validation.

## Scope

- Render existing signature widget appearance streams.
- Detect unsigned, invalid, or unverifiable validation states as metadata only.
- Document the boundary between appearance rendering and validation.
- Add fixtures with visible signatures and signature panels.

## Non-Goals

- Validate certificate chains.
- Provide legal signature status.
- Mutate signed documents.

## Deliverables

- Signature appearance rendering coverage.
- Validation-boundary policy.
- Fixture report for signed business PDFs.

## Acceptance Criteria

- Visible signature appearances render natively when present.
- API and CLI text do not imply cryptographic validation.
- Missing appearances follow the annotation fallback policy.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run signed PDF fixture comparisons.
- Run metadata boundary tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
