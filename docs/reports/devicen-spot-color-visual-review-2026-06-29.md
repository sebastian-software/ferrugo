# DeviceN Spot Color Visual Review 2026-06-29

Milestone: 0191.

## Summary

Added a focused visual-review suite for Separation, DeviceN, CMYK-alternate
tint transforms, and overprint-adjacent spot-color thumbnails. The goal is
understandable RGB preview output for common business and print-adjacent
documents, not proofing-level colorimetry or separations output.

## Fixture Coverage

New generated fixtures:

| Fixture | Family | Coverage |
| --- | --- | --- |
| `spot-letterhead-separation.pdf` | `separation-business` | Letterhead-style Separation brand bar and tint rows. |
| `spot-invoice-devicen-stamp.pdf` | `devicen-business` | Invoice-style DeviceN approval stamp with table context. |
| `spot-cmyk-tint-swatch.pdf` | `separation-cmyk` | CMYK-alternate spot tint swatches at 25%, 55%, and 100%. |

Added `fixtures/spot-color-visual-review-manifest.tsv`, which combines the new
samples with existing Separation, DeviceN, overprint, and prepress spot-color
baselines.

## Policy Boundary

Supported for thumbnail review:

- Separation and DeviceN vector fill/stroke content.
- Bounded Type 2 tint transforms.
- DeviceGray, DeviceRGB, and DeviceCMYK alternate conversion to RGB.
- Visible spot-color approximations in business and print-adjacent pages.

Out of scope:

- certified print proofing;
- separations or plate output;
- proprietary spot-color libraries;
- arbitrary sampled tint functions;
- treating category-local color drift thresholds as global visual defaults.

## Native Support Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --fail-on-fallback --max-edge 180 --output target/spot-color-0191-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | --- |
| 7 | 7 | 0 | `{}` |

All seven families rendered 1/1 natively.

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 129600 --output target/spot-color-0191-benchmark.json
```

Result:

| Family | Total | Native | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `devicen-baseline` | 1 | 1 | 0 | 0 | 2.514 | 2.514 | 57600 |
| `devicen-business` | 1 | 1 | 0 | 0 | 10.182 | 10.182 | 95760 |
| `overprint-baseline` | 1 | 1 | 0 | 0 | 29.348 | 29.348 | 57600 |
| `prepress-boundary` | 1 | 1 | 0 | 0 | 5.704 | 5.704 | 97200 |
| `separation-baseline` | 1 | 1 | 0 | 0 | 35.350 | 35.350 | 57600 |
| `separation-business` | 1 | 1 | 0 | 0 | 5.039 | 5.039 | 93600 |
| `separation-cmyk` | 1 | 1 | 0 | 0 | 4.799 | 4.799 | 93600 |

The first benchmark attempt was started in parallel with Poppler visual diff
and was killed with exit 137. The standalone repeat passed and is the recorded
result.

## Visual Review

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --max-edge 180 --max-mae 24 --max-p95 140 --max-changed-ratio 0.45 --timeout 30 --output target/spot-color-0191-poppler-visual-diff.json
```

Thresholds are intentionally category-local:

| Metric | Threshold |
| --- | ---: |
| Mean absolute error | 24.000 |
| p95 channel delta | 140 |
| Changed ratio | 0.450 |

Result: 7 total, 0 exact, 7 accepted drift, 0 blockers, 0 native errors, 0
reference errors.

| Fixture | Status | Mean abs error | p95 delta | Changed ratio |
| --- | --- | ---: | ---: | ---: |
| `devicen-spot-color.pdf` | accepted drift | 14.285 | 76 | 0.319444 |
| `overprint-spot-approximation.pdf` | accepted drift | 0.042 | 0 | 0.000278 |
| `prepress-spot-overprint-boundary.pdf` | accepted drift | 2.436 | 0 | 0.030329 |
| `separation-spot-color.pdf` | accepted drift | 9.666 | 23 | 0.428194 |
| `spot-cmyk-tint-swatch.pdf` | accepted drift | 5.032 | 41 | 0.182094 |
| `spot-invoice-devicen-stamp.pdf` | accepted drift | 6.593 | 48 | 0.113576 |
| `spot-letterhead-separation.pdf` | accepted drift | 1.258 | 0 | 0.044231 |

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs scripts/generate_fixtures.py fixtures/corpus-manifest.tsv fixtures/spot-color-visual-review-manifest.tsv docs/corpus-taxonomy.md docs/backend/native.md docs/policies/prepress-boundary.md docs/milestones/0191-devicen-spot-color-visual-review-samples.md docs/milestones/README.md docs/reports/devicen-spot-color-visual-review-2026-06-29.md
cargo test -p ferrugo-native spot_color -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --fail-on-fallback --max-edge 180 --output target/spot-color-0191-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 129600 --output target/spot-color-0191-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spot-color-visual-review-manifest.tsv --include-family separation-business --include-family devicen-business --include-family separation-cmyk --include-family separation-baseline --include-family devicen-baseline --include-family overprint-baseline --include-family prepress-boundary --max-edge 180 --max-mae 24 --max-p95 140 --max-changed-ratio 0.45 --timeout 30 --output target/spot-color-0191-poppler-visual-diff.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
