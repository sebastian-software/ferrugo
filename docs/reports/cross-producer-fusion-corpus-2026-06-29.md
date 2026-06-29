# Cross-Producer Typical Document Fusion Corpus

Date: 2026-06-29
Milestone: 0216

## Summary

Milestone 0216 adds a fused typical-document corpus that groups equivalent
workflows across producer styles instead of treating each producer family as a
separate checklist. The corpus uses existing generated reductions only; it does
not add private PDFs, external files, screenshots, rendered pixels, document
hashes, or real producer output.

New artifacts:

- `fixtures/cross-producer-fusion-manifest.tsv`
- `fixtures/cross-producer-fusion-matrix.tsv`
- `scripts/check_cross_producer_fusion_corpus.sh`

## Workflow Coverage

| Workflow | Family | Rows | Producer examples | Native status |
| --- | --- | ---: | --- | --- |
| Report export | `fused-report` | 4 | Writer, browser print, financial report, server report. | 4 native, 0 fallback, 0 errors. |
| Tabular statement | `fused-table-statement` | 4 | Spreadsheet, bank statement, accounting invoice, government notice. | 4 native, 0 fallback, 0 errors. |
| Form with marks | `fused-form` | 3 | Government form, business form, WebKit print form. | 3 native, 0 fallback, 0 errors. |
| Scan ingest | `fused-scan` | 4 | Mailroom scanner, mobile scan app, legal scan assembly, OCR overlay. | 4 native, 0 fallback, 0 errors. |
| Dashboard/map export | `fused-dashboard-map` | 5 | Presentation handout, browser dashboard, analytics dashboard, GIS exports. | 5 native, 0 fallback, 0 errors. |
| Unsupported boundary | `fused-unsupported-boundary` | 2 | Layered presentation, fax scanner. | 2 typed unsupported, 0 errors. |

## Matrix Decisions

The fusion matrix records:

- normalized workflow;
- synthetic producer label and version style;
- document family;
- feature pressure;
- server profile;
- expected native status;
- current status or typed cause;
- owner route;
- fixture minimization note.

Every workflow and every family has at least two producer entries. Supported
families are native-only runnable in CI with `--fail-on-fallback`. Unsupported
boundary rows stay visible and route to existing owner milestones:

| Fixture | Bucket | Owner route |
| --- | --- | --- |
| `fixtures/generated/optional-content-ocmd.pdf` | `graphics.optional-content` | 0192 optional-content-ui-state |
| `fixtures/generated/unsupported-ccitt-image.pdf` | `image.filter` | 0209 rust-native-image-codec |

## Privacy And Minimization

The corpus is limited to generated fixture PDFs created by
`scripts/generate_fixtures.py` under `MIT OR Apache-2.0`. The fusion check
rejects:

- missing fixture files;
- non-generated sources;
- unexpected licenses;
- rows without `producer:*`, `workflow:*`, or `expected:*` tags;
- `privacy:private` or `privacy:local-only` tags;
- workflows or families with fewer than two producers;
- matrix rows missing owner routes for non-supported status;
- matrix rows without generated minimization notes.

The producer report declares:

```text
no PDF bytes, rendered pixels, extracted text, private filenames, or document hashes
```

## Gate Evidence

Supported fusion gate:
`target/cross-producer-fusion-0216-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `fused-dashboard-map` | 5 | 5 | 0 | 0 |
| `fused-form` | 3 | 3 | 0 | 0 |
| `fused-report` | 4 | 4 | 0 | 0 |
| `fused-scan` | 4 | 4 | 0 | 0 |
| `fused-table-statement` | 4 | 4 | 0 | 0 |
| **Total** | **20** | **20** | **0** | **0** |

Unsupported boundary classification:
`target/cross-producer-fusion-0216-boundary-classification.json`

| Family | Total | Fallback required | Buckets | Errors |
| --- | ---: | ---: | --- | ---: |
| `fused-unsupported-boundary` | 2 | 2 | `graphics.optional-content`, `image.filter` | 0 |

Producer report:
`target/cross-producer-fusion-0216-producer-report.json`

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 22 | 20 | 2 | 0 |

## Validation

Commands run:

```sh
bash scripts/check_cross_producer_fusion_corpus.sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --fail-on-fallback --max-edge 160 --output target/cross-producer-fusion-0216-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-unsupported-boundary --max-edge 160 --output target/cross-producer-fusion-0216-boundary-classification.json
cargo run -p ferrugo-cli --no-default-features -- producer-regression-report fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --max-edge 160 --output target/cross-producer-fusion-0216-producer-report.json
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check -- fixtures/cross-producer-fusion-manifest.tsv fixtures/cross-producer-fusion-matrix.tsv scripts/check_cross_producer_fusion_corpus.sh docs/corpus-taxonomy.md docs/policies/producer-regression-bisect-workflow.md docs/milestones/README.md docs/milestones/0216-cross-producer-typical-document-fusion-corpus.md docs/reports/cross-producer-fusion-corpus-2026-06-29.md
```
