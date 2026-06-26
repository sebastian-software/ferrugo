# 0163: Producer Compatibility Matrix Expansion

Status: done
Phase: 30
Size: medium
Depends on: 0162

## Goal

Expand the corpus around common PDF producers so native-renderer coverage is
measured by the documents users actually encounter.

## Scope

- Add or classify fixtures from office suites, browsers, scanners, email
  clients, design tools, accounting tools, and government form generators.
- Track producer, version, document family, and known feature pressure in the
  manifest.
- Keep private or licensed samples out of the committed corpus unless their
  license permits inclusion.
- Add a producer matrix report that separates supported, unsupported, and
  visually degraded cases.

## Non-Goals

- Add confidential user documents to the repository.
- Treat producer coverage as a guarantee for every document from that producer.
- Implement feature fixes discovered by the matrix in this milestone.

## Deliverables

- Expanded producer compatibility manifest fields.
- Producer matrix report.
- Fixture generation or ingestion notes for each added source.

## Acceptance Criteria

- Producer metadata is present for representative typical-document fixtures.
- Unsupported producer cases have typed categories.
- The matrix identifies high-value follow-up milestones.

## Validation

- Run fixture manifest validation.
- Run native-only supported corpus gate.
- Run visual comparison for producer fixture subsets.
- Run benchmark summary for newly added samples.

## Completion Notes

- Added `fixtures/producer-compatibility-manifest.tsv` as the CLI-compatible
  producer subset gate.
- Added `fixtures/producer-compatibility-matrix.tsv` with producer,
  producer-version style, document family, workflow, feature pressure, current
  status, and owner route fields.
- Added `docs/reports/producer-compatibility-matrix-2026-06-26.md`.
- Supported producer subset covers office suites, browsers, scanners,
  accounting/banking, government forms, and a PDF 2.0 producer baseline with
  13/13 native renders, 0 fallbacks, and 0 errors.
- Typed unsupported producer-style boundaries are tracked for layered
  presentations (`graphics.optional-content`) and fax/scanner image codecs
  (`image.filter`).
- Email-client and design-tool producer reductions remain explicit follow-up
  gaps for later corpus milestones.
- Matrix column validation, supported fallback gate, unsupported classification,
  PDFium visual subset, native benchmark, and formatting checks passed.
