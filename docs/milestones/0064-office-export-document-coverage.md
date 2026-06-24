# 0064: Office Export Document Coverage

Status: todo
Phase: 10
Size: medium
Depends on: 0063

## Goal

Improve native rendering for PDFs exported from common office applications.

## Scope

- Target word-processor, spreadsheet, and slide-deck PDF exports.
- Measure text, table, image, clipping, and transparency gaps in those files.
- Add focused fixes for the highest-volume unsupported constructs.
- Add representative generated fixtures where committed samples are not viable.

## Non-Goals

- Implement office file parsing.
- Guarantee pixel-perfect parity for every export engine.
- Optimize print-production workflows.

## Deliverables

- Office-export fixture set or generation scripts.
- Native renderer fixes for observed blockers.
- Differential report against PDFium.

## Acceptance Criteria

- Representative office-export documents render natively without fallback.
- Remaining visual differences are categorized and prioritized.
- Unsupported errors identify the missing renderer feature.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run office-export corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
