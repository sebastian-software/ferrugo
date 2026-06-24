# 0010: Thumbnail API Facade

Status: done
Phase: 0
Size: small
Depends on: 0009

## Goal

Define the backend-neutral Rust thumbnail API facade.

## Scope

- Define PDF input representation.
- Define thumbnail options.
- Define thumbnail output.
- Define backend trait or equivalent abstraction.
- Define typed error enum.

## Non-Goals

- Bind PDFium.
- Implement rendering.
- Add Node-API types.

## Deliverables

- Rust API for single-page thumbnail rendering.
- Defaults for `page_index = 0`, `max_edge = 1024`, PNG/RGBA output, and
  `timeout = 5s`.
- Error classes for encrypted, malformed, unsupported, timeout, and internal
  failures.

## Acceptance Criteria

- The API can represent the Phase 0 thumbnail contract.
- PDFium handles do not leak into the public API.
- Defaults are explicit and tested.

## Validation

- Add unit tests for defaults and error display where useful.
- Run `cargo test`.

## Completion Notes

Completed on 2026-06-24.

- Added backend-neutral input, option, output, backend trait, and error types
  to `pdfrust-thumbnail`.
- Added tests for Phase 0 defaults, buffer layout validation, and stable error
  display.
- Kept PDFium-specific handles and naming out of the public API.
