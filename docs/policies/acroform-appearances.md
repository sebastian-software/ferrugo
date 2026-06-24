# AcroForm Appearance Policy

Status: accepted for 0053, updated for 0092.
Date: 2026-06-24. Updated: 2026-06-25.

The native renderer treats AcroForm widgets as static page annotations for
thumbnail rendering. It may render an existing normal appearance stream, but it
does not implement an interactive form engine.

## Supported

- Widget annotations reachable from a page `/Annots` array.
- Existing `/AP /N` Form XObject appearance streams.
- Existing `/AP /N` appearance state dictionaries selected by `/AS`.
- Static text-field, button, and signature placeholder appearances when their
  appearance stream uses already supported drawing operators.
- Bounded synthetic thumbnail appearances for missing-appearance text fields,
  choice fields, checkboxes, and radio buttons when the widget dictionary
  exposes common `/FT`, `/V`, `/AS`, `/Ff`, and `/Rect` values.

## Unsupported

- Editing or filling fields.
- Calculating field values.
- Running document, field, annotation, or action JavaScript.
- Validating digital signatures or interpreting `/ByteRange` contents.
- XFA forms.
- Matching every viewer-specific widget style.
- Executing JavaScript, XFA, calculations, or dynamic validation to derive a
  field value.

## Missing Or Dynamic Appearances

If a widget does not have a usable normal appearance stream, the native renderer
may synthesize a static thumbnail appearance for common text, choice, checkbox,
and radio widgets. Existing normal appearances remain authoritative. Text and
choice fallback values are drawn only when the synthesized content can use a
page font resource; otherwise the widget frame remains visible and the value is
skipped rather than failing the page render.

Malformed appearance streams follow the same policy as other annotation
appearances: they must not execute interactive behavior, and they should not
abort unrelated page content unless the malformed object also breaks required
page parsing.

## Synthesis Limits

Synthetic widget appearances are deterministic and bounded by the widget
rectangle. They use simple fill/stroke geometry and do not mutate the document,
write new PDF objects, calculate inherited field state beyond the widget
dictionary, or persist generated appearances.

## Security Boundary

The renderer must never execute form scripts or external actions while rendering
thumbnails. Field values are data for future appearance generation decisions,
not executable instructions.
