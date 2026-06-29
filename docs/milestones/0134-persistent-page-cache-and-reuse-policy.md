# 0134: Persistent Page Cache And Reuse Policy

Status: done
Phase: 24
Size: medium
Depends on: 0133

## Goal

Define whether native rendering should support persistent or reusable page
artifacts across repeated thumbnail requests.

## Scope

- Identify cacheable artifacts such as parsed objects, decoded fonts, glyphs,
  images, and display lists.
- Separate in-memory reuse from any on-disk persistence.
- Define cache keys that include document identity, page, options, and version.
- Measure memory savings and invalidation risks.

## Non-Goals

- Add a global unbounded cache.
- Persist private document content by default.
- Require consumers to use caching.

## Deliverables

- Page cache policy.
- Prototype or decision record for reusable artifacts.
- Benchmark report for repeated render requests.

## Acceptance Criteria

- Cache keys prevent cross-document contamination.
- Reuse improves repeated renders or is rejected with evidence.
- Memory and privacy tradeoffs are explicitly documented.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run repeated-render benchmarks.
- Run cache isolation tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added the native `NativePageCachePolicy::IsolatedRender` default and
  `NativePageCacheKey` shape for caller-owned reusable page artifacts.
- Added `benchmark-repeat-native`, which reports cache policy, versioned
  cache keys, first-render timings, repeated-render timings, and budget
  failures.
- Added `fixtures/page-cache-policy-manifest.tsv` and benchmark artifact
  `target/page-cache-0134-repeat-benchmark.json`.
- Documented the decision to reject a default persistent/global page cache for
  now because repeated-render timings stayed close to first-render timings
  while persistence would add memory, invalidation, and privacy risk.
- Report: `docs/reports/page-cache-reuse-policy-2026-06-25.md`.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p ferrugo-native native_page_cache -- --nocapture`
  - `cargo test -p ferrugo-cli repeat_benchmark -- --nocapture`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/page-cache-policy-manifest.tsv --include-family small --include-family business --include-family repeated-resources --include-family vector-stress --repetitions 3 --max-edge 160 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/page-cache-0134-repeat-benchmark.json`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `cargo test --workspace --no-default-features`
