# 0187: Incremental Document Streaming Memory Budget

Status: todo
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

Empty until done.
