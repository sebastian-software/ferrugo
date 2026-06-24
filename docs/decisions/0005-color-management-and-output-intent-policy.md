# 0005: Color Management And Output Intent Policy

Status: accepted
Date: 2026-06-24

## Context

The native renderer targets thumbnails for common office, browser, report, form,
and scanned documents. These workflows need predictable colors and explicit
errors, but they do not require print-proof color management.

The renderer already supports the common process color cases needed by the
current corpus: DeviceGray, DeviceRGB, DeviceCMYK, Indexed DeviceGray/RGB, and
calibrated Gray/RGB approximation. ICCBased, Lab, Separation, DeviceN, and spot
color workflows need a stricter policy because silent approximation can be
misleading for callers that care about color-managed output.

## Decision

The native renderer treats OutputIntent dictionaries as rendering metadata in
the current thumbnail path. A catalog OutputIntent does not cause native ICC
profile parsing, profile conversion, or a PDFium fallback when page content uses
otherwise supported DeviceRGB/DeviceGray/DeviceCMYK color spaces.

Accepted thumbnail conversions:

- DeviceGray maps directly to grayscale RGBA.
- DeviceRGB maps directly to RGBA.
- DeviceCMYK uses the existing bounded approximate CMYK-to-RGB conversion.
- Indexed DeviceGray/RGB keeps compact indexed samples and resolves palette
  colors during rasterization.
- CalGray and CalRGB are accepted as DeviceGray/DeviceRGB approximations for
  thumbnails.

Unsupported color workflows remain typed unsupported errors:

- ICCBased image color spaces.
- Lab color spaces.
- Separation and DeviceN spot-color workflows.
- Output proofing or print-production color simulation.

No native C color-management dependency is added in this phase. A future ICC
implementation must come with corpus evidence, a memory profile, dependency
review, and regression tests for unsupported-to-supported behavior.

## Consequences

- Common RGB, grayscale, CMYK, Indexed, and calibrated office/browser/scan
  documents continue rendering natively without requiring PDFium.
- OutputIntent metadata is documented and test-covered without introducing a
  profile parser or unbounded profile memory use.
- Color-managed and spot-color workflows stay explicit unsupported cases until
  they are justified by real corpus coverage and a bounded implementation.
