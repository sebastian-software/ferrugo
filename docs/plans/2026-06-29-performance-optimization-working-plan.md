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
- [ ] Treat changes below 5% as noise unless repeated runs prove otherwise.
- [ ] Repeat and inspect any 5-10% change before calling it meaningful.
- [ ] Add no performance dependency without profile evidence and a short
  "why std is not enough" note in the change.
- [ ] Keep `unsafe` out of renderer hot paths unless a safe API cannot express
  the operation, the block is isolated, and the safety invariant is documented.
- [ ] Do not update public README performance claims until at least two stable
  matrix runs agree.

## Acceptance Criteria

These criteria apply to every optimization block unless a narrower follow-up
document explicitly overrides them.

Baseline acceptance:

- [ ] Two release-mode matrix runs on the same host have comparable top-10
  Ferrugo fixture rankings.
- [ ] Report artifacts include backend versions/commands, OS, CPU, Rust
  version, fixture manifest, `max_edge`, iterations, warmup, timeout, and RSS
  availability.
- [ ] Missing PDFium is acceptable only when the report records `missing-tool`;
  PDFium is required before publishing comparison claims.
- [ ] Poppler timing is treated as a cold-process reference, not as an
  in-process renderer peer.

Optimization-block acceptance:

- [ ] The block targets one fixture family and one profile-backed bottleneck.
- [ ] Target fixtures improve by at least 10% in p95/wall time, or peak RSS /
  allocation volume drops by at least 10%.
- [ ] No new fallback bucket, error class, timeout, or crash appears on the
  focused fixture set.
- [ ] Visual output is reviewed against existing differential artifacts and,
  when available, against Poppler/PDFium reference renders.
- [ ] The change passes:
  `cargo fmt --all --check`,
  `cargo check --workspace --no-default-features`,
  `cargo test --workspace --no-default-features`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Before/after matrix artifacts are kept locally with enough naming context
  to revisit the result.

Dependency and hardware-aware acceptance:

- [ ] Prefer `std` and safe slice APIs first.
- [ ] Choose stack-inline data structures only after size histograms show that
  the inline capacity is correct for real fixtures.
- [ ] Keep scratch buffers request-local or session-local; no hidden global
  cache.
- [ ] Any SIMD, pointer-copy, arena, or thread-pool change keeps a simple scalar
  or safe fallback path unless the crate boundary makes that impossible.

## Phase 0: Baseline Hardening

Goal: make the first optimization target defensible.

- [x] Add or document a release-mode path for `scripts/generate_performance_matrix.sh`.
- [ ] Run the full starter matrix in release mode with `native + poppler`.
- [ ] Run the same matrix with PDFium once `FERRUGO_PDFIUM_LIBRARY` is available.
- [ ] Store baseline artifacts under `target/performance-matrix-baseline-*`.
- [ ] Record host details: OS, CPU, Rust version, Poppler path, PDFium path, and
  whether RSS was available.
- [ ] Run the matrix twice and compare rank stability for the top 10 Ferrugo
  fixtures.
- [x] Treat Poppler as a useful cold-process and visual reference. Do not use
  Poppler outliers as hard optimization targets until the report gains host
  timing reliability flags.

Suggested wrapper command:

```sh
OUTPUT=target/performance-matrix-baseline-release.json \
REPORT=target/performance-matrix-baseline-release.md \
ARTIFACT_DIR=target/performance-matrix-baseline-artifacts \
ITERATIONS=5 \
TIMEOUT=60 \
./scripts/generate_performance_matrix.sh
```

Smoke variant:

```sh
PROFILE=dev ./scripts/generate_performance_matrix.sh
```

Equivalent direct command:

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

PDFium path policy:

- keep local PDFium paths in `FERRUGO_PDFIUM_LIBRARY` and
  `DYLD_LIBRARY_PATH`;
- do not commit absolute maintainer paths;
- record the resolved command/path in the matrix report;
- keep native-only runs valid by marking PDFium as `missing-tool`.

## Phase 1: Renderer Timing Attribution

Goal: split Ferrugo time into phases before changing hot paths.

- [x] Add opt-in native timing attribution for:
  - load, xref, object graph, and page tree;
  - stream decode;
  - content tokenization;
  - display-list build;
  - resource decode;
  - raster paths;
  - raster text;
  - raster images;
  - PNG/output encoding.
- [x] Include attribution in a machine-readable report without leaking PDF bytes
  or rendered pixels.
- [x] Add focused tests for phase-field presence and volatile-field redaction.
- [ ] Run attribution on the top 5 Ferrugo slow fixtures from the matrix.
- [ ] Pick the first optimization block from attribution, not from assumptions.

Likely implementation shape:

- [x] reuse or extend `trace-native` for per-render phase timings;
- [x] keep attribution disabled by default;
- [x] avoid global state and keep measurements request-local.

Current caveat:

- `content_tokenize` only covers explicit token scans outside display-list
  construction. Builder-internal tokenization is still included in
  `display_list_build` until the builder APIs expose a cleaner split.

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

## Hardware-Aware Rust Notes

Goal: use Rust's memory model and the host CPU well without prematurely
outsmarting the compiler.

Default choices:

- Use `Vec<T>` for large or genuinely dynamic contiguous data. Prefer
  `with_capacity` when the upper bound is known, reuse buffers across phases,
  and avoid repeated grow/copy cycles inside pixel or path loops.
- Use slices in APIs: `&[T]`, `&mut [T]`, `&str`, and `&[u8]` keep ownership
  local and make hot code easier to profile.
- Prefer row-major, cache-friendly traversal for raster buffers. Keep inner
  loops branch-light and make clipping decisions before entering them.
- Prefer safe bulk-copy APIs such as `copy_from_slice`, `copy_within`,
  `clone_from_slice`, and `extend_from_slice`. LLVM can lower these to optimized
  `memcpy`/`memmove` patterns for the target CPU.

Candidate crates and when to consider them:

- [`smallvec`](https://docs.rs/smallvec/latest/smallvec/): consider for tiny,
  hot vectors such as path operands, short clip stacks, or compact text-state
  runs after histograms show most values fit inline. Pick the inline size from
  fixture data, not taste. Watch for larger stack frames and larger enum
  variants.
- [`arrayvec`](https://docs.rs/arrayvec/latest/arrayvec/): consider when a hard
  PDF or renderer limit gives a fixed maximum and overflow should be handled as
  a normal error.
- [`memchr`](https://docs.rs/memchr/latest/memchr/): consider for tokenizer or
  stream scanning if profiles show byte-search loops dominating.
- [`bumpalo`](https://docs.rs/bumpalo/latest/bumpalo/): consider for
  request-local arenas only when allocation profiles show many short-lived
  objects with the same lifetime. Arena memory must be bounded by request
  budgets.

Copy and pointer rules:

- Treat "memcpy optimization" as "make the safe slice operation obvious" first.
- Use `std::ptr::copy_nonoverlapping` only if safe slice APIs cannot express the
  operation, after a benchmark proves the win, and with a documented safety
  invariant beside the `unsafe` block.
- Do not add hand-written pointer loops for style. They must beat the safe
  implementation on the target fixture set.

SIMD and concurrency rules:

- Write a simple scalar implementation first. Consider SIMD only after profiles
  show the inner loop dominates and the scalar version has been cleaned up.
- Any SIMD path needs correctness fixtures, a scalar fallback, and target
  feature gating.
- Avoid internal Rayon-style parallelism inside one page render for now. Server
  deployments can already parallelize across requests/pages, and hidden inner
  parallelism can increase peak RSS. Revisit this after scheduler benchmarks
  show a clear need.

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

## Settled Decisions

- [x] `scripts/generate_performance_matrix.sh` defaults to release mode.
  Use `PROFILE=dev` only for smoke runs.
- [x] PDFium library location stays environment-driven. Do not commit machine
  paths.
- [x] Default macOS profiling order: `sample` first, Instruments when call-tree
  detail is needed, Samply as an optional flamegraph-friendly path.
- [x] Add host timing reliability flags to the matrix report in a follow-up, but
  do not block the first optimization block on that field.

## Remaining Questions

- [ ] Which exact fixture becomes optimization block 1 after two baseline runs?
- [ ] What family-specific thresholds should replace the global 10% rule after
  we understand variance?
- [ ] Should any focused performance subset become CI-gated, or should all
  benchmark budgets remain maintainer-local for now?
- [ ] Which `smallvec` inline capacities are justified by real path/token/clip
  histograms?
- [ ] Which memory tool should be the default for allocation evidence on macOS:
  Instruments Allocations, heaptrack-equivalent tooling, or targeted counters in
  the renderer?
