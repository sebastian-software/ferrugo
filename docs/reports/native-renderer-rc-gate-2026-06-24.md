# Native Renderer Release Candidate Gate 2026-06-24

Milestone: 0080.
Decision: no-go for broad primary production renderer status.

The Rust-native renderer is strong enough for native-first experiments and
category-limited rollout behind explicit PDFium fallback, but it is not yet a
release candidate for broad primary production rendering of the targeted
typical-document surface.

## Evidence Summary

Generated fixture corpus:

| Signal | Result |
| --- | ---: |
| Fixtures | 52 |
| Native rendered | 50 |
| Native fallback required | 1 |
| Expected error fixtures | 1 |
| Metadata comparisons against PDFium | 52 / 52 match |
| Native smoke benchmark budget failures | 3 |
| PDFium smoke benchmark budget failures | 1 |

The expected error fixture is `encrypted-placeholder.pdf`. It is not a release
blocker because both backends report an encrypted input rather than producing
silent output.

## Release Criteria

For this gate, native primary status requires all of these:

- Native renders every non-encrypted committed fixture without requiring PDFium
  fallback.
- Metadata comparison against PDFium matches across the generated corpus.
- Smoke benchmark failures are either expected policy errors or documented
  non-blocking performance outliers.
- Visual fidelity has enough automated or reviewed evidence to classify
  category risk.
- Native-only validation succeeds for the supported release surface.

The current run satisfies metadata parity and most category render coverage, but
it fails native-only coverage and visual-fidelity confidence.

## Corpus Results

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/rc-0080-fallback-summary.json
```

Summary:

| Family | Total | Native rendered | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| browser-print | 4 | 4 | 0 | 0 |
| form | 6 | 6 | 0 | 0 |
| mixed-layout | 9 | 8 | 0 | 1 encrypted |
| office-export | 10 | 10 | 0 | 0 |
| presentation | 4 | 3 | 1 | 0 |
| report | 12 | 12 | 0 | 0 |
| scan | 7 | 7 | 0 | 0 |

Strict native-only gate:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --fail-on-fallback --output target/rc-0080-native-only-gate.json
```

Result: failed with `1 native fallback(s) required`.

The fallback is `fixtures/generated/optional-content-ocmd.pdf` in the
presentation family, bucketed as `graphics.optional-content`.

## Metadata Comparison

Corpus-wide metadata comparison was run by invoking the PDFium-enabled
`compare-metadata` command for every manifest entry.

Summary artifact:

```text
target/rc-0080-metadata-comparison-summary.json
```

Result:

| Total | Matches | Mismatches | Process errors |
| ---: | ---: | ---: | ---: |
| 52 | 52 | 0 | 0 |

This is good evidence for page-count and page-geometry parity. It is not visual
fidelity evidence.

## Benchmark Results

Native command:

```text
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/rc-0080-benchmark-native.json
```

PDFium command:

```text
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- benchmark-pdfium fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/rc-0080-benchmark-pdfium.json
```

Native summary:

| Total | Rendered | Fallback | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 52 | 50 | 1 | 1 | 3 |

PDFium summary:

| Total | Rendered | Fallback | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 52 | 51 | 0 | 1 | 1 |

Native budget failures:

| Fixture | Family | Violation | Notes |
| --- | --- | --- | --- |
| `encrypted-placeholder.pdf` | mixed-layout | `render_error` | Expected encrypted policy result |
| `optional-content-ocmd.pdf` | presentation | `native_fallback` | Release blocker |
| `vector-stress.pdf` | report | `render_time` | 2865.649 ms at smoke budget |

PDFium only reports the encrypted fixture as a budget failure.

## Visual Fidelity

No automated full-corpus pixel-diff dashboard exists yet. Existing fixture tests
exercise rendering paths and earlier reports document category-specific visual
risks, but this gate cannot claim broad visual fidelity readiness without the
later visual-diff workflow.

Known fidelity risks that remain relevant:

- Text-heavy fixtures still rely on the current text rasterization policy, with
  shaped/CJK fidelity caveats documented in earlier reports.
- `vector-stress.pdf` renders but is slow enough to exceed the smoke budget.
- Optional-content membership policy still requires PDFium fallback for one
  valid presentation fixture.

## Decision

No-go for broad primary production renderer status.

Allowed next state:

- Continue native-first rendering for controlled categories that pass without
  fallback.
- Keep PDFium-enabled fallback available for unsupported native features.
- Treat the generated corpus as a strong but insufficient proxy until real-world
  corpus ingestion and visual diffing land.

## Blockers

1. `optional-content-ocmd.pdf` requires fallback in the presentation family.
2. `vector-stress.pdf` exceeds the smoke render-time budget.
3. Full-corpus visual-diff automation is missing.
4. Text fidelity risk remains category-dependent despite passing render tests.

## Follow-Up Plan

- 0081 should turn these RC blockers into a prioritized backlog with subsystem
  owners and measurable gates.
- 0083 should add real-world corpus ingestion before another broad readiness
  decision.
- 0084 should add the visual diff dashboard needed for visual fidelity claims.
- 0096 should use the benchmark evidence to profile vector/report hot paths.
- The optional-content blocker should stay ahead of any fallback-removal or
  native-only packaging milestone.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo test -p ferrugo-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/rc-0080-fallback-summary.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/rc-0080-benchmark-native.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- benchmark-pdfium fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/rc-0080-benchmark-pdfium.json
```

All non-strict validation commands completed successfully. The strict
native-only fallback gate failed as expected and is recorded as release-blocker
evidence.
