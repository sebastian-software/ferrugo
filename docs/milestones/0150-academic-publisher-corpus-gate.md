# 0150: Academic Publisher Corpus Gate

Status: todo
Phase: 27
Size: medium
Depends on: 0149

## Goal

Validate native rendering against academic papers, technical reports, books,
equation-heavy documents, citations, and multi-column layouts.

## Scope

- Add public or synthetic fixtures for papers, reports, equation pages, figures,
  footnotes, references, and two-column layouts.
- Track embedded font subsets, ligatures, math symbols, and vector figures.
- Classify failures that affect readability or page structure.
- Keep long-document samples small enough for local validation.

## Non-Goals

- Parse semantic citations or equations.
- Implement text selection or reflow.
- Guarantee full publisher-specific PDF conformance.

## Deliverables

- Academic corpus entries.
- Multi-column and equation-page visual report.
- Follow-up backlog for text and vector fidelity gaps.

## Acceptance Criteria

- Common academic pages render natively with readable text and figures.
- Math and symbol failures are explicit and traceable to font handling gaps.
- Multi-column layout geometry stays stable across platforms.

## Validation

- Run academic-family visual comparison.
- Run font subset regression tests.
- Run cross-platform determinism subset if available.
- Run native-only supported corpus gate.

## Completion Notes

Empty until done.
