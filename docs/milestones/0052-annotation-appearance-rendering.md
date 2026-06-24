# 0052: Annotation Appearance Rendering

Status: todo
Phase: 7
Size: medium
Depends on: 0051

## Goal

Render annotation appearances that are visible in common reviewed, signed, or
commented PDFs.

## Scope

- Resolve annotation dictionaries from page objects.
- Render normal appearance streams for supported annotation types.
- Support link, text markup, stamp, and widget appearance handling as driven by
  fixtures.
- Define fallback behavior for annotations without usable appearances.

## Non-Goals

- Interactive annotation editing.
- JavaScript actions.
- Full PDF form behavior.

## Deliverables

- Annotation appearance discovery and render path.
- Fixtures for visible annotation appearances.
- Unsupported annotation diagnostics.

## Acceptance Criteria

- PDFs with appearance streams show visible annotations in thumbnails.
- Missing or unsupported appearances do not abort otherwise renderable pages.
- Action dictionaries are ignored or reported without execution.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for annotation fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
