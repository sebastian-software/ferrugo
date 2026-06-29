# Annotation Popup Stamp And FreeText Fidelity

Date: 2026-06-29
Milestone: 0207

## Decision

Common static annotation workflows are covered without adding interactive
annotation behavior. Existing annotation appearance streams remain
authoritative, supported missing-appearance markup remains bounded, popup state
is inert metadata for thumbnails, and FreeText without a usable appearance
stream remains a typed `annotation.appearance` unsupported boundary.

## Corpus

0207 adds `fixtures/annotation-popup-stamp-freetext-manifest.tsv` and three
generated fixtures:

| Fixture | Family | Purpose |
| --- | --- | --- |
| `freetext-annotation-appearance.pdf` | `appearance-stream` | FreeText annotation with explicit normal appearance stream. |
| `stamp-annotation-rotated-appearance.pdf` | `stamp-appearance` | Print-visible rotated stamp annotation with explicit normal appearance. |
| `popup-annotation-inert-state.pdf` | `popup-boundary` | Popup metadata remains inert while the parent text note renders statically. |

The manifest also reuses existing annotation appearance, highlight, print-state,
synthesized markup, inert link, text-note, and unsupported FreeText fixtures.

## Native Support Gates

Supported-family command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --fail-on-fallback --max-edge 160 --output target/annotation-0207-supported.json
```

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 10 | 10 | 0 | 0 |

Unsupported-boundary command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family unsupported-synthesis --max-edge 160 --output target/annotation-0207-unsupported.json
```

| Total | Native rendered | Fallbacks | Fallback category | Errors |
| ---: | ---: | ---: | --- | ---: |
| 1 | 0 | 1 | `annotation.appearance` | 0 |

## Benchmark And Operator Coverage

| Gate | Total | Native rendered | Fallbacks | Errors | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| Benchmark | 10 | 10 | 0 | 0 | 0 |

| Total fixtures | Scanned | Errors | Total operators | Implemented | Partial | Unsupported | Ignored |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 10 | 1 | 207 | 193 | 14 | 0 | 0 |

The operator scan error is the expected FreeText-without-appearance unsupported
boundary. It is covered separately by the support gate above.

## Visual Review

Poppler is used here only as an independent review oracle. It is not part of
the supported runtime path.

```sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/annotation-0207-poppler.json
```

| Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 2 | 5 | 0 | 0 | 3 |

The reference errors are Poppler-side review failures/timeouts in the local run,
not native renderer failures.

## Validation

Commands run:

```text
python3 scripts/generate_fixtures.py
cargo fmt --check
python3 -m py_compile scripts/generate_fixtures.py
cargo test -p pdfrust-native popup_stamp_freetext -- --nocapture
cargo test -p pdfrust-native annotation -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --fail-on-fallback --max-edge 160 --output target/annotation-0207-supported.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family unsupported-synthesis --max-edge 160 --output target/annotation-0207-unsupported.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/annotation-0207-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --include-family unsupported-synthesis --output target/annotation-0207-operators.json
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/annotation-0207-poppler.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
