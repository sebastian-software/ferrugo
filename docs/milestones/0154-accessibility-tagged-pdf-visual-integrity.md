# 0154: Accessibility Tagged PDF Visual Integrity

Status: todo
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

Empty until done.
