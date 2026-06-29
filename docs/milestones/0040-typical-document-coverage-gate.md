# 0040: Typical Document Coverage Gate

Status: done
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

- Added `docs/reports/typical-document-coverage-2026-06-24.md` with the
  committed generated seed corpus, local-only corpus category manifest, native
  renderer outcomes, and ranked renderer gaps.
- Updated `.gitignore` to keep `fixtures/local-corpus/` out of Git.
- Updated `docs/fixtures.md` to list the newer Image XObject and Form XObject
  generated fixtures.
- Seed coverage summary:
  - recognizable: `page-size-letter.pdf`, `vector-paths.pdf`, `text-page.pdf`
    via ASCII fallback, and `image-xobject.pdf`.
  - unsupported: `form-xobject.pdf` in the combined native render path.
  - failing gap: `inline-image.pdf` currently renders as blank white
    (`nonwhite=0`) because inline image stream execution is not implemented.
- Ranked next gaps: inline image streams, Form XObject integration in native
  rendering, real font rendering, broader image filters/color spaces, and
  advanced stroke/clipping/transparency/patterns.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/page-size-letter.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-page-size-letter-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/vector-paths.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-vector-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/text-page.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-text-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/image-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-image-xobject-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/form-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-form-native.png` returned `render error [unsupported]`.
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/inline-image.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-inline-image-native.png` produced a blank native image with `nonwhite=0`.
  - `cargo clippy --all-targets --all-features -- -D warnings`
