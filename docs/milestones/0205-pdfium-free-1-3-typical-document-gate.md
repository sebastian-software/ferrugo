# 0205: PDFium-Free 1.3 Typical Document Gate

Status: done
Phase: 38
Size: medium
Depends on: 0204

## Goal

Make the PDFium-free 1.3 decision using the expanded typical-document corpus,
native renderer scorecard, and memory/performance evidence from phase 38.

## Scope

- Run the 1.3 native-only validation matrix across supported typical-document
  families.
- Compare 1.3 coverage, fidelity, unsupported categories, memory, and throughput
  against the 1.2 readiness baseline.
- Decide release, stabilize, or defer based on measured evidence.
- Produce the next implementation backlog without relying on runtime PDFium.

## Non-Goals

- Claim complete PDF specification support.
- Hide unsupported categories behind aggregate pass rates.
- Reintroduce PDFium runtime fallback to pass the gate.

## Deliverables

- PDFium-free 1.3 typical-document report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.3 backlog.

## Acceptance Criteria

- Typical document families pass native-only gates at documented thresholds.
- Performance and memory budgets are met for supported profiles.
- Unsupported boundaries remain typed, visible, and documented for consumers.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.3 supported corpus gate.
- Run visual validation with the PDFium-free oracle strategy.
- Run benchmark, memory, server, and package profile checks.
- Run WASM and low-memory profile checks as non-blocking compatibility signals
  unless they expose shared renderer correctness, safety, or unbounded resource
  defects.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Produced `docs/reports/pdfium-free-1-3-readiness-2026-06-29.md`.
- Decision: stabilize the scoped PDFium-free server/runtime path for 1.3, but
  defer a broad PDFium-replacement claim.
- Fresh 1.3 scorecard artifact: `target/readiness-0205-scorecard/scorecard.json`.
  Weighted score is `94.04`, but `presentation` remains below the `88.00`
  per-family threshold at `86.09`.
- Primary corpus support remains 203 total, 190 native rendered, 12 typed
  unsupported, and 1 encrypted policy error.
- Native-only release, fuzz smoke, serverless profile, server batch, all-features
  Clippy, and WASM smoke gates passed.
- Poppler visual review produced no native errors, but still shows blockers in
  office chart/vector effects, dense spreadsheet grids, and layout-stress
  fixtures.

Validation run:

- `scripts/generate_coverage_scorecard.sh target/readiness-0205-scorecard`
- `bash scripts/check_native_only_release.sh`
- `bash scripts/check_fuzz_smoke.sh`
- `env FERRUGO_SERVERLESS_OUTPUT=target/serverless-profile-0205.json FERRUGO_SERVERLESS_PACKAGE_LIST=target/serverless-profile-0205-ferrugo-cli-package-files.txt scripts/measure_serverless_profile.sh`
- `bash scripts/check_wasm_smoke.sh`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-office-chart-poppler.json`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-spreadsheet-poppler.json`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-layout-poppler.json`
