# 0094: Structure Outline Metadata And Page Labels

Status: completed
Phase: 16
Size: small
Depends on: 0093

## Goal

Expose common non-rendering document metadata needed by consumers that replace
PDFium-backed document inspection.

## Scope

- Parse document info, XMP presence, outlines, page labels, named destinations,
  and tagged PDF structure presence.
- Keep unsupported or huge metadata trees bounded.
- Add API tests that do not require rendering pixels.
- Document which metadata is parsed, ignored, or intentionally unsupported.

## Non-Goals

- Build accessibility extraction.
- Interpret all tagged PDF semantics.
- Render outlines or viewer UI.

## Deliverables

- Native metadata inspection API coverage.
- Metadata fixture set.
- Support matrix updates.

## Acceptance Criteria

- Downstream users can avoid PDFium for common metadata lookups.
- Large metadata structures cannot cause unbounded memory growth.
- Unsupported structure features are explicit and test-covered.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run metadata fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Extended `DocumentMetadata` with document info, structure presence, outline,
  and page-label fields.
- Added native classic-document extraction for `/Info`, XMP presence, outlines,
  named destinations, tagged-PDF presence, and direct page labels.
- Added generated fixture `metadata-outline-page-labels.pdf` plus manifest
  coverage.
- Documented support boundaries in `docs/policies/document-metadata.md`.
- Evidence report: `docs/reports/structure-outline-metadata-2026-06-25.md`.
