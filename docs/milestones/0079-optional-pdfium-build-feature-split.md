# 0079: Optional PDFium Build Feature Split

Status: todo
Phase: 13
Size: medium
Depends on: 0078

## Goal

Make PDFium an optional build capability instead of a required part of normal
native renderer use.

## Scope

- Split PDFium-dependent crates, features, tests, and CLI paths cleanly.
- Keep native-only builds compiling and testing without PDFium libraries.
- Preserve comparison and fallback workflows when the PDFium feature is enabled.
- Document packaging implications for downstream users.

## Non-Goals

- Delete PDFium support.
- Remove differential testing.
- Break users who still need PDFium fallback.

## Deliverables

- Feature-gated PDFium integration.
- Native-only CI or local validation path.
- Packaging documentation for native-only and PDFium-enabled builds.

## Acceptance Criteria

- `cargo test` succeeds in native-only configuration.
- PDFium-enabled builds retain comparison and fallback behavior.
- Dependency graph and binary packaging are smaller in native-only mode.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check` with native-only features.
- Run `cargo test` with native-only features.
- Run PDFium-enabled comparison tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
