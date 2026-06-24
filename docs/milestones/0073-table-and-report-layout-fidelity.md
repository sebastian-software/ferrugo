# 0073: Table And Report Layout Fidelity

Status: todo
Phase: 12
Size: medium
Depends on: 0072

## Goal

Improve native fidelity for reports, invoices, statements, and table-heavy PDFs.

## Scope

- Add fixtures with dense ruled tables, small text, logos, and repeated headers.
- Tune stroke, text, and image placement at thumbnail sizes.
- Track legibility and missing-element regressions separately from pixel deltas.
- Add multi-page report comparison coverage.

## Non-Goals

- Extract table structure.
- Infer semantic document fields.
- Optimize for arbitrary zoom levels before thumbnail parity is stable.

## Deliverables

- Report and table fixture suite.
- Native fixes for common small-text and ruling-line issues.
- Multi-page comparison output for representative reports.

## Acceptance Criteria

- Typical reports render with visible text, table lines, and logos.
- Multi-page documents preserve stable dimensions and page ordering.
- Remaining differences have clear renderer feature labels.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run report corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
