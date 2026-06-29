# 0214: Incremental Parser And Object Cache Fusion

Status: done
Phase: 40
Size: medium
Depends on: 0213

## Goal

Fuse incremental parsing with the native object and resource cache so large,
linearized, and partially accessed PDFs avoid unnecessary memory and I/O work.

## Scope

- Profile parser, xref, object stream, page tree, resource, and render-cache
  lifetimes across long documents.
- Add bounded object cache reuse between first-page render, navigation, search,
  and thumbnail workflows.
- Validate incremental updates, hybrid references, linearized files, and corrupt
  but common recovery cases.
- Document cache invalidation and recovery rules.

## Non-Goals

- Build a full random-access PDF editing model.
- Keep every indirect object resident for speed.
- Reuse invalid cached objects after recovery paths detect corruption.

## Deliverables

- Incremental parser and object cache design note.
- Cache fusion patch set.
- Long-document memory and navigation report.

## Acceptance Criteria

- Fast-first-page and page navigation avoid avoidable reparsing.
- Large and linearized documents meet memory budgets.
- Incremental update and recovery behavior remains deterministic.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run linearized and long-document navigation benchmarks.
- Run incremental update and corrupt-PDF recovery corpus checks.
- Run cache invalidation snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added a document-local compressed-object lookup index for xref-stream
  documents so `ModernDocument::get_object` avoids repeated scans across all
  loaded object streams.
- Kept compressed object values parsed on demand from validated decoded object
  stream bytes instead of retaining every indirect object value.
- Documented cache invalidation and recovery rules in
  `docs/policies/incremental-and-hybrid-references.md`.
- Added `fixtures/object-cache-fusion-manifest.tsv` covering linearized,
  malformed linearization fallback, incremental update, deleted incremental
  object, hybrid reference, recoverable xref drift, and long-document navigation
  fixtures.
