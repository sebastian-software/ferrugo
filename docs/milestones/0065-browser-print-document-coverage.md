# 0065: Browser Print Document Coverage

Status: todo
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

Empty until done.
