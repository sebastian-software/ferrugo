# 0141: PDFium Runtime Deletion Execution

Status: done
Phase: 26
Size: medium
Depends on: 0140

## Goal

Remove PDFium from normal runtime rendering paths after the GA2 evidence shows
that supported document families are covered by the Rust-native renderer.

## Scope

- Delete or disable PDFium-backed runtime dispatch in library and CLI surfaces.
- Keep maintainer-only comparison tooling behind explicit feature flags.
- Remove PDFium runtime assumptions from default examples and docs.
- Add regression checks proving supported rendering works without PDFium assets.

## Non-Goals

- Delete historical reports or comparison evidence.
- Remove maintainer visual-diff tooling before replacement evidence exists.
- Claim unsupported PDF categories are now supported.

## Deliverables

- Runtime deletion patch set.
- Native-only validation report.
- Updated docs describing PDFium as comparison-only tooling.

## Acceptance Criteria

- Default rendering paths do not link, load, or shell out to PDFium.
- PDFium-enabled code is opt-in and clearly maintainer-only.
- Supported corpus gates pass in native-only configuration.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run supported corpus gate without PDFium installed or configured.
- Run package dry-runs for native-only crates.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Removed PDFium runtime fallback dispatch from `render` / `render-auto`.
- `--allow-pdfium-fallback` now fails with a usage error for normal render
  commands; `--native-only` / `--no-pdfium-fallback` remain compatibility
  no-ops because normal render paths are already native-only.
- Kept explicit maintainer PDFium commands behind `--features pdfium`:
  `render-pdfium`, `render-isolated`, `compare-metadata`,
  `benchmark-pdfium`, and `visual-diff`.
- Native-only supported corpus gate passed without PDFium env:
  67/67 `browser-print`, `office-export`, and `form` fixtures rendered
  natively with zero fallback and zero errors.
- Added `docs/reports/pdfium-runtime-deletion-2026-06-26.md` and updated the
  native, PDFium, packaging, error, and maintenance-backlog docs.
