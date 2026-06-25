# 0098: Native-Only Packaging And Consumer Migration

Status: done
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

Completed on 2026-06-25.

- Added versioned internal path dependencies and inherited package description
  metadata across the Rust crates.
- Expanded `docs/packaging.md` with native-only feature examples, consumer
  migration checklist, PDFium maintainer workflow boundaries, and release
  order.
- Recorded dependency graph comparison: native-only `pdfrust-cli` has 24
  dependency-tree lines and no `pdfrust-pdfium`; PDFium-enabled has 26 and
  includes `pdfrust-pdfium`.
- Validated host native-only builds on `aarch64-apple-darwin` with
  `cargo check --workspace --no-default-features` and
  `cargo test --workspace --no-default-features`.
- Validated maintainer PDFium feature smoke with
  `cargo test -p pdfrust-cli --features pdfium`.
- Ran package dry-runs for leaf crates `pdfrust-syntax` and
  `pdfrust-thumbnail`; full `pdfrust-cli` packaging is documented as blocked
  until internal crates are released in order.
- See `docs/reports/native-only-packaging-validation-2026-06-25.md`.
