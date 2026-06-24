# 0014: Error Taxonomy Mapping

Status: done
Phase: 0
Size: small
Depends on: 0011

## Goal

Map PDFium/backend failures into the project's stable thumbnail error classes.

## Scope

- Map encrypted or password-protected PDFs.
- Map malformed PDFs.
- Map unsupported features.
- Map timeout behavior.
- Map internal backend failures.

## Non-Goals

- Perfectly preserve every PDFium error code.
- Design Node-specific error classes.
- Solve process isolation.

## Deliverables

- Error mapping implementation.
- CLI error messages that match the taxonomy.
- Documentation for approximate mappings.

## Acceptance Criteria

- Each required error class has at least one test or documented manual probe.
- Unknown backend failures map to internal error.
- Error messages are stable enough for CLI use.

## Validation

- Run fixture or manual probes for malformed and encrypted files where
  available.
- Run unit tests for error mapping.

## Completion Notes

Completed on 2026-06-24.

- Added `ThumbnailErrorClass` and `ThumbnailError::class()`.
- Mapped known PDFium error codes to stable thumbnail classes.
- Added CLI render error output with the stable class.
- Documented the taxonomy in `docs/errors.md`.
- Timeout is reserved in the facade taxonomy; live timeout probes require the
  local PDFium backend.
