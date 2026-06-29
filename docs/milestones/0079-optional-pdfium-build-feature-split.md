# 0079: Optional PDFium Build Feature Split

Status: done
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

Completed on 2026-06-24.

- Made `ferrugo-pdfium` an optional `ferrugo-cli` dependency behind the
  `pdfium` feature.
- Added workspace `default-members` so root-level builds focus on the
  native-only stack by default.
- Gated PDFium CLI paths while preserving command names with clear native-only
  usage errors.
- Preserved PDFium-enabled fallback, direct render, isolated render,
  comparison, and benchmark workflows under `--features pdfium`.
- Documented native-only and PDFium-enabled packaging in `docs/packaging.md`.
- Validation passed:
  `cargo fmt --check`,
  `cargo check --no-default-features`,
  `cargo test --no-default-features`,
  `cargo test -p ferrugo-cli --features pdfium`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Runtime probes passed:
  PDFium-enabled `compare-metadata`,
  PDFium-enabled `render-auto` fallback,
  native-only `render-pdfium` disabled-path check.

Implementation commit:

- `387822a feat: make pdfium cli support optional`
