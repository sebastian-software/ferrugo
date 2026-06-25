# 0168: Email Client And Web Archive PDF Coverage

Status: todo
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

Empty until done.
