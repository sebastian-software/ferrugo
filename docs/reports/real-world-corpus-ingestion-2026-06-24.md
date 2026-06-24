# Real-World Corpus Ingestion 2026-06-24

Milestone: 0083.

## Implemented Slice

- Added `docs/policies/corpus-intake.md` with privacy, license, metadata, size,
  and local-only reporting rules.
- Added `fixtures/real-world-style-manifest.tsv`, a privacy-safe
  synthetic-realistic manifest over existing generated PDFs.
- Updated `docs/corpus-taxonomy.md` to describe the real-world-style manifest
  and expected-backend feature tags.

No private or third-party PDFs were committed.

## Manifest Entries

| Category | Fixture | Expected backend |
| --- | --- | --- |
| invoice | `office-table.pdf` | `expected:native` |
| statement | `multi-page-report.pdf` | `expected:native` |
| scanned-packet | `scanned-page.pdf` | `expected:native` |
| form | `acroform-text-field.pdf` | `expected:native` |
| browser-export | `vector-paths.pdf` | `expected:native` |
| office-export | `text-page.pdf` | `expected:native` |
| report | `vector-stress.pdf` | `expected:native`, `perf-risk` |
| presentation | `optional-content-ocmd.pdf` | `expected:pdfium-fallback` |
| secure-document | `encrypted-placeholder.pdf` | `expected:error-encrypted` |
| malformed-recovery | `malformed-xref-offset-drift.pdf` | `expected:native` |

Every row includes path, category, source, license, page count, feature tags,
and a retirement-oriented note.

## Validation Results

Manifest structure and file presence:

- 10 manifest rows.
- 10 existing PDF paths.
- 10 rows with an `expected:*` feature tag.
- Added committed text footprint: 2183 bytes for the manifest and 2332 bytes
  for the intake policy.

Commands:

```text
cargo run -p pdfrust-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --output target/0083-real-world-style-metadata.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --max-edge 160 --output target/0083-real-world-style-fallback-summary.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- benchmark-pdfium fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/0083-real-world-style-pdfium-benchmark.json
```

The existing corpus commands take an input directory and classify every PDF in
that directory. Because the new manifest intentionally covers only the
real-world-style seed rows, the remaining generated fixtures are reported under
`unclassified`.

Seed-category native results:

| Category | Native result |
| --- | --- |
| invoice | rendered |
| statement | rendered |
| scanned-packet | rendered |
| form | rendered |
| browser-export | rendered |
| office-export | rendered |
| report | rendered, perf-risk retained |
| presentation | fallback required: `graphics.optional-content` |
| secure-document | expected `encrypted` error |
| malformed-recovery | rendered |

PDFium benchmark comparison for the same manifest categories rendered all
non-encrypted seed entries and reported only the expected encrypted error.

## Repository Size Impact

No new PDF binaries were added. The committed size increase is text-only and
small:

```text
2183 fixtures/real-world-style-manifest.tsv
2332 docs/policies/corpus-intake.md
```

## Remaining Limits

- This is not a real private/local corpus run.
- The manifest is a synthetic-realistic seed, not evidence of production
  distribution.
- Follow-up work should add local-only aggregate reporting once real samples
  are available outside Git.
