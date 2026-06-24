# 0017: Run Local PDFium Build

Status: todo
Phase: 1
Size: medium
Depends on: 0016

## Goal

Run the pinned PDFium checkout and minimal GN/Ninja build on a machine with
`depot_tools`.

## Scope

- Install or point to `depot_tools`.
- Run the checkout recipe.
- Run GN generation with the Phase 0 args.
- Build the PDFium target needed by `pdfrust-pdfium`.
- Record the exact dynamic-library path.

## Non-Goals

- Optimize binary size.
- Package PDFium for distribution.
- Add Node-API bindings.

## Deliverables

- Updated `docs/measurements/pdfium-build-baseline.md`.
- Confirmed `PDFRUST_PDFIUM_LIBRARY` value.
- Build failure log if the build does not complete.

## Acceptance Criteria

- `gn gen` and `ninja` either succeed or have exact failure output recorded.
- The produced PDFium library path is documented.

## Validation

- Run the backend smoke probe against the built library.

## Completion Notes

Empty until done.
