# Performance Optimization Working Plan

Status: active working document.
Date: 2026-06-29.

## Purpose

Ferrugo now has a first renderer performance matrix. This document is the
editable work board for turning that measurement infrastructure into real
renderer speed and memory improvements.

This is not a claim document. It should change as new benchmark reports,
profiles, and regressions teach us where the real bottlenecks are.

## Current State

- [x] `benchmark-matrix` exists for `native`, `pdfium`, and `poppler`.
- [x] `cold-process` records startup-inclusive wall time, exit status, output
  size, output dimensions, and RSS fields when the host can expose them.
- [x] `hot-render` records warmup plus measured distributions for in-process
  Ferrugo and PDFium runs.
- [x] Poppler is represented as an external-process reference and marked
  `not-applicable` for hot-render.
- [x] The starter manifest covers `small-text`, `office-export`, `scan`,
  `browser-print`, `form`, `presentation`, `report/vector`, and
  `mixed-layout`.
- [ ] No renderer optimization has happened from this plan yet.

## Operating Rules

- [ ] Optimize only after a matrix run and a profile identify the bottleneck.
- [ ] Use release builds for performance decisions. Dev-build timings are smoke
  checks only.
- [ ] Compare before and after on the same machine, same fixtures, same
  `max_edge`, same backend set, and same iteration count.
- [ ] Keep optimization PRs small enough to explain with one primary profile
  finding.
- [ ] Accept a block only with at least 10% improvement on target fixtures or a
  clear memory reduction, with no new fallback or visual-regression evidence.
- [ ] Do not update public README performance claims until at least two stable
  matrix runs agree.

## Phase 0: Baseline Hardening

Goal: make the first optimization target defensible.

- [ ] Add or document a release-mode path for `scripts/generate_performance_matrix.sh`.
- [ ] Run the full starter matrix in release mode with `native + poppler`.
- [ ] Run the same matrix with PDFium once `FERRUGO_PDFIUM_LIBRARY` is available.
- [ ] Store baseline artifacts under `target/performance-matrix-baseline-*`.
- [ ] Record host details: OS, CPU, Rust version, Poppler path, PDFium path, and
  whether RSS was available.
- [ ] Run the matrix twice and compare rank stability for the top 10 Ferrugo
  fixtures.
- [ ] Decide whether Poppler outliers on this host are useful timing references
  or only functional reference rows.

Suggested command:

```sh
cargo run -p ferrugo-cli --release --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge 160 \
  --iterations 5 \
  --warmup 1 \
  --timeout 60 \
  --output target/performance-matrix-baseline-release.json \
  --report target/performance-matrix-baseline-release.md \
  --artifact-dir target/performance-matrix-baseline-artifacts
```

PDFium variant:

```sh
FERRUGO_PDFIUM_LIBRARY=/path/to/libpdfium.dylib \
DYLD_LIBRARY_PATH=/path/to/pdfium/lib \
cargo run -p ferrugo-cli --release --features pdfium -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge 160 \
  --iterations 5 \
  --warmup 1 \
  --timeout 60 \
  --output target/performance-matrix-baseline-pdfium-release.json \
  --report target/performance-matrix-baseline-pdfium-release.md \
  --artifact-dir target/performance-matrix-baseline-pdfium-artifacts
```

## Phase 1: Renderer Timing Attribution

Goal: split Ferrugo time into phases before changing hot paths.

- [ ] Add opt-in native timing attribution for:
  - load, xref, object graph, and page tree;
  - stream decode;
  - content tokenization;
  - display-list build;
  - resource decode;
  - raster paths;
  - raster text;
  - raster images;
  - PNG/output encoding.
- [ ] Include attribution in a machine-readable report without leaking PDF bytes
  or rendered pixels.
- [ ] Add focused tests for phase-field presence and volatile-field redaction.
- [ ] Run attribution on the top 5 Ferrugo slow fixtures from the matrix.
- [ ] Pick the first optimization block from attribution, not from assumptions.

Likely implementation shape:

- reuse or extend `trace-native` for per-render phase timings;
- keep attribution disabled by default;
- avoid global state and keep measurements request-local.

## Phase 2: Vector And Report Hot Paths

Goal: improve the expected first bottleneck family without changing semantics.

Candidate fixtures:

- `fixtures/generated/vector-stress.pdf`
- `fixtures/generated/technical-hatch-clipping.pdf`
- `fixtures/generated/technical-linework-dimensions.pdf`
- `fixtures/generated/prepress-trim-bleed-marks.pdf`
- `fixtures/generated/technical-large-coordinate-plan.pdf`

Work items:

- [ ] Profile top vector fixtures with `sample`, Instruments, or Samply.
- [ ] Confirm whether time is in display-list construction, path flattening,
  clipping, stroke rasterization, or pixel loops.
- [ ] Add device-bounds culling before expensive raster work.
- [ ] Add fast paths for axis-aligned filled rectangles.
- [ ] Add fast paths for axis-aligned hairlines and simple strokes.
- [ ] Flatten reusable paths once per display item instead of per raster pass.
- [ ] Apply clip/intersection checks before entering expensive pixel loops.
- [ ] Add regression fixtures or targeted tests around clipping and hairline
  correctness.
- [ ] Re-run matrix before and after each block.

Acceptance:

- [ ] At least 10% improvement on the selected vector fixtures or a documented
  reason why the profile disproved the candidate.
- [ ] No new fallback categories.
- [ ] No unacceptable Poppler/PDFium visual drift on the touched fixture set.

## Phase 3: Allocation And Clone Audit

Goal: reduce avoidable work in hot paths after phase attribution exposes where
allocations matter.

- [ ] Run Clippy with perf lints as part of the normal all-target/all-feature
  gate.
- [ ] Review hotpath `Vec` creation and growth.
- [ ] Review `String`, `PathBuf`, and large enum clones inside loops.
- [ ] Remove intermediate `.collect()` calls where the consumer can stream.
- [ ] Inspect large enum variants if profiles show copy pressure.
- [ ] Add before/after allocation evidence where tooling is available.

Acceptance:

- [ ] Matrix timing improves or memory high-water drops on a target fixture set.
- [ ] Code remains simpler or equally readable; no clever allocation trick
  without a measured win.

## Phase 4: Image And Scan Track

Goal: make scan/image-heavy documents fast without increasing peak memory.

- [ ] Identify image-heavy fixtures from matrix and existing image reports.
- [ ] Profile decode, color conversion, alpha/soft-mask work, and output encode.
- [ ] Add downsample-aware decode where the source image is much larger than the
  target raster.
- [ ] Avoid full RGBA expansion when the target raster is smaller and direct
  sampling is possible.
- [ ] Reuse SoftMask/alpha scratch buffers within a render request.
- [ ] Investigate cropped decode when the CTM/clip excludes large image areas.

Acceptance:

- [ ] Clear time or memory reduction on scan/image fixtures.
- [ ] No regression on masks, ICC conversions, predictor images, or transparent
  image fixtures.

## Phase 5: Session Cache, But Bounded

Goal: improve batch and multi-page workloads without introducing hidden global
state.

- [ ] Keep global caches out of the renderer path.
- [ ] Define an explicit request/session cache object for batch or multi-page
  rendering.
- [ ] Budget cache entries by bytes and item count.
- [ ] Cache parsed document/page tree data only inside the request/session.
- [ ] Cache decoded shared resources only when identity and budget are clear.
- [ ] Make cache use visible in benchmark output.

Acceptance:

- [ ] Repeat/batch benchmark shows improvement.
- [ ] Low-memory profile remains bounded.
- [ ] Cache invalidation is tied to document identity and render options.

## Phase 6: Benchmark Gates And Claims

Goal: turn stable evidence into guardrails, not premature marketing.

- [ ] Promote stable fixture subsets into budgeted CI gates only after variance
  is understood.
- [ ] Keep the full matrix as a local maintainer tool until tool availability is
  reliable on CI.
- [ ] Add a "performance claim update" checklist before changing README copy.
- [ ] Keep MuPDF as v2 comparison backlog, not a blocker for the first
  optimization wave.

Claim checklist:

- [ ] Two stable matrix runs.
- [ ] Same host or clearly documented host differences.
- [ ] Reference renderer versions recorded.
- [ ] No known host/tool timeout artifact driving the conclusion.
- [ ] Result phrased by workload family, not as broad renderer parity.

## Current Best Guess

The first optimization block should probably be vector/report rendering, but
that remains a hypothesis until the release matrix and top-fixture profiles are
captured. The most likely high-value candidates are:

1. device-bounds culling before raster work;
2. simple rect and hairline fast paths;
3. flatten-once path reuse;
4. clip-before-loop checks.

If profiling points elsewhere, this section should be edited before code
changes start.

## Open Questions

- [ ] Where should the local PDFium dylib live for repeatable maintainer runs?
- [ ] Should `scripts/generate_performance_matrix.sh` default to release mode or
  keep smoke mode as default and expose `PROFILE=release`?
- [ ] Which profiler should be the default documented path on macOS: `sample`,
  Instruments, or Samply?
- [ ] Should the matrix report gain explicit "host timing reliability" flags for
  Poppler or RSS availability?
