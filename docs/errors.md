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
