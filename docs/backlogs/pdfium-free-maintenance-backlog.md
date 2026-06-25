# PDFium-Free Maintenance Backlog

Status: accepted for 0120.
Date: 2026-06-25.

This backlog separates normal native-only operation from maintainer-only PDFium
comparison infrastructure. Each deletion item should be small and reversible.

## Keep

| Area | Decision | Reason |
| --- | --- | --- |
| `pdfrust-pdfium` crate | Keep as optional maintainer tooling. | It is the current local oracle for metadata, PDFium benchmarks, and visual diffs. |
| `render-pdfium` / `render-isolated` | Keep behind `--features pdfium`. | Maintainers still need direct oracle renders and process-isolated probes. |
| `compare-metadata` | Keep behind `--features pdfium`. | Native metadata expansion still needs an oracle for page-count and page-size parity. |
| `benchmark-pdfium` | Keep behind `--features pdfium`. | Performance regression work needs a reference backend. |
| `visual-diff` | Keep behind `--features pdfium`. | Pixel-diff blocker triage currently depends on a PDFium oracle. |
| PDFium build docs and measurements | Keep as historical and maintainer setup docs. | They make oracle runs reproducible without bundling PDFium. |

## Delete Candidates

| Candidate | Earliest milestone | Risk | Rollback |
| --- | --- | --- | --- |
| `render-worker` alias for direct PDFium rendering | 0142 | Low: scripts may still use the old alias. | Re-add the alias to the CLI match arm. |
| `PDFRUST_ALLOW_PDFIUM_FALLBACK` environment fallback opt-in | 0142 or later | Medium: local maintainer sweeps may rely on env-driven fallback. | Restore env parsing in `FallbackPolicy::default()`. |
| Production docs that suggest PDFium fallback for supported families | 0120 complete | Low: docs-only cleanup. | Restore wording from this backlog/report. |
| Any default-feature PDFium dependency edge | Immediately if found | High if missed, because it would reintroduce runtime packaging baggage. | Revert the dependency or feature change. |

## Deferred Until Native Coverage Lands

| Area | Blocker | Required evidence before deletion |
| --- | --- | --- |
| `render` / `render-auto --allow-pdfium-fallback` | Unsupported categories remain. | Full supported target surface has 0 native fallback buckets or a documented non-PDFium fallback strategy. |
| Optional-content PDFium fallback | `graphics.optional-content` / OCMD gaps. | Optional-content membership policy renders natively or returns accepted typed unsupported outcomes for out-of-scope cases. |
| Specialized image codec fallback | `image.filter` gaps for CCITT, JBIG2, JPX. | Codec policy either implements pure-Rust support or stable unsupported handling for target document families. |
| Pattern/mesh fallback | `graphics.pattern-shading` gaps. | Mesh shading and pattern fixtures no longer require PDFium for target families. |
| Form/XFA fallback probes | Dynamic XFA and appearance fidelity gaps. | Static/dynamic form policy has native output or explicit non-rendering unsupported classification. |

## Native-Only Release Rule

Normal supported-document rendering must use:

- default features or `--no-default-features`
- `render-native`, `render --native-only`, or `render-auto --native-only`
- supported-family fallback gates with `--fail-on-fallback`

Maintainer PDFium commands must be isolated in explicit `--features pdfium`
jobs and must not be required for normal package installation, deployment, or
native-only smoke tests.
