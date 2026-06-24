# 0015: Differential Baseline Format

Status: todo
Phase: 0
Size: small
Depends on: 0012, 0014

## Goal

Define the metadata format for comparing future Rust-native renderer output
against PDFium-backed thumbnail output.

## Scope

- Record fixture identity.
- Record backend identity.
- Record page index, max edge, background, format, dimensions, and error class.
- Record pixel output path or digest.
- Record tolerance policy placeholder.

## Non-Goals

- Implement the Rust-native renderer.
- Create a large visual regression suite.
- Store large bitmap artifacts in Git by default.

## Deliverables

- Baseline metadata format.
- Example baseline record for a generated fixture.
- Documentation for where large artifacts live.

## Acceptance Criteria

- The format can represent success and failure cases.
- Baselines are backend-neutral.
- Large outputs are not accidentally committed unless explicitly intended.

## Validation

- Generate one example metadata record.
- Confirm it references the generated fixture and PDFium backend.

## Completion Notes

Empty until done.

