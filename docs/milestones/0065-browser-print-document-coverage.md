# 0065: Browser Print Document Coverage

Status: done
Phase: 10
Size: medium
Depends on: 0064

## Goal

Cover PDFs produced by browser print and save-to-PDF workflows.

## Scope

- Target Chrome, Safari, and Firefox generated PDFs.
- Improve rendering of CSS-derived clipping, images, links, transparency, and
  embedded font subsets.
- Add deterministic HTML-to-PDF fixture generation where possible.
- Track browser-specific feature gaps separately.

## Non-Goals

- Implement HTML or CSS rendering.
- Depend on a live browser in normal unit tests.
- Treat browser output as canonical PDF specification behavior.

## Deliverables

- Browser-print fixture manifest.
- Native fixes for common browser PDF constructs.
- Documentation of browser-specific remaining gaps.

## Acceptance Criteria

- Representative browser PDFs render through native by default.
- Browser-print regressions are covered by tests or corpus checks.
- Fallback telemetry distinguishes browser-print blockers.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run browser-print corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Used the committed `browser-print` corpus family in
  `fixtures/corpus-manifest.tsv` as deterministic, license-safe browser-print
  proxies for page geometry, vector paths, clipping, and inline images.
- Added `docs/reports/browser-print-coverage-2026-06-24.md`.
- Browser-print corpus summary at `--max-edge 120` reported 4 total fixtures,
  4 native renders, 1.000 native pass rate, 0 fallbacks, and 0 errors.
- PDFium differential smoke at `--max-edge 260` rendered all 4 browser-print
  fixtures successfully through native and direct PDFium with matching PNG
  dimensions.
- Browser-specific remaining gaps are documented separately from generic
  renderer failures and mapped to fallback categories.
- Validation: `cargo fmt --check`, `cargo check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test --quiet`, browser-print corpus summary, and PDFium differential
  smoke.
