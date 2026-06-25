# 0185: PDF/A And Archival Document Conformance Boundary

Status: todo
Phase: 34
Size: medium
Depends on: 0184

## Goal

Define and validate the renderer boundary for PDF/A and archival documents so
long-lived records render predictably without claiming full conformance
validation.

## Scope

- Detect PDF/A metadata and common archival profile markers.
- Add archival fixtures with embedded fonts, output intents, metadata, and
  long-lived producer quirks.
- Verify rendering behavior for embedded-font and color-profile-heavy files.
- Document what the renderer does and does not validate for conformance.

## Non-Goals

- Build a PDF/A validator.
- Certify legal or archival compliance.
- Ignore visual failures because the file is archival.

## Deliverables

- PDF/A boundary policy.
- Archival fixture report.
- Typed unsupported or warning categories for archival-only gaps.

## Acceptance Criteria

- PDF/A profile markers are visible in metadata reports.
- Typical archival records render through the native backend.
- Compliance validation boundaries are explicit.

## Validation

- Run native-only `cargo test`.
- Run archival fixture classification.
- Run visual comparisons for archival fixtures.
- Review metadata and support matrix documentation.

## Completion Notes

Empty until done.
