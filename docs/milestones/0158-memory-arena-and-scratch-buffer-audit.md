# 0158: Memory Arena And Scratch Buffer Audit

Status: todo
Phase: 29
Size: medium
Depends on: 0157

## Goal

Audit renderer allocation patterns and introduce or validate reusable scratch
buffers where they materially reduce churn without unsafe complexity.

## Scope

- Profile allocation hot paths for parsing, decoding, display-list building,
  rasterization, text layout, and image conversion.
- Identify buffers that can be reused safely across pages or render passes.
- Add hard capacity limits and reset behavior for reusable scratch state.
- Document tradeoffs for memory retention versus allocation savings.

## Non-Goals

- Add custom allocators without measured need.
- Use unsafe arenas for convenience.
- Optimize cold paths that do not affect typical documents.

## Deliverables

- Allocation profile report.
- Scratch-buffer improvement patches or documented no-op decisions.
- Memory budget updates.

## Acceptance Criteria

- Hot allocation sources are measured and categorized.
- Reusable buffers have explicit bounds and reset semantics.
- Memory improvements do not change rendering output.

## Validation

- Run allocation-sensitive benchmarks.
- Run renderer visual comparison subset.
- Run native-only `cargo test`.
- Run memory profile before and after changes.

## Completion Notes

Empty until done.
