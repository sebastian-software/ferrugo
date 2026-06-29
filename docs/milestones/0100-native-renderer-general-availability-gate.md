# 0100: Native Renderer General Availability Gate

Status: done
Phase: 17
Size: medium
Depends on: 0099

## Goal

Decide whether the Rust renderer can be declared generally available for the
targeted typical-document surface without PDFium as a normal dependency.

## Scope

- Run native-only tests, corpus gates, benchmarks, fuzz smoke checks, and package
  validation.
- Compare supported-category output against the latest PDFium-enabled baseline.
- Review remaining unsupported categories, fallback policy, and rollback plan.
- Produce the GA decision and post-GA maintenance backlog.

## Non-Goals

- Claim full PDF specification coverage.
- Remove maintainer-only PDFium comparison infrastructure.
- Ship without documented unsupported and degraded categories.

## Deliverables

- Native renderer GA report.
- Go/no-go decision for PDFium-free normal operation.
- Post-GA maintenance and deletion backlog.

## Acceptance Criteria

- GA decision is based on measured fidelity, performance, memory, safety, and
  packaging evidence.
- Normal supported-document operation does not require PDFium.
- Remaining PDFium usage is maintainer-only, emergency-only, or explicitly
  scoped to unsupported categories.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run renderer benchmark suite.
- Run fuzz smoke targets.
- Run package validation.
- Run PDFium-enabled comparison baseline.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added `docs/reports/native-renderer-ga-gate-2026-06-25.md`.
- Decision: no broad visual GA yet. The native renderer is fallback-free for
  the supported-family technical gate (`browser-print`, `office-export`, and
  `form`), but PDFium visual comparison still reports supported-family fidelity
  blockers.
- Supported native-only gate: 30 total, 30 native rendered, 0 fallback, 0
  errors.
- Native benchmark: 75 total, 69 native rendered, 5 fallback required, 1 error,
  6 budget failures; supported families had 0 budget failures.
- PDFium visual diff baseline: 75 total, 26 exact, 13 accepted drift, 30
  blockers, 5 native errors, 0 PDFium errors, 1 both-error case.
- Fuzz smoke targets completed: `primitive_parse` 165, `xref_load` 154,
  `stream_decode` 154, `content_tokenize` 165, `render_setup` 165.
- Package dry-runs completed for `ferrugo-syntax` and `ferrugo-thumbnail`.
- Validation completed with native-only fmt/check/test, supported corpus gate,
  benchmark, PDFium visual baseline, PDFium feature test, and all-feature
  clippy.
