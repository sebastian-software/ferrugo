# 0089: JPX JBIG2 And Specialized Image Codec Policy

Status: todo
Phase: 15
Size: medium
Depends on: 0088

## Goal

Decide and implement the practical policy for specialized image codecs that
block native rendering of scanned and office-exported PDFs.

## Scope

- Inventory JPX, JBIG2, CCITT, and uncommon image filter usage in the corpus.
- Choose pure Rust, optional dependency, fallback, or unsupported handling per
  codec.
- Add deterministic errors for unsupported codec paths.
- Implement the highest-impact codec slice if the policy selects one.

## Non-Goals

- Ship unsafe decoder bindings without a reviewable safety boundary.
- Implement every rare image filter before measuring corpus impact.
- Silently fall back to PDFium in native-default mode.

## Deliverables

- Specialized codec decision record.
- Codec support or explicit unsupported-error implementation.
- Corpus report showing remaining image-codec blockers.

## Acceptance Criteria

- Each specialized image codec has a documented native strategy.
- Supported codecs respect memory and decompression budgets.
- Unsupported codecs produce actionable errors and support matrix entries.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-codec corpus classification.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
