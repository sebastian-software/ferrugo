# Annotation Fallback Policy

Status: accepted for 0091.
Date: 2026-06-25.

The native renderer may synthesize static thumbnail appearances for common page
annotations that do not provide a usable normal appearance stream. Existing
`/AP /N` appearance streams remain authoritative.

## Supported Fallbacks

- `/Highlight`: filled QuadPoints or Rect using the annotation color, default
  yellow.
- `/Underline`: stroked lower QuadPoints edge using the annotation color,
  default red.
- `/Square`: stroked rectangle using the annotation color.
- `/Circle`: bounded polygonal ellipse using the annotation color.
- `/Text`: small static note icon using the annotation color and a bounded
  20-unit icon footprint.

## Non-Visual Metadata

- `/Link` annotations without appearance streams remain invisible.
- URI, action, destination, popup, and contents metadata are not executed or
  displayed by thumbnail rendering.
- Unknown annotation subtypes without appearances are skipped.

## Bounds And Performance

Fallback drawing is serialized into a small synthetic content stream and then
uses the existing path rasterizer. QuadPoint synthesis is capped at 32 quads per
annotation. Circle fallback uses a 12-segment polygonal ellipse instead of
cubic curves to keep review markup rendering bounded.

## Security Boundary

The renderer must never execute annotation actions, JavaScript, launch actions,
or external links while rendering thumbnails. Annotation dictionaries are visual
metadata only unless a future explicit non-rendering API exposes them.
