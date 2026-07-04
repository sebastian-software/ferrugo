# Project Audit and Next-Steps Plan, 2026-07-04

Status: accepted planning report.
Scope: full-project audit across five dimensions — PDFium coupling, benchmark
infrastructure, renderer performance, API/binding readiness, and
quality/conformance — with the resulting issue plan.

This report is the evidence base for the post-1.0 direction epic
([#118](https://github.com/sebastian-software/ferrugo/issues/118)) and the
issues [#96](https://github.com/sebastian-software/ferrugo/issues/96) through
[#117](https://github.com/sebastian-software/ferrugo/issues/117).

## Product direction confirmed by this audit

1. Be measurably faster than established renderers on the scoped preview
   workloads, proven by statistically sound, reproducible benchmarks.
2. Be better for real documents: close the highest-frequency real-world gaps
   and guard robustness in CI.
3. PDFium-free product: the binding crate leaves the workspace. PDFium may
   appear at most as an external process oracle in benchmark tooling. This
   supersedes the "retain maintainer tooling" position in ADR-0008 and the
   2026-06-29 comparison-tool removal decision; the prerequisites named there
   (native golden gate, multi-oracle records) have since landed.
4. Node-API bindings as the first non-Rust consumer surface, resolving the
   decision path in #73.

## Finding 1: PDFium is the only in-process oracle left

Poppler (`pdftoppm`) and Ghostscript (`gs`) are already invoked as external
child processes with graceful missing-tool handling. PDFium is the odd one
out: a 713-line hand-rolled dlopen FFI crate (`ferrugo-pdfium`, 88 `unsafe`
occurrences, published at 0.3.1) plus roughly 130 feature-gated references in
the CLI (`render-pdfium`, `render-worker`/`render-isolated`,
`compare-metadata`, `benchmark-pdfium`, `visual-diff`, and the
`MatrixBackend::Pdfium` hot/cold arms).

The existing quarantine already guarantees PDFium is neither a default
dependency nor a shipped asset; it does not push toward deletion, and
`ferrugo-simd` is not covered by it. Plan: externalize the oracle first
(#109), then remove the crate, feature, publish entry, and harden the
quarantine into a workspace-wide gate (#116). Open sub-decisions are recorded
in #116 (fate of `compare-metadata` and `render-isolated`).

## Finding 2: benchmark governance is mature, the numbers are not yet trustworthy

The plumbing and governance are unusually good: honest missing-tool caveats,
a written performance-claims policy, an oracle role matrix, and a golden
gate. The trust gaps concentrate in three places:

- **Statistics.** Default 3 iterations; with 3 samples the p95 resolves to
  the maximum. No stddev, coefficient of variation, or outlier handling
  exists anywhere in the harness (#96).
- **Corpus.** Every published number is measured on ferrugo's own generated
  fixtures — 11 single-page thumbnails in the performance matrix, no real
  producer output, no multi-page or high-DPI families (#97).
- **Reproducibility.** Peak RSS parsing assumes BSD `/usr/bin/time -l`
  (macOS-only), oracle versions are pinned in prose only, output
  normalization across renderers is checked post-hoc rather than enforced,
  and cold-process native times are dominated by a ~15 ms startup floor
  (#98). No benchmark runs in CI (#115).

The README performance snapshot is stale relative to the newest matrix
reports; #114 makes the README section generated from the promoted report so
the two cannot drift.

## Finding 3: the renderer closed the algorithmic gap; constant factors remain

The 2026-07-02 raster architecture gap analysis was acted on fast: analytic
coverage fills, stroke-to-fill, adaptive flattening, and banded replay all
landed, taking `vector-stress` from ~139 ms to ~1.8 ms. The old
distance-predicate stroke routes are `#[cfg(test)]` parity oracles now — and
the optimization backlog's hot-symbol list still names them, so fresh
profiles are a hard prerequisite for the next wave (#100).

The remaining distance to the reference engines is constant-factor work:

- `ferrugo-simd` is scalar-only with per-channel `f64` blend math; the span
  blitters that gate SIMD adoption now exist, so the kernels can be
  vectorized with integer lowp math and runtime dispatch (#110).
- Antialiased partial-coverage edge pixels still blend per pixel; on thin
  linework nearly every covered pixel is partial coverage (#113).
- Font faces are re-parsed per glyph outline extraction, and the glyph
  raster cache is 256 entries, fallback-fonts-only, per-pass — text-heavy and
  multi-page workloads re-do work PDFium caches document-wide (#111).
- Parallel banding is proven (~3-4x on large pages) but opt-in; the Type3
  cache boundary and a memory-bounded worker policy are what keep it from
  being the default (#112).

## Finding 4: the facade is binding-ready except for three blockers

`NativeBackend` is `Copy + Send + Sync`, outputs are owned buffers, inputs
are borrowed slices, and the error taxonomy is stable and string-addressable
— a clean shape for napi-rs. Three blockers stand in the way:

1. PNG encoding lives only in the CLI while `OutputFormat::Png` is public
   facade API no backend honors (#106).
2. `ThumbnailOptions.timeout` is never read in the library path; timeouts
   only exist in the CLI's PDFium subprocess supervisor. Single-page renders
   cannot be cancelled (#107).
3. `ferrugo-native` re-exports renderer telemetry types that would be
   semver-locked at 1.0; docs enforcement (`missing_docs`, docs.rs metadata)
   and panic-lint protection are absent (#108).

With those closed, #117 defines the Node package: `AsyncTask` rendering on
the stateless backend, typed error mapping, TypeScript definitions, and
prebuilds. `NativeDocumentSession` stays out of the binding surface (RefCell
caches, `!Sync`).

## Finding 5: robustness is strong, the process around it is not

The good news: a brace-aware scan finds only 13 guarded production panic
sites across the workspace, zero `unsafe` in the native crates, real
overflow/budget discipline, and a minimal dependency set. The gaps are
process-level:

- **No PR CI at all.** Only `publish.yml` exists; fmt/clippy/tests/gates run
  on maintainer discipline alone (#99).
- **Fuzzing is a deterministic smoke test**, not coverage-guided fuzzing:
  fixed seeds, hardcoded mutations, 4096-byte cap, 4 adversarial fixtures
  (#104).
- **The golden gate is 5 hash-only self-baselines** with the count
  hard-coded; correctness against reference renderers is maintainer-local
  and perceptually naive (#105).

Real-world conformance gaps, ranked by expected frequency in preview
workloads: permissions-only encryption is rejected outright — the single
most common real-world failure class (#101); `LZWDecode`/`RunLengthDecode`
are absent entirely (#102); CCITT (then JBIG2/JPX later) force scan-family
fallbacks (#103).

## Issue map

| Phase | Issues |
| --- | --- |
| 1. Foundations | #99 PR CI, #96 benchmark statistics, #100 fresh profiles |
| 2. PDFium exit | #109 external oracle, #116 binding removal |
| 3. Benchmark credibility | #97 corpus, #98 fairness/reproducibility, #115 scheduled CI, #114 README pipeline |
| 4. Performance queue | #110 SIMD kernels, #113 edge-pixel spans, #111 font/glyph caches, #112 parallel banding default |
| 5. Coverage and robustness | #101 encryption, #102 LZW/RunLength, #103 CCITT, #104 fuzzing, #105 golden/SSIM |
| 6. API and Node | #106 PNG in library, #107 timeout/cancel, #108 API hygiene, #117 Node binding |

Ordering rules live in the epic (#118). The audit evidence in this report
should be re-checked against fresh profiles (#100) before the Phase 4 queue
is executed.
