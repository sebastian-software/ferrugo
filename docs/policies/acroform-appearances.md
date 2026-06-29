# AcroForm Appearance Policy

Status: accepted for 0053, updated for 0194 and 0206.
Date: 2026-06-24. Updated: 2026-06-29.

The native renderer treats AcroForm widgets as static page annotations for
thumbnail rendering. It may render an existing normal appearance stream, but it
does not implement an interactive form engine.

## Supported

- Widget annotations reachable from a page `/Annots` array.
- Existing `/AP /N` Form XObject appearance streams.
- Existing `/AP /N` appearance state dictionaries selected by `/AS`.
- Static text-field, button, and signature placeholder appearances when their
  appearance stream uses already supported drawing operators.
- Static choice/combo-box and rotated field appearances when the source PDF has
  already generated a normal appearance stream.
- XFA/AcroForm hybrid documents when static page content or static widget
  appearances are already present and do not require an XFA runtime.
- Already-flattened form exports where field values have been written into
  ordinary page content by the producer.
- Bounded synthetic thumbnail appearances for missing-appearance text fields,
  choice fields, checkboxes, and radio buttons when the widget dictionary
  exposes common `/FT`, `/V`, `/AS`, `/Ff`, and `/Rect` values.

## Unsupported

- Editing or filling fields.
- Calculating field values.
- Running document, field, annotation, or action JavaScript.
- Validating digital signatures or interpreting `/ByteRange` contents.
- Dynamic XFA documents that require an XFA runtime to synthesize layout or
  field appearances.
- Matching every viewer-specific widget style.
- Executing JavaScript, XFA, calculations, or dynamic validation to derive a
  field value.

## XFA Boundary

The native renderer detects `/AcroForm /XFA` before page rendering. If the
document also exposes non-empty AcroForm `/Fields`, native rendering may
continue and use the existing static page, annotation, and widget appearance
paths. If `/XFA` is present without static fields, rendering returns the
unsupported bucket `form.xfa-dynamic`.

The renderer does not decode XFA packets to build layout, execute scripts, or
derive field values. XFA data is treated as a policy signal only.

## Digital Signature Boundary

Signature widgets are rendered as static AcroForm widget appearances. The
native metadata path may report that signature fields and `/ByteRange` metadata
are present, but this is a presence signal only. It is not certificate-chain
validation, hash verification, legal status, or proof that `/Contents` is
well-formed.

The renderer must not mutate signed documents, recalculate signature values, or
interpret signature contents while generating thumbnails.

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

## Mutation And Preview Boundary

The default render mode is document-state rendering: existing `/AP /N`
appearance streams and `/AP /N` appearance-state dictionaries selected by `/AS`
are authoritative, even when field values such as `/V` look newer than the
embedded appearance. This keeps thumbnail rendering read-only and matches the
source bytes rather than acting as a viewer-side form engine.

Callers that need to preview changed field values must request an explicit form
mutation mode through the facade. The Rust-native backend currently rejects that
mode with `unsupported` and bucket `form.appearance-mutation`; it does not
silently update `/V`, `/AS`, `/AP`, widget dictionaries, incremental revisions,
or flattened page content.

Later form-editing or flattening support must live behind an explicit API that
owns generated appearance data and save semantics. Thumbnail rendering may reuse
already-mutated or flattened source PDFs, but it must not persist synthetic
appearances back into the input document.

## Synthesis Limits

Synthetic widget appearances are deterministic and bounded by the widget
rectangle. They use simple fill/stroke geometry and do not mutate the document,
write new PDF objects, calculate inherited field state beyond the widget
dictionary, or persist generated appearances.

## Security Boundary

The renderer must never execute form scripts or external actions while rendering
thumbnails. Field values are data for future appearance generation decisions,
not executable instructions.
