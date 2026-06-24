# 0109: Transparency Isolation Knockout And Luminosity Masks

Status: todo
Phase: 19
Size: medium
Depends on: 0108

## Goal

Close remaining transparency gaps for isolated groups, knockout groups, and
luminosity soft masks in typical office and design PDFs.

## Scope

- Implement isolated and knockout group compositing rules.
- Support alpha and luminosity soft mask conversion.
- Keep intermediate surfaces bounded by page and group dimensions.
- Add fixtures with shadows, overlays, watermarks, and masked images.

## Non-Goals

- Implement print-production transparency flattening.
- Allocate full-resolution group surfaces when thumbnails can be clipped.
- Hide unsupported blend interactions.

## Deliverables

- Transparency group compositing updates.
- Surface allocation metrics.
- Visual comparison report for transparency fixtures.

## Acceptance Criteria

- Common transparency groups render without PDFium fallback.
- Intermediate surfaces are clipped and memory-bounded.
- Unsupported transparency cases return typed reasons with fixture evidence.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run transparency visual comparisons.
- Run memory-budget stress fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
