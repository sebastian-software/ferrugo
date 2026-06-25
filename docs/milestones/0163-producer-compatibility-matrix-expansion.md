# 0163: Producer Compatibility Matrix Expansion

Status: todo
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

Empty until done.
