# Golden Image And SSIM Oracle Report - 2026-07-05

This report records the first widened native golden-image gate and the matching
Poppler oracle trend surface after SSIM/DSSIM reporting was added to
`visual-diff-poppler`.

The committed golden manifest contains 33 exact native hash baselines:

- 32 generated fixtures: four samples each for `browser-print`,
  `email-web-archive`, `form`, `mixed-layout`, `office-export`, `presentation`,
  `report`, and `scan`;
- 1 public real-world fixture: `real-scan`.

The other public real-world fixtures remain oracle evidence rather than golden
hash rows because the current native renderer returns native errors for them.

## Commands

```sh
bash scripts/check_native_golden_images.sh

cargo run -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest /tmp/ferrugo-generated-oracle-manifest.tsv \
  --include-family browser-print \
  --include-family email-web-archive \
  --include-family form \
  --include-family mixed-layout \
  --include-family office-export \
  --include-family presentation \
  --include-family report \
  --include-family scan \
  --max-edge 160 \
  --timeout 30 \
  --output target/issue-105-generated-poppler-visual-diff.json

cargo run -p ferrugo --no-default-features -- visual-diff-poppler fixtures/real-world \
  --manifest /tmp/ferrugo-real-world-oracle-manifest.tsv \
  --include-family real-browser-print \
  --include-family real-office-export \
  --include-family real-report \
  --include-family real-scan \
  --max-edge 160 \
  --timeout 30 \
  --output target/issue-105-real-world-poppler-visual-diff.json
```

`pdftoppm` version: `26.06.0`.

## Generated Corpus Oracle

Top-level summary: 32 total, 7 exact, 14 accepted drift, 11 blockers, 0 native
errors, 0 reference errors.

| Family | Total | Exact | Accepted drift | Blockers | Comparable | Mean SSIM | Min SSIM | Max DSSIM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 4 | 1 | 3 | 0 | 4 | 0.997690 | 0.991022 | 0.004489 |
| `email-web-archive` | 4 | 0 | 4 | 0 | 4 | 0.996401 | 0.989639 | 0.005180 |
| `form` | 4 | 2 | 2 | 0 | 4 | 0.947334 | 0.869615 | 0.065193 |
| `mixed-layout` | 4 | 0 | 2 | 2 | 4 | 0.918625 | 0.811825 | 0.094088 |
| `office-export` | 4 | 0 | 0 | 4 | 4 | 0.884115 | 0.796507 | 0.101746 |
| `presentation` | 4 | 3 | 0 | 1 | 4 | 0.962744 | 0.850978 | 0.074511 |
| `report` | 4 | 1 | 0 | 3 | 4 | 0.845330 | 0.549458 | 0.225271 |
| `scan` | 4 | 0 | 3 | 1 | 4 | 0.746685 | 0.007211 | 0.496395 |

## Real-World Oracle

Top-level summary: 4 total, 0 exact, 0 accepted drift, 1 blocker, 3 native
errors, 0 reference errors.

| Family | Total | Blockers | Native errors | Comparable | Mean SSIM | Min SSIM | Max DSSIM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `real-browser-print` | 1 | 0 | 1 | 0 | null | null | null |
| `real-office-export` | 1 | 0 | 1 | 0 | null | null | null |
| `real-report` | 1 | 0 | 1 | 0 | null | null | null |
| `real-scan` | 1 | 1 | 0 | 1 | 0.422494 | 0.422494 | 0.288753 |

## Interpretation

The widened golden gate is a self-regression gate: every committed golden row
must render natively and match exact dimensions, decoded RGBA bytes, and
deterministic PNG bytes.

The Poppler oracle report is not an all-clear fidelity claim. It establishes
baseline per-family SSIM/DSSIM drift data so future runs can distinguish stable
status counts from perceptual drift changes.
