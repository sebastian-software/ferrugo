# 0059: Native Backend Facade Parity

Status: todo
Phase: 8
Size: medium
Depends on: 0058

## Goal

Make the native Rust renderer satisfy the same thumbnail facade contract as the
PDFium backend for supported documents.

## Scope

- Align success, unsupported, malformed, timeout, and budget error mapping.
- Ensure output image dimensions, page selection, and background behavior match
  the facade contract.
- Add backend-selection tests that exercise both native and PDFium paths.
- Document supported and fallback behavior for downstream callers.

## Non-Goals

- Remove PDFium.
- Force native rendering for unsupported documents.
- Add new product APIs.

## Deliverables

- Facade parity tests.
- Backend-selection documentation.
- Error mapping cleanup where needed.

## Acceptance Criteria

- Supported documents can switch between PDFium and native backends without API
  changes.
- Unsupported native cases fall back or fail according to documented policy.
- Tests prove the facade contract is backend-neutral for supported paths.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run representative thumbnail API tests with both backends.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
