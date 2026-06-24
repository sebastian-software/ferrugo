# 0067: Mixed Text Image Page Fidelity

Status: todo
Phase: 10
Size: medium
Depends on: 0066

## Goal

Improve fidelity for pages that combine selectable text, placed images, vector
marks, and clipping.

## Scope

- Test ordering between images, paths, text, forms, and transparency boundaries.
- Fix common z-order and clipping regressions.
- Add mixed-layout fixtures based on reports, invoices, and handouts.
- Measure visual deltas against PDFium with stable thresholds.

## Non-Goals

- Add new codecs unless required by an accepted fixture.
- Implement interactive document behavior.
- Tune for single-pixel parity before structural correctness.

## Deliverables

- Mixed-layout fixture set.
- Renderer fixes for ordering and clipping gaps.
- Differential thresholds for mixed-layout pages.

## Acceptance Criteria

- Representative mixed-layout pages render without missing major elements.
- Z-order and clip behavior match PDFium for supported features.
- Known remaining fidelity gaps are documented by feature.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run mixed-layout visual comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
