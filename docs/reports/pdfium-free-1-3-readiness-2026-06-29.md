# PDFium-Free 1.3 Readiness 2026-06-29

Milestone 0205 makes the 1.3 decision from the current native-only
server/runtime evidence and the expanded phase 38 typical-document gates.

## Decision

Stabilize the scoped PDFium-free server/runtime path for 1.3, but defer a broad
PDFium-replacement claim.

The runtime path remains healthy: PDFium is absent from the supported native
package path, native-only release checks pass, server batch rendering stays
inside budget, serverless startup and first-render budgets pass, fuzz smoke
passes, and the WASM smoke remains green as a compatibility signal.

The broad replacement claim is still blocked. The weighted 1.3 score is
`94.04`, which meets the aggregate threshold, but `presentation` scores `86.09`
against the per-family minimum of `88.00`. The primary corpus also still has 12
typed unsupported rows plus one encrypted policy error. Visual review adds no
native errors, but still shows chart, dense-grid, and layout-stress parity
blockers.

## 1.3 Scorecard

Command:

```sh
scripts/generate_coverage_scorecard.sh target/readiness-0205-scorecard
```

Artifacts:

- `target/readiness-0205-scorecard/scorecard.json`
- `target/readiness-0205-scorecard/scorecard.md`
- `target/readiness-0205-scorecard/dashboard/dashboard.json`

Summary:

| Weighted score | Native rendered | Typed unsupported | Errors | Server batch budget failures |
| ---: | ---: | ---: | --- | ---: |
| 94.04 | 190 | 12 | 1 encrypted | 0 |

Family scorecard:

| Family | Score | Native pass | Unsupported | Errors | Partial operators |
| --- | ---: | ---: | ---: | ---: | ---: |
| `form` | 99.89 | 24 | 0 | 0 | 9 |
| `office-export` | 98.67 | 62 | 1 | 0 | 29 |
| `report` | 93.49 | 46 | 4 | 0 | 44 |
| `scan` | 91.11 | 24 | 3 | 0 | 0 |
| `mixed-layout` | 90.87 | 24 | 2 | 1 encrypted | 9 |
| `presentation` | 86.09 | 10 | 2 | 0 | 15 |

The encrypted mixed-layout row is a policy outcome, not a native renderer crash
or PDFium fallback.

Unsupported categories:

| Category | Count | Routed milestone |
| --- | ---: | --- |
| `image.filter` | 3 | 0209 Rust-Native Image Codec Deployment Policy |
| `graphics.optional-content` | 2 | 0211 PDF Operator Semantic Snapshot Suite |
| `graphics.transparency` | 2 | 0213 Transparency Stack Memory Optimization |
| `graphics.color-management` | 1 | 0208 Color Managed Print Preview Extended Gate |
| `graphics.pattern-shading` | 1 | 0204 Office Chart SmartArt And Vector Effect Fidelity |
| `annotation.appearance` | 1 | 0207 Annotation Popup Stamp And FreeText Fidelity |
| `form.xfa-dynamic` | 1 | 0206 Form Filling Appearance Update And Flattening Coverage |
| `text.font-program` | 1 | 0202 Text Selection Geometry And Search Highlight Parity |

## Runtime, Packaging, And Security

Native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

Result: passed.

This covered native-only check/test, plugin-free distribution, PDFium
quarantine, CLI package file inspection, leaf package dry-runs, and all-features
Clippy. Registry-backed workspace package verification remained optional and was
skipped because `PDFRUST_NATIVE_RELEASE_VERIFY_REGISTRY` was not set.

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

## Performance And Server Profiles

Scorecard server batch artifact:

- `target/readiness-0205-scorecard/dashboard/batch.json`

| Jobs | Native rendered | Fallbacks | Errors | Budget failures | Throughput/sec |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 0 | 0 | 0 | 44.297 |

Serverless artifact:

- `target/serverless-profile-0205.json`

| Binary bytes | Startup p95 ms | First-render p95 ms | Budget failures |
| ---: | ---: | ---: | ---: |
| 1,017,344 | 310.456 | 6.665 | 0 |

WASM smoke:

```sh
bash scripts/check_wasm_smoke.sh
```

Result: passed. WASM remains a compatibility signal for this gate; it does not
override the server-side PDFium replacement decision.

## Visual Validation

The PDFium-free oracle strategy remains unchanged: runtime readiness is
native-only, while visual comparisons are maintainer evidence from an
independent renderer. 0205 uses Poppler review artifacts and does not add a
PDFium runtime dependency.

Artifacts:

- `target/readiness-0205-office-chart-poppler.json`
- `target/readiness-0205-spreadsheet-poppler.json`
- `target/readiness-0205-layout-poppler.json`

Summary:

| Gate | Total | Accepted drift | Blockers | Native errors | Reference errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| Office chart/vector effects | 10 | 4 | 2 | 0 | 4 |
| Spreadsheet grid | 7 | 1 | 3 | 0 | 3 |
| Layout stress | 7 | 0 | 5 | 0 | 2 |

Native visual blockers:

| Fixture | Gate | Subsystem | Main metrics |
| --- | --- | --- | --- |
| `slide-rotated-callout.pdf` | Office chart/vector | `page-geometry` | MAE 2.730, p95 8, changed 0.839 |
| `slide-title-gradient.pdf` | Office chart/vector | `vector-graphics` | MAE 7.213, p95 55, changed 0.503 |
| `spreadsheet-dense-numeric-grid.pdf` | Spreadsheet grid | `rendering-core` | MAE 23.690, p95 113, changed 0.309 |
| `spreadsheet-frozen-header.pdf` | Spreadsheet grid | `rendering-core` | MAE 26.454, p95 221, changed 0.182 |
| `spreadsheet-vector-stress-grid.pdf` | Spreadsheet grid | `vector-graphics` | MAE 47.737, p95 186, changed 0.447 |
| `layout-columns-footnotes-table-stress.pdf` | Layout stress | `rendering-core` | MAE 44.093, p95 214, changed 0.395 |
| `office-report-header-footer-link.pdf` | Layout stress | `rendering-core` | MAE 9.315, p95 32, changed 0.234 |
| `reference-footnote-layout.pdf` | Layout stress | `rendering-core` | MAE 12.056, p95 108, changed 0.135 |
| `scientific-two-column-paper.pdf` | Layout stress | `rendering-core` | MAE 12.576, p95 104, changed 0.137 |
| `spreadsheet-dense-numeric-grid.pdf` | Layout stress | `rendering-core` | MAE 23.690, p95 113, changed 0.309 |

Reference errors are Poppler timeouts or reference-tool failures in the local
review run. They are not native renderer errors, but they remain visible so
future review tooling can separate native regressions from oracle limits.

## Post-1.3 Backlog

1. `image.filter`: implement or explicitly deploy-policy CCITT/JBIG2/JPX before
   claiming broad scan/fax/archive support.
2. `graphics.optional-content`: finish OCMD and usage-application semantics so
   presentation score clears the per-family threshold.
3. `graphics.transparency`: reduce soft-mask and transparency-stack boundaries
   for report/dashboard documents.
4. Dense grid and layout fidelity: reduce repeated thin-stroke, small-text, and
   multi-column placement drift from the 0203 visual gates.
5. `graphics.color-management`: keep black point compensation typed until the
   0208 print-preview policy lands.
6. `annotation.appearance` and `form.xfa-dynamic`: finish the 0206/0207 form and
   annotation gates without hiding unsupported behavior.
7. `text.font-program`: decide whether the residual office-export font-program
   row belongs in a follow-up to 0202 or in the 0212 font-cache work.

## Validation Commands

```text
scripts/generate_coverage_scorecard.sh target/readiness-0205-scorecard
bash scripts/check_native_only_release.sh
bash scripts/check_fuzz_smoke.sh
env PDFRUST_SERVERLESS_OUTPUT=target/serverless-profile-0205.json PDFRUST_SERVERLESS_PACKAGE_LIST=target/serverless-profile-0205-pdfrust-cli-package-files.txt scripts/measure_serverless_profile.sh
bash scripts/check_wasm_smoke.sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-office-chart-poppler.json
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-spreadsheet-poppler.json
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/readiness-0205-layout-poppler.json
```
