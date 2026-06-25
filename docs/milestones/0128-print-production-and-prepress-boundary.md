# 0128: Print Production And Prepress Boundary

Status: done
Phase: 23
Size: medium
Depends on: 0127

## Goal

Define and validate the native renderer boundary for print-production PDFs with
bleed boxes, output intents, spot colors, overprint, and trim marks.

## Scope

- Add prepress-style fixtures with trim, bleed, registration marks, and output
  intents.
- Document approximation boundaries for spot colors and overprint.
- Ensure page box selection stays explicit and testable.
- Track visual differences that are acceptable for thumbnails but not print
  proofing.

## Non-Goals

- Provide print-proofing guarantees.
- Implement full prepress validation.
- Replace color-managed production workflows.

## Deliverables

- Prepress boundary policy.
- Print-production fixture family.
- Visual-diff report with thumbnail-oriented tolerances.

## Acceptance Criteria

- Thumbnail rendering remains useful for print-oriented PDFs.
- Print-proof-only features are documented as approximations or unsupported.
- Page box and output-intent behavior is covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run prepress visual comparisons.
- Run color and page-box fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated prepress boundary fixtures covering trim/bleed marks,
  OutputIntent metadata with page boxes, registration color bars, and
  spot-color overprint approximation.
- Added `fixtures/prepress-boundary-manifest.tsv` with eight focused rows
  across `trim-bleed`, `output-intent`, `registration`, and `spot-overprint`
  families.
- Added native regression coverage for prepress rendering and CropBox-derived
  first-page metadata.
- Documented the thumbnail renderer boundary in
  `docs/policies/prepress-boundary.md`: CropBox is the visible thumbnail box,
  BleedBox/TrimBox are context, OutputIntents are not proofing, and spot
  colors/overprint are approximated.
- Native fallback gate: 8/8 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 8/8 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle with prepress thumbnail thresholds
  `--max-mae 6.5 --max-p95 42 --max-changed-ratio 0.13`: 2 exact matches,
  6 accepted drift, 0 blockers, 0 native render errors, 0 PDFium render
  errors.
- Report: `docs/reports/prepress-boundary-fidelity-2026-06-25.md`.
