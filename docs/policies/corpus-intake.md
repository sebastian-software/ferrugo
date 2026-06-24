# Corpus Intake Policy

Status: accepted.
Date: 2026-06-24.

The validation corpus must improve PDFium-retirement evidence without storing
private or legally ambiguous documents in Git.

## Allowed Inputs

- Generated fixtures from repository scripts.
- Synthetic-realistic documents that contain no private data.
- Public documents only when the license permits redistribution and the source
  is recorded.
- Local-only private samples under `fixtures/local-corpus/`, excluded from Git,
  with aggregated metadata only.

## Disallowed Inputs

- Customer, employee, financial, health, legal, or other private documents.
- Screenshots, rendered pages, extracted text, hashes, or filenames that can
  identify private documents.
- Large binary fixtures without a size, memory, and product-value rationale.
- Third-party PDFs without a clear redistribution license.

## Metadata Requirements

Every committed corpus row must include:

- relative path
- document category
- reproducible source
- license
- page count
- feature tags
- notes explaining why the fixture matters for PDFium retirement

Expected backend behavior is encoded as a feature tag:

- `expected:native`
- `expected:pdfium-fallback`
- `expected:error-encrypted`
- `expected:error-malformed`

Use additional tags such as `perf-risk`, `visual-risk`, or `memory-risk` when a
fixture needs special interpretation.

## Categories

The real-world-style corpus uses production-shaped categories:

- `invoice`
- `report`
- `scanned-packet`
- `form`
- `statement`
- `browser-export`
- `office-export`
- `presentation`
- `secure-document`
- `malformed-recovery`

These categories may map back to broader generated-corpus families when
reporting historical coverage.

## Size Policy

Committed fixtures should stay small enough for normal test and benchmark runs.
Large or many-page examples need a written reason tied to memory, streaming, or
performance gates. Prefer generated minimal reproductions over storing full
source documents.

## Local-Only Reporting

Private local corpus results must be reported only as aggregates:

- category
- document count
- coarse page-count range
- native rendered count
- fallback count by bucket
- error count by public class

Do not publish document names, hashes, extracted text, screenshots, or rendered
outputs from private samples.
