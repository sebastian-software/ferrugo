# 0175: Native Render Trace And Operator Replay Tool

Status: todo
Phase: 32
Size: medium
Depends on: 0174

## Goal

Add maintainer tooling for inspecting native render behavior without relying on
PDFium comparison logs.

## Scope

- Capture compact render traces for parser events, content operators,
  display-list items, resource lookups, and typed unsupported outcomes.
- Add an operator replay mode for small reduced fixtures.
- Bound trace size and redact or omit document data that is not needed for
  debugging.
- Document how traces support issue triage and regression reduction.

## Non-Goals

- Log full PDF contents by default.
- Make tracing part of the normal runtime hot path.
- Build a graphical debugger in this milestone.

## Deliverables

- Native render trace format.
- Operator replay command or maintainer tool.
- Trace-size and privacy policy documentation.

## Acceptance Criteria

- Maintainers can reproduce a reduced render issue from a bounded trace.
- Tracing is opt-in and has explicit size limits.
- Runtime performance is unchanged when tracing is disabled.

## Validation

- Run native-only `cargo test`.
- Run trace/replay tests on small fixtures.
- Run benchmark comparison with tracing disabled.
- Review trace output for accidental large data capture.

## Completion Notes

Empty until done.
