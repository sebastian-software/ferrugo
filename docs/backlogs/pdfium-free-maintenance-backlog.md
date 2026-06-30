# PDFium-Free Maintenance Backlog

Status: accepted for 0120.
Date: 2026-06-25.

This backlog separates normal native-only operation from maintainer-only PDFium
comparison infrastructure. Each deletion item should be small and reversible.

## Keep

| Area | Decision | Reason |
| --- | --- | --- |
| `ferrugo-pdfium` crate | Keep as optional maintainer tooling. | It is the current local oracle for metadata, PDFium benchmarks, and visual diffs. |
| `render-pdfium` / `render-isolated` | Keep behind `--features pdfium`. | Maintainers still need direct oracle renders and process-isolated probes. |
| `compare-metadata` | Keep behind `--features pdfium`. | Native metadata expansion still needs an oracle for page-count and page-size parity. |
| `benchmark-pdfium` | Keep behind `--features pdfium`. | Performance regression work needs a reference backend. |
| `visual-diff` | Keep behind `--features pdfium`. | Pixel-diff blocker triage currently depends on a PDFium oracle. |
| PDFium build docs and measurements | Keep as historical and maintainer setup docs. | They make oracle runs reproducible without bundling PDFium. |

## Delete Candidates

| Candidate | Earliest action | Risk | Rollback |
| --- | --- | --- | --- |
| Production docs that suggest PDFium fallback for supported families | 0120 complete | Low: docs-only cleanup. | Restore wording from this backlog/report. |
| Any default-feature PDFium dependency edge | Immediately if found | High if missed, because it would reintroduce runtime packaging baggage. | Revert the dependency or feature change. |

## Deleted From Runtime Paths

| Item | Removed in | Rollback |
| --- | --- | --- |
| `render` / `render-auto --allow-pdfium-fallback` runtime retry | 0141 | Reintroduce the fallback branch in `render_auto_thumbnail`. |
| `FERRUGO_ALLOW_PDFIUM_FALLBACK` environment runtime opt-in | 0141 | Restore env parsing and fallback policy state. |
| `FERRUGO_DENY_FALLBACK_REASONS` targeted runtime denial | 0141 | Restore fallback policy parsing if runtime fallback returns. |
| Direct `render-worker` CLI invocation | 0142 | Keep the private entry point guarded by `FERRUGO_PDFIUM_RENDER_WORKER`; direct invocation now fails with a usage error. |

## Deferred Until Native Coverage Lands

| Area | Blocker | Required evidence before deletion |
| --- | --- | --- |
| Optional-content PDFium fallback | `graphics.optional-content` / OCMD gaps. | Optional-content membership policy renders natively or returns accepted typed unsupported outcomes for out-of-scope cases. |
| Specialized image codec fallback | `image.filter` gaps for CCITT, JBIG2, JPX. | Codec policy either implements pure-Rust support or stable unsupported handling for target document families. |
| Pattern/mesh fallback | `graphics.pattern-shading` gaps. | Mesh shading and pattern fixtures no longer require PDFium for target families. |
| Form/XFA fallback probes | Dynamic XFA and appearance fidelity gaps. | Static/dynamic form policy has native output or explicit non-rendering unsupported classification. |

## Native-Only Release Rule

Normal supported-document rendering must use:

- default features or `--no-default-features`
- `render`, `render-auto`, or `render-native`
- supported-family fallback gates with `--fail-on-fallback`

Maintainer PDFium commands must be isolated in explicit `--features pdfium`
jobs and must not be required for normal package installation, deployment, or
native-only smoke tests.
