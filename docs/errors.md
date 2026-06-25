# Thumbnail Error Taxonomy

Status: accepted Phase 0 taxonomy.
Date: 2026-06-24.

The public thumbnail facade exposes five stable error classes:

| Class | Meaning | PDFium mapping |
| --- | --- | --- |
| `encrypted` | Password-protected or security-restricted PDF. | `FPDF_ERR_PASSWORD`, `FPDF_ERR_SECURITY` |
| `malformed` | File cannot be read as a valid PDF. | `FPDF_ERR_FILE`, `FPDF_ERR_FORMAT`, local file-read failure |
| `unsupported` | Valid input or request cannot be handled by the current backend. | `FPDF_ERR_PAGE` for unavailable page operations; future unsupported feature probes |
| `timeout` | Rendering exceeded the configured timeout. | Enforced by the isolated render parent; direct in-process PDFium calls cannot provide hard cancellation |
| `internal` | Backend, linkage, allocation, or unknown failure. | `FPDF_ERR_UNKNOWN`, `FPDF_ERR_SUCCESS` in an error path, unrecognized codes |

The CLI includes the class in render failures:

```text
render error [malformed]: PDF is malformed
```

PDFium exposes coarse error codes, so mappings are intentionally approximate.
The stable class is for callers and baselines; detailed backend diagnostics can
still include local context.

Direct in-process PDFium rendering cannot safely stop a running native call.
Hard timeout behavior is provided by the isolated render parent, which
terminates the worker process and returns the `timeout` class.

## Native Unsupported Feature Buckets

Native renderer diagnostics can attach a stable internal feature bucket while
preserving the public `unsupported` class:

| Bucket | Meaning | Typical owner |
| --- | --- | --- |
| `renderer.inline-image-stream` | Content stream uses `BI`/`ID`/`EI` inline image data. | image execution pull-forward |
| `renderer.form-xobject-composition` | Page requires nested Form XObject items in the combined render order. | Form integration pull-forward |
| `text.font-program` | Page requires Base14, Type1, TrueType, CFF, or embedded font programs. | 0042 |
| `text.cmap-tounicode` | Page requires CMap or ToUnicode character-code mapping. | 0043 |
| `text.glyph-outline` | Page requires real glyph outline extraction. | 0044 |
| `image.color-space` | Image or fill uses unsupported color-space conversion or decode arrays. | 0046 |
| `image.filter` | Stream uses unsupported image codecs or predictors. | 0047 |
| `graphics.transparency` | Page requires unsupported soft-mask forms, transparency groups, or blend modes. | 0048-0049 |
| `graphics.pattern-shading` | Page requires tiling patterns or shadings. | 0050 |
| `graphics.stroke-clip` | Page depends on stroke joins, caps, dashes, or clipping fidelity. | 0051 |
| `annotation.appearance` | Annotation appearance is missing, malformed, dynamically generated, or requires unsupported form/interaction behavior. | 0052 |
| `form.acroform` | AcroForm widget appearance is missing, dynamic, XFA-backed, script-backed, or requires validation/editing behavior. | 0053 |
| `form.xfa-dynamic` | Document declares XFA without static AcroForm fields or appearances, requiring an XFA runtime to render faithfully. | 0111 |
| `graphics.optional-content` | Optional content uses unsupported membership, usage application, intent, or viewer-state policy. | 0054 |
| `xref.incremental-hybrid` | Incremental or hybrid-reference structure is cyclic, over budget, malformed, or requires unsupported compressed hybrid entries. | 0055 |
| `security.encryption` | Document declares encryption metadata and cannot be interpreted without an explicit password/decryption policy. | 0056 |
| `parser.recovery` | Malformed structure requires bounded parser recovery, such as small xref object-offset drift. | 0057 |
| `renderer.memory-budget` | Rendering exceeded a configured native memory or cache budget and should not be treated as an internal crash. | 0058-0059 |

These buckets are not API classes. They make support matrices and corpus
reports stable without forcing downstream callers to depend on milestone-level
implementation details.

Native runtime rendering preserves the public `unsupported` class and includes
the bucket in the error message when one is available. Generic unsupported
outcomes use `native.unsupported` until a narrower bucket is available. Corpus
commands such as `summarize-fallbacks --fail-on-fallback` turn these diagnostics
into local or CI failure gates without loading PDFium.
