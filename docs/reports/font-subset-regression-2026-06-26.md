# Font Subset Regression 2026-06-26

## Summary

Milestone 0136 expands the Rust-native font regression corpus with reduced
fixtures for common subset font patterns that appear in office, browser-print,
report, and publishing exports. The new fixtures cover TrueType widths, CFF
ToUnicode mapping, Type0 CID descendant widths, repeated Type3 CharProcs, and
deterministic fallback for a subset-prefixed missing font.

The gate renders all five fixtures through the Rust-native backend with no
PDFium fallback, no errors, and no benchmark budget failures.

## Fixture Matrix

| Family | Fixture | Feature focus |
| --- | --- | --- |
| `truetype-subset` | `fixtures/generated/subset-truetype-widths.pdf` | Subset-prefixed TrueType with explicit `/Widths` and embedded `FontFile2`. |
| `cff-subset` | `fixtures/generated/subset-cff-tounicode.pdf` | Subset-prefixed CFF `FontFile3` plus explicit ToUnicode mapping. |
| `cid-subset` | `fixtures/generated/subset-cid-widths.pdf` | Type0 CID font with `/Identity-H`, descendant `/W`, and ToUnicode. |
| `type3-subset` | `fixtures/generated/subset-type3-repeated-charprocs.pdf` | Repeated Type3 CharProc glyph reuse and width advancement. |
| `missing-font-subset` | `fixtures/generated/subset-missing-font.pdf` | Subset-prefixed missing font resolved through deterministic fallback. |

The same five files are registered in `fixtures/font-subset-manifest.tsv` for
focused gates and in `fixtures/corpus-manifest.tsv` under the `office-export`
family for broader corpus coverage.

## Native Fallback Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/font-subset-manifest.tsv --include-family truetype-subset --include-family cff-subset --include-family cid-subset --include-family type3-subset --include-family missing-font-subset --fail-on-fallback --max-edge 160 --output target/font-subset-0136-supported-gate.json
```

Summary:

| Metric | Value |
| --- | ---: |
| Fixtures | 5 |
| Native rendered | 5 |
| Fallback required | 0 |
| Errors | 0 |
| Native pass rate | 1.000 |

Every family reported one native render and a pass rate of `1.000`.

## Benchmark Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/font-subset-manifest.tsv --include-family truetype-subset --include-family cff-subset --include-family cid-subset --include-family type3-subset --include-family missing-font-subset --iterations 2 --max-edge 160 --max-ms 1000 --max-output-bytes 1048576 --output target/font-subset-0136-benchmark.json
```

Summary:

| Metric | Value |
| --- | ---: |
| Fixtures | 5 |
| Iterations | 2 |
| Native rendered | 5 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |

Per-family timing:

| Family | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: |
| `cff-subset` | 0.805 | 0.805 | 55,680 |
| `cid-subset` | 0.691 | 0.691 | 55,680 |
| `missing-font-subset` | 0.430 | 0.430 | 51,200 |
| `truetype-subset` | 0.396 | 0.396 | 55,680 |
| `type3-subset` | 1.145 | 1.145 | 47,360 |

The Type3 fixture is the slowest of this small set because it repeats CharProc
path rendering, but it remains well under the 1000 ms budget.

## Validation

- `cargo fmt --check`
- `cargo test -p ferrugo-native font_subset_regression -- --nocapture`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks ...`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native ...`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
