# 0187: Incremental Document Streaming Memory Budget

Status: done
Phase: 35
Size: medium
Depends on: 0186

## Goal

Keep large and incrementally loaded documents usable by bounding memory growth
while parsing, loading page resources, and rendering previews.

## Scope

- Measure memory behavior for long documents, linearized files, and large
  resource dictionaries.
- Add streaming or lazy-loading boundaries where full-document retention is not
  required.
- Define eviction points for page-local parsed resources.
- Document unsupported cases that require full-file availability.

## Non-Goals

- Implement random access over every remote transport.
- Rewrite all object storage in one milestone.
- Trade correctness for lower peak memory.

## Deliverables

- Incremental memory budget report.
- Lazy-loading improvements or follow-up backlog.
- Large-document regression fixtures.

## Acceptance Criteria

- Peak memory is measured for representative long documents.
- Page preview rendering avoids unnecessary full-resource retention.
- Streaming limitations are explicit in public docs.

## Validation

- Run native-only `cargo test`.
- Run long-document benchmark and memory profiles.
- Run linearized and incremental loading tests.
- Review memory budget documentation.

## Completion Notes

- Added `FirstPagePreviewMemory` to the native first-page preview API so
  callers can observe input bytes, parsed object count, parsed object bytes,
  linearized first-page section size, and whether the preview used a
  first-page-only loader.
- Refactored native rendering so `render_first_page_preview` renders the same
  loaded document instance used for load-mode/memory reporting instead of
  performing a second parser pass.
- Added focused tests proving `linearized-first-page.pdf` retains fewer parsed
  objects and object bytes than the full classic loader, while malformed
  linearization stays a safe full-document fallback.
- Added `fixtures/incremental-memory-budget-manifest.tsv` and
  `docs/reports/incremental-streaming-memory-budget-2026-06-29.md` for
  long-document, linearized, page-targeted, repeated-resource, and large-resource
  budget coverage.
- Streaming limitations remain explicit: local file/byte inputs are still
  required, remote range fetching is not implemented, and malformed or
  unsupported linearization falls back to full-file parsing for correctness.
