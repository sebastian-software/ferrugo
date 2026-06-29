# PDF Operator Semantic Snapshot Suite

Date: 2026-06-29
Milestone: 0211

## Decision

The first operator semantic snapshot suite is in place. It freezes
operator-level semantics for reduced text, path, inline-image, and pattern
fixtures using normalized operator coverage rather than pixel-only comparison.

This suite is intentionally narrow. It catches high-impact drift in common
operator classes while keeping visual corpus gates responsible for full-page
fidelity.

## Snapshot Corpus

0211 adds `fixtures/operator-semantic-snapshot-manifest.tsv`.

| Family | Fixture | Snapshot focus |
| --- | --- | --- |
| `text-state` | `text-page.pdf` | `BT`, `Tf`, `Td`, `Tj`, `ET` text object semantics. |
| `path-state` | `vector-paths.pdf` | Graphics state, move/line, stroke, rectangle, and fill semantics. |
| `image-state` | `inline-image.pdf` | Inline image recognition and placement operator envelope. |
| `pattern-state` | `tiling-pattern.pdf` | Pattern color-space operators as partial, typed `image.color-space` semantics. |

## Rust Snapshot Test

`operator_coverage_should_match_semantic_snapshots` freezes:

- total operator count per fixture;
- inline-image count;
- operator support status;
- typed fallback bucket for partial pattern color-space operators.

This catches renderer coverage drift before it appears only as a visual diff.

## CLI Operator Gate

```sh
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --output target/operator-snapshot-0211-operators.json
```

| Total | Scanned | Errors | Operators | Inline images | Implemented | Partial | Unsupported | Ignored |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 0 | 24 | 1 | 22 | 2 | 0 | 0 |

The two partial operators are `cs` and `scn` in the pattern fixture. Both remain
typed as `image.color-space` rather than silently becoming unsupported or
incorrectly marked fully implemented.

## Visual Smoke

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/operator-snapshot-0211-poppler.json
```

| Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 1 | 2 | 0 | 0 | 1 |

The reference error is a Poppler-side review issue in the local run, not a
native renderer error. No reduced fixture produced a visual blocker.

## Validation

Commands run:

```text
cargo fmt --check
cargo test -p ferrugo-native operator_coverage_should_match_semantic_snapshots -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --output target/operator-snapshot-0211-operators.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/operator-snapshot-0211-poppler.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
