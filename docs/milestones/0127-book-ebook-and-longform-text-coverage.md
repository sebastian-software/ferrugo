# 0127: Book Ebook And Longform Text Coverage

Status: todo
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

Empty until done.
