# 0168: Email Client And Web Archive PDF Coverage

Status: done
Phase: 31
Size: medium
Depends on: 0167

## Goal

Add coverage for PDFs produced from email clients and saved web archives, a
common source of mixed fonts, inline images, attachments, and irregular page
layout.

## Scope

- Add fixtures for email print-to-PDF output from representative producers.
- Cover mixed text encodings, inline images, link annotations, page headers, and
  long message threads.
- Classify attachment and portfolio behavior when the visual page is separate
  from embedded files.
- Fix bounded renderer issues that affect common email-style documents.

## Non-Goals

- Parse email formats directly.
- Extract or render arbitrary embedded attachments.
- Support active content in archived web pages.

## Deliverables

- Email and web-archive PDF fixture set.
- Coverage report by producer and feature pressure.
- Native renderer fixes or typed unsupported classifications.

## Acceptance Criteria

- Representative email-style PDFs render natively with stable output.
- Embedded or attached non-page content is handled by policy.
- Memory use remains bounded for long message threads.

## Validation

- Run native-only `cargo test`.
- Run email/web-archive fixture visual comparison.
- Run long-document memory profile subset.
- Run fallback summary for new fixtures.

## Completion Notes

Completed on 2026-06-26.

Report:

- `docs/reports/email-web-archive-coverage-2026-06-26.md`

Implemented:

- Added four generated email/web-archive fixtures for a multi-page email
  thread, inline image plus link, saved web archive layout, and inert email
  attachment summary.
- Added `fixtures/email-web-archive-manifest.tsv` combining the new fixtures
  with existing embedded-file, file-attachment annotation, and portfolio policy
  baselines.
- Registered the new fixtures in `fixtures/corpus-manifest.tsv`.
- Added native tests for rendering, attachment policy metadata, and bounded
  parallel sampling of the multi-page thread.

Validation:

- `cargo test -p pdfrust-native email -- --nocapture`
- Email/web-archive native support gate: 7 total, 7 native rendered, 0
  fallbacks, 0 errors.
- Email/web-archive benchmark: 7 total, 7 native rendered, 0 fallbacks, 0
  errors, 0 budget failures.
- Low-memory email-thread benchmark: 1 total, 1 native rendered, 0 fallbacks, 0
  errors, 0 budget failures.
- Maintainer visual comparison: 7 total, 1 exact, 2 accepted drift, 4 blockers,
  0 native errors, 0 PDFium errors.
