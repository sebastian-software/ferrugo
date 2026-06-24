# 0083: Real-World Corpus Ingestion And Classification

Status: todo
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

Empty until done.
