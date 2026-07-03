# Default Banded Replay Final Coverage

Date: 2026-07-03
Issue: #28

## Summary

This report refreshes the default-profile generated-corpus banding coverage
after the later raster work in #25 and #26. The default policy remains:

- render below `160,000` output pixels with the single target;
- render supported pages at or above `160,000` output pixels with serial
  64-row banding;
- keep parallel replay opt-in through the explicit parallel profiles.

The refreshed corpus run shows no above-threshold native-rendered fixture left
on the single-target path. The remaining default-profile single-target renders
are below the rollout threshold, not current guard fallbacks.

## Coverage Command

```bash
target/release/ferrugo benchmark-native fixtures/generated --native-profile default --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue28-final/default-corpus-coverage-rss.json
```

Artifact:

- `target/issue28-final/default-corpus-coverage-rss.json`

## Coverage Result

| Metric | Count |
| --- | ---: |
| Total fixtures | 233 |
| Native rendered | 217 |
| Fallback required | 12 |
| Errors | 4 |
| Budget failures | 16 |
| Native rendered with banding | 9 |
| Native rendered single-target | 208 |
| Above-threshold native single-target renders | 0 |

The process RSS sampler returned `null` in this run, so this report does not
replace the high-DPI before/after RSS evidence in
`docs/reports/default-banding-rollout-2026-07-03.md`. It closes the remaining
coverage question: no native-rendered generated fixture above the default
rollout threshold is still falling back to single-target replay.

## Banded Fixtures

| Fixture | Output | Bands | Active target peak bytes | Active target reduction |
| --- | ---: | ---: | ---: | ---: |
| `academic-publisher-first-page.pdf` | 360x480 | 8 | 92,160 | 866 permille |
| `browser-chromium-article-print.pdf` | 360x460 | 8 | 92,160 | 860 permille |
| `engineering-large-transform-detail.pdf` | 1024x683 | 11 | 262,144 | 906 permille |
| `high-dpi-preview-fidelity.pdf` | 480x360 | 6 | 122,880 | 822 permille |
| `image-heavy-repeated-xobject-report.pdf` | 400x480 | 8 | 102,400 | 866 permille |
| `layout-columns-footnotes-table-stress.pdf` | 400x520 | 9 | 102,400 | 876 permille |
| `page-size-letter.pdf` | 612x792 | 13 | 156,672 | 919 permille |
| `scientific-two-column-paper.pdf` | 360x480 | 8 | 92,160 | 866 permille |
| `technical-large-coordinate-plan.pdf` | 1024x614 | 10 | 262,144 | 895 permille |

## Closeout

The earlier #28 slices supplied the Type3 byte-parity matrix and default-profile
RSS evidence. With this refreshed coverage run:

- Type3 remains covered by the banded-vs-single parity matrix;
- default-profile banding remains thresholded and serial;
- every native-rendered generated fixture above the default threshold bands;
- below-threshold single-target replay remains the documented default policy.

No additional #28 code change is needed unless the default rollout threshold is
changed or new above-threshold fixtures expose an unsupported banded surface.

## Validation

```bash
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated --native-profile default --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue28-final/default-corpus-coverage-rss.json
```
