# Corpus Governance Dashboard 2026-06-26

Milestone: 0179

## Summary

Added a native-only corpus dashboard generation flow and governance policy for
fixture provenance, license handling, privacy, coverage, performance, memory,
and regression visibility.

The dashboard script composes existing CLI gates instead of introducing a web
service. It writes generated artifacts under `target/corpus-dashboard/` and
keeps committed manifests and policy docs as the source of truth.

## Added

- `scripts/generate_corpus_dashboard.sh`
- `docs/policies/corpus-governance.md`

## Dashboard Artifacts

Command:

```sh
bash scripts/generate_corpus_dashboard.sh target/corpus-dashboard-0179
```

Generated:

| Artifact | Purpose |
| --- | --- |
| `metadata.json` | Native metadata extraction with manifest context. |
| `local-corpus-validation.json` | Aggregate local corpus metadata validation. |
| `support.json` | Native support, fallback buckets, and public error classes. |
| `operators.json` | Operator coverage and unsupported operator status. |
| `performance.json` | Native performance sample for report/presentation fixtures. |
| `batch.json` | Server batch isolation/performance sample. |
| `dashboard.json` | Compact release-decision summary linking the artifacts. |

## Dashboard Result

Native support classification over the primary generated families:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 187 | 176 | 10 | 1 encrypted |

Fallback categories:

| Category | Count |
| --- | ---: |
| `form.xfa-dynamic` | 1 |
| `graphics.color-management` | 1 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |
| `graphics.transparency` | 2 |
| `image.filter` | 3 |
| `text.font-program` | 1 |

Operator coverage:

| Total | Scanned | Errors | Operators | Inline images |
| ---: | ---: | ---: | ---: | ---: |
| 187 | 186 | 1 | 9652 | 0 |

Performance sample:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 53 | 48 | 5 | 0 | 5 |

Server batch sample:

| Inputs | Jobs | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 16 | 16 | 0 | 0 | 0 |

Local corpus metadata validation used `fixtures/local-corpus.example.toml` and
passed with 2 aggregate samples, 5 aggregate documents, and 2 synthetic
replacement links, without exposing private document names or payload data.

## Governance Rules

`docs/policies/corpus-governance.md` defines:

- required manifest metadata;
- generated, public, private, and local-only fixture handling;
- review rules for adding and removing fixtures;
- regression owner/category/severity/status expectations;
- dashboard privacy boundaries.

## Validation

- `bash scripts/generate_corpus_dashboard.sh target/corpus-dashboard-0179`
- `cargo run -p pdfrust-cli --no-default-features -- validate-local-corpus fixtures/local-corpus.example.toml --allow-missing`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
