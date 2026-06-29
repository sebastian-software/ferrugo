# 0200: PDFium-Free 1.2 Readiness Gate

Status: done
Phase: 37
Size: medium
Depends on: 0199

## Goal

Make the PDFium-free 1.2 release decision using expanded document-family
coverage, native-only packaging, performance budgets, and explicit unsupported
boundaries.

## Scope

- Run the complete native-only validation matrix for the 1.2 corpus.
- Compare coverage, fidelity, memory, startup, and throughput against 1.1.
- Verify runtime PDFium remains absent from supported packages.
- Decide release, stabilize, or defer based on measured evidence.

## Non-Goals

- Claim complete PDF specification support.
- Reopen PDFium runtime fallback to pass the gate.
- Ship without documented unsupported boundaries.

## Deliverables

- PDFium-free 1.2 readiness report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.2 backlog.

## Acceptance Criteria

- Typical-document families pass native-only release gates.
- Packaging and deployment profiles remain PDFium-free.
- Performance, memory, and fidelity budgets are documented and acceptable.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run visual validation using the PDFium-free oracle strategy.
- Run benchmark, memory, server, and package profile checks.
- Run WASM and low-memory profile checks as non-blocking compatibility signals
  unless they expose shared renderer correctness, safety, or unbounded resource
  defects.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Produced `docs/reports/pdfium-free-1-2-readiness-2026-06-29.md`.
- Decision: stabilize the scoped PDFium-free server/runtime path, but defer a
  broad PDFium replacement claim for 1.2.
- Fresh 1.2 dashboard: primary families total 203, native rendered 190,
  fallback required 12, encrypted policy errors 1.
- Native-only release gate passed after tightening the plugin-free dependency
  scan to avoid matching `hyperlink` as the `hyper` crate.
- Fuzz smoke passed after updating `render_setup` for the current
  `ThumbnailOptions` fields.
- Serverless profile passed with a 1,017,344-byte stripped binary and 0 budget
  failures.
- Validation:
  - `bash scripts/generate_corpus_dashboard.sh target/readiness-0200-dashboard`
  - `bash scripts/check_native_only_release.sh`
  - `bash scripts/check_fuzz_smoke.sh`
  - `scripts/measure_serverless_profile.sh`
  - `cargo fmt --check`
  - `git diff --check -- scripts/check_plugin_free_distribution.sh fuzz/fuzz_targets/render_setup.rs docs/milestones/0200-pdfium-free-1-2-readiness-gate.md docs/milestones/README.md docs/reports/pdfium-free-1-2-readiness-2026-06-29.md`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
