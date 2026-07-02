# Raster Flattening Policy

Status: active.
Date: 2026-07-02.

Ferrugo flattens cubic Bezier path segments adaptively in device space before
fill, stroke, clip, and pattern rasterization. The renderer applies the
effective page transform first, then subdivides each cubic until the maximum
distance from either control point to the emitted line chord is at most `0.5`
device pixels.

This policy is intentionally device-space based. Thumbnail renders should not
pay for invisible sub-pixel curve detail, while larger zoom or scale transforms
can emit more segments when the curve would otherwise drift visibly.

## Budget Boundary

Adaptive flattening keeps using the existing `max_flattened_segments` path
complexity budget. If a curve-heavy path would exceed that budget, rendering
returns the same typed path-complexity error as the previous fixed subdivision
route.

Stroked cubic curves keep a 12-segment floor while stroke rasterization still
uses line-based distance predicates. That keeps existing row-bucket routing
available for dense stroked curves. Revisit this floor when stroke-to-fill
lowering lands.

## Diagnostics

`ferrugo trace-native` reports a `path_flattening_summary` block with:

- `device_tolerance_px`
- `path_items`
- `clip_items`
- `cubic_curves`
- `flattened_edges`
- `curve_segments`
- `max_curve_segments_per_curve`
- `max_flattened_edges_per_item`

Use these counters when deciding whether a curve-heavy fixture benefits from
adaptive flattening or is blocked by a different raster bottleneck.

## Validation

Changes to this policy need:

- unit coverage for lower thumbnail-scale segment counts and higher segment
  counts under larger device transforms;
- native render coverage on generated vector fixtures;
- visual review on the touched vector fixture set under
  [visual diff thresholds](visual-diff-thresholds.md);
- release-build performance evidence following the
  [performance claims policy](performance-claims.md) before making speed claims.
