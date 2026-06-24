# 0083: Real-World Corpus Ingestion And Classification

Status: done
Phase: 14
Size: medium
Depends on: 0082

## Goal

Expand the validation corpus with privacy-safe real-world document samples so
native renderer coverage reflects typical production inputs.

## Scope

- Define intake rules for sanitized or synthetic-realistic documents.
- Add categories for invoices, reports, scanned packets, forms, statements,
  browser exports, and office exports.
- Capture document traits that affect renderer behavior.
- Keep fixture metadata compact and searchable.

## Non-Goals

- Store private customer documents.
- Optimize for rare PDF authoring edge cases before common categories.
- Add large fixtures without size and memory justification.

## Deliverables

- Corpus intake policy.
- Expanded manifest taxonomy.
- Initial real-world-style fixture batch or documented placeholders.

## Acceptance Criteria

- New corpus entries explain why they matter for PDFium retirement.
- Each fixture has category, feature tags, page count, and expected backend.
- Privacy and repository size constraints are documented.

## Validation

- Run manifest validation.
- Run native corpus summary.
- Run PDFium comparison where available.
- Review fixture size impact.

## Completion Notes

Completed on 2026-06-24.

- Added privacy-safe corpus intake policy at `docs/policies/corpus-intake.md`.
- Added `fixtures/real-world-style-manifest.tsv` with 10 synthetic-realistic
  seed entries across invoice, statement, scanned packet, form, browser export,
  office export, report, presentation, secure document, and malformed recovery
  categories.
- Updated `docs/corpus-taxonomy.md` with the real-world-style manifest contract
  and `expected:*` backend tags.
- Published `docs/reports/real-world-corpus-ingestion-2026-06-24.md`.

Validation passed:

- manifest path and `expected:*` tag validation for all 10 rows,
- `extract-corpus-metadata` with the new manifest,
- native `summarize-fallbacks` with the new manifest,
- PDFium-enabled benchmark comparison with the new manifest,
- fixture size review confirmed no new PDF binaries.
