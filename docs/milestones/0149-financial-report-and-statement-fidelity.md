# 0149: Financial Report And Statement Fidelity

Status: done
Phase: 27
Size: medium
Depends on: 0148

## Goal

Improve fidelity for financial reports, account statements, invoices, and other
dense business documents with strict table and number alignment.

## Scope

- Add fixtures for statements, invoices, annual-report pages, dense tables, and
  chart-heavy financial summaries.
- Track text alignment, table rules, decimal columns, headers, and footers.
- Measure visual drift in regions that affect readability and extraction trust.
- Classify failures by text, vector, image, color, and layout subsystems.

## Non-Goals

- Extract financial data.
- Validate accounting correctness.
- Support private customer documents as fixtures.

## Deliverables

- Financial-document fixture set.
- Region-aware fidelity report.
- Prioritized backlog for dense table and report rendering.

## Acceptance Criteria

- Representative financial documents render natively and remain readable.
- Numeric alignment regressions are detected by visual or structural checks.
- Unsupported cases have explicit typed diagnostics.

## Validation

- Run financial-family visual comparison.
- Run table-heavy rendering smoke tests.
- Run native-only supported corpus gate.
- Run benchmark subset for dense pages.

## Completion Notes

Completed on 2026-06-26.

- Added three synthetic financial-document fixtures:
  `financial-annual-report-page.pdf`,
  `financial-cashflow-statement.pdf`, and
  `financial-chart-summary.pdf`.
- Added `fixtures/financial-document-manifest.tsv` for invoice, statement,
  annual-report, cashflow, chart-summary, and report-statement slices.
- Extended the native business-document smoke test to compile and render the
  new financial fixtures.
- Native supported gate is green at 8/8 rendered, 0 fallbacks, and 0 errors.
- Dense-page benchmark reports 0 budget failures.
- Visual oracle reports 8 fidelity blockers and 0 native/PDFium errors; those
  deltas are routed to table-rule, decimal-alignment, chart-geometry, and
  text-spacing follow-ups.
- Report:
  `docs/reports/financial-report-statement-fidelity-2026-06-26.md`.
