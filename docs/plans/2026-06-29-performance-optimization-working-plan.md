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
- [x] First renderer optimization landed: stroke join geometry is prepared once
  per stroke instead of recomputed inside every pixel/sample test.

## Operating Rules

- [ ] Optimize only after a matrix run and a profile identify the bottleneck.
- [ ] Use release builds for performance decisions. Dev-build timings are smoke
  checks only.
- [ ] Compare before and after on the same machine, same fixtures, same
  `max_edge`, same backend set, and same iteration count.
- [ ] Keep optimization PRs small enough to explain with one primary profile
  finding.
- [ ] Accept a block with at least 10% improvement on target fixtures, or accept
  a 5-10% improvement when repeated runs confirm it and the change is part of a
  clear cumulative optimization track.
- [ ] Accept a clear memory reduction under the same repeatability rule, with no
  new fallback or visual-regression evidence.
- [ ] Treat changes below 5% as noise unless repeated runs prove otherwise.
- [ ] Repeat and inspect any 5-10% change before calling it meaningful.
- [ ] Add no performance dependency without profile evidence and a short
  "why std is not enough" note in the change.
- [ ] Keep `unsafe` out of renderer hot paths unless a safe API cannot express
  the operation, the block is isolated, and the safety invariant is documented.
- [ ] Do not update public README performance claims until at least two stable
  matrix runs agree.

## Settled Questions For The First Optimization Wave

These questions are settled for the first performance wave. Reopen a decision
only when benchmark evidence or product requirements change. The goal is to
keep the work decisive without pretending the first numbers are public claims.

| Question | Working answer | Acceptance impact |
| --- | --- | --- |
| What is the first workload target? | `report/vector`, starting with `vector-stress`. | Phase 2 work must improve the focused vector set before moving to image-heavy or text-heavy work. |
| What counts as a meaningful speed win? | At least 10% on p95 or wall time for the target fixtures as a standalone win; 5-10% can land when repeated and clearly cumulative. | No commit should claim a performance win from a single noisy run. Small wins need stronger repeat evidence and no protection-set regression. |
| What counts as a meaningful memory win? | At least 10% lower peak RSS, allocation count, allocation bytes, or renderer-owned scratch memory as a standalone win; 5-10% can land when repeated and clearly cumulative. | Memory claims need a named metric, not just intuition from code review. Small wins need a named cumulative track. |
| Which references matter first? | PDFium for in-process comparison when available; Poppler as cold-process and visual reference. | Native-only work may proceed, but public comparison claims wait for PDFium evidence. |
| How strict is visual fidelity during speed work? | No new fallback bucket, error class, crash, timeout, or obvious visual drift on the touched fixture set. | Fast paths must prove they preserve clipping, transforms, alpha, and stroke semantics for their supported shape. |
| Are WASM and low-memory primary constraints now? | No. Server-side rendering is the primary model; low-memory remains a bounded-cache discipline, not a WASM-first architecture driver. | Avoid optimizing for WASM-specific constraints unless a later product requirement reopens this. |
| Are global caches allowed? | No. Only explicit request/session caches with visible benchmark configuration. | Any cache PR must expose budget and lifecycle in code and benchmark output. |
| Is internal page parallelism allowed? | Not in the first wave. Parallelize across requests/pages before adding hidden inner parallelism. | Rayon/thread-pool changes need separate scheduler and RSS evidence. |
| When do we add a dependency? | Only when profile evidence shows `std` is not enough and the crate has a narrow, justified role. | Dependency PRs need a short local rationale plus before/after data. |
| Which hardware do we optimize for first? | Commodity 64-bit server CPUs running native Rust release builds. | Avoid WASM-first or embedded-first tradeoffs unless a later product requirement changes the target. |
| Do we accept hardware-specific fast paths? | Yes, but only behind runtime/compile-time feature checks with a scalar fallback. | SIMD or target-feature PRs need correctness coverage on the fallback path and benchmark evidence on the accelerated path. |
| Can `SmallVec` replace hot `Vec`s broadly? | No. It is a measured tool for short, hot, high-frequency collections only. | A `SmallVec` PR needs length histograms and must show that stack growth does not hurt cache locality. |
| Can bulk-copy or `memcpy` style changes be optimized directly? | Prefer safe slice APIs first; let LLVM lower obvious contiguous copies. | Raw pointer copies require a benchmark win, a local safety invariant, and a safe implementation that was insufficient. |
| What metric decides allocation wins? | Allocation count, allocation bytes, peak RSS, or explicit scratch-buffer high-water, depending on the block. | The metric must be named before implementation and reported after the benchmark run. |

Not settled globally:

- Public speed or memory claims against PDFium, Poppler, or MuPDF. Those wait
  for repeated reference-renderer runs on a documented host.
- CI performance budgets. The benchmark matrix should stabilize first; early
  CI gates would mostly encode local variance.
- WASM-first constraints. Server-side native rendering is the current product
  shape; WASM can be revisited when it becomes a concrete delivery target.
- Broad dependency policy. Each performance dependency is still a local
  decision with local evidence.

## Questions To Close Before The Next Optimization Wave

These are the questions that should be considered settled for the first vector
wave. Reopen them only when benchmark evidence or product requirements change.

- [x] Should the next work optimize toward server-side native rendering rather
  than WASM? Yes. Native server-side rendering is the target for now.
- [x] Should low-memory behavior drive the architecture? Partly. Keep memory
  bounded and visible, but do not trade away broad server performance for a
  WASM-style low-memory constraint.
- [x] Should performance work favor algorithmic culling before SIMD? Yes.
  Remove wasted raster work before vectorizing the remaining work.
- [x] Should PDFium/Poppler parity block local native improvements? No. Local
  native improvements can land with focused Ferrugo evidence, while public
  comparison claims wait for reference-renderer runs.
- [x] Should dependency additions be normal for performance work? No. They are
  allowed, but each one needs profile evidence, a narrow job, and a rollback
  story.
- [x] Should sub-5% benchmark wins be committed? No, not as performance wins.
  Record them as rejected or inconclusive unless repeated runs show a larger
  effect.
- [x] Should repeated 5-10% wins be allowed? Yes. They can land when they are
  stable across repeated runs, have no protection-set regression, and fit a
  cumulative optimization track.
- [x] Should we optimize for average latency or tail latency first? Tail first.
  Use p95 for acceptance, with mean as supporting evidence.
- [x] Should memory improvements be accepted without speed wins? Yes, when the
  memory metric is named before the change and improves by at least 10% without
  visual or fallback regressions.

## Acceptance Criteria

These criteria apply to every optimization block unless a narrower follow-up
document explicitly overrides them.

Definition of done for one optimization commit:

- The commit changes one bottleneck class: for example path rasterization,
  image decode, output encoding, object loading, or allocation churn.
- The plan records the baseline artifact, the profile finding, the chosen
  metric, and the before/after result.
- The result clears the threshold: at least 10% p95 or wall-time improvement on
  target fixtures, or at least 10% lower peak RSS/allocation volume/scratch
  high-water as a standalone win. A 5-10% result can be accepted when repeated
  runs confirm it, the protection set does not regress, and the commit is
  explicitly part of a cumulative track.
- The focused fixture set has no new crash, timeout, fallback bucket, error
  class, output-dimension change, or obvious visual drift.
- The validation commands relevant to the touched surface pass before the next
  optimization starts.

Baseline acceptance:

- [ ] Two release-mode matrix runs on the same host have comparable top-10
  Ferrugo fixture rankings.
- [ ] Report artifacts include backend versions/commands, OS, CPU, Rust
  version, available core count, memory size when practical, fixture manifest,
  `max_edge`, iterations, warmup, timeout, and RSS availability.
- [ ] Missing PDFium is acceptable only when the report records `missing-tool`;
  PDFium is required before publishing comparison claims.
- [ ] Poppler timing is treated as a cold-process reference, not as an
  in-process renderer peer.
- [ ] Any host/tool caveat that affects trust in the numbers is written into
  the report or the working-plan notes.

Optimization-block acceptance:

- [ ] The block targets one fixture family and one profile-backed bottleneck.
- [ ] Target fixtures improve by at least 10% in p95/wall time, or peak RSS /
  allocation volume drops by at least 10%; alternatively, a 5-10% improvement
  is repeated, protection-set-neutral, and recorded as cumulative.
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
- [ ] The commit message and plan update name the bottleneck and the measured
  effect. If the candidate did not help, the plan records that negative result.

Dependency and hardware-aware acceptance:

- [ ] Prefer `std` and safe slice APIs first.
- [ ] Choose stack-inline data structures only after size histograms show that
  the inline capacity is correct for real fixtures.
- [ ] Record p50, p95, p99, and max length for any collection proposed for
  `SmallVec`, `ArrayVec`, or arena allocation.
- [ ] Keep hot structs and enum variants size-aware. A dependency that removes
  allocations but bloats every display item must prove a net win.
- [ ] Keep scratch buffers request-local or session-local; no hidden global
  cache.
- [ ] Any SIMD, pointer-copy, arena, or thread-pool change keeps a simple scalar
  or safe fallback path unless the crate boundary makes that impossible.
- [ ] Any `unsafe` code must have a local safety comment, a focused test, and a
  benchmark showing why safe APIs were insufficient.
- [ ] Any change that increases stack frame size or enum size must be checked
  against representative fixture data.
- [ ] Any target-feature-specific path must be gated and must not change output
  dimensions, alpha semantics, clipping, or fallback classification.
- [ ] Any bulk-copy optimization must describe source/destination overlap
  semantics and prefer `copy_from_slice`, `copy_within`, or `extend_from_slice`
  before pointer APIs.

## Phase 0: Baseline Hardening

Goal: make the first optimization target defensible.

- [x] Add or document a release-mode path for `scripts/generate_performance_matrix.sh`.
- [x] Run the full starter matrix in release mode with `native + poppler`.
- [ ] Run the same matrix with PDFium once `FERRUGO_PDFIUM_LIBRARY` is available.
- [x] Store baseline artifacts under `target/performance-matrix-baseline-*`.
- [ ] Record host details: OS, CPU, Rust version, Poppler path, PDFium path, and
  whether RSS was available.
- [x] Run the matrix twice and compare rank stability for the top 10 Ferrugo
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
- [x] Run attribution on the top 5 Ferrugo slow fixtures from the matrix.
- [x] Pick the first optimization block from attribution, not from assumptions.

Likely implementation shape:

- [x] reuse or extend `trace-native` for per-render phase timings;
- [x] keep attribution disabled by default;
- [x] avoid global state and keep measurements request-local.

Current caveat:

- `content_tokenize` only covers explicit token scans outside display-list
  construction. Builder-internal tokenization is still included in
  `display_list_build` until the builder APIs expose a cleaner split.

Baseline artifacts from 2026-06-29:

- `target/performance-matrix-baseline-starter-release-1.json`
- `target/performance-matrix-baseline-starter-release-1.md`
- `target/performance-matrix-baseline-starter-release-2.json`
- `target/performance-matrix-baseline-starter-release-2.md`

Attribution traces from 2026-06-29:

- `target/native-trace-vector-stress.json`: total 11.462 ms, raster paths
  11.270 ms.
- `target/native-trace-technical-hatch-clipping.json`: total 4.192 ms, raster
  paths 3.991 ms.
- `target/native-trace-prepress-trim-bleed-marks.json`: total 2.456 ms, raster
  paths 2.339 ms.
- `target/native-trace-mixed-text-image.json`: total 2.021 ms, raster paths
  1.843 ms.
- `target/native-trace-technical-linework-dimensions.json`: total 1.632 ms,
  raster paths 1.467 ms.

First optimization block:

- Target fixture: `fixtures/generated/vector-stress.pdf`.
- Target family: `report/vector`.
- Target phase: `raster_paths`.
- Initial target result: reduce `vector-stress` hot-render p95 by at least 10%
  without fallback or visual drift on the report/vector starter set.

## Phase 2: Vector And Report Hot Paths

Goal: improve the expected first bottleneck family without changing semantics.

Candidate fixtures:

- `fixtures/generated/vector-stress.pdf`
- `fixtures/generated/technical-hatch-clipping.pdf`
- `fixtures/generated/technical-linework-dimensions.pdf`
- `fixtures/generated/prepress-trim-bleed-marks.pdf`
- `fixtures/generated/technical-large-coordinate-plan.pdf`

Work items:

- [x] Profile top vector fixtures with `sample`, Instruments, or Samply.
- [x] Confirm whether time is in display-list construction, path flattening,
  clipping, stroke rasterization, or pixel loops.
- [ ] Add device-bounds culling before expensive raster work.
- [ ] Add fast paths for axis-aligned filled rectangles.
- [ ] Add fast paths for axis-aligned hairlines and simple strokes.
- [ ] Flatten reusable paths once per display item instead of per raster pass.
- [x] Apply clip/intersection checks before entering expensive pixel loops.
- [x] Precompute bevel/miter stroke join geometry once per stroke instead of
  normalizing join segments for every candidate pixel/sample.
- [x] Skip per-segment stroke distance checks when the candidate point is
  outside conservative padded segment bounds.
- [ ] Add regression fixtures or targeted tests around clipping and hairline
  correctness.
- [x] Add targeted tests for prepared stroke join geometry and degenerate join
  segments.
- [x] Add targeted tests for conservative stroke segment bounds.
- [x] Re-run matrix before and after each block.

Acceptance:

- [x] At least 10% improvement on the selected vector fixtures or a documented
  reason why the profile disproved the candidate.
- [x] No new fallback categories.
- [x] No unacceptable Poppler/PDFium visual drift on the touched fixture set.

First vector optimization result from 2026-06-29:

- Change: prepared bevel/miter stroke join triangles once in `stroke_path`.
- Profile evidence: previous `sample` run showed `stroke_path` dominating
  `vector-stress`, with visible time under `point_in_join_side` and `hypot`.
- Before: `target/benchmark-native-vector-stress-profile.json` mean
  `10.956 ms` over 5000 iterations.
- After: `target/benchmark-native-vector-stress-prepared-joins.json` mean
  `9.554 ms` over 5000 iterations, about 12.8% faster.
- Hot matrix after: `target/performance-matrix-vector-stress-prepared-joins.json`
  p95 `9.900 ms` over 30 measured iterations after 3 warmups.
- Baseline hot matrix p95: `10.929-11.012 ms`, so the p95 improvement is about
  9.4-10.1% depending on the baseline run.
- Visual/fallback check: current native PNG
  `target/native-vector-stress-prepared-joins.png` is byte-identical to
  `target/performance-matrix-baseline-starter-artifacts-1/native-cold-process-vector-stress.png`;
  focused matrix status remained `rendered` with no fallback bucket or error.

Second vector optimization result from 2026-06-29:

- Change: `point_in_stroke` now rejects sample points outside each line segment's
  padded device bounds before running the more expensive distance calculation.
- Before: `target/benchmark-native-vector-stress-prepared-joins.json` mean
  `9.554 ms` over 5000 iterations.
- After: `target/benchmark-native-vector-stress-line-bounds.json` mean
  `6.588 ms` over 5000 iterations, about 31.0% faster than the previous block.
- Hot matrix after: `target/performance-matrix-vector-stress-line-bounds.json`
  p95 `6.709 ms` over 30 measured iterations after 3 warmups.
- Compared with the original baseline hot matrix p95 `10.929-11.012 ms`, the
  cumulative p95 improvement on `vector-stress` is about 38.6-39.1%.
- Visual/fallback check: current native PNG
  `target/native-vector-stress-line-bounds.png` is byte-identical to
  `target/performance-matrix-baseline-starter-artifacts-1/native-cold-process-vector-stress.png`;
  focused matrix status remained `rendered` with no fallback bucket or error.

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: skip `point_in_active_clips` calls inside
  `stroke_path` when the active clip list is empty.
- Result: `target/benchmark-native-vector-stress-skip-empty-clips.json` mean
  `6.490 ms` vs `target/benchmark-native-vector-stress-line-bounds.json` mean
  `6.588 ms`, about 1.5% faster.
- Decision: below the 5% noise threshold, so the code change was reverted and
  should not be treated as a proven optimization.

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: precompute padded line bounds into a
  `PreparedStrokeLine` vector before entering `point_in_stroke`.
- Result: `target/benchmark-native-vector-stress-prepared-line-bounds.json`
  mean `7.122 ms` vs `target/benchmark-native-vector-stress-line-bounds.json`
  mean `6.588 ms`, about 8.1% slower on `vector-stress`.
- Secondary result:
  `target/benchmark-native-technical-hatch-prepared-line-bounds.json` mean
  `3.679 ms` vs
  `target/benchmark-native-technical-hatch-profile-after-stroke-culling.json`
  mean `3.830 ms`, about 3.9% faster on `technical-hatch-clipping`.
- Decision: mixed result with a clear regression on the primary vector target
  and a sub-5% gain on the secondary target, so the code change was reverted.

Third vector optimization result from 2026-06-29:

- Change: active clip paths now store device-space bounds, and fill/stroke
  raster bounds are intersected with those clip bounds before entering
  expensive pixel/sample loops.
- Target fixture: `fixtures/generated/technical-hatch-clipping.pdf`.
- Before: `target/benchmark-native-technical-hatch-profile-after-stroke-culling.json`
  mean `3.830 ms` over 8000 iterations.
- After: `target/benchmark-native-technical-hatch-clip-bounds.json` mean
  `2.842 ms` over 5000 iterations, about 25.8% faster.
- Regression guard: `target/benchmark-native-vector-stress-clip-bounds.json`
  mean `6.595 ms` vs `target/benchmark-native-vector-stress-line-bounds.json`
  mean `6.588 ms`, effectively neutral on `vector-stress`.
- Hot matrix after:
  `target/performance-matrix-report-vector-clip-bounds.json` reports
  `technical-hatch-clipping` p95 `3.018 ms` over 30 measured iterations after
  3 warmups.
- Previous hot matrix:
  `target/performance-matrix-report-vector-after-stroke-culling.json` reported
  `technical-hatch-clipping` p95 `3.912 ms`, so the p95 improvement is about
  22.9%.
- Visual/fallback check: current native PNG
  `target/native-technical-hatch-clip-bounds.png` is byte-identical to
  `target/performance-matrix-baseline-starter-artifacts-1/native-cold-process-technical-hatch-clipping.png`;
  focused matrix status remained `rendered` with no fallback bucket or error.

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: direct axis-aligned stroke hit-testing for
  horizontal and vertical lines with butt, square, and round caps.
- Result: `target/benchmark-native-vector-stress-axis-stroke-fast-path.json`
  mean `6.553 ms` vs `target/benchmark-native-vector-stress-clip-bounds.json`
  mean `6.595 ms`, about 0.6% faster on `vector-stress`.
- Secondary results:
  `target/benchmark-native-technical-hatch-axis-stroke-fast-path.json` mean
  `2.884 ms` vs `target/benchmark-native-technical-hatch-clip-bounds.json`
  mean `2.842 ms`, about 1.5% slower; and
  `target/benchmark-native-technical-linework-axis-stroke-fast-path.json` mean
  `0.899 ms`, only a small improvement on the linework fixture.
- Decision: below the 5% noise threshold and with a small regression on
  `technical-hatch-clipping`, so the code change was reverted.

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: direct single-segment stroke hit-testing
  for paths with exactly one line and no joins, bypassing the generic
  line-slice iterator and empty join path inside `stroke_path`.
- Result:
  `target/benchmark-native-vector-stress-single-stroke-fast-path.json` mean
  `6.507 ms` vs `target/benchmark-native-vector-stress-clip-bounds.json` mean
  `6.595 ms`, about 1.3% faster on `vector-stress`.
- Secondary results:
  `target/benchmark-native-technical-hatch-single-stroke-fast-path.json` mean
  `2.851 ms` vs `target/benchmark-native-technical-hatch-clip-bounds.json`
  mean `2.842 ms`, effectively neutral to slightly slower; and
  `target/benchmark-native-technical-linework-single-stroke-fast-path.json`
  mean `0.897 ms`, only a small improvement on the linework fixture.
- Decision: below the 5% noise threshold, so the code change was reverted.

Current profile after accepted vector optimizations:

- `target/sample-vector-stress-current.txt` still shows `stroke_path` as the
  dominant top-of-stack entry on `vector-stress`, with `fill_path`, blending,
  flattening, tokenization, and allocation work far smaller.
- `target/native-trace-vector-stress-current.json` reports total `6.491 ms`,
  with `raster_paths` at `6.304 ms`; load, decode, tokenization, display-list
  build, resource decode, text, and image phases are not the current target.
- Decision: continue path-raster work only for profile-backed algorithmic wins.
  The next broad track should include Phase 3 allocation/clone evidence and a
  later revisit of larger raster algorithms, not more sub-5% micro-fast-paths.

## Hardware-Aware Rust Notes

Goal: use Rust's memory model and the host CPU well without prematurely
outsmarting the compiler.

Short version:

- Prefer fewer operations over faster operations.
- Prefer borrowing and buffer reuse over copying.
- Prefer safe contiguous slice operations over handwritten pointer loops.
- Prefer measured, fixture-specific data structure changes over broad
  "performance crate" rewrites.
- Prefer scalar clarity first; add SIMD only after the cleaned-up scalar path is
  still the profile hotspot.

Working model:

- First make the renderer do less work: cull outside device bounds, intersect
  clip bounds early, reuse flattened geometry, and avoid repeated decode or
  allocation.
- Then make the remaining work friendlier to the CPU: contiguous memory,
  predictable branches, row-major traversal, and bulk operations that the
  compiler can recognize.
- Only then consider specialized crates, SIMD, arenas, or pointer-level copies.
  These tools are useful, but they should not hide an algorithm that still
  spends cycles on pixels or paths that cannot affect the output.

Default choices and modern Rust toolbox:

- Use `Vec<T>` for large or genuinely dynamic contiguous data. Prefer
  `with_capacity` when the upper bound is known, reuse buffers across phases,
  and avoid repeated grow/copy cycles inside pixel or path loops.
- Keep `Vec<T>` when the data often escapes the current loop or lives inside
  long-lived display-list/resource structures. Removing one allocation is not a
  win if every stored item becomes wider and hurts cache locality.
- Use `Box<[T]>` when a buffer becomes immutable and should not carry spare
  capacity. This can make ownership and memory accounting clearer after build
  phases such as display-list construction.
- Use `Arc<[T]>` only for shared immutable data across request-local workers or
  session cache entries. Do not introduce `Arc` just to work around borrowing.
- Use slices in APIs: `&[T]`, `&mut [T]`, `&str`, and `&[u8]` keep ownership
  local and make hot code easier to profile.
- Prefer row-major, cache-friendly traversal for raster buffers. Keep inner
  loops branch-light and make clipping decisions before entering them.
- Prefer safe bulk-copy APIs such as `copy_from_slice`, `copy_within`,
  `clone_from_slice`, and `extend_from_slice`. LLVM can lower these to optimized
  `memcpy`/`memmove` patterns for the target CPU.
- Prefer `chunks_exact`, `array_chunks`, and slice splitting helpers when they
  make row or pixel structure explicit without indexing ambiguity.
- Prefer squared-distance comparisons over `sqrt`/`hypot` in inner loops when
  the math allows it.
- Prefer separating rare cases from hot loops. For example, handle complex
  caps, joins, alpha, or clips outside the tightest loop when a simpler fast
  path can prove the same output for common cases.
- Prefer compact plain-data structs for geometry hot paths. Small `Copy` types
  passed by value are fine; large owned structs and large enum variants should
  stay out of per-pixel work.

Decision matrix:

| Tool | Use when | Avoid when | Required evidence |
| --- | --- | --- | --- |
| `Vec<T>` | Size is dynamic, large, or long-lived. | The collection is tiny, created in a very hot loop, and usually fits a known bound. | Capacity or allocation profile if changing existing code. |
| `SmallVec<[T; N]>` | Most real fixture lengths fit inline and allocation cost is visible. | `N` is guessed, the type is stored in many display items, or stack/cache cost grows. | p50/p95/p99/max length histogram plus before/after benchmark. |
| `ArrayVec<T, N>` | A real invariant caps length and overflow is a renderer error or fallback. | The limit is only "probably enough". | Spec/code invariant and overflow test. |
| Safe slice copy | Source and destination are contiguous and semantics are expressible safely. | Copy can be avoided by borrowing or reusing scratch. | Code clarity plus benchmark if in a hot path. |
| Raw pointer copy | Safe APIs cannot express the needed non-overlap/aliasing contract fast enough. | The change is only stylistic or marginal. | Safety comment, focused test, and measured win. |
| SIMD | A cleaned-up scalar inner loop still dominates profiles. | The algorithm still wastes work outside bounds or clips. | Scalar fallback, target gating, visual tests, benchmark win. |
| Arena allocation | Many short-lived objects share one request lifetime. | Objects escape request/session boundaries or memory budgets are unclear. | Allocation profile, request budget, and peak-memory comparison. |

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
- [`bytemuck`](https://docs.rs/bytemuck/latest/bytemuck/): consider only for
  well-defined plain-data casts at boundaries where layout is explicit. Avoid it
  for PDF object models or types with padding-sensitive semantics.
- [`bumpalo`](https://docs.rs/bumpalo/latest/bumpalo/): consider for
  request-local arenas only when allocation profiles show many short-lived
  objects with the same lifetime. Arena memory must be bounded by request
  budgets.
- [`rayon`](https://docs.rs/rayon/latest/rayon/): defer for now. It can help
  batch-level or page-level parallelism later, but hidden per-page parallelism
  needs scheduler, cancellation, and RSS evidence first.
- [`wide`](https://docs.rs/wide/latest/wide/) or explicit
  `std::arch` intrinsics: consider only after the scalar inner loop is clean,
  branch-light, and still dominant in profiles. Keep the scalar fallback as the
  reference implementation.

Copy, `memcpy`, and pointer rules:

- Treat "memcpy optimization" as "make the safe slice operation obvious" first.
- Prefer one bulk copy over many tiny copies. If a loop copies predictable
  contiguous ranges, reshape it toward slice ranges before considering pointer
  code.
- Avoid copying at all when borrowing or reusing a scratch buffer is possible.
  The fastest `memcpy` is still slower than no copy in a hot loop.
- Use `std::ptr::copy_nonoverlapping` only if safe slice APIs cannot express the
  operation, after a benchmark proves the win, and with a documented safety
  invariant beside the `unsafe` block.
- Do not add hand-written pointer loops for style. They must beat the safe
  implementation on the target fixture set.
- Keep overlapping-copy semantics explicit: `copy_from_slice` /
  `copy_nonoverlapping` for non-overlap, `copy_within` / `ptr::copy` only when
  overlap is intended.

Small buffer rules:

- Measure length distributions before choosing `SmallVec<[T; N]>`. Record p50,
  p95, p99, and max for the candidate collection.
- Pick `N` to cover the common case without bloating every item. For hot
  display-list items, a larger inline size can hurt cache locality even when it
  removes allocations.
- Prefer a small local `SmallVec` inside a hot builder over storing `SmallVec`
  in persistent renderer data unless the persistent size increase is measured.
- Keep `Vec<T>` when sizes are usually large, highly variable, or stored inside
  long-lived structs.
- Prefer `ArrayVec<T, N>` only when overflow has clear semantics and `N` is a
  real invariant, not a guess.

SIMD and concurrency rules:

- Write a simple scalar implementation first. Consider SIMD only after profiles
  show the inner loop dominates and the scalar version has been cleaned up.
- Any SIMD path needs correctness fixtures, a scalar fallback, and target
  feature gating.
- Prefer algorithmic wins before SIMD: culling, clipping, squared-distance math,
  branch reduction, and fewer passes usually beat vectorizing wasted work.
- Avoid internal Rayon-style parallelism inside one page render for now. Server
  deployments can already parallelize across requests/pages, and hidden inner
  parallelism can increase peak RSS. Revisit this after scheduler benchmarks
  show a clear need.

## Phase 3: Allocation And Clone Audit

Goal: reduce avoidable work in hot paths after phase attribution exposes where
allocations matter.

- [x] Run Clippy with perf lints as part of the normal all-target/all-feature
  gate.
- [x] Review hotpath `Vec` creation and growth.
- [ ] Review `String`, `PathBuf`, and large enum clones inside loops.
- [ ] Remove intermediate `.collect()` calls where the consumer can stream.
- [ ] Inspect large enum variants if profiles show copy pressure.
- [ ] Add before/after allocation evidence where tooling is available.

Acceptance:

- [ ] Matrix timing improves or memory high-water drops on a target fixture set.
- [ ] Code remains simpler or equally readable; no clever allocation trick
  without a measured win.

Clippy perf audit from 2026-06-29:

- `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::perf`
  completed without warnings.
- No code change was made from Clippy alone; further allocation work needs
  targeted profile or allocation evidence.

Hotpath `Vec` audit from 2026-06-29:

- `target/sample-vector-stress-current.txt` showed only a few allocation samples
  under `flatten_path_segments` and display-list construction, while
  `stroke_path` dominated the profile.
- Reviewed `flatten_path_segments`, `stroke_joins_from_subpaths`,
  `dashed_subpath_line_segments`, and the hairline snapping `.collect::<Vec<_>>()`
  paths.
- Decision: do not introduce `SmallVec`, broad preallocation, or a custom arena
  yet. The current evidence does not show allocation pressure large enough to
  satisfy the optimization-block acceptance criteria.

## Phase 4: Image And Scan Track

Goal: make scan/image-heavy documents fast without increasing peak memory.

- [x] Identify image-heavy fixtures from matrix and existing image reports.
- [x] Profile decode, color conversion, alpha/soft-mask work, and output encode.
- [ ] Add downsample-aware decode where the source image is much larger than the
  target raster.
- [ ] Avoid full RGBA expansion when the target raster is smaller and direct
  sampling is possible.
- [x] Avoid full inverse-matrix work per pixel for axis-aligned image
  placements.
- [ ] Reuse SoftMask/alpha scratch buffers within a render request.
- [ ] Investigate cropped decode when the CTM/clip excludes large image areas.

Acceptance:

- [x] Clear time or memory reduction on scan/image fixtures.
- [x] No regression on masks, ICC conversions, predictor images, or transparent
  image fixtures.

Image-heavy baseline and attribution update from 2026-06-29:

- Focused matrix:
  `target/performance-matrix-image-heavy-current.json`, native hot-render,
  `fixtures/image-heavy-memory-manifest.tsv`, `--max-edge 160`,
  30 measured iterations after 3 warmups.
- Slowest p95 fixtures: `mobile-mixed-compression-scan.pdf` `1.061 ms`,
  `image-heavy-repeated-xobject-report.pdf` `0.904 ms`,
  `image-heavy-rotated-mask-sheet.pdf` `0.830 ms`,
  `scanner-large-image-budget.pdf` `0.602 ms`.
- Initial traces showed the top mixed image fixtures as `raster_paths`-dominant
  with `raster_images: 0.000 ms`, because content-order rasterization was
  measured as one path phase.
- Added ordered-display-list phase attribution so `trace-native` records image
  work inside mixed z-order pages without changing paint order, clip state, or
  raster output.
- Post-change traces now expose image work:
  `mobile-mixed-compression-scan.pdf` `raster_images: 0.142 ms`,
  `image-heavy-repeated-xobject-report.pdf` `raster_images: 0.125 ms`,
  `image-heavy-rotated-mask-sheet.pdf` `raster_images: 0.107 ms`.
- Post-change matrix remained fully rendered with no fallback/error records:
  `target/performance-matrix-image-heavy-phase-attribution.json`. P95 stayed
  effectively neutral for the top fixture (`1.061 ms` -> `1.056 ms`) and
  improved on repeated-xobject/rotated-mask in this run, but this change is
  treated as profiling infrastructure, not a speed claim.
- Decision: the next image optimization should target actual `RasterImages`
  work only after a deeper sample/Instruments profile shows whether image
  sampling, soft-mask alpha, DCT decode, or path overlays dominate the target
  fixture.

Longer sample on `mobile-mixed-compression-scan.pdf` from 2026-06-29:

- Command artifact:
  `target/performance-matrix-mobile-mixed-profile-run.json`, native hot-render,
  `--include-family mixed-compression`, `--max-edge 160`, 50,000 measured
  iterations after 10 warmups.
- Sample artifact:
  `target/sample-mobile-mixed-compression-phase-attribution.txt`, 10-second
  macOS `sample` run against the long benchmark process.
- p95 for the focused long run was `1.114 ms`; the fixture remained rendered
  with no fallback or error records.
- Top profile entries:
  `stroke_path` `5596` samples, `ImageSampleCache::sample` `480` samples,
  `draw_image` `262` samples, `source_over` `65` samples, `blend_pixel`
  `19` samples.
- Resource decode also showed image decode work, especially Flate/miniz and a
  smaller JPEG header/decode component, but it was not the dominant total stack
  for this focused run.
- Decision: do not start Phase 4 with DCT, SoftMask, or RGBA-expansion
  micro-optimizations. For the current slowest image-heavy fixture, the
  dominant work is still path stroke overlay. The next code candidate should be
  a profile-backed simple stroke/rect overlay optimization, or the target
  should switch to a fixture where `RasterImages` dominates after attribution.

Rejected vector candidate from 2026-06-29:

- Change tested locally but not kept: a semantically tight axis-aligned
  rectangle stroke fast path for butt-cap, miter-join rectangle outlines.
- First broad variant was fast on `mobile-mixed-compression-scan.pdf`, but was
  rejected because it could change corner semantics compared with the generic
  stroke hit test.
- Tightened variant result:
  `target/performance-matrix-mobile-mixed-rect-stroke-fast-path-final.json`
  p95 `0.995 ms` vs
  `target/performance-matrix-image-heavy-phase-attribution.json` p95
  `1.056 ms`, about 5.8% faster on the mixed-compression target.
- Protection result:
  `target/performance-matrix-report-vector-rect-stroke-fast-path-final.json`
  showed `vector-stress` p95 `6.689 ms` and
  `technical-hatch-clipping` p95 `3.072 ms`, worse than the accepted
  clip-bounds vector protection set.
- Decision: reverted. The target win was in the repeatable 5-10% range, but it
  was not protection-set-neutral.

First image raster optimization result from 2026-06-29:

- Change: axis-aligned image placement now uses a dedicated raster loop. It
  computes Y coverage and source Y once per row, avoids a full inverse-matrix
  point transform per pixel, and fast-paths fully covered pixels to coverage
  `1.0` while preserving the existing edge-overlap calculation.
- Profile evidence: phase attribution and focused traces showed true image
  raster work on pure image fixtures, while the larger mixed target remained
  stroke-overlay dominated. The change targets the measured `raster_images`
  component without adding dependencies or `unsafe`.
- A/B baseline:
  `target/performance-matrix-image-heavy-axis-ab-baseline.json`, native
  hot-render, `fixtures/image-heavy-memory-manifest.tsv`, `--max-edge 160`,
  300 measured iterations after 20 warmups.
- After:
  `target/performance-matrix-image-heavy-axis-final.json`, same command shape
  and host after the final Clippy cleanup.
- Results: `soft-mask-image.pdf` p95 `0.101 ms` -> `0.085 ms` (~15.8%),
  `dct-image.pdf` p95 `0.087 ms` -> `0.079 ms` (~9.2%), and
  `predictor-image.pdf` p95 `0.069 ms` -> `0.063 ms` (~8.7%).
  `scanner-large-image-budget.pdf` p95 `0.629 ms` -> `0.604 ms` (~4.0%).
  Larger mixed pages improved less: `mobile-mixed-compression-scan.pdf` p95
  `1.070 ms` -> `1.046 ms` (~2.2%) because path strokes still dominate.
- Protection result:
  `target/performance-matrix-report-vector-axis-image-protection.json` stayed
  neutral to better on the report/vector set, with no fallback or error
  records.
- Visual check: current renders in `target/axis-image-visual-current/` are
  byte-identical to existing native baseline PNGs for `dct-image`,
  `predictor-image`, `scanner-large-image-budget`, and `soft-mask-image`.

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

The first optimization block is vector/report path rasterization. The first two
release matrix runs and `trace-native` attribution agree that `vector-stress` is
the dominant hot-render target and that `raster_paths` accounts for nearly all
of the traced render time on the report/vector candidates.

The most likely high-value candidates are:

1. device-bounds culling before raster work;
2. simple rect and hairline fast paths;
3. flatten-once path reuse;
4. clip-before-loop checks.

If deeper profiler samples point elsewhere inside path rasterization, this
section should be edited before code changes start.

## Settled Decisions

- [x] `scripts/generate_performance_matrix.sh` defaults to release mode.
  Use `PROFILE=dev` only for smoke runs.
- [x] PDFium library location stays environment-driven. Do not commit machine
  paths.
- [x] Default macOS profiling order: `sample` first, Instruments when call-tree
  detail is needed, Samply as an optional flamegraph-friendly path.
- [x] Add host timing reliability flags to the matrix report in a follow-up, but
  do not block the first optimization block on that field.
- [x] First optimization block is `report/vector` path rasterization, starting
  with `fixtures/generated/vector-stress.pdf`.

## Remaining Questions

- [ ] What family-specific standalone and cumulative thresholds should replace
  the default 10% / repeatable 5-10% rule after we understand variance?
- [ ] Should any focused performance subset become CI-gated, or should all
  benchmark budgets remain maintainer-local for now?
- [ ] Which `smallvec` inline capacities are justified by real path/token/clip
  histograms?
- [ ] Which memory tool should be the default for allocation evidence on macOS:
  Instruments Allocations, heaptrack-equivalent tooling, or targeted counters in
  the renderer?
