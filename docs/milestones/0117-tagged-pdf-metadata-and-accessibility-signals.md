# 0117: Tagged PDF Metadata And Accessibility Signals

Status: done
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

Completed on 2026-06-25.

- Commit: `02abfff feat: expose tagged pdf accessibility metadata`.
- Added `AccessibilityMetadata` to `DocumentMetadata` with language,
  `/MarkInfo /Marked`, RoleMap presence, structure role count,
  marked-content reference presence, and traversal truncation.
- Added bounded native structure-tree traversal with cycle protection and a
  malformed structure-tree error path.
- Added generated tagged and malformed structure fixtures:
  `fixtures/generated/tagged-accessibility-metadata.pdf` and
  `fixtures/generated/malformed-tagged-structure.pdf`.
- Updated CLI metadata JSON to emit an `accessibility` object.
- Validation artifacts:
  - `target/tagged-0117-metadata.json`: total 106, tagged fixture success with
    `language: "en-US"`, `mark_info_marked: true`, RoleMap, one role, and
    marked-content reference; malformed fixture returns `error_class:
    "malformed"`.
  - `target/tagged-0117-supported-gate.json`: total 46, native rendered 46,
    fallback required 0, errors `{}`.
  - `target/tagged-0117-benchmark.json`: total 106, native rendered 99,
    fallback required 6, errors 1, budget failures 7.
  - `target/tagged-0117-visual-diff.json`: total 106, exact 35, accepted drift
    22, blockers 42, native errors 6, PDFium errors 0, both errors 1.
  - New fixture visual results: malformed structure exact; tagged
    accessibility fixture accepted drift with MAE 0.515, changed ratio
    0.012500, p95 0, max channel delta 129.
