# Transparency Stack Memory Optimization

Milestone: 0213
Date: 2026-06-29

## Summary

Transparency group rasterization now reuses a pass-local scratch
`RasterDevice` for same-sized group intermediates. The renderer still clips the
intermediate to the group device bounds and enforces
`max_transparency_group_pixels` before allocation. The scratch surface is
cleared before reuse, stays scoped to one rasterization pass, and is not shared
across documents, pages, or worker jobs.

Nested transparency groups keep a separate recursive rasterization context, so
parent and child intermediates cannot alias. Unsupported transparency semantics,
such as luminosity soft masks and unsupported blend modes, remain typed
`graphics.transparency` boundaries rather than memory-budget failures.

## Coverage

Added `fixtures/transparency-stack-memory-manifest.tsv` with nine existing
generated fixtures:

| Family | Count | Purpose |
| --- | ---: | --- |
| `alpha-stack` | 1 | ExtGState alpha baseline. |
| `group-stack` | 3 | Isolated, knockout, and overlapping-alpha group surfaces. |
| `soft-mask-stack` | 1 | Image soft-mask alpha surface. |
| `office-transparency-stack` | 2 | Clipped group and repeated office vector effects. |
| `presentation-transparency-stack` | 1 | Layered slide image, tint, and shadow. |
| `chart-transparency-stack` | 1 | Chart slide with translucent rotated callout. |

## Native Gate

Artifact: `target/transparency-stack-0213-supported-gate.json`

Result:

- Total: 9
- Native rendered: 9
- Fallback required: 0
- Errors: 0

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/transparency-stack-memory-manifest.tsv --include-family alpha-stack --include-family group-stack --include-family soft-mask-stack --include-family office-transparency-stack --include-family presentation-transparency-stack --include-family chart-transparency-stack --fail-on-fallback --max-edge 160 --output target/transparency-stack-0213-supported-gate.json
```

## Low-Memory Benchmark

Artifact: `target/transparency-stack-0213-low-memory-benchmark.json`

Result:

- Total: 9
- Native rendered: 9
- Fallback required: 0
- Errors: 0
- Budget failures: 0
- Slowest family mean: `office-transparency-stack` at `23.784ms`
- Largest family output bytes: `office-transparency-stack` at `138880`

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/transparency-stack-memory-manifest.tsv --include-family alpha-stack --include-family group-stack --include-family soft-mask-stack --include-family office-transparency-stack --include-family presentation-transparency-stack --include-family chart-transparency-stack --native-profile low-memory --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/transparency-stack-0213-low-memory-benchmark.json
```

## Focused Tests

```bash
cargo test -p ferrugo-render form_transparency_group_should_reuse_same_sized_scratch_surface -- --nocapture
cargo test -p ferrugo-render transparency_group -- --nocapture
cargo test -p ferrugo-native transparency -- --nocapture
```

All focused tests passed locally.

## Workspace Validation

```bash
cargo fmt --check
git diff --check -- crates/ferrugo-render/src/lib.rs docs/backend/native.md docs/reports/transparency-stack-memory-2026-06-29.md fixtures/transparency-stack-memory-manifest.tsv
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All workspace validation commands passed locally. The repository still has the
pre-existing unstaged `.gitignore` whitespace change, which is unrelated to this
milestone and was not modified here.
