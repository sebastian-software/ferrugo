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

These buckets are not API classes. They make support matrices and corpus
reports stable without forcing downstream callers to depend on milestone-level
implementation details.
