# Cross-Producer Regression Bisect 2026-06-29

Milestone: 0190.

## Summary

Added `pdfrust-cli producer-regression-report`, a native-only report that turns
manifest producer metadata into actionable regression triage output. The command
requires a manifest, filters directory inputs to manifest-listed fixtures, and
groups outcomes by producer, family, and feature tags.

This milestone does not build a hosted regression service and does not publish
private fixture details.

## CLI Contract

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- producer-regression-report fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --max-edge 160 --output target/producer-regression-0190-report.json
```

Output fields:

| Field | Purpose |
| --- | --- |
| `summary` | Total, native rendered, fallback required, and error counts. |
| `producer_groups` | Outcome groups keyed by `producer:*` manifest feature tags. |
| `family_groups` | Outcome groups keyed by the manifest family column. |
| `feature_groups` | Outcome groups keyed by non-expected, non-producer feature tags. |
| `records` | Per-fixture records with committed fixture IDs or redacted local IDs. |

Privacy field:

```text
no PDF bytes, rendered pixels, extracted text, private filenames, or document hashes
```

## Current Producer Report

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 15 | 13 | 2 | 0 |

Supported producer families:

| Family | Total | Native rendered | Pass rate |
| --- | ---: | ---: | ---: |
| `accounting` | 2 | 2 | 1.000 |
| `browser` | 3 | 3 | 1.000 |
| `government` | 2 | 2 | 1.000 |
| `office-suite` | 3 | 3 | 1.000 |
| `pdf20` | 1 | 1 | 1.000 |
| `scanner` | 2 | 2 | 1.000 |

Typed fallback producer groups:

| Producer group | Family | Bucket | Affected features | Milestone route |
| --- | --- | --- | --- | --- |
| `layered-presentation-export` | `unsupported-boundary` | `graphics.optional-content` | `ocmd`, `optional-content` | `0192 optional-content-ui-state` |
| `fax-scanner-export` | `unsupported-boundary` | `image.filter` | `ccitt`, `codec`, `image`, `unsupported` | `0209 rust-native-image-codec` |

## Simulated Failed Classification

The existing unsupported-boundary fixtures exercise the failed-fixture
classification path without adding private inputs:

- `optional-content-ocmd.pdf` proves that an optional-content producer boundary
  is routed to 0192 instead of being hidden in a generic fallback count.
- `unsupported-ccitt-image.pdf` proves that a fax/scanner codec boundary is
  routed to 0209 with the `image.filter` bucket.

Both records keep committed generated fixture IDs. Local-only fixtures would be
reported as `local-only-####` IDs.

## Workflow

The local workflow is documented in
`docs/policies/producer-regression-bisect-workflow.md`.

Use that workflow to:

- rerun the producer report on the same checkout as the failed gate;
- identify producer, family, and feature clusters;
- route typed unsupported boundaries before opening renderer regressions;
- bisect with the smallest affected manifest family;
- keep private fixture details out of committed artifacts and issue text.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-cli/src/main.rs docs/milestones/0190-cross-producer-regression-bisect-workflow.md docs/milestones/README.md docs/policies/corpus-governance.md docs/policies/producer-regression-bisect-workflow.md docs/reports/cross-producer-regression-bisect-2026-06-29.md
cargo test -p pdfrust-cli producer_regression -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- producer-regression-report fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --max-edge 160 --output target/producer-regression-0190-report.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
