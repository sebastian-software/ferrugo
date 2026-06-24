# 0040: Typical Document Coverage Gate

Status: todo
Phase: 4
Size: medium
Depends on: 0039

## Goal

Establish a coverage gate for typical thumbnail documents and decide the next
renderer focus from evidence.

## Scope

- Define a local corpus manifest for office exports, browser PDFs, invoices,
  scanned pages, image-heavy PDFs, and vector-heavy PDFs.
- Keep private or licensed PDFs outside Git.
- Record expected behavior by category: render, unsupported, encrypted, or
  malformed.
- Run Rust backend and PDFium backend comparisons for the corpus metadata.
- Identify the next highest-value renderer gaps.

## Non-Goals

- Claim full PDFium parity.
- Commit private PDFs.
- Implement every missing feature found by the corpus.
- Ship Node-API packaging.

## Deliverables

- Typical-document corpus manifest.
- Coverage report comparing Rust backend and PDFium backend outcomes.
- Follow-up milestones for the highest-impact gaps.

## Acceptance Criteria

- The project can say which common document categories are recognizable,
  unsupported, or failing.
- Rust renderer gaps are ranked by product impact and implementation risk.
- Follow-up milestones are small enough to validate independently.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run the local corpus comparison command.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
