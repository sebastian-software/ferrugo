# 0097: Fuzzing And Adversarial PDF Hardening

Status: done
Phase: 16
Size: medium
Depends on: 0096

## Goal

Increase confidence that native parsing and rendering fail safely on malformed
or adversarial PDFs.

## Scope

- Add fuzz targets for primitive parsing, xref loading, stream decoding, content
  tokenization, and page rendering setup.
- Add regression fixtures for discovered crashes, panics, and excessive work.
- Enforce recursion, allocation, stream, and operator budgets.
- Document how to run smoke fuzzing locally.

## Non-Goals

- Make fuzzing mandatory for every local test run.
- Treat all malformed PDFs as recoverable.
- Hide panics behind broad catch-all errors without fixing root causes.

## Deliverables

- Fuzz target suite.
- Crash regression corpus.
- Adversarial PDF hardening report.

## Acceptance Criteria

- Fuzz smoke runs find no panics in targeted parser and render setup paths.
- Known adversarial inputs fail with stable errors.
- Resource exhaustion limits are covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run fuzz smoke targets.
- Run malformed corpus checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added a standalone optional `fuzz/` Cargo package with smoke targets for
  primitive parsing, xref/object loading, stream decoding, content tokenization,
  and native render setup.
- Added reduced adversarial corpus inputs under `fixtures/adversarial/`.
- Added a primitive nesting budget
  (`ferrugo_syntax::DEFAULT_MAX_PRIMITIVE_NESTING`) and a regression test for
  excessive nested arrays.
- Added content and native backend regressions for unterminated inline-image
  tokenization and truncated malformed PDF setup.
- Fuzz smoke completed without panics:
  `primitive_parse` 165 cases, `content_tokenize` 165 cases, `stream_decode`
  154 cases, `xref_load` 154 cases, and `render_setup` 165 cases.
- See `docs/fuzzing.md` and
  `docs/reports/fuzzing-adversarial-hardening-2026-06-25.md`.
