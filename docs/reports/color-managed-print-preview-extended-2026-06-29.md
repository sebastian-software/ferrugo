# Color Managed Print Preview Extended Gate

Date: 2026-06-29
Milestone: 0208

## Decision

The 0208 server-side print-preview runtime gate passes without PDFium. The
Rust-native renderer covers the scoped OutputIntent, ICCBased image,
DeviceCMYK, registration, spot-color approximation, overprint approximation,
prepress page-box, and print-visible annotation fixtures with zero runtime
fallbacks.

Press-proof colorimetry is still outside scope. CMYK and spot-color visual
parity differences remain documented review gaps rather than runtime blockers.

## Corpus

0208 adds `fixtures/color-managed-print-preview-manifest.tsv`.

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `output-intent` | 2 | OutputIntent metadata and prepress page boxes. |
| `process-color` | 1 | DeviceCMYK image conversion baseline. |
| `icc-image` | 3 | ICCBased RGB, Gray, and CMYK image paths plus transform-cache policy. |
| `registration` | 1 | Registration marks and process color bars. |
| `spot-overprint` | 4 | Separation, DeviceN, and overprint RGB-thumbnail approximations. |
| `print-state` | 1 | Print-visible annotation state. |

## Native Support Gate

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --fail-on-fallback --max-edge 180 --output target/color-print-0208-supported.json
```

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 12 | 12 | 0 | 0 |

## Benchmark And Operators

| Gate | Total | Native rendered | Fallbacks | Errors | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| Benchmark | 12 | 12 | 0 | 0 | 0 |

| Total fixtures | Scanned | Errors | Total operators | Implemented | Partial | Unsupported | Ignored |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 12 | 12 | 0 | 195 | 179 | 16 | 0 | 0 |

The partial operators are the expected policy-dependent color, spot-color, and
graphics-state subsets. No operator in this gate is classified unsupported.

## Visual Review

Poppler is used only as an independent review oracle.

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --max-edge 120 --max-mae 10 --max-p95 72 --max-changed-ratio 0.25 --output target/color-print-0208-poppler.json
```

| Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 12 | 0 | 5 | 4 | 0 | 3 |

Review blockers:

| Fixture | Reason |
| --- | --- |
| `cmyk-image.pdf` | CMYK conversion differs from Poppler's color-managed output. |
| `icc-cmyk-image.pdf` | ICCBased CMYK follows the native bounded approximation path. |
| `devicen-spot-color.pdf` | DeviceN spot-color approximation differs from Poppler. |
| `separation-spot-color.pdf` | Separation spot-color approximation differs from Poppler. |

These are accepted as documented approximation gaps for thumbnails. They should
remain visible in future color-fidelity burn-down work, but they do not require
PDFium in the 0208 runtime path.

## Cache And Memory Signal

`cargo test -p ferrugo-render icc_transform_cache -- --nocapture` passed the
focused cache tests:

- repeated ICCBased resource builds reuse the transform cache;
- a one-entry cache evicts deterministically by budget.

The native grouped print-preview test also renders the 0208 corpus with
`AnnotationMode::Print`, covering the print-state behavior that the current CLI
summary and benchmark commands do not parameterize.

## Validation

Commands run:

```text
cargo fmt --check
cargo test -p ferrugo-native color_managed_print_preview -- --nocapture
cargo test -p ferrugo-render icc_transform_cache -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --fail-on-fallback --max-edge 180 --output target/color-print-0208-supported.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/color-print-0208-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --output target/color-print-0208-operators.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/color-managed-print-preview-manifest.tsv --include-family output-intent --include-family process-color --include-family icc-image --include-family registration --include-family spot-overprint --include-family print-state --max-edge 120 --max-mae 10 --max-p95 72 --max-changed-ratio 0.25 --output target/color-print-0208-poppler.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
