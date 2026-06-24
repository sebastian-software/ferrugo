# 0098: Native-Only Packaging And Consumer Migration

Status: todo
Phase: 17
Size: medium
Depends on: 0097

## Goal

Prepare downstream consumers to use the Rust renderer without bundling PDFium.

## Scope

- Produce native-only package metadata, feature examples, and size comparisons.
- Add migration notes for API, CLI, CI, and deployment environments.
- Verify native-only builds on supported target platforms.
- Keep PDFium-enabled comparison tooling available for maintainers.

## Non-Goals

- Remove PDFium fallback code.
- Support untested target triples.
- Change public APIs without migration notes.

## Deliverables

- Native-only packaging guide.
- Consumer migration checklist.
- Platform build validation report.

## Acceptance Criteria

- Consumers can build and test without PDFium artifacts.
- Package size and dependency changes are documented.
- PDFium-enabled maintainer workflows remain available.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run PDFium-enabled comparison smoke tests.
- Run package dry-run or equivalent local packaging check.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
