# Telemetry And Diagnostics Privacy

Status: accepted for 0198.
Date: 2026-06-29.

Renderer diagnostics are opt-in artifacts controlled by the embedding
application or maintainer CLI command. `pdfrust` does not collect hosted
telemetry, does not enable background upload, and must not require diagnostics
for successful rendering.

## Field Classes

| Class | Examples | Sharing Rule |
| --- | --- | --- |
| Safe | error class, unsupported bucket, stage hint, page count, page dimensions, timings, memory limits, selected render options | May be attached to public issues when no surrounding path or manifest data identifies a private document. |
| Sensitive | input path, manifest source, manifest license, manifest notes, producer IDs, local fixture IDs | Review before sharing; redact for private, customer, local-only, or reference-only inputs. |
| Local-only | PDF bytes, content streams, operands, text snippets, image samples, rendered pixels, document-info fields, document hashes | Do not include in default artifacts or public reports. |
| Experimental | newly proposed diagnostic fields, coarse hashes, sampled layout snapshots, allocator/RSS telemetry | Must be reviewed and classified before becoming part of a shareable artifact. |

## Default Bundle Rules

Native diagnostic bundles may include:

- normalized fixture ID or redacted local-only ID;
- safe render options;
- safe page count and page-size metadata;
- stage timing and coarse stage hints;
- high-level error class and stable unsupported bucket;
- native memory and cache limits.

Native diagnostic bundles must not include:

- PDF bytes;
- rendered pixel buffers;
- extracted text or text snippets;
- raw content stream operands;
- image samples;
- title, author, subject, keywords, or other document-info fields;
- private paths or private manifest notes.

Private or local-only manifest entries must use a synthetic `local-only-NNNN`
identifier and redact manifest details before the artifact leaves the document
trust boundary.

## Telemetry Control

Telemetry is application-controlled. Libraries and CLI commands may produce
local JSON artifacts only when requested by the caller. Applications that build
upload or aggregation workflows must:

- opt in explicitly;
- document retention and access controls;
- apply this field classification before upload;
- keep raw documents, rendered pages, and text payloads out of telemetry by
  default.

## Issue Reporting Checklist

Before attaching a diagnostic artifact to a public issue:

1. Confirm `privacy.includes_pdf_bytes`, `privacy.includes_rendered_pixels`,
   `privacy.includes_document_info`, `privacy.includes_text_samples`, and
   `privacy.includes_private_paths` are all `false`.
2. Confirm private/local-only inputs use a redacted fixture ID.
3. Remove manifest notes, source details, or producer IDs that identify a
   customer, user, contract, account, or internal system.
4. Prefer a reduced synthetic fixture when the issue requires visual proof or
   document content.

Maintainer-only traces follow the same boundary. They may record operator names
and aggregate counts, but not operands, stream bytes, text strings, image
samples, or rendered pixels.
