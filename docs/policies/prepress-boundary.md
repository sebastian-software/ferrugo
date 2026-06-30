# Prepress Boundary Policy

Status: accepted for milestone 0128.
Date: 2026-06-25.

This policy defines how the native renderer treats print-production PDFs while
the project remains focused on thumbnails and preview rendering.

## Rendering Contract

The native renderer should produce useful thumbnails for print-oriented PDFs
when their visible content is expressible through supported page geometry,
paths, text, colors, transparency, images, and forms.

The native renderer does not provide print-proofing guarantees. It must not be
used as evidence for plate separation, calibrated proof output, press-ready
color validation, trapping, imposition, or standards conformance.

## Page Boxes

CropBox is the selected visible page box for native thumbnails when present;
MediaBox is used otherwise. This matches the existing thumbnail contract and is
covered by native tests.

BleedBox and TrimBox are accepted as document context in fixtures and reports,
but they do not currently select the rendered thumbnail boundary. A later
decision point can add explicit page-box selection if consumers need bleed or
trim preview modes.

## OutputIntents And Color

Catalog OutputIntents are accepted as metadata/context and must not make an
otherwise renderable thumbnail fail.

OutputIntents do not currently trigger color-managed proofing. Device colors,
ICCBased colors, Separation colors, and DeviceN colors continue to follow the
renderer approximation paths used for thumbnail output.

## Spot Colors And Overprint

Spot colors and overprint are thumbnail approximations. Separation and DeviceN
tint transforms may be used to derive visible RGB output, and overprint flags
may be retained as renderer diagnostics, but the result is not a separations or
press-overprint proof.

Spot-color visual review should use
`fixtures/spot-color-visual-review-manifest.tsv` when changes affect
Separation, DeviceN, DeviceCMYK alternate conversion, or tint-transform
evaluation. Review thresholds must be recorded in the review report because
RGB approximation drift is expected and category-local.

Prepress visual diffs may use broader thumbnail-oriented thresholds than the
global defaults when the review report records the threshold values and the
reason. Those thresholds are local evidence for this document category, not a
global replacement for strict visual review.

## Printer Marks

Trim marks, registration marks, color bars, and similar printer marks should
render as ordinary vector content when they are inside the selected visible
page box. Marks outside CropBox may be clipped by the thumbnail contract.

## PDFium Role

PDFium remains a maintainer-only visual oracle for triage. It is not part of
the native renderer contract and does not define proofing-level correctness for
prepress features.
