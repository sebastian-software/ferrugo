# 0121: Invoice Statement And Business Form Corpus Gate

Status: todo
Phase: 22
Size: medium
Depends on: 0120

## Goal

Prove native rendering coverage for common invoices, account statements,
receipts, and business forms that represent everyday document traffic.

## Scope

- Expand generated and reviewed corpus categories for invoices and statements.
- Cover barcodes, logos, dense totals tables, stamps, and signature blocks.
- Track visual-diff outcomes by business-document subtype.
- Reduce recurring private-corpus gaps into shareable synthetic fixtures.

## Non-Goals

- Extract accounting data.
- Validate form semantics.
- Guarantee parity for every vendor-specific template.

## Deliverables

- Business-document corpus gate.
- Synthetic fixtures for recurring invoice and statement features.
- Coverage report with blocker categories and follow-up backlog.

## Acceptance Criteria

- Typical invoices and statements render natively or fail with typed reasons.
- Business-document blockers are grouped by renderer feature.
- Private examples are represented by sanitized fixture reductions where useful.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run business-document corpus comparisons.
- Run native benchmark for business-document fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
