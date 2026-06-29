# 0220: PDFium-Free 1.4 Readiness Gate

Status: done
Phase: 41
Size: medium
Depends on: 0219

## Goal

Make the PDFium-free 1.4 release decision using cross-producer typical-document
coverage, server scheduler tuning, constrained server evidence, and a clear
unsupported-feature SLA. WASM and mobile low-memory results inform compatibility
backlog decisions but are not primary release blockers by themselves.

## Scope

- Run the complete native-only 1.4 validation matrix across supported document
  families and primary server deployment profiles.
- Compare 1.4 coverage, fidelity, memory, throughput, unsupported categories,
  and consumer-facing diagnostics against the 1.3 baseline.
- Verify PDFium is absent from supported runtime, package, CI, and deployment
  paths.
- Decide release, stabilize, or defer based on measured evidence.

## Non-Goals

- Claim complete PDF specification support.
- Hide unsupported behavior behind non-public diagnostics.
- Retain PDFium comparison tooling without a fresh explicit decision.

## Deliverables

- PDFium-free 1.4 readiness report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.4 backlog.

## Acceptance Criteria

- Cross-producer typical-document families pass native-only release gates.
- Supported desktop and server profiles meet documented release budgets.
- WASM, mobile, embedded, and low-memory profile failures are classified as
  compatibility backlog unless they expose shared renderer correctness, safety,
  or unbounded resource defects.
- Consumer-facing unsupported behavior is stable and documented.
- No supported path requires PDFium.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full 1.4 supported corpus gate.
- Run independent visual oracle validation.
- Run benchmark, memory, server, and package profile checks.
- Run low-end and WASM profile checks as secondary compatibility signals.
- Run security and fuzz smoke suite.
- Run repository scan for unsupported PDFium runtime references.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Produced `docs/reports/pdfium-free-1-4-readiness-2026-06-29.md`.
- Decision: stabilize the scoped PDFium-free server/runtime path for 1.4, but
  defer a broad PDFium-replacement claim.
- Native-only release, security fuzz smoke, server batch, serverless,
  scheduler, low-end, WASM, and PDFium-quarantine gates passed.
- The 1.4 scorecard reached `94.15`, but `presentation` remains below the
  per-family threshold at `86.09`.
- Independent Poppler cross-producer visual validation found 7 visual blockers
  with 0 native errors and 0 reference errors.
- Post-1.4 backlog is ranked in the readiness report.

Validation:

- `scripts/generate_coverage_scorecard.sh target/readiness-0220-scorecard`
- `bash scripts/check_native_only_release.sh`
- `bash scripts/check_fuzz_smoke.sh`
- `env PDFRUST_SERVERLESS_OUTPUT=target/serverless-profile-0220.json PDFRUST_SERVERLESS_PACKAGE_LIST=target/serverless-profile-0220-pdfrust-cli-package-files.txt scripts/measure_serverless_profile.sh`
- `bash scripts/check_scheduler_tuning_matrix.sh`
- `bash scripts/check_low_end_reliability_matrix.sh`
- `bash scripts/check_wasm_smoke.sh`
- `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --max-edge 120 --max-mae 12 --max-p95 96 --max-changed-ratio 0.30 --timeout 30 --output target/readiness-0220-cross-producer-poppler.json`
- `cargo fmt --check`
- `git diff --check -- docs/milestones/0220-pdfium-free-1-4-readiness-gate.md docs/milestones/README.md docs/reports/pdfium-free-1-4-readiness-2026-06-29.md`
