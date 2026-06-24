# Office Export Coverage

Date: 2026-06-24.
Milestone: 0064.

## Scope

The committed office-export corpus is generated, license-safe, and tracked in
`fixtures/corpus-manifest.tsv`. It now covers simple office text, embedded font
resolution, ToUnicode mapping, Encoding Differences, fragmented text spacing,
and a ruled table layout with text cells.

## Corpus Summary

Command:

```sh
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/office-expanded-summary.json
```

Result:

| Family | Total | Native rendered | Native pass rate | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `office-export` | 6 | 6 | 1.000 | 0 | 0 |

## PDFium Differential Smoke

Native and direct PDFium rendering were run at `--max-edge 260` using the local
PDFium dylib at
`/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib`.

| Fixture | Native | PDFium | Native size | PDFium size |
| --- | --- | --- | --- | --- |
| `embedded-font.pdf` | ok | ok | 180x100 | 180x100 |
| `encoding-differences.pdf` | ok | ok | 160x100 | 160x100 |
| `office-table.pdf` | ok | ok | 260x160 | 260x160 |
| `text-page.pdf` | ok | ok | 260x139 | 260x139 |
| `text-spacing.pdf` | ok | ok | 260x120 | 260x120 |
| `tounicode-text.pdf` | ok | ok | 160x100 | 160x100 |

## Remaining Differences

No office-export fixture requires PDFium fallback. Remaining visual risk is
categorized as text fidelity, not unsupported rendering:

- fallback bitmap glyphs are still visibly different from PDFium text
  rasterization;
- advanced office exports may still expose CID/subset fonts, shaping, vertical
  writing, or complex tables not represented by this committed corpus;
- dense spreadsheet output should be expanded in 0073 with smaller text,
  repeated table headers, and logos.

Unsupported office failures should use the existing `text.font-program`,
`text.cmap-tounicode`, `text.glyph-outline`, `graphics.stroke-clip`, or
`renderer.memory-budget` fallback categories when those blockers appear.
