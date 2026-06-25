# 0130: Legal Contract And Redaction Document Coverage

Status: todo
Phase: 23
Size: medium
Depends on: 0129

## Goal

Validate native rendering for contracts, legal filings, redacted PDFs, and
signature-heavy documents that mix text, annotations, stamps, and scans.

## Scope

- Add fixtures for contracts, filing-style pages, visible redactions, and
  signature blocks.
- Cover stamps, highlights, comments, typed fields, and scanned attachments.
- Track annotation and form fallback outcomes for legal-document subtypes.
- Verify that redaction visuals render as page content or appearances.

## Non-Goals

- Validate legal signatures.
- Determine whether redactions are semantically secure.
- Extract contract clauses or legal metadata.

## Deliverables

- Legal-document fixture family.
- Annotation/form coverage report for contracts and filings.
- Redaction visual-rendering policy notes.

## Acceptance Criteria

- Visible redactions, stamps, and signatures appear in native thumbnails.
- Missing appearance cases are typed, synthesized, or documented.
- Legal-document blockers are traceable to concrete renderer features.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run legal-document visual comparisons.
- Run annotation and form fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
