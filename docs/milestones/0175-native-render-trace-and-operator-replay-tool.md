# 0175: Native Render Trace And Operator Replay Tool

Status: done
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

Completed on 2026-06-26.

Implemented opt-in CLI maintainer tooling:

- `pdfrust-cli trace-native` emits bounded native render traces with metadata,
  render outcome, aggregate operator coverage, typed unsupported outcomes, and
  capped operator events.
- `pdfrust-cli replay-operators` reads native trace JSON and emits compact
  operator replay counts for reduced fixture triage.
- `docs/policies/native-render-trace.md` defines the trace format, privacy
  boundary, size limits, and replay boundary.
- `docs/reports/native-render-trace-operator-replay-2026-06-26.md` records the
  smoke commands, artifact sizes, privacy review, and disabled benchmark.

Validation completed:

- `cargo test -p pdfrust-cli trace -- --nocapture`
- `cargo test -p pdfrust-cli replay_operator -- --nocapture`
- Trace/replay smoke commands on `fixtures/generated/vector-paths.pdf`.
- Disabled native benchmark on the report corpus family.
