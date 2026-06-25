# 0131: Low-Memory Embedded Renderer Profile

Status: todo
Phase: 24
Size: medium
Depends on: 0130

## Goal

Define a low-memory native rendering profile suitable for embedded consumers,
serverless jobs, and constrained batch processing.

## Scope

- Add configurable caps for decoded images, glyph caches, display lists, and
  intermediate surfaces.
- Measure failure behavior under intentionally tight budgets.
- Keep budget errors typed and actionable.
- Document recommended low-memory settings for thumbnail use cases.

## Non-Goals

- Guarantee all documents render under tiny limits.
- Replace full-fidelity desktop rendering defaults.
- Add allocator-specific tuning in this slice.

## Deliverables

- Low-memory renderer profile.
- Budget stress fixtures and tests.
- Memory report with expected failure categories.

## Acceptance Criteria

- Tight budgets produce deterministic typed errors instead of OOM risk.
- Common thumbnails still render under documented low-memory settings.
- Cache and buffer limits are observable in diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run low-memory budget corpus comparisons.
- Run memory stress benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
