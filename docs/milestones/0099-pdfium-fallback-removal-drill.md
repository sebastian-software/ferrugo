# 0099: PDFium Fallback Removal Drill

Status: done
Phase: 17
Size: medium
Depends on: 0098

## Goal

Run a controlled drill that disables PDFium fallback for supported document
categories and measures the remaining operational risk.

## Scope

- Add a native-only validation mode that treats accidental PDFium use as a
  failure.
- Run supported corpus categories through the native-only path.
- Record unsupported categories and required user-facing errors.
- Identify any fallback paths that can be deleted immediately.

## Non-Goals

- Delete comparison tooling.
- Pretend unsupported categories are supported.
- Remove emergency fallback without a rollback path.

## Deliverables

- PDFium fallback removal drill report.
- Native-only supported-category gate.
- Deletion candidates for fallback code and configuration.

## Acceptance Criteria

- Supported categories pass without invoking PDFium.
- Remaining fallback paths are justified by documented unsupported categories.
- The drill produces a clear delete, defer, or keep decision per fallback path.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run native-only supported corpus gate.
- Run PDFium-enabled comparison smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added `summarize-fallbacks --include-family <family>` so supported corpus
  categories can be gated explicitly instead of relying on full-corpus fallback
  totals.
- Ran a native-only supported-category gate for `browser-print`,
  `office-export`, and `form`: 30 fixtures rendered natively with 0 fallback
  requirements and 0 errors.
- Recorded full-corpus fallback risk: `graphics.optional-content` (1),
  `graphics.pattern-shading` (1), and `image.filter` (3); encrypted input
  remains a native `encrypted` error rather than a fallback category.
- Documented delete/defer/keep decisions for production fallback usage,
  explicit `--allow-pdfium-fallback`, environment-driven fallback, and
  maintainer comparison commands.
- See `docs/reports/pdfium-fallback-removal-drill-2026-06-25.md`.
