# Native Renderer 1.3 Hardening Gate

Date: 2026-06-29
Milestone: 0210

## Decision

The 1.3 Rust-native renderer hardening gate passes as a scoped server-side,
PDFium-free runtime path. The hardening run did not expose crashes, panics,
untyped failures, server batch budget failures, fuzz-smoke failures,
native-only packaging failures, or WASM smoke failures.

This is not a claim that every PDF feature is native. The release risk list is
explicit: 12 typed fallback rows remain in the 1.3 scorecard corpus, plus one
encrypted policy error. These remain visible follow-up work and must not be
hidden by automatic PDFium runtime fallback.

## Scorecard

Artifact: `target/hardening-0210-scorecard/scorecard.json`

| Metric | Value |
| --- | ---: |
| Weighted score | 94.15 |
| Family count | 6 |
| Native rendered | 196 |
| Fallback required | 12 |
| Encrypted policy errors | 1 |
| Server batch budget failures | 0 |

Support gate artifact:
`target/hardening-0210-scorecard/dashboard/support.json`

| Family | Total | Native | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| `form` | 27 | 27 | 0 | 0 |
| `mixed-layout` | 30 | 27 | 2 | 1 encrypted |
| `office-export` | 63 | 62 | 1 | 0 |
| `presentation` | 12 | 10 | 2 | 0 |
| `report` | 50 | 46 | 4 | 0 |
| `scan` | 27 | 24 | 3 | 0 |

## Release Risk List

| Bucket | Count | Release stance |
| --- | ---: | --- |
| `image.filter` | 3 | Deferred by 0209 codec deployment policy for CCITT, JBIG2, and JPX. |
| `graphics.optional-content` | 2 | Needs OCMD/usage-application follow-up before broad presentation claims. |
| `graphics.transparency` | 2 | Needs transparency-stack hardening before broad design/prepress claims. |
| `annotation.appearance` | 1 | FreeText without appearance remains typed unsupported. |
| `form.xfa-dynamic` | 1 | Dynamic XFA remains outside static native rendering scope. |
| `graphics.color-management` | 1 | Advanced color-management boundary remains explicit. |
| `graphics.pattern-shading` | 1 | Unsupported mesh/pattern shading remains explicit. |
| `text.font-program` | 1 | Color/emoji font program boundary remains explicit. |
| `encrypted` | 1 | Stable policy error, not a crash or malformed untyped failure. |

## Determinism And Repeat Signal

Artifact: `target/hardening-0210-repeat.json`

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 | 0 |

The repeat benchmark used three repetitions across supported real-world-style
families. It validates stable repeated render behavior, cache-policy reporting,
and budget conformance for the hardening slice.

## Batch, Package, Fuzz, And WASM Gates

| Gate | Result |
| --- | --- |
| Server batch sample | 16 jobs, 16 native, 0 fallbacks, 0 errors, 0 budget failures, 44.327 jobs/sec. |
| Fuzz smoke | Passed: primitive parse, xref load, stream decode, content tokenize, render setup. |
| Native-only release | Passed: native check/test, plugin-free distribution, PDFium quarantine, package dry-runs, all-features clippy. |
| WASM smoke | Passed: 728680-byte artifact, compile 2.305 ms, instantiate 0.089 ms, smoke 5.886 ms. |

## Validation

Commands run:

```text
bash scripts/generate_coverage_scorecard.sh target/hardening-0210-scorecard
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --include-family invoice --include-family statement --include-family scanned-packet --include-family form --include-family browser-export --include-family office-export --include-family report --include-family malformed-recovery --repetitions 3 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/hardening-0210-repeat.json
bash scripts/check_fuzz_smoke.sh
bash scripts/check_native_only_release.sh
bash scripts/check_wasm_smoke.sh
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
