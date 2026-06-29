# Form Filling Appearance Update And Flattening Coverage

Date: 2026-06-29
Milestone: 0206

## Decision

Common filled and already-flattened form exports are covered as static native
render inputs. This milestone does not add a form editor, save pipeline, or
native form-flattening writer. Existing producer-generated appearances remain
authoritative, bounded missing-appearance synthesis remains a thumbnail fallback,
and dynamic XFA without static AcroForm fields remains a typed unsupported
boundary.

## Corpus

0206 adds `fixtures/form-filling-flattening-manifest.tsv` and three generated
fixtures:

| Fixture | Family | Purpose |
| --- | --- | --- |
| `acroform-combo-box-appearance.pdf` | `existing-appearance` | Filled choice/combo-box widget with explicit normal appearance stream. |
| `acroform-rotated-text-field-appearance.pdf` | `existing-appearance` | Filled rotated text-field widget with explicit normal appearance stream. |
| `flattened-form-export.pdf` | `flattened-static` | Filled form exported as ordinary static page content. |

The manifest also reuses existing text, checkbox, radio, signature, synthesized
missing-appearance, static-XFA, dynamic-XFA, business-form, and government-form
fixtures.

## Policy Boundary

`docs/policies/acroform-appearances.md` now explicitly covers:

- source-generated choice/combo-box and rotated field appearances;
- already-flattened form exports as ordinary page content;
- continued rejection of native form mutation and native form-flattening writes;
- dynamic XFA without static AcroForm fields as `form.xfa-dynamic`.

## Native Support Gate

Supported-family command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --fail-on-fallback --max-edge 160 --output target/form-filling-0206-supported.json
```

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 15 | 15 | 0 | 0 |

XFA-boundary command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family xfa-boundary --max-edge 160 --output target/form-filling-0206-xfa-boundary.json
```

| Total | Native rendered | Fallbacks | Fallback category | Errors |
| ---: | ---: | ---: | --- | ---: |
| 2 | 1 | 1 | `form.xfa-dynamic` | 0 |

## Benchmark And Operator Coverage

| Gate | Total | Native rendered | Fallbacks | Errors | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| Benchmark | 15 | 15 | 0 | 0 | 0 |

| Total fixtures | Scanned | Errors | Total operators | Implemented | Partial | Unsupported | Ignored |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 17 | 17 | 0 | 462 | 453 | 9 | 0 | 0 |

## Visual Review

Poppler is used here only as an independent review oracle. It is not part of
the supported runtime path.

```sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family flattened-static --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/form-filling-0206-poppler.json
```

| Total | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: |
| 11 | 7 | 1 | 0 | 3 |

The one native visual blocker is the existing
`e-signature-contract-workflow.pdf` signature-boundary fixture: MAE `25.571`,
p95 `209`, changed ratio `0.180939`. It is retained as follow-up fidelity work
for signature appearance composition. It does not indicate a fallback, crash, or
form-mutation write.

Reference errors are Poppler-side review failures/timeouts in the local run and
are tracked separately from native renderer support.

## Validation

Commands run:

```text
python3 scripts/generate_fixtures.py
cargo fmt --check
python3 -m py_compile scripts/generate_fixtures.py
cargo test -p pdfrust-native native_backend_should_render_generated_form_filling_flattening_fixtures -- --nocapture
cargo test -p pdfrust-native acroform -- --nocapture
cargo test -p pdfrust-native form_filling -- --nocapture
cargo test -p pdfrust-native signature_presence -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --fail-on-fallback --max-edge 160 --output target/form-filling-0206-supported.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family xfa-boundary --max-edge 160 --output target/form-filling-0206-xfa-boundary.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/form-filling-0206-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --include-family xfa-boundary --output target/form-filling-0206-operators.json
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family flattened-static --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/form-filling-0206-poppler.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
