# 0082: Native Default API And CLI Stabilization

Status: done
Phase: 14
Size: medium
Depends on: 0081

## Goal

Stabilize the native renderer as the default API and CLI path for supported
documents while keeping PDFium available only as an explicit fallback.

## Scope

- Make backend selection predictable across library and CLI entry points.
- Add native-default tests for supported corpus categories.
- Preserve explicit PDFium selection for comparison and emergency fallback.
- Document API compatibility and error behavior for downstream users.

## Non-Goals

- Remove PDFium code.
- Promise full PDF specification coverage.
- Add new rendering features without a linked retirement blocker.

## Deliverables

- Native-default API and CLI behavior.
- Backend-selection documentation.
- Regression tests for native-default supported documents.

## Acceptance Criteria

- Supported fixtures render through native paths without PDFium opt-in.
- PDFium is never selected silently for supported documents.
- Unsupported documents return stable errors or explicit fallback decisions.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run native-default CLI smoke tests.
- Run PDFium-enabled comparison smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Changed `render` / `render-auto` so PDFium fallback requires explicit
  `--allow-pdfium-fallback` opt-in.
- Preserved explicit PDFium commands behind the `pdfium` feature.
- Added regression coverage for supported native default rendering, default
  fallback denial, and explicit fallback flag parsing.
- Documented backend-selection behavior in `docs/backend/native.md`,
  `docs/backend/pdfium.md`, and `docs/packaging.md`.
- Published
  `docs/reports/native-default-api-cli-stabilization-2026-06-24.md`.

Validation passed:

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo test -p ferrugo-cli --no-default-features`
- `cargo test -p ferrugo-cli --features pdfium`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Implementation commit:

- `975b105 feat: require explicit pdfium fallback opt-in`
