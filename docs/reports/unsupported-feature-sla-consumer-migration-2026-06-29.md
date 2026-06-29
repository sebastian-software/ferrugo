# Unsupported Feature SLA And Consumer Migration

Date: 2026-06-29
Milestone: 0219

## Summary

Milestone 0219 turns typed unsupported outcomes into a consumer-facing SLA and
native-only migration guide. The public contract remains class-first routing
through `ThumbnailError::class()`, with stable unsupported buckets available for
feature-specific telemetry, support copy, and backlog routing.

New artifacts:

- `docs/policies/unsupported-feature-sla.md`
- `docs/guides/native-only-consumer-migration.md`
- `scripts/check_unsupported_feature_sla.sh`

## Public API Example

The thumbnail facade now includes a focused consumer migration test. It verifies
that applications can route:

- `image.filter` to scan codec review;
- `form.xfa-dynamic` to producer migration;
- generic unsupported outcomes to a native-feature backlog;
- malformed inputs separately from unsupported features.

This example uses only `ThumbnailErrorClass` and
`unsupported_feature_bucket()`, not backend-internal renderer state or display
message parsing.

## SLA Classification Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/unsupported-0219-classification.json
```

Result:

| Total | Native rendered | Typed unsupported | Malformed errors | Encrypted errors |
| ---: | ---: | ---: | ---: | ---: |
| 233 | 217 | 12 | 3 | 1 |

Unsupported buckets:

| Bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 2 |
| `graphics.transparency` | 2 |
| `annotation.appearance` | 1 |
| `form.xfa-dynamic` | 1 |
| `graphics.color-management` | 1 |
| `graphics.pattern-shading` | 1 |
| `text.font-program` | 1 |

Policy errors are not unsupported buckets: the corpus still has 3 malformed
fixtures and 1 encrypted fixture.

## Strict Supported-Family Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family email-web-archive --include-family form --fail-on-fallback --max-edge 160 --output target/unsupported-0219-supported-families.json
```

Result:

| Families | Total | Native rendered | Typed unsupported | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print`, `email-web-archive`, `form` | 46 | 46 | 0 | 0 |

This gate is the consumer-facing proof that supported families can be operated
without hidden runtime PDFium fallback.

## Package Dry-Runs

Package file-list dry-runs passed for:

- `pdfrust-thumbnail`
- `pdfrust-cli`

The dry-runs keep public API examples and migration docs aligned with packaged
consumer artifacts.

## Validation

Commands run:

```sh
bash scripts/check_unsupported_feature_sla.sh
cargo test -p pdfrust-thumbnail consumer_migration -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/unsupported-0219-classification.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family email-web-archive --include-family form --fail-on-fallback --max-edge 160 --output target/unsupported-0219-supported-families.json
cargo package -p pdfrust-thumbnail --allow-dirty --no-verify --list
cargo package -p pdfrust-cli --allow-dirty --no-verify --list
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check -- crates/pdfrust-thumbnail/src/lib.rs docs/errors.md docs/packaging.md docs/policies/unsupported-feature-sla.md docs/guides/native-only-consumer-migration.md docs/reports/unsupported-feature-sla-consumer-migration-2026-06-29.md scripts/check_unsupported_feature_sla.sh docs/milestones/README.md docs/milestones/0219-unsupported-feature-sla-and-consumer-migration-guide.md
```
