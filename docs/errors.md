# Thumbnail Error Taxonomy

Status: accepted Phase 0 taxonomy.
Date: 2026-06-24.

The public thumbnail facade exposes five stable error classes. The consumer SLA
for these classes is documented in `docs/policies/unsupported-feature-sla.md`.

| Class | Meaning | PDFium mapping |
| --- | --- | --- |
| `encrypted` | Password-protected or security-restricted PDF. | `FPDF_ERR_PASSWORD`, `FPDF_ERR_SECURITY` |
| `malformed` | File cannot be read as a valid PDF. | `FPDF_ERR_FILE`, `FPDF_ERR_FORMAT`, local file-read failure |
| `unsupported` | Valid input or request cannot be handled by the current backend. | `FPDF_ERR_PAGE` for unavailable page operations; native unsupported feature probes |
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

Native renderer diagnostics can attach a stable feature bucket while preserving
the public `unsupported` class. The bucket constants are exposed by
`ferrugo_thumbnail::unsupported_feature_buckets`, and the full stable set is
available as `ferrugo_thumbnail::STABLE_UNSUPPORTED_FEATURE_BUCKETS`.

| Bucket | Meaning | Typical owner |
| --- | --- | --- |
| `native.unsupported` | Valid input uses an unsupported native renderer feature without a more specific bucket. | triage |
| `renderer.memory-budget` | Rendering exceeded a configured native memory, pixel, or cache budget and should not be treated as an internal crash. | 0058-0059 |
| `renderer.form-xobject-composition` | Page requires nested Form XObject items in the combined render order. | Form integration pull-forward |
| `graphics.optional-content` | Optional content uses unsupported membership, usage application, intent, or viewer-state policy. | 0054, 0192 |
| `graphics.color-management` | Page uses unsupported color-management behavior. | 0075, 0208 |
| `graphics.pattern-shading` | Page requires unsupported patterns or shadings. | 0050 |
| `graphics.stroke-clip` | Page depends on stroke joins, caps, dashes, or clipping fidelity. | 0051 |
| `graphics.transparency` | Page requires unsupported soft-mask forms, transparency groups, blend, or overprint behavior. | 0048-0049, 0213 |
| `annotation.appearance` | Annotation appearance is missing, malformed, dynamically generated, or requires unsupported form/interaction behavior. | 0052, 0193 |
| `image.color-space` | Image or fill uses unsupported color-space conversion or decode arrays. | 0046 |
| `image.filter` | Stream uses unsupported image codecs, filters, or predictors. | 0047, 0209 |
| `form.xfa-dynamic` | Document declares XFA without static AcroForm fields or appearances, requiring an XFA runtime to render faithfully. | 0111 |
| `form.appearance-mutation` | Caller requested viewer-side AcroForm value or appearance-state mutation that thumbnail rendering must not synthesize silently. | 0194 |
| `text.cmap-tounicode` | Page requires unsupported CMap or ToUnicode character-code mapping. | 0043 |
| `text.font-program` | Page requires unsupported font program, encoding, or fallback behavior. | 0042, 0203 |
| `text.glyph-outline` | Page requires unsupported glyph outline extraction. | 0044 |

These buckets are stable diagnostics, not separate high-level API classes. Code
that only needs retry/fallback routing should branch on
`ThumbnailError::class()`. Code that needs user-facing support categories,
telemetry, or feature-specific fallback can additionally read
`ThumbnailError::unsupported_feature_bucket()`.

Native runtime rendering preserves the public `unsupported` class and includes
the bucket in the error message when one is available. Generic unsupported
outcomes use `native.unsupported` until a narrower bucket is available. Corpus
commands such as `summarize-fallbacks --fail-on-fallback` turn these diagnostics
into local or CI failure gates without loading PDFium.

## Consumer Handling

Recommended application behavior:

| Error class | Suggested handling |
| --- | --- |
| `unsupported` with bucket | Treat as a valid PDF that needs an unsupported renderer feature. Use the bucket for telemetry, support copy, or an alternate renderer path. |
| `unsupported` without bucket | Treat as valid but unsupported; avoid parsing the display message for control flow. |
| `malformed` | Treat as invalid or unrecoverable within the parser recovery budget. Do not retry as an unsupported feature. |
| `encrypted` | Ask for password/decryption policy; do not treat as malformed or unsupported. |
| `timeout` | Retry only under an explicit timeout or isolation policy. |
| `internal` | Treat as a bug or environment failure; diagnostic text is not stable. |

See `docs/guides/native-only-consumer-migration.md` for a public API routing
example that does not inspect internal renderer state.
