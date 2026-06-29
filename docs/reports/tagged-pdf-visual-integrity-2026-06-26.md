# Tagged PDF Visual Integrity

Date: 2026-06-26.
Milestone: 0154.

## Summary

The tagged-PDF visual corpus now has a focused manifest at
`fixtures/tagged-pdf-visual-manifest.tsv`. It covers accessibility metadata and
marked-content wrappers across common visual surfaces while preserving the
boundary that metadata is not a drawing command.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `tagged-report-visual-integrity.pdf` | Tagged report page with marked header/table/chart content and structure roles. |
| `tagged-form-visual-integrity.pdf` | Tagged form page with an AcroForm checkbox widget and structure metadata. |
| `tagged-office-alt-text.pdf` | Tagged office-style page with a figure, alt text in the structure element, and a visual table. |
| `tagged-structure-heavy-report.pdf` | Tagged report with 64 marked-content entries for bounded structure traversal and low-memory profiling. |

The existing `tagged-accessibility-metadata.pdf` baseline remains in the
focused manifest.

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --fail-on-fallback --max-edge 160 --output target/tagged-0154-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 5 | 5 | 0 | 0 |

Family result:

| Family | Total | Native rendered | Fallback required |
| --- | ---: | ---: | ---: |
| `metadata-baseline` | 1 | 1 | 0 |
| `structure-heavy` | 1 | 1 | 0 |
| `tagged-form` | 1 | 1 | 0 |
| `tagged-office` | 1 | 1 | 0 |
| `tagged-report` | 1 | 1 | 0 |

## Metadata Diagnostics

Focused native tests assert the new fixtures report:

- `Lang` as `en-US`.
- `/MarkInfo /Marked true`.
- `/StructTreeRoot` and RoleMap presence.
- Marked-content references.
- Bounded structure role counts, including 65 roles for
  `tagged-structure-heavy-report.pdf`.
- No traversal truncation.

Command:

```sh
cargo test -p ferrugo-native tagged_visual -- --nocapture
cargo test -p ferrugo-native accessibility -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --output target/tagged-0154-metadata.json
```

`extract-corpus-metadata` currently emits the full generated fixture directory;
the native tests are the focused assertion gate for this milestone.

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/tagged-0154-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `metadata-baseline` | 1 | 5.456 | 5.456 | 0 |
| `structure-heavy` | 1 | 46.057 | 46.057 | 0 |
| `tagged-form` | 1 | 37.903 | 37.903 | 0 |
| `tagged-office` | 1 | 38.667 | 38.667 | 0 |
| `tagged-report` | 1 | 37.468 | 37.468 | 0 |

Low-memory structure-heavy command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family structure-heavy --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/tagged-0154-low-memory-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `structure-heavy` | 1 | 49.392 | 49.392 | 0 |

## Visual Oracle

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/tagged-0154-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 0 | 1 | 4 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `rendering-core` | 4 | 1 | 3 | 0 |
| `text-fonts` | 1 | 0 | 1 | 0 |

Fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `tagged-accessibility-metadata.pdf` | accepted drift | `rendering-core` | 0.515 | 0 | 0.012500 |
| `tagged-form-visual-integrity.pdf` | blocker | `rendering-core` | 8.164 | 38 | 0.131776 |
| `tagged-office-alt-text.pdf` | blocker | `text-fonts` | 6.047 | 40 | 0.122547 |
| `tagged-report-visual-integrity.pdf` | blocker | `rendering-core` | 10.531 | 68 | 0.159644 |
| `tagged-structure-heavy-report.pdf` | blocker | `rendering-core` | 16.009 | 83 | 0.265977 |

These blockers are renderer fidelity deltas on otherwise supported page
content. The structure metadata itself does not cause native errors, fallback,
or unbounded traversal.

## Boundary

- Tagged PDF metadata is reported through metadata diagnostics.
- Marked content is non-visual except for the content stream operators inside
  the marked sequence.
- Alt text and reading-order semantics are not interpreted for visual rendering.
- Full reading-order extraction remains deferred to milestone 0182.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `tagged-report-visual-integrity.pdf` | 1,669 |
| `tagged-form-visual-integrity.pdf` | 1,830 |
| `tagged-office-alt-text.pdf` | 1,834 |
| `tagged-structure-heavy-report.pdf` | 11,676 |
| **Total new PDF bytes** | **17,009** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- New fixture content is synthetic and contains no real accessibility tree,
  office export, signer, customer, or user data.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo test -p ferrugo-native tagged_visual -- --nocapture
cargo test -p ferrugo-native accessibility -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --fail-on-fallback --max-edge 160 --output target/tagged-0154-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --output target/tagged-0154-metadata.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/tagged-0154-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family structure-heavy --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/tagged-0154-low-memory-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/tagged-0154-visual-diff.json
wc -c fixtures/generated/tagged-report-visual-integrity.pdf fixtures/generated/tagged-form-visual-integrity.pdf fixtures/generated/tagged-office-alt-text.pdf fixtures/generated/tagged-structure-heavy-report.pdf
find fixtures/generated -name '*.pdf' -size +512k -print
```
