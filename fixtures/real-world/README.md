# Real-World Benchmark Fixtures

Status: accepted #97 seed tier.
Date: 2026-07-04.

These PDFs are committed because they are small, public, redistributable, and
exercise real producer output that the generated fixture corpus cannot fully
represent. They are benchmark and compatibility inputs only; do not use their
results as broad renderer claims without the performance-claims checklist.

The license basis for the federal fixtures follows the repository corpus intake
policy: U.S. federal government works are public-domain for U.S. copyright
purposes, subject to the normal government-material caveats for logos,
trademarks, privacy, endorsement, and third-party material. Each source was
reviewed as a public federal source before committing.

| File | Family | Source | License | Pages | SHA-256 | Producer notes |
| --- | --- | --- | --- | ---: | --- | --- |
| `irs-fw9.pdf` | `real-office-export` | `https://www.irs.gov/pub/irs-pdf/fw9.pdf` | Public domain, U.S. federal government work | 6 | `2d420cbb4123dcf1fb82595b2359cfbb5d81f00b9df9d359fcc7af361d093f53` | Designer 6.5, XFA form, JavaScript, tagged |
| `usagov-report-scams-chrome-print.pdf` | `real-browser-print` | Chrome print of `https://www.usa.gov/where-report-scams` on 2026-07-04 | Public domain, U.S. federal government work | 2 | `4be8b878a00d0b675b607f90bcaad0e1fab41381ac0209b11533947def0a2110` | HeadlessChrome 150, Skia/PDF m150, tagged |
| `uspto-patent-6289801.pdf` | `real-scan` | `https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6289801` | Public domain, U.S. patent document | 6 | `8f6734f846ce01b7e15b431b810a2c283047576f3ab120fa2162d8dd85d8ee4a` | USPTO PDF Builder, large scanned pages |
| `cdc-data-brief-492.pdf` | `real-report` | `https://www.cdc.gov/nchs/data/databriefs/db492.pdf` | Public domain, U.S. federal government work | 8 | `02156ddfa7d88f2998c91d1a43875605b0b7d21844a648774332e1d8a2e3da77` | Adobe InDesign 19.3, charts, tagged |

Run the real-world matrix seed with:

```sh
INPUT=fixtures/real-world MAX_EDGE=1024 ITERATIONS=5 WARMUP=1 \
  OUTPUT=target/real-world-performance-matrix.json \
  REPORT=target/real-world-performance-matrix.md \
  ARTIFACT_DIR=target/real-world-performance-matrix-artifacts \
  bash scripts/generate_performance_matrix.sh
```

For private or reference-only PDFs, keep files under `fixtures/local-corpus/`
and create an untracked TSV that follows `fixtures/local-corpus-matrix.example.tsv`.
