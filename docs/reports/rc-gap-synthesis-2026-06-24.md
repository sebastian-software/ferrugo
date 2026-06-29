# RC Gap Synthesis 2026-06-24

Milestone: 0081.
Source gate: 0080 native renderer release candidate report.

This backlog is ordered by evidence from the 0080 generated-corpus run. It is
not a broad PDF feature wish list.

## Current RC Blockers

| Priority | Gap | Category | Subsystem owner | Type | PDFium role | Acceptance gate |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | Optional-content membership policy fallback in `optional-content-ocmd.pdf` | presentation | native optional content / resource policy | correctness | required fallback | Native renders the fixture without `graphics.optional-content` fallback and the presentation family reaches 4/4 native renders |
| 2 | No automated full-corpus visual diff | all visual categories | visual diff tooling / review workflow | correctness / rollout | oracle baseline | A committed workflow compares native and PDFium rasters by family with thresholds and review artifacts |
| 3 | `vector-stress.pdf` smoke render-time violation | report | path rasterization / vector hot path | performance | benchmark oracle | Native smoke benchmark has no non-policy `render_time` failure at `--max-edge 160 --max-ms 1000` |
| 4 | Text fidelity remains category-dependent | office-export, browser-print, report | text rasterizer / font shaping | correctness | quality fallback | Visual diff or reviewed fixtures classify text-heavy families as rendered/degraded with explicit thresholds |
| 5 | Generated corpus is not enough real-world evidence | all target categories | corpus ingestion | rollout | oracle baseline | Real-world corpus manifest covers office, browser, report, form, scan, presentation, encrypted, and malformed categories |

The encrypted fixture is not a release blocker. It remains an expected error
policy case until password/security support is explicitly scoped.

## Backlog By Work Type

Correctness:

- Implement or explicitly downgrade optional-content membership policy handling.
- Add visual-diff review artifacts before any broad production-primary claim.
- Classify text-heavy output quality with thresholds, not only render success.

Performance:

- Profile `vector-stress.pdf` and separate parser/display-list time from path
  flattening/rasterization time.
- Add a non-flaky benchmark budget after the hot path is understood.

Memory:

- No new memory blocker was observed in 0080.
- Keep deterministic raster/image/font budgets as the primary guard until
  larger real-world documents are ingested.

Packaging/API:

- 0079 completed the optional PDFium feature split.
- No API blocker blocks the next evidence wave; default native-only packaging
  should remain gated by corpus evidence.

Rollout:

- Native-first remains acceptable only for categories that render without
  fallback and have reviewed fidelity expectations.
- Broad primary status waits for visual diffing and real-world corpus coverage.

## PDFium Retirement Order

1. Keep PDFium as oracle for all visual and metadata comparisons.
2. Remove PDFium fallback from categories that have 100% native render pass,
   reviewed visual output, and stable benchmark behavior.
3. Keep PDFium fallback for presentation until `optional-content-ocmd.pdf`
   renders natively or is explicitly classified as unsupported outside the
   supported release surface.
4. Keep PDFium fallback for fidelity-sensitive text-heavy documents until visual
   diff results classify acceptable degradation.
5. Only run a fallback removal drill after real-world corpus ingestion and
   visual-diff workflow are in place.

## Next Milestone Mapping

| Milestone | Why it is next |
| --- | --- |
| 0082 Native Default API And CLI Stabilization | Keep native-first behavior explicit while RC blockers remain |
| 0083 Real-World Corpus Ingestion And Classification | Replace generated-corpus-only evidence with real document coverage |
| 0084 Visual Diff Dashboard And Review Workflow | Provide the missing visual fidelity gate |
| 0096 Hot Path Profiling And Raster Optimization | Use the `vector-stress.pdf` benchmark failure to guide optimization |
| 0099 PDFium Fallback Removal Drill | Defer until 0083/0084 evidence exists |

## Validation

Reviewed `docs/reports/native-renderer-rc-gate-2026-06-24.md` and reran the
0080 corpus summary command:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/rc-0080-fallback-summary.json
```

Current generated-corpus evidence:

- 52 fixtures total.
- 50 native rendered.
- 1 expected encrypted error.
- 1 native fallback, `graphics.optional-content`, in the presentation family.
