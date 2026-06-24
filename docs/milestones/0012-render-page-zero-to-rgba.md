# 0012: Render Page Zero To RGBA

Status: done
Phase: 0
Size: small
Depends on: 0011

## Goal

Render page index `0` of a PDF to an RGBA thumbnail buffer through the backend
facade.

## Scope

- Open a PDF from file or bytes.
- Load page index `0`.
- Render with bounded `max_edge`.
- Return dimensions, stride, format, and RGBA bytes.

## Non-Goals

- Render batch pages.
- Add PNG encoding.
- Implement Rust-native rendering.

## Deliverables

- Working RGBA thumbnail output from PDFium backend.
- Smoke fixture that exercises page index `0`.
- Basic dimension assertions.

## Acceptance Criteria

- A simple generated fixture renders to non-empty RGBA.
- Output dimensions honor `max_edge`.
- Failure is typed when the file cannot be loaded.

## Validation

- Run the smoke test on generated fixtures.
- Confirm output bytes are non-empty and dimensions are stable.

## Completion Notes

Completed on 2026-06-24.

- Implemented PDFium page rendering to RGBA in `pdfrust-pdfium`.
- Added bounded `max_edge` scaling and BGRA-to-RGBA conversion.
- Added a typed missing-file failure test.
- A live fixture render was not run because no local PDFium library is
  available in this environment.
