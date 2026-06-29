# 0193: Annotation Print Preview Fidelity Gate

Status: done
Phase: 36
Size: medium
Depends on: 0192

## Goal

Validate annotation appearance behavior for print-preview-like workflows where
comments, stamps, highlights, and form marks must be visible or hidden
predictably.

## Scope

- Add fixtures for printable, non-printable, hidden, and no-view annotations.
- Verify annotation appearance streams, opacity, z-order, and clipping.
- Document policy differences between screen preview and print preview.
- Add typed unsupported reasons for unsupported synthesized appearances.

## Non-Goals

- Implement interactive annotation editing.
- Flatten annotations into source documents.
- Treat malicious annotation actions as renderable content.

## Deliverables

- Annotation print preview coverage report.
- Fixture updates for common annotation flags.
- Policy update for preview modes.

## Acceptance Criteria

- Annotation flags produce documented screen and print-preview behavior.
- Common appearance streams render without PDFium.
- Unsupported synthesis remains specific and typed.

## Validation

- Run native-only `cargo test`.
- Run annotation fixture visual comparisons.
- Run preview mode classification tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `AnnotationMode` to thumbnail options and wired `render-native
  --annotation-mode screen|print`.
- Applied PDF annotation `/F` visibility flags for screen and print-preview
  rendering: hidden/invisible suppress both modes, no-view suppresses screen
  only, and print mode requires the print bit.
- Added generated fixtures and `fixtures/annotation-print-preview-manifest.tsv`
  for appearance streams, flags, synthesized markup, inert links, and
  unsupported FreeText synthesis.
- Promoted `annotation.appearance` to a public stable unsupported-feature bucket
  and used it for appearance-free FreeText annotations.
- Documented preview policy and validation in
  `docs/policies/annotation-fallbacks.md`,
  `docs/backend/native.md`, and
  `docs/reports/annotation-print-preview-fidelity-2026-06-29.md`.
- Validation completed on 2026-06-29: focused annotation tests, CLI
  annotation-mode parser test, supported fallback gate, unsupported FreeText
  gate, Poppler visual comparison for supported annotation families, and broad
  workspace gates.
