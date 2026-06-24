# 0117: Tagged PDF Metadata And Accessibility Signals

Status: todo
Phase: 21
Size: medium
Depends on: 0116

## Goal

Expose useful tagged-PDF and accessibility metadata without coupling visual
thumbnail rendering to full document reflow semantics.

## Scope

- Parse structure tree presence and basic role metadata.
- Extract page labels, language, title, and marked-content signals.
- Report accessibility metadata alongside render diagnostics.
- Add fixtures for tagged office exports and reports.

## Non-Goals

- Implement screen-reader output.
- Reflow tagged content.
- Treat accessibility metadata as a rendering prerequisite.

## Deliverables

- Tagged-PDF metadata extraction path.
- Metadata report schema update.
- Fixture report for tagged documents.

## Acceptance Criteria

- Tagged documents render independently of metadata extraction success.
- Metadata extraction is bounded and fails with typed errors.
- Reports identify tagged, untagged, and malformed structure trees.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run tagged-PDF metadata fixture tests.
- Run render comparisons to confirm no visual regressions.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
