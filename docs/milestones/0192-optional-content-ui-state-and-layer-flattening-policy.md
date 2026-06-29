# 0192: Optional Content UI State And Layer Flattening Policy

Status: done
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

- Added `DocumentMetadata.optional_content` with bounded OCG/default-state
  metadata and unsupported behavior flags for `/D /AS`, `/OCMD`, and direct
  OCG dictionaries.
- Added nested and usage-application optional-content fixtures plus
  `fixtures/optional-content-ui-state-manifest.tsv`.
- Kept rendering deterministic: default-on/off and nested OCGs render natively;
  dynamic usage applications and OCMD policies stay typed
  `graphics.optional-content` fallback boundaries.
- Documented flattening, unsupported UI state, and consumer metadata routing in
  `docs/policies/optional-content.md`,
  `docs/backend/native.md`, and
  `docs/reports/optional-content-ui-state-2026-06-29.md`.
- Validation completed on 2026-06-29:
  `cargo test -p pdfrust-native optional_content -- --nocapture`, supported
  optional-content fallback gate, unsupported boundary gate, Poppler visual diff
  for default-on/default-off/nested fixtures, single-file metadata extraction,
  and broad workspace gates.
