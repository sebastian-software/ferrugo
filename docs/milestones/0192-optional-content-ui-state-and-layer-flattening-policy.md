# 0192: Optional Content UI State And Layer Flattening Policy

Status: todo
Phase: 36
Size: medium
Depends on: 0191

## Goal

Define how optional content groups, default layer states, and flattened output
should behave in native rendering and viewer integration.

## Scope

- Add fixtures for default-on, default-off, nested, and usage-based optional
  content groups.
- Expose enough layer metadata for consumers to present or flatten layer state.
- Document unsupported dynamic UI state and print/export behavior.
- Ensure hidden layers do not paint pixels by default.

## Non-Goals

- Build a full viewer layer panel.
- Implement every usage intent or JavaScript-driven layer behavior.
- Render hidden optional content to improve visual similarity.

## Deliverables

- Optional content policy update.
- Layer metadata or classification tests.
- Visual fixtures for layer state behavior.

## Acceptance Criteria

- Default layer visibility is deterministic.
- Hidden content does not leak into raster output.
- Consumers can identify unsupported layer behavior.

## Validation

- Run native-only `cargo test`.
- Run optional content visual comparisons.
- Run metadata classification tests.
- Review policy docs for runtime PDFium references.

## Completion Notes

Empty until done.
