# 0145: Office Suite Regression Corpus Refresh

Status: todo
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

Empty until done.
