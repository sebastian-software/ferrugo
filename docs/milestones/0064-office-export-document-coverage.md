# 0064: Office Export Document Coverage

Status: done
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

- Added generated `fixtures/generated/office-table.pdf`, covering an
  office-style ruled table with header fill and text cells.
- Added `office-table.pdf` to `fixtures/corpus-manifest.tsv` under the
  `office-export` family with source, license, page-count, feature, and note
  metadata.
- Added native backend smoke coverage for the office table fixture.
- Added `docs/reports/office-export-coverage-2026-06-24.md`.
- Office-export corpus summary at `--max-edge 120` reported 6 total fixtures,
  6 native renders, 1.000 native pass rate, 0 fallbacks, and 0 errors.
- PDFium differential smoke at `--max-edge 260` rendered all 6 office-export
  fixtures successfully through native and direct PDFium with matching PNG
  dimensions.
- Remaining differences are visual text fidelity risks, not unsupported
  fallbacks; dense spreadsheet/report-specific fidelity is deferred to 0073.
- Validation: `cargo fmt --check`, `cargo check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test --quiet`, manifest/PDF set comparison, office-export corpus
  summary, and PDFium differential smoke.
