# AcroForm Appearance Policy

Status: accepted for 0053.
Date: 2026-06-24.

The native renderer treats AcroForm widgets as static page annotations for
thumbnail rendering. It may render an existing normal appearance stream, but it
does not implement an interactive form engine.

## Supported

- Widget annotations reachable from a page `/Annots` array.
- Existing `/AP /N` Form XObject appearance streams.
- Existing `/AP /N` appearance state dictionaries selected by `/AS`.
- Static text-field, button, and signature placeholder appearances when their
  appearance stream uses already supported drawing operators.

## Unsupported

- Editing or filling fields.
- Calculating field values.
- Running document, field, annotation, or action JavaScript.
- Validating digital signatures or interpreting `/ByteRange` contents.
- XFA forms.
- Generating synthetic appearances from `/DA`, `/V`, `/DV`, fonts, or widget
  style dictionaries.

## Missing Or Dynamic Appearances

If a widget does not have a usable normal appearance stream, the native renderer
does not synthesize one in 0053. The page remains otherwise renderable and the
widget is skipped. Synthetic fallback appearances require corpus evidence that
they materially improve typical thumbnails without hiding missing form-engine
semantics.

Malformed appearance streams follow the same policy as other annotation
appearances: they must not execute interactive behavior, and they should not
abort unrelated page content unless the malformed object also breaks required
page parsing.

## Security Boundary

The renderer must never execute form scripts or external actions while rendering
thumbnails. Field values are data for future appearance generation decisions,
not executable instructions.
