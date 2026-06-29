# Unsupported Feature SLA

Status: accepted for 0219.
Date: 2026-06-29.

The native renderer treats unsupported PDF features as stable, typed consumer
outcomes. Unsupported does not mean malformed input, crash, retryable timeout,
or permission failure. It means the input is valid enough to inspect, but the
current native renderer does not yet promise faithful output for a required
feature.

## Public Contract

Consumers must route first by `ThumbnailError::class()`:

| Class | SLA | Retry guidance |
| --- | --- | --- |
| `unsupported` | Valid input or request outside the supported native feature set. | Do not retry through hidden PDFium fallback. Use the bucket for support, telemetry, alternate processing, or backlog routing. |
| `malformed` | Invalid or unrecoverable PDF within the parser recovery budget. | Do not classify as unsupported. Ask the producer for a repaired file or run an explicit repair workflow outside the renderer. |
| `encrypted` | Password-protected or security-restricted input. | Ask for an explicit password/decryption policy. |
| `timeout` | Rendering exceeded caller policy. | Retry only under a deliberate timeout/isolation policy. |
| `internal` | Renderer bug or environment failure. | Treat as a defect; message text is diagnostic and not a stable API. |

When the class is `unsupported`, consumers may additionally read
`ThumbnailError::unsupported_feature_bucket()`. Bucket names are stable
diagnostics exposed by `pdfrust_thumbnail::unsupported_feature_buckets` and
listed in `pdfrust_thumbnail::STABLE_UNSUPPORTED_FEATURE_BUCKETS`.

Do not parse `Display` messages for control flow. Message text can improve
without a semver-major API change as long as class and bucket behavior remains
stable.

## Stable Buckets

| Bucket | SLA category | Release impact |
| --- | --- | --- |
| `native.unsupported` | Generic unsupported native boundary. | Backlog candidate until narrowed. |
| `renderer.memory-budget` | Caller or renderer budget prevented a bounded render. | Release blocker only when supported server profiles cannot fit documented budgets. |
| `renderer.form-xobject-composition` | Unsupported nested Form XObject composition. | Blocker for workflows that require that form composition class. |
| `graphics.optional-content` | Unsupported optional-content membership, usage, intent, or viewer-state policy. | Documented limit unless a target claim includes interactive layer fidelity. |
| `graphics.color-management` | Unsupported color-management behavior. | Blocker for print/archival/color-critical claims; otherwise documented limit. |
| `graphics.pattern-shading` | Unsupported pattern or shading behavior. | Blocker for chart/vector claims when observed in target corpus. |
| `graphics.stroke-clip` | Unsupported stroke, dash, or clipping fidelity. | Blocker for technical drawing or vector-fidelity claims. |
| `graphics.transparency` | Unsupported soft-mask, blend, transparency, or overprint behavior. | Blocker for report/dashboard visual-fidelity claims when observed. |
| `annotation.appearance` | Unsupported or unsafe annotation appearance synthesis. | Documented limit unless annotation visual fidelity is in scope. |
| `image.color-space` | Unsupported image color-space conversion or decode behavior. | Blocker for image-heavy or print-preview claims when observed. |
| `image.filter` | Unsupported image codec, filter, or predictor. | Blocker for scan/fax/archive claims when observed. |
| `form.xfa-dynamic` | Dynamic XFA runtime required for faithful rendering. | Accepted deferral; route to producer migration or external XFA handling. |
| `form.appearance-mutation` | Caller requested unsupported form value or appearance-state mutation. | Documented request boundary; use explicit form-flattening workflows. |
| `text.cmap-tounicode` | Unsupported CMap or ToUnicode mapping behavior. | Blocker for text-heavy claims when observed in target corpus. |
| `text.font-program` | Unsupported font program, encoding, or fallback behavior. | Blocker for office-export claims when visible text is affected. |
| `text.glyph-outline` | Unsupported glyph outline behavior. | Blocker for font-fidelity claims when observed. |

## Outcome Categories

- Successful: native render or metadata extraction completes within documented
  profile budgets.
- Degraded: native render succeeds but a documented policy says fidelity is
  approximate for that feature class.
- Unsupported: the renderer returns `ThumbnailErrorClass::Unsupported`, usually
  with a stable bucket.
- Failed: malformed, encrypted, timeout, or internal outcomes. These are not
  unsupported feature backlog items unless a later triage converts them to a
  typed unsupported bucket.

## Release Blocking Rules

Unsupported buckets block a release claim only when all of the following are
true:

1. The bucket appears in a supported target family or documented deployment
   profile for that release.
2. The feature affects visible output, required metadata, or bounded server
   operation for that claim.
3. The issue is not already documented as an accepted profile limitation.

Buckets observed only in secondary profiles such as WASM, mobile, or embedded
low-memory runs do not block server-side PDFium replacement unless the same
class exposes shared renderer correctness, safety, or unbounded-resource
behavior.

## Consumer Reporting

Telemetry and support reports may include:

- high-level class;
- unsupported bucket when present;
- renderer version;
- native profile;
- page index and bounded dimensions;
- fixture family or producer classification when known.

Reports must not include raw document bytes, extracted private text, or
absolute user paths. See `docs/reports/native-renderer-telemetry-privacy-2026-06-29.md`.

## Related Guidance

- `docs/errors.md`
- `docs/guides/native-only-consumer-migration.md`
- `docs/policies/native-renderer-api-semver.md`
- `docs/policies/native-conformance-triage.md`
