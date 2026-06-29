# 0217: Low-End Device Reliability Sweep

Status: done
Phase: 41
Size: medium
Depends on: 0216

## Goal

Validate low-end Rust-native rendering reliability as a secondary profile sweep
using realistic typical-document workflows. Constrained server behavior remains
important; mobile browser and embedded findings are compatibility signals unless
they reveal shared renderer defects.

## Scope

- Define low-memory, low-thread, high-latency I/O, and reduced canvas-size test
  profiles.
- Run typical-document workflows for thumbnails, first page, page navigation,
  search highlighting, and batch rendering.
- Measure peak memory, scratch allocation reuse, cache eviction, timeout, and
  recovery behavior.
- Document profile-specific unsupported or degraded modes.
- Promote shared renderer correctness, safety, and unbounded resource issues to
  the main server-side backlog.

## Non-Goals

- Optimize for devices below documented minimum requirements.
- Treat low-end profiles as a reason to reduce desktop fidelity.
- Hide profile failures behind PDFium fallback.
- Block server-side release gates solely on mobile or embedded profile limits.

## Deliverables

- Low-end reliability profile matrix.
- Memory, timeout, and degradation report.
- Profile-specific budget updates.

## Acceptance Criteria

- Supported low-end profiles complete typical workflows without panics.
- Memory and timeout budgets are enforced and documented.
- Degraded behavior is typed and visible to consumers.
- Server-constrained failures are classified separately from browser or embedded
  profile limitations.

## Validation

- Run native-only `cargo test`.
- Run low-memory renderer profile.
- Run WASM low-memory browser gate.
- Run server constrained batch gate.
- Run deterministic render checks for constrained profiles.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added `fixtures/low-end-reliability-profile-matrix.tsv` with low-memory,
  server-constrained batch, WASM smoke, and deterministic reduced-canvas
  profiles.
- Added `scripts/check_low_end_reliability_matrix.sh` to validate profile
  coverage, blocking scopes, target-local artifacts, and budget notes.
- Updated renderer memory budget and server batch policies with the 0217
  constrained profile limits.
- Report: `docs/reports/low-end-device-reliability-sweep-2026-06-29.md`.

Validation:

- `bash scripts/check_low_end_reliability_matrix.sh`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --fail-on-fallback --max-edge 120 --native-profile low-memory --output target/low-end-0217-low-memory-summary.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --native-profile low-memory --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/low-end-0217-low-memory-repeat.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 2 --pages-per-input 12 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/low-end-0217-server-constrained-batch.json`
- `cargo run -p ferrugo-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-a.png`
- `cargo run -p ferrugo-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-b.png`
- `cmp -s target/low-end-0217-deterministic-a.png target/low-end-0217-deterministic-b.png`
- `bash scripts/check_wasm_smoke.sh`
- `cargo fmt --check`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check -- fixtures/low-end-reliability-profile-matrix.tsv scripts/check_low_end_reliability_matrix.sh docs/policies/renderer-memory-budgets.md docs/policies/server-batch-rendering.md docs/milestones/README.md docs/milestones/0217-low-end-device-reliability-sweep.md docs/reports/low-end-device-reliability-sweep-2026-06-29.md`
