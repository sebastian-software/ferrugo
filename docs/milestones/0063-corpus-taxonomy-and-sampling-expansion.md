# 0063: Corpus Taxonomy And Sampling Expansion

Status: todo
Phase: 9
Size: medium
Depends on: 0062

## Goal

Expand the typical-document corpus so native renderer progress is measured
against realistic document families instead of isolated fixtures only.

## Scope

- Define corpus buckets for office exports, browser printouts, scans, reports,
  forms, presentations, and mixed-layout documents.
- Add metadata capture for page count, size, filters, fonts, annotations, and
  transparency features.
- Keep corpus fixtures license-safe and reproducible.
- Add sampling notes for private local corpora that cannot be committed.

## Non-Goals

- Commit proprietary or personally sensitive PDFs.
- Build a full PDF feature classifier.
- Replace focused unit fixtures.

## Deliverables

- Corpus taxonomy documentation.
- Fixture metadata extraction output.
- Expanded local comparison manifest.

## Acceptance Criteria

- Corpus runs can report pass rates per document family.
- Each committed fixture has source and license notes.
- Private corpus usage is documented without leaking document contents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run corpus metadata extraction.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
