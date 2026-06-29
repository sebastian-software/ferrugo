# Browser Print Coverage

Date: 2026-06-24.
Milestone: 0065.

## Scope

The committed browser-print corpus uses deterministic generated PDFs as
license-safe proxies for print-to-PDF output. Normal tests do not depend on a
live Chrome, Safari, or Firefox installation. Real browser exports should be
sampled locally through `fixtures/local-corpus/metadata.toml` and reported only
as aggregate family results.

Committed browser-print coverage currently includes:

- page geometry and full-page fill (`page-size-letter.pdf`);
- vector paths and fills (`vector-paths.pdf`);
- even-odd clipping (`clipped-paths.pdf`);
- unfiltered inline image execution (`inline-image.pdf`).

## Corpus Summary

Command:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/browser-print-summary.json
```

Result:

| Family | Total | Native rendered | Native pass rate | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 4 | 4 | 1.000 | 0 | 0 |

## PDFium Differential Smoke

Native and direct PDFium rendering were run at `--max-edge 260` using the local
PDFium dylib at
`/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`.

| Fixture | Native | PDFium | Native size | PDFium size |
| --- | --- | --- | --- | --- |
| `clipped-paths.pdf` | ok | ok | 120x120 | 120x120 |
| `inline-image.pdf` | ok | ok | 120x120 | 120x120 |
| `page-size-letter.pdf` | ok | ok | 201x260 | 201x260 |
| `vector-paths.pdf` | ok | ok | 220x180 | 220x180 |

## Remaining Browser-Specific Gaps

No committed browser-print fixture currently requires PDFium fallback. Remaining
browser-specific risks are tracked separately from generic rendering failures:

- filtered inline images should use `image.filter`;
- ICC/profiled or CSS color conversions should use `image.color-space`;
- complex clipping or stroke fidelity should use `graphics.stroke-clip`;
- link annotations and generated appearance differences should use
  `annotation.appearance`;
- embedded subset font gaps should use `text.font-program`,
  `text.cmap-tounicode`, or `text.glyph-outline`.

Live Chrome/Safari/Firefox exports remain a local-corpus sampling task until a
deterministic offline generation path is added.
