# 0124: Technical Drawing And CAD PDF Fidelity

Status: todo
Phase: 22
Size: medium
Depends on: 0123

## Goal

Cover technical drawings and CAD-style PDFs that stress thin vector geometry,
large coordinate systems, clipping, and repeated symbols.

## Scope

- Add technical drawing fixtures with fine lines, hatches, labels, and symbols.
- Validate large page boxes, user units, and precision-sensitive transforms.
- Track path flattening, clipping, dash, and join fidelity.
- Keep vector workloads bounded by explicit segment and raster budgets.

## Non-Goals

- Parse CAD source formats.
- Support interactive layer toggling beyond existing optional-content policy.
- Guarantee print-production exactness for every engineering drawing.

## Deliverables

- Technical drawing fixture family.
- Vector fidelity report for drawing-style pages.
- Budget notes for large coordinate systems and repeated geometry.

## Acceptance Criteria

- Typical technical drawing thumbnails render without geometry collapse.
- Fine strokes remain visible where PDFium renders visible marks.
- Excessive geometry fails with typed budget errors instead of unbounded work.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run technical drawing visual comparisons.
- Run vector and memory budget benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
