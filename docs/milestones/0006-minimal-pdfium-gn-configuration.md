# 0006: Minimal PDFium GN Configuration

Status: done
Phase: 0
Size: small
Depends on: 0005

## Goal

Create and document the minimal PDFium GN configuration for thumbnail rendering.

## Scope

- Disable V8.
- Disable XFA.
- Use AGG.
- Disable Skia.
- Prefer a complete static library where practical.
- Record any platform-specific adjustments.

## Non-Goals

- Optimize final binary size.
- Build release packages.
- Support every target platform.

## Deliverables

- Documented GN args.
- A short explanation for each non-default flag.
- Notes on any build flags that failed or were unavailable.

## Acceptance Criteria

- The configuration is enough to start a PDFium build.
- V8 and XFA are disabled.
- Skia is not the selected render path.
- Any deviation from the Phase 0 decision baseline is documented.

## Validation

- Run GN generation with the documented args.
- Record command output or a summary.

## Completion Notes

Completed on 2026-06-24.

- Added `docs/build/pdfium-gn-args.md`.
- Documented the cut-down AGG, no-V8, no-XFA, no-Skia configuration.
- Local `gn gen` could not be executed because `depot_tools` is not installed
  in this environment.
