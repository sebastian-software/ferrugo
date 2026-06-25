# 0155: Incremental Loading Interactive Preview

Status: todo
Phase: 28
Size: medium
Depends on: 0154

## Goal

Support fast first-page and partial-document preview behavior for large PDFs
without reintroducing PDFium or loading the entire document eagerly.

## Scope

- Audit parse and render paths for first-page latency.
- Reuse linearization, xref, and page scheduling work for preview APIs.
- Add cancellation and partial-result behavior for long renders.
- Measure memory and latency on large documents.

## Non-Goals

- Build a full viewer UI.
- Stream from arbitrary remote protocols in this milestone.
- Sacrifice deterministic output for lower latency.

## Deliverables

- Incremental preview API or internal boundary.
- First-page latency report.
- Cancellation and partial-render tests.

## Acceptance Criteria

- First-page preview does not require full-document rendering for supported
  inputs.
- Cancellation releases work promptly and safely.
- Memory use remains bounded for large documents.

## Validation

- Run first-page latency benchmark.
- Run cancellation tests.
- Run native-only supported corpus gate.
- Run large-document memory profile.

## Completion Notes

Empty until done.
