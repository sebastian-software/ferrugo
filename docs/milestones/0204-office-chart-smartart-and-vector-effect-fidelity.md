# 0204: Office Chart SmartArt And Vector Effect Fidelity

Status: todo
Phase: 38
Size: medium
Depends on: 0203

## Goal

Improve Rust-native rendering for common office chart, diagram, SmartArt-style,
and vector-effect PDFs produced by presentation and document suites.

## Scope

- Add reduced fixtures for chart fills, shadows, clipped legends, connectors,
  grouped vector effects, and gradient-heavy diagram exports.
- Measure transparency, clipping, pattern, and text-overlay interactions inside
  nested form XObjects.
- Track chart-specific visual drift separately from generic vector stress.
- Prioritize implementation work that improves typical office documents without
  adding unbounded caches.

## Non-Goals

- Reconstruct editable chart or SmartArt semantics.
- Support every proprietary office effect exactly.
- Add PDFium runtime fallback for chart pages.

## Deliverables

- Office chart and vector-effect corpus.
- Fidelity report with reduced failure examples.
- Prioritized renderer fixes for high-impact effects.

## Acceptance Criteria

- Common chart and diagram exports render with stable fills, strokes, clipping,
  and labels.
- Unsupported effects produce typed diagnostics.
- Nested vector effects do not exceed memory or recursion budgets.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run office chart visual comparisons.
- Run form XObject recursion and memory-budget checks.
- Run operator coverage scan for chart fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
