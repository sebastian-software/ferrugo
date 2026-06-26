# Government Form And Certificate Coverage

Date: 2026-06-26.
Milestone: 0148.

## Summary

The government/form corpus now has a focused manifest at
`fixtures/government-form-manifest.tsv`. It separates supported static
government-style forms from the intentionally unsupported dynamic-XFA boundary.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `government-permit-checkbox-form.pdf` | Permit-style form with AcroForm checkbox appearance, stamp marker, barcode marker, and signature line. |
| `government-certificate-seal-signature.pdf` | Certificate-style page with strict border geometry, seal marker, QR-style marker, signature line, and page label metadata. |
| `government-tax-notice-barcode.pdf` | Tax-notice reduction with dense table rules, stamp marker, barcode marker, and synthetic footer text. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family permit --include-family certificate --include-family tax-notice --include-family widget-appearance --include-family signature-appearance --include-family static-xfa --include-family business-form --fail-on-fallback --max-edge 160 --output target/government-0148-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 |

Supported family result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `business-form` | 1 | 1 | 0 | 0 |
| `certificate` | 1 | 1 | 0 | 0 |
| `permit` | 1 | 1 | 0 | 0 |
| `signature-appearance` | 2 | 2 | 0 | 0 |
| `static-xfa` | 1 | 1 | 0 | 0 |
| `tax-notice` | 1 | 1 | 0 | 0 |
| `widget-appearance` | 1 | 1 | 0 | 0 |

## Dynamic Form Boundary

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family dynamic-xfa-unsupported --max-edge 160 --output target/government-0148-dynamic-backlog.json
```

Result:

| Family | Total | Native rendered | Fallback required | Fallback category |
| --- | ---: | ---: | ---: | --- |
| `dynamic-xfa-unsupported` | 1 | 0 | 1 | `form.xfa-dynamic` |

Dynamic XFA remains an explicit unsupported policy boundary. Static government
forms and certificate pages are not blocked by this boundary.

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family permit --include-family certificate --include-family tax-notice --include-family widget-appearance --include-family signature-appearance --include-family static-xfa --include-family business-form --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/government-0148-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0 | 2 | 6 | 0 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `government-certificate-seal-signature.pdf` | blocker | `annotations-forms` | 10.515 | 37 | 0.161568 |
| `government-permit-checkbox-form.pdf` | blocker | `rendering-core` | 5.248 | 35 | 0.139440 |
| `government-tax-notice-barcode.pdf` | blocker | `rendering-core` | 8.870 | 67 | 0.151347 |

These blockers are visual-fidelity deltas, not native runtime fallbacks. They
route to widget appearance parity, line/table geometry, and stamp/barcode
composition work.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `government-permit-checkbox-form.pdf` | 1,937 |
| `government-certificate-seal-signature.pdf` | 1,331 |
| `government-tax-notice-barcode.pdf` | 1,374 |
| **Total new PDF bytes** | **4,642** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- `rg -n "private|customer|confidential|personal|production|PII|@" ...`
  returned only synthetic "no private data" fixture text plus an existing
  confidentiality clause in an unrelated contract fixture generator.
- New fixture content is synthetic and has no private identity-document source.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family permit --include-family certificate --include-family tax-notice --include-family widget-appearance --include-family signature-appearance --include-family static-xfa --include-family business-form --fail-on-fallback --max-edge 160 --output target/government-0148-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family dynamic-xfa-unsupported --max-edge 160 --output target/government-0148-dynamic-backlog.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/government-form-manifest.tsv --include-family permit --include-family certificate --include-family tax-notice --include-family widget-appearance --include-family signature-appearance --include-family static-xfa --include-family business-form --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/government-0148-visual-diff.json
cargo test -p pdfrust-native acroform -- --nocapture
cargo test -p pdfrust-native signature -- --nocapture
cargo test -p pdfrust-native annotation_appearance -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/government-permit-checkbox-form.pdf fixtures/generated/government-certificate-seal-signature.pdf fixtures/generated/government-tax-notice-barcode.pdf
rg -n "private|customer|confidential|personal|production|PII|@" fixtures/corpus-manifest.tsv fixtures/government-form-manifest.tsv scripts/generate_fixtures.py
```
