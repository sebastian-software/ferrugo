# 0091: Annotation Without Appearance Fallbacks

Status: todo
Phase: 15
Size: medium
Depends on: 0090

## Goal

Render common annotations that do not include appearance streams without relying
on PDFium to synthesize them.

## Scope

- Add native fallback rendering for links, highlights, underlines, squares,
  circles, and simple text notes where appropriate.
- Keep appearance streams authoritative when present.
- Define which annotation types remain non-rendering metadata.
- Add fixtures for office comments, browser links, and review markup.

## Non-Goals

- Build interactive editing behavior.
- Render pop-up UI chrome.
- Guess complex vendor-specific annotation styles.

## Deliverables

- Annotation fallback renderer.
- Annotation policy documentation.
- Fixture and comparison updates.

## Acceptance Criteria

- Common appearance-free markup is visible in native renders.
- Link rectangles and non-visual metadata do not create misleading artifacts.
- Unsupported annotation types are documented and stable.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run annotation visual comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
