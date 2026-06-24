# Optional Content Policy

Status: accepted for 0054.
Date: 2026-06-24.

The native renderer applies the document's default optional content visibility
for thumbnail rendering. It does not expose interactive layer toggles.

## Supported

- Page `/Resources /Properties` entries that reference `/Type /OCG` objects.
- Marked content sequences using `/OC <property> BDC ... EMC`.
- Catalog `/OCProperties /D` default configuration with `/BaseState`, `/ON`,
  and `/OFF`.
- Nested marked content, where any hidden enclosing layer hides the nested
  drawing operations.

## Unsupported

- Optional content membership dictionaries (`/Type /OCMD`).
- `/OCProperties /D /AS` usage application arrays.
- User-selectable layer state.
- Intent-specific rendering and viewer preference handling.
- Direct OCG dictionaries that cannot be matched to catalog default-state
  references.

Unsupported optional content policy returns the public `unsupported` class
instead of rendering a potentially misleading thumbnail. Unknown non-`/OC`
marked content remains neutral and does not affect visibility.

## Default Behavior

When no optional content properties are present, the content stream is rendered
unchanged. When optional content is present but the catalog has no default
configuration, the default base state is treated as visible. `/OFF` entries hide
matching groups, and `/ON` entries show matching groups even when the base state
is `/OFF`.
