# 0181: PDF 2.0 Feature Usage Corpus Gate

Status: todo
Phase: 34
Size: medium
Depends on: 0180

## Goal

Measure which PDF 2.0 features appear in typical documents and decide which
ones must be supported, approximated, or typed as unsupported for the native
renderer.

## Scope

- Expand corpus metadata with PDF version and producer feature flags.
- Detect common PDF 2.0 structures that affect visual output.
- Add typed unsupported reasons for observed features outside the current
  renderer boundary.
- Rank implementation follow-ups by real document frequency and render impact.

## Non-Goals

- Implement complete PDF 2.0 coverage.
- Treat rare archival features as release blockers without corpus evidence.
- Reintroduce PDFium runtime fallback.

## Deliverables

- PDF 2.0 usage report.
- Corpus feature classification updates.
- Ranked follow-up backlog.

## Acceptance Criteria

- PDF 2.0 documents are detected and classified in corpus reports.
- Visual-impacting unsupported features have stable typed reasons.
- The 1.2 roadmap has evidence-backed PDF 2.0 priorities.

## Validation

- Run native-only `cargo test`.
- Run corpus classification across PDF version families.
- Run supported corpus fallback summary.
- Review unsupported feature categories for ambiguity.

## Completion Notes

Empty until done.
