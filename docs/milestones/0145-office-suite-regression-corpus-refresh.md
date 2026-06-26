# 0145: Office Suite Regression Corpus Refresh

Status: done
Phase: 26
Size: medium
Depends on: 0144

## Goal

Refresh the office-export corpus so Word, Excel, PowerPoint, LibreOffice, and
browser-generated business documents remain first-class native renderer targets.

## Scope

- Add or refresh office-export fixtures across text, tables, charts, images,
  hyperlinks, headers, footers, and page backgrounds.
- Record generator metadata where it can be shared safely.
- Classify failures by renderer subsystem.
- Keep fixtures small enough for routine local validation.

## Non-Goals

- Store private or customer documents.
- Target editable document reconstruction.
- Support every export setting from every office suite.

## Deliverables

- Refreshed office-export corpus entries.
- Family-level visual report.
- Backlog entries for office-specific fidelity gaps.

## Acceptance Criteria

- Office-export family coverage includes common mixed-content documents.
- New fixtures are reproducible or have documented provenance.
- Native renderer failures are explicit and actionable.

## Validation

- Run corpus manifest validation.
- Run office-export family visual comparison.
- Run native-only supported corpus gate.
- Run fixture size and privacy checks.

## Completion Notes

- Added generated Office-suite fixtures for a header/footer/link report, a
  spreadsheet with chart and comments, and a presentation handout.
- Registered the new fixtures in `fixtures/corpus-manifest.tsv` under
  `office-export`, bringing the family to 47 fixtures.
- Native supported gate passed: 47/47 office-export fixtures rendered
  natively, with 0 fallbacks and 0 errors.
- PDFium visual oracle passed as a measurement run and classified 44/47
  office-export fixtures as fidelity blockers, including all three new mixed
  Office fixtures; this feeds the rendering-core and table/grid backlog rather
  than blocking runtime support.
- Fixture size review confirmed the three new PDFs add only 6,020 bytes and no
  generated PDF exceeds the 512 KiB routine-validation threshold.
- Report: `docs/reports/office-suite-regression-corpus-refresh-2026-06-26.md`.
