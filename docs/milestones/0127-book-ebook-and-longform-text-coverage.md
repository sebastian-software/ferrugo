# 0127: Book Ebook And Longform Text Coverage

Status: done
Phase: 23
Size: medium
Depends on: 0126

## Goal

Cover book-like PDFs and longform text documents with many pages, repeated
fonts, page labels, front matter, and scanned or mixed illustrations.

## Scope

- Add fixtures for books, manuals, ebooks, and longform text samples.
- Exercise page labels, outlines, repeated font resources, and page cache reuse.
- Verify first page, chapter page, and interior page sampling.
- Track memory growth across long documents.

## Non-Goals

- Implement ebook format support.
- Add text reflow.
- Build a reader UI.

## Deliverables

- Longform text corpus gate.
- Multi-page sampling and cache report.
- Memory profile for repeated font and image resources.

## Acceptance Criteria

- Representative longform pages render natively without unbounded state growth.
- Page labels and outlines remain available for metadata consumers.
- Repeated resources are reused through bounded caches.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run longform corpus comparisons.
- Run long-document memory benchmark.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated fixtures for book front matter with page labels and
  outlines, illustrated manual pages, narrow ebook pages, and longform repeated
  font/image resources.
- Added `fixtures/longform-text-manifest.tsv` with eight focused rows across
  `book`, `manual`, `ebook`, and `repeated-resources` families.
- Added native regression coverage for longform rendering, book metadata,
  frontmatter/chapter/interior page sampling, and bounded cache diagnostics.
- Native fallback gate: 8/8 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 8/8 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle: 1 exact match, 7 strict-threshold blockers, 0 native
  render errors, 0 PDFium render errors.
- Report: `docs/reports/longform-text-fidelity-2026-06-25.md`.
- Follow-up on 2026-06-28 added deterministic Times-Roman advance widths for
  standard-base-font fallback text. The book fixture remains a strict Poppler
  blocker, but its measured drift improved slightly without introducing native
  or reference render errors.
