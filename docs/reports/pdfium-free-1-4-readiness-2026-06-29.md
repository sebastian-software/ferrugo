# PDFium-Free 1.4 Readiness 2026-06-29

Milestone 0220 makes the 1.4 decision from the current native-only runtime,
server, cross-producer, low-memory, WASM, security, and consumer diagnostic
evidence.

## Decision

Stabilize the scoped PDFium-free server/runtime path for 1.4, but defer a broad
PDFium-replacement claim.

The supported runtime path is healthy: native-only check/test passed, PDFium is
absent from supported package/runtime paths, server batch scheduling is bounded,
serverless and WASM profile gates pass, low-end and scheduler matrices pass,
fuzz smoke passes, and unsupported behavior now has a consumer-facing SLA.

The broad replacement claim is still not ready. The weighted runtime score is
`94.15`, but `presentation` remains below the per-family threshold at `86.09`.
The primary corpus still has 12 typed unsupported rows plus 1 encrypted policy
error. The independent Poppler cross-producer fusion slice has 7 visual
blockers, mostly in scan/rendering-core, page geometry, and forms.

## 1.4 Scorecard

Command:

```sh
scripts/generate_coverage_scorecard.sh target/readiness-0220-scorecard
```

Artifacts:

- `target/readiness-0220-scorecard/scorecard.json`
- `target/readiness-0220-scorecard/scorecard.md`
- `target/readiness-0220-scorecard/dashboard/dashboard.json`

Summary:

| Weighted score | Native rendered | Typed unsupported | Errors | Server batch budget failures |
| ---: | ---: | ---: | --- | ---: |
| 94.15 | 196 | 12 | 1 encrypted | 0 |

Family scorecard:

| Family | Score | Native pass | Unsupported | Errors | Partial operators |
| --- | ---: | ---: | ---: | ---: | ---: |
| `form` | 99.89 | 27 | 0 | 0 | 9 |
| `office-export` | 98.67 | 62 | 1 | 0 | 29 |
| `report` | 93.49 | 46 | 4 | 0 | 44 |
| `scan` | 91.11 | 24 | 3 | 0 | 0 |
| `mixed-layout` | 91.69 | 27 | 2 | 1 encrypted | 14 |
| `presentation` | 86.09 | 10 | 2 | 0 | 15 |

Unsupported categories:

| Category | Count | Release impact |
| --- | ---: | --- |
| `image.filter` | 3 | Blocks broad scan/fax/archive claims. |
| `graphics.optional-content` | 2 | Keeps presentation below the per-family threshold. |
| `graphics.transparency` | 2 | Blocks broad report/dashboard visual-fidelity claims. |
| `annotation.appearance` | 1 | Documented mixed-layout boundary. |
| `form.xfa-dynamic` | 1 | Accepted dynamic-XFA deferral. |
| `graphics.color-management` | 1 | Print/color-critical deferral. |
| `graphics.pattern-shading` | 1 | Vector/chart deferral. |
| `text.font-program` | 1 | Office-export text/font follow-up. |

## Runtime, Packaging, And Security

Native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

Result: passed.

This covered native-only `cargo check`, native-only `cargo test`,
plugin-free distribution, PDFium quarantine, CLI package file inspection, leaf
package dry-runs, and all-features Clippy. Registry-backed workspace package
verification remained optional and was skipped because
`FERRUGO_NATIVE_RELEASE_VERIFY_REGISTRY` was not set.

Fuzz smoke:

```sh
bash scripts/check_fuzz_smoke.sh
```

Result: passed.

| Target | Cases | Result |
| --- | ---: | --- |
| `primitive_parse` | 165 | passed |
| `xref_load` | 154 | passed |
| `stream_decode` | 154 | passed |
| `content_tokenize` | 165 | passed |
| `render_setup` | 176 | passed |

PDFium runtime references remain quarantined for supported paths. The 0215
decision keeps PDFium comparison tooling as maintainer-only oracle tooling
behind `--features pdfium`; it is not part of runtime fallback.

## Server And Compatibility Profiles

Scorecard server batch artifact:

- `target/readiness-0220-scorecard/dashboard/batch.json`

| Jobs | Native rendered | Fallbacks | Errors | Budget failures | Throughput/sec | P95 ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 0 | 0 | 0 | 44.481 | 138.103 |

Serverless artifact:

- `target/serverless-profile-0220.json`

| Binary bytes | Startup p95 ms | First-render p95 ms | Budget failures |
| ---: | ---: | ---: | ---: |
| 1,017,504 | 304.950 | 5.666 | 0 |

Scheduler and low-end profile matrices:

| Gate | Result |
| --- | --- |
| `bash scripts/check_scheduler_tuning_matrix.sh` | passed |
| `bash scripts/check_low_end_reliability_matrix.sh` | passed |

WASM smoke:

| Artifact size bytes | Compile ms | Instantiate ms | Smoke ms | Result |
| ---: | ---: | ---: | ---: | --- |
| 730359 | 1.277 | 0.069 | 5.421 | passed |

WASM, mobile, embedded, and low-memory results remain compatibility signals for
this release. None of the passing secondary checks expose shared renderer
correctness, safety, or unbounded-resource defects that would override the
server-side decision.

## Independent Visual Oracle

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --max-edge 120 --max-mae 12 --max-p95 96 --max-changed-ratio 0.30 --timeout 30 --output target/readiness-0220-cross-producer-poppler.json
```

Result:

| Total | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: |
| 20 | 13 | 7 | 0 | 0 |

Blockers:

| Fixture | Family | Subsystem |
| --- | --- | --- |
| `business-form-stamp-signature.pdf` | `fused-form` | `annotations-forms` |
| `dashboard-heatmap-overlay.pdf` | `fused-dashboard-map` | `rendering-core` |
| `financial-annual-report-page.pdf` | `fused-report` | `page-geometry` |
| `mobile-mixed-compression-scan.pdf` | `fused-scan` | `rendering-core` |
| `office-spreadsheet-chart-comments.pdf` | `fused-table-statement` | `rendering-core` |
| `scanner-ocr-form-overlay.pdf` | `fused-scan` | `rendering-core` |
| `scanner-skewed-mailroom-page.pdf` | `fused-scan` | `page-geometry` |

These are visual-fidelity blockers for broad replacement claims, not runtime
PDFium dependencies or native render errors.

## Consumer Diagnostics

0219 completed the unsupported-feature SLA and migration guide:

- `docs/policies/unsupported-feature-sla.md`
- `docs/guides/native-only-consumer-migration.md`
- `docs/reports/unsupported-feature-sla-consumer-migration-2026-06-29.md`

Applications can distinguish successful, degraded, unsupported, and failed
outcomes through stable public APIs. Unsupported buckets are stable diagnostic
inputs for telemetry and backlog routing, not instructions to reintroduce
hidden runtime PDFium fallback.

## Post-1.4 Backlog

1. `image.filter`: implement or isolate CCITT/JBIG2/JPX strategy before broad
   scan/fax/archive claims; the visual oracle also shows scan rendering-core
   and page-geometry blockers.
2. `graphics.optional-content`: finish optional-content semantics so
   presentation clears the per-family readiness threshold.
3. Cross-producer visual fidelity: reduce the 7 Poppler blockers in scan,
   forms, dashboard/report geometry, and spreadsheet/table rendering.
4. `graphics.transparency`: continue reducing soft-mask/blend boundaries for
   report/dashboard documents.
5. `annotation.appearance`: close mixed-layout annotation appearance gaps that
   are now visible in both SLA and visual oracle evidence.
6. Dense spreadsheet/table and layout fidelity: keep reducing rendering-core
   drift from 0203 and the 0220 fusion slice.
7. `graphics.color-management` and `graphics.pattern-shading`: keep print and
   vector/chart deferrals visible until target corpus frequency justifies
   native implementation.
8. `text.font-program`: triage the residual office-export font-program row.
9. `form.xfa-dynamic`: keep dynamic XFA as an accepted unsupported boundary.

## Validation Commands

```text
scripts/generate_coverage_scorecard.sh target/readiness-0220-scorecard
bash scripts/check_native_only_release.sh
bash scripts/check_fuzz_smoke.sh
env FERRUGO_SERVERLESS_OUTPUT=target/serverless-profile-0220.json FERRUGO_SERVERLESS_PACKAGE_LIST=target/serverless-profile-0220-ferrugo-cli-package-files.txt scripts/measure_serverless_profile.sh
bash scripts/check_scheduler_tuning_matrix.sh
bash scripts/check_low_end_reliability_matrix.sh
bash scripts/check_wasm_smoke.sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --max-edge 120 --max-mae 12 --max-p95 96 --max-changed-ratio 0.30 --timeout 30 --output target/readiness-0220-cross-producer-poppler.json
cargo fmt --check
git diff --check -- docs/milestones/0220-pdfium-free-1-4-readiness-gate.md docs/milestones/README.md docs/reports/pdfium-free-1-4-readiness-2026-06-29.md
```
