# PDF 2.0 Feature Usage Corpus 2026-06-28

Milestone: 0181.

## Decision

Keep PDF 2.0 support evidence-driven for the 1.2 roadmap. The current generated
corpus contains three PDF 2.0 fixtures: two are accepted native-rendering cases,
and one is a visual-impacting typed unsupported color-management boundary.

This is not a broad PDF 2.0 support claim. It is a usage and priority gate for
the PDFium-free native renderer.

## Usage Classifier

Added `classify-pdf20-usage` to `pdfrust-cli`. The command combines:

- PDF header version detection;
- catalog `/Version /2.0` detection;
- manifest feature tags such as `pdf-2.0`, `associated-files`, and
  `black-point-compensation`;
- native render outcome classification.

The JSON report is privacy-safe and does not persist PDF bytes, rendered pixels,
text samples, stream bytes, or operands.

## Corpus Result

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- classify-pdf20-usage fixtures/generated \
  --manifest fixtures/pdf20-compatibility-manifest.tsv \
  --max-edge 160 \
  --output target/pdf20-0181-usage.json
```

Summary:

| Metric | Count |
| --- | ---: |
| Total generated PDFs scanned | 211 |
| PDF 2.0 documents detected | 3 |
| Native rendered | 2 |
| Typed unsupported | 1 |
| Errors | 0 |

Feature counts:

| Feature | Count | Policy | Impact |
| --- | ---: | --- | --- |
| `pdf-2.0-version-marker` | 3 | Accept existing render path | Visual-supported |
| `catalog-version` | 3 | Accept existing render path | Non-visual |
| `associated-files` | 1 | Ignore metadata-only for thumbnails | Non-visual |
| `black-point-compensation` | 1 | Typed unsupported | Visual-unsupported |

## Native Gate

Supported PDF 2.0 subset:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/pdf20-compatibility-manifest.tsv \
  --include-family accepted-office \
  --include-family accepted-associated-file \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/pdf20-0181-supported-gate.json
```

Result: 2 total, 2 native rendered, 0 fallback required, 0 errors.

Full PDF 2.0 classification:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/pdf20-compatibility-manifest.tsv \
  --include-family accepted-office \
  --include-family accepted-associated-file \
  --include-family unsupported-color-management \
  --max-edge 160 \
  --output target/pdf20-0181-classification.json
```

Result: 3 total, 2 native rendered, 1 fallback required, 0 errors. The only
fallback category is `graphics.color-management` for
`black-point-compensation`.

## 1.2 Priority Ranking

| Rank | Feature | Observed documents | Bucket | Recommendation |
| ---: | --- | ---: | --- | --- |
| 1 | `black-point-compensation` | 1 | `graphics.color-management` | Keep typed unsupported for 1.2 unless real-corpus frequency rises; implement only with color-threshold evidence. |
| 2 | `associated-files` | 1 | n/a | Keep accepted as metadata-only for thumbnails and preserve regression fixtures. |
| 3 | `catalog-version` | 3 | n/a | Keep accepting when render operators stay in existing supported paths. |

## Validation

Commands run:

```sh
cargo fmt
cargo test -p pdfrust-cli pdf20_usage -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- classify-pdf20-usage fixtures/generated --manifest fixtures/pdf20-compatibility-manifest.tsv --max-edge 160 --output target/pdf20-0181-usage.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/pdf20-compatibility-manifest.tsv --include-family accepted-office --include-family accepted-associated-file --fail-on-fallback --max-edge 160 --output target/pdf20-0181-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/pdf20-compatibility-manifest.tsv --include-family accepted-office --include-family accepted-associated-file --include-family unsupported-color-management --max-edge 160 --output target/pdf20-0181-classification.json
```
