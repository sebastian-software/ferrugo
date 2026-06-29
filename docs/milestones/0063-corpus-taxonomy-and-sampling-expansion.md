# 0063: Corpus Taxonomy And Sampling Expansion

Status: done
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

- Added `docs/corpus-taxonomy.md` with committed corpus families:
  `office-export`, `browser-print`, `scan`, `report`, `form`,
  `presentation`, and `mixed-layout`.
- Added `fixtures/corpus-manifest.tsv` with one row for each of the 39
  committed generated PDFs. Each row records path, family, reproducible source,
  license, page count, feature tags, and notes.
- Documented private local corpus sampling rules that avoid filenames,
  customer names, text extraction, hashes, screenshots, and rendered outputs.
- Extended `summarize-fallbacks` with `--manifest` so corpus runs report
  native pass rates per document family.
- Added `extract-corpus-metadata` to capture committed fixture metadata:
  manifest source/license/features plus native page count and page sizes.
- Validation: manifest TSV has 7 columns on every row, covers exactly the
  39 committed `fixtures/generated/*.pdf` files, and
  `cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --output target/fallback-summary-with-families.json`
  reported 37 native renders, 1 `graphics.optional-content` fallback, and 1
  encrypted input.
- Validation:
  `cargo run -p ferrugo-cli -- extract-corpus-metadata fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/corpus-metadata.json`
  produced 39 metadata records.
- Validation: `cargo fmt --check`, `cargo check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --quiet`.
