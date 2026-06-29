# Optional Content Policy

Status: accepted for 0054, updated for 0192.
Date: 2026-06-29.

The native renderer applies the document's default optional content visibility
for thumbnail rendering. It does not expose interactive layer toggles.

## Supported

- Page `/Resources /Properties` entries that reference `/Type /OCG` objects.
- Marked content sequences using `/OC <property> BDC ... EMC`.
- Catalog `/OCProperties /D` default configuration with `/BaseState`, `/ON`,
  and `/OFF`.
- Nested marked content, where any hidden enclosing layer hides the nested
  drawing operations.
- Bounded document metadata for optional content group count, default base
  state, default `/ON` and `/OFF` counts, and unsupported behavior flags.

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

Metadata inspection is more permissive than rendering. It reports `/D /AS`,
`/OCMD`, and direct OCG dictionary signals through
`DocumentMetadata.optional_content.has_unsupported_behavior` so consumers can
show policy state, decide whether flattening is safe, or route the document to
a fallback path without attempting native rasterization first.

## Default Behavior

When no optional content properties are present, the content stream is rendered
unchanged. When optional content is present but the catalog has no default
configuration, the default base state is treated as visible. `/OFF` entries hide
matching groups, and `/ON` entries show matching groups even when the base state
is `/OFF`.

Flattened native output follows this default visibility only. Hidden groups are
not painted for visual similarity, print/export-specific `/AS` usage rules are
not interpreted, and user layer toggles are not persisted or replayed. A
consumer that needs interactive layer panels should use the exposed metadata as
classification input and maintain its own UI state outside the native thumbnail
renderer.
