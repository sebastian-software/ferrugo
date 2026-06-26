# 0154: Accessibility Tagged PDF Visual Integrity

Status: done
Phase: 28
Size: medium
Depends on: 0153

## Goal

Ensure tagged PDFs, accessibility metadata, structure trees, alt text, and
reading-order metadata do not break or distort visual native rendering.

## Scope

- Add tagged PDF fixtures across reports, forms, and office exports.
- Verify visual rendering is independent from structure metadata parsing.
- Preserve diagnostics for tags, structure tree, language, and metadata.
- Identify any visual regressions caused by metadata-heavy files.

## Non-Goals

- Build a screen-reader API.
- Guarantee semantic reading order extraction.
- Treat accessibility metadata as visual drawing commands.

## Deliverables

- Tagged PDF corpus entries.
- Visual integrity report for metadata-heavy documents.
- Metadata diagnostics updates if needed.

## Acceptance Criteria

- Tagged PDFs render visually like their untagged counterparts where content is
  otherwise supported.
- Structure metadata does not cause crashes or unbounded memory growth.
- Unsupported accessibility semantics are documented separately from visuals.

## Validation

- Run tagged-PDF visual comparison.
- Run metadata parsing tests.
- Run memory profile for structure-heavy fixtures.
- Run native-only supported corpus gate.

## Completion Notes

Completed on 2026-06-26.

- Added a focused tagged-PDF visual manifest at
  `fixtures/tagged-pdf-visual-manifest.tsv`.
- Added synthetic tagged fixtures across report, form, office/figure alt-text,
  and structure-heavy metadata surfaces:
  `tagged-report-visual-integrity.pdf`,
  `tagged-form-visual-integrity.pdf`,
  `tagged-office-alt-text.pdf`, and
  `tagged-structure-heavy-report.pdf`.
- Added native smoke coverage proving tagged metadata and marked-content
  wrappers do not block visual rendering.
- Added metadata assertions for language, `/MarkInfo`, `/StructTreeRoot`,
  RoleMap presence, marked-content references, and bounded role counts.
- Native supported gate is green at 5/5 rendered, 0 fallbacks, and 0 errors.
  Default and low-memory benchmark gates report 0 budget failures.
- PDFium visual comparison reports 1 accepted drift and 4 blockers across
  `rendering-core` and `text-fonts`; these are visual fidelity deltas, not
  accessibility-metadata failures.
- Report: `docs/reports/tagged-pdf-visual-integrity-2026-06-26.md`.
