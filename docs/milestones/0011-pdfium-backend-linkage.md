# 0011: PDFium Backend Linkage

Status: todo
Phase: 0
Size: medium
Depends on: 0006, 0010

## Goal

Connect the Rust thumbnail facade to a locally built PDFium library.

## Scope

- Add a PDFium backend crate or module.
- Link against the local cut-down PDFium build.
- Initialize and shut down PDFium safely.
- Keep backend execution serialized.

## Non-Goals

- Ship PDFium binaries.
- Support all platforms.
- Add Node-API bindings.

## Deliverables

- Local PDFium backend implementation shell.
- Build configuration for a local PDFium path.
- Documentation for required environment variables or config.

## Acceptance Criteria

- `cargo check` or equivalent backend build passes with a configured PDFium
  path.
- Backend internals do not leak PDFium handles to public API consumers.
- Serialization strategy is documented.

## Validation

- Run local build with PDFium path configured.
- Run a smoke test that initializes PDFium and reports version/build details if
  available.

## Completion Notes

Empty until done.

