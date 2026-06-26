# 0171: Long Document Navigation And Page Cache Gate

Status: done
Phase: 32
Size: medium
Depends on: 0170

## Goal

Make long-document preview workflows efficient enough for typical viewer use
without retaining unbounded page state.

## Scope

- Add fixtures and synthetic documents with many pages, repeated resources, and
  mixed page complexity.
- Define page cache keys, eviction behavior, and memory accounting.
- Optimize first-page, next-page, and random-page render workflows.
- Add cancellation or early-exit behavior where render work can be abandoned.

## Non-Goals

- Build a full viewer UI.
- Keep every rendered page resident in memory.
- Add unsafe shared state for cache speed.

## Deliverables

- Long-document benchmark report.
- Page cache and eviction policy updates.
- Tests for repeated resource reuse and cache limits.

## Acceptance Criteria

- Long-document rendering has bounded peak memory.
- Repeated resources are reused without stale-page artifacts.
- First-page and random-page timings are measured and documented.

## Validation

- Run native-only `cargo test`.
- Run long-document benchmark suite.
- Run memory profile with cache pressure.
- Run visual comparison for sampled pages.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

Report:

- `docs/reports/long-document-navigation-page-cache-gate-2026-06-26.md`

Implemented:

- Added `fixtures/long-document-navigation-manifest.tsv` for long-document
  navigation, repeated resource, book, report, and statement samples.
- Added `long-document-navigation-deck.pdf`, a 12-page generated fixture with
  repeated font/image resources and first/next/random-page sampling coverage.
- Added native tests for bounded scheduler navigation sampling and
  caller-owned cache-key isolation across page/background variants.

Validation:

- `cargo test -p pdfrust-native long_document_navigation -- --nocapture`
- `cargo test -p pdfrust-native native_page_cache -- --nocapture`
- Long-document supported gate: 5 total, 5 native rendered, 0 fallbacks, 0
  errors.
- Long-document benchmark: 5 total, 5 native rendered, 0 fallbacks, 0 errors, 0
  budget failures.
- Repeat benchmark: 5 total, 5 native rendered, 0 fallbacks, 0 errors, 0 budget
  failures under `isolated-render`.
- Batch memory profile: 10 jobs, 10 native rendered, 0 fallbacks, 0 errors, 0
  budget failures with `max_in_flight_pixels = 51200`.
- Maintainer visual comparison: 5 total, 0 exact, 0 accepted drift, 5 fidelity
  blockers, 0 native errors, 0 PDFium errors.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
