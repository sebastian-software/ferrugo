# Native Renderer 1.3 Coverage Scorecard

Date: 2026-06-29
Milestone: 0201

## Decision

Milestone 0201 establishes a reproducible native-only scorecard for the 1.3
renderer work. The current weighted score is `94.04`, but the scorecard is not a
release pass: `presentation` is below the proposed per-family threshold, and the
12 typed unsupported rows still block broad PDFium replacement claims.

PDFium remains outside the supported runtime path. Visual drift is tracked as a
separate validation channel and is not folded into the runtime score.

## Artifacts

Command:

```sh
scripts/generate_coverage_scorecard.sh target/coverage-scorecard-0201
```

Artifacts:

- `target/coverage-scorecard-0201/scorecard.json`
- `target/coverage-scorecard-0201/scorecard.md`
- `target/coverage-scorecard-0201/dashboard/dashboard.json`

## Score Formula

The baseline favors typical-document impact over raw PDF feature count.

```text
family_score =
  100 * ((0.8 * native_pass_rate) + (0.2 * operator_maturity_rate))

operator_maturity_rate =
  (implemented + ignored + 0.5 * partial) / total_operators

weighted_score =
  sum(family_score * typical_document_weight)
```

The score is intentionally runtime-focused. Visual drift remains a separate
channel so unsupported or visually weak areas cannot disappear into a single
aggregate number.

## Family Scorecard

| Family | Weight | Score | Native pass | Unsupported | Errors | Partial operators | Memory budget | Timeout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `report` | 0.24 | 93.49 | 92.0% | 4 | 0 | 44 | 0 | 0 |
| `office-export` | 0.22 | 98.67 | 98.4% | 1 | 0 | 29 | n/a | n/a |
| `scan` | 0.18 | 91.11 | 88.9% | 3 | 0 | 0 | n/a | n/a |
| `form` | 0.14 | 99.89 | 100.0% | 0 | 0 | 9 | n/a | n/a |
| `mixed-layout` | 0.12 | 90.87 | 88.9% | 2 | 1 encrypted | 9 | n/a | n/a |
| `presentation` | 0.10 | 86.09 | 83.3% | 2 | 0 | 15 | 0 | 0 |

The encrypted mixed-layout row is a security policy outcome rather than a native
renderer crash or PDFium fallback.

## Weighted Gap Queue

| Category | Count | Weighted gap points | Severity | Routed milestone |
| --- | ---: | ---: | --- | --- |
| `image.filter` | 3 | 2.000 | high | 0209 Rust-Native Image Codec Deployment Policy |
| `graphics.optional-content` | 2 | 1.667 | medium | 0211 PDF Operator Semantic Snapshot Suite |
| `graphics.transparency` | 2 | 0.960 | high | 0213 Transparency Stack Memory Optimization |
| `graphics.color-management` | 1 | 0.480 | medium | 0208 Color Managed Print Preview Extended Gate |
| `graphics.pattern-shading` | 1 | 0.480 | medium | 0204 Office Chart SmartArt And Vector Effect Fidelity |
| `annotation.appearance` | 1 | 0.444 | medium | 0207 Annotation Popup Stamp And FreeText Fidelity |
| `form.xfa-dynamic` | 1 | 0.444 | documented boundary | 0206 Form Filling Appearance Update And Flattening Coverage |
| `text.font-program` | 1 | 0.349 | high | 0202 Text Selection Geometry And Search Highlight Parity |

The ranking shows that the next high-impact work is not edge-case cleanup:
scanned image codecs, optional-content semantics, transparency, print color, and
office/chart/vector surfaces are all typical-document blockers.

## 1.3 Thresholds

| Threshold | Value |
| --- | ---: |
| Weighted score minimum | 94.00 |
| Per-family score minimum | 88.00 |
| Supported-family native pass rate | 100.0% |
| Supported-family error budget | 0 |
| Server batch budget failures | 0 |
| Runtime PDFium allowed | no |

These thresholds are strict enough to block a weak family even if the weighted
average passes. That matters for 1.3 because `presentation` is below threshold
today despite the aggregate score landing above 94.

## Validation

Commands run:

```sh
scripts/generate_coverage_scorecard.sh target/coverage-scorecard-0201
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
