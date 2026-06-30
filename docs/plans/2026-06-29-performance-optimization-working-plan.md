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
- [ ] Treat 10% as the default acceptance target for an optimization block or a
  standalone performance claim, not for every individual commit.
- [ ] Accept a 5-10% commit when repeated runs confirm it and the change is part
  of a clear cumulative optimization track.
- [ ] Let multiple confirmed 5% wins compound when they attack the same
  bottleneck class and keep the protection set neutral.
- [ ] Allow correctness guards, instrumentation, and documentation commits
  without speed claims when they reduce risk for the next optimization.
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
| What counts as a meaningful speed win? | At least 10% on p95 or wall time for the target fixtures as a standalone win; repeated 5-10% wins can land and compound when they stay on the same bottleneck track. | No commit should claim a performance win from a single noisy run. Small wins need stronger repeat evidence, no protection-set regression, and a named cumulative track. |
| What counts as a meaningful memory win? | At least 10% lower peak RSS, allocation count, allocation bytes, or renderer-owned scratch memory as a standalone win; repeated 5-10% wins can land and compound when they stay on the same memory track. | Memory claims need a named metric, not just intuition from code review. Small wins need a named cumulative track. |
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
- [x] Should memory improvements be accepted without speed wins? Yes. A 10%
  reduction is a standalone memory win; repeated 5-10% reductions can land when
  they are part of a named cumulative track and have no visual or fallback
  regressions.

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
  explicitly part of a cumulative track. Several confirmed 5-10% commits can
  close one optimization block together; each commit just needs honest evidence
  and a clear relationship to the same bottleneck.
- Correctness guards, benchmark instrumentation, and planning commits do not
  need a speed threshold, but must avoid claiming a performance win.
- The focused fixture set has no new crash, timeout, fallback bucket, error
  class, output-dimension change, or obvious visual drift.
- The validation commands relevant to the touched surface pass before the next
  optimization starts.

Baseline acceptance:

- [ ] Two release-mode matrix runs on the same host have comparable top-10
  Ferrugo fixture rankings.
- [x] Report artifacts include backend versions/commands, OS, CPU, Rust
  version, available core count, memory size when practical, fixture manifest,
  `max_edge`, iterations, warmup, timeout, and RSS availability.
- [ ] Missing PDFium is acceptable only when the report records `missing-tool`;
  PDFium is required before publishing comparison claims.
- [ ] Poppler timing is treated as a cold-process reference, not as an
  in-process renderer peer.
- [x] Any host/tool caveat that affects trust in the numbers is written into
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
- [x] Record host details: OS, CPU, Rust version, Poppler path, PDFium path, and
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

Resource attribution refinement from 2026-06-30:

- Change: `trace-native` and repeat-benchmark phase timing JSON now keep
  `resource_decode` as the compatible aggregate and also expose
  `resource_graphics`, `resource_forms`, `resource_images`, `resource_fonts`,
  and `resource_annotations`.
- Purpose: the first image/scan optimization wave kept hitting a broad
  `resource_decode` bucket. The subfields make the next profile loop tell
  whether a fixture is actually image decode, font loading, form resolution, or
  general graphics resources.
- Smoke artifact: `target/trace-scanner-large-resource-subphases.json` confirms
  the scanner fixture's resource time is overwhelmingly image-owned:
  `resource_decode` `8.960 ms`, `resource_images` `8.841 ms`,
  `resource_forms` `0.114 ms`, and the other resource buckets near zero in the
  dev trace. Treat the absolute timings as dev-build smoke, not performance
  claims.
- Decision: accept as profiling infrastructure. The next image/scan code
  candidate should target `resource_images` specifically, not generic resource
  loading.

Baseline artifacts from 2026-06-29:

- `target/performance-matrix-baseline-starter-release-1.json`
- `target/performance-matrix-baseline-starter-release-1.md`
- `target/performance-matrix-baseline-starter-release-2.json`
- `target/performance-matrix-baseline-starter-release-2.md`

Host details recorded on 2026-06-29:

- OS: macOS 26.5.1, build 25F80.
- Architecture: arm64 / `aarch64-apple-darwin`.
- CPU: Apple M1 Ultra, 20 logical CPUs.
- Memory: 64 GiB reported by Node `os.totalmem()`.
- Rust: `rustc 1.95.0-nightly (842bd5be2 2026-01-29)`, LLVM 22.1.0.
- Poppler: `pdftoppm` version 26.05.0 from the Codex runtime dependency
  bundle. The absolute local runtime path is intentionally not committed.
- PDFium: `FERRUGO_PDFIUM_LIBRARY` was not set in this shell, so PDFium matrix
  runs remain deferred and reports must use `missing-tool` until configured.
- RSS caveat: the local sandbox rejected `ps`, so `current_rss_kib()` reports
  no RSS samples in this environment. Treat RSS fields from this run as
  unavailable rather than as zero memory use.

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
- [x] Add device-bounds culling before expensive raster work.
- [x] Add fast paths for axis-aligned filled rectangles.
- [x] Add a fast path for simple non-axis-aligned strokes; keep the earlier
  narrow axis-aligned hairline candidate rejected unless new evidence reopens
  it.
- [ ] Flatten reusable paths once per display item instead of per raster pass.
- [x] Apply clip/intersection checks before entering expensive pixel loops.
- [x] Precompute bevel/miter stroke join geometry once per stroke instead of
  normalizing join segments for every candidate pixel/sample.
- [x] Skip per-segment stroke distance checks when the candidate point is
  outside conservative padded segment bounds.
- [x] Add regression fixtures or targeted tests around clipping and hairline
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

Third vector guard result from 2026-06-30:

- Change: path rasterization now checks transformed device bounds before
  `flatten_path_segments`, with conservative stroke padding for caps and miter
  joins.
- Evidence: `rasterize_paths_should_cull_off_device_paths_before_flattening`
  proves a fully off-device path is skipped before it can hit the configured
  flattening limit.
- Baseline artifact:
  `target/performance-matrix-technical-cull-baseline.json`.
- After artifact: `target/performance-matrix-technical-cull-after.json`.
- Technical drawing result: the current floorplan, schematic, large-coordinate,
  and transform-detail fixtures did not contain enough off-device path work to
  produce a meaningful speed win; p95 changes stayed between -0.2% and +2.5%.
- Protection set artifact: `target/performance-matrix-report-vector-cull-after.json`.
  All 4 `report/vector` records remained `rendered`, with no fallback bucket or
  error. `vector-stress` p95 changed from `6.616 ms` to `6.531 ms`; the smaller
  fixture p95 deltas stayed in the local-noise range.
- Performance claim: none. This is a culling guard that prevents wasted
  flattening and off-device complexity errors; it should not be counted as a
  measured fixture speedup yet.

Rejected candidate from 2026-06-30:

- Candidate: add an axis-aligned line-body shortcut inside `point_in_stroke`
  for Butt and Square caps before falling back to generic line projection.
- Baseline artifact:
  `target/performance-matrix-report-vector-hairline-baseline.json`.
- Candidate artifact:
  `target/performance-matrix-report-vector-axis-line-fastpath-after.json`.
- Result on the focused `report/vector` hot-render set with 100 measured
  iterations and 10 warmups: `vector-stress` p95 improved only from `6.616 ms`
  to `6.542 ms` (about 1.1%), while `technical-hatch-clipping` regressed from
  `2.807 ms` to `2.909 ms` and `prepress-trim-bleed-marks` regressed from
  `1.098 ms` to `1.430 ms`.
- Decision: reverted. The candidate is below the repeated 5% threshold and is
  not protection-set-neutral, so it should not land as a performance commit.

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

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: per-sample `ActiveClip` bounds checks
  before calling `point_in_path` inside `point_in_active_clips`.
- Rationale: active clip bounds are already used to shrink raster pixel bounds;
  this candidate tried to avoid polygon tests for samples that still reached
  the inner clip predicate.
- Baseline:
  `target/performance-matrix-report-vector-clip-point-bounds-before.json`,
  native hot-render, `report/vector`, `--max-edge 160`, 200 measured
  iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-report-vector-clip-point-bounds-after.json`, same
  command and host.
- Result: `technical-hatch-clipping.pdf` p95 improved `2.896 ms` -> `2.818 ms`
  (~2.7%), `technical-linework-dimensions.pdf` p95 improved `0.944 ms` ->
  `0.909 ms` (~3.7%), and `vector-stress.pdf` was effectively neutral
  (`6.488 ms` -> `6.464 ms`).
- Decision: below the 5% noise threshold and not enough evidence for a
  cumulative track, so the code change was reverted.

Rejected candidate from 2026-06-30:

- Change tested locally but not kept: compute pixel-center bounds once for the
  axis-aligned rectangle fill fast path, and skip `point_in_active_clips` inside
  that loop when no active clips exist.
- Rationale: this targeted table, dashboard, and office-export pages that use
  many simple filled rectangles, while preserving the existing center-sample
  semantics for rectangle coverage.
- Baselines:
  `target/benchmark-native-office-table-rect-baseline.json`,
  `target/benchmark-native-spreadsheet-grid-rect-baseline.json`, and
  `target/benchmark-native-dashboard-rect-baseline.json`, each
  `benchmark-native`, `--max-edge 160`, 3000 iterations.
- Candidate:
  `target/benchmark-native-office-table-rect-center-bounds.json`,
  `target/benchmark-native-spreadsheet-grid-rect-center-bounds.json`, and
  `target/benchmark-native-dashboard-rect-center-bounds.json`, same command
  shape and host.
- Result: `office-table.pdf` mean improved `0.289 ms` -> `0.282 ms` (~2.4%),
  `spreadsheet-dense-numeric-grid.pdf` improved `0.778 ms` -> `0.761 ms`
  (~2.2%), and `browser-firefox-dashboard-print.pdf` improved `0.730 ms` ->
  `0.708 ms` (~3.0%).
- Decision: consistent but below the 5% acceptance threshold, so the code
  change was reverted and should not be treated as a cumulative optimization
  unless future fixtures show a larger rectangle-fill bottleneck.

Rejected candidate from 2026-06-29:

- Change tested locally but not kept: cull path display items before
  `flatten_path_segments` by transforming approximate `PathDisplayItem` bounds
  into device space and intersecting them with active clip pixel bounds.
- Rationale: this implements the intended "device-bounds culling before
  expensive raster work" shape for fully offscreen path items, but it adds an
  extra segment-bounds scan for every visible path.
- Baseline:
  `target/performance-matrix-report-vector-clip-point-bounds-before.json`,
  native hot-render, `report/vector`, `--max-edge 160`, 200 measured
  iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-report-vector-preflatten-cull-after.json`, same
  command and host.
- Result: `technical-linework-dimensions.pdf` p95 improved `0.944 ms` ->
  `0.920 ms` (~2.5%), `vector-stress.pdf` was effectively neutral
  (`6.488 ms` -> `6.467 ms`), and `technical-hatch-clipping.pdf` regressed
  slightly (`2.896 ms` -> `2.924 ms`).
- Decision: below the 5% noise threshold and mixed across the protection set.
  Keep culling after flattening for now; revisit pre-flatten culling only with
  fixtures that contain many fully offscreen paths.

Rejected candidate from 2026-06-30:

- Change tested locally but not kept: cache `FlattenedPath` bounds while
  flattening so `flattened_bounds` and rectangle-fill detection can avoid
  scanning subpath points later.
- Rationale: this targeted repeated path-bound scans in raster hot paths, but
  it also increased every flattened path value by one optional bounds payload.
- Baselines:
  `target/performance-matrix-flattened-bounds-baseline.json` and
  `target/performance-matrix-flattened-bounds-spreadsheet-baseline.json`,
  native hot-render, `--max-edge 160`, 100 measured iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-flattened-bounds-after.json` and
  `target/performance-matrix-flattened-bounds-spreadsheet-after.json`, same
  command shape and host.
- Result: chart/dashboard/map/vector fixtures mostly regressed, including
  `chart-combo-legend.pdf` p95 `0.840 ms` -> `0.900 ms` (~7.1% slower),
  `clipped-paths.pdf` p95 `0.589 ms` -> `0.630 ms` (~7.0% slower), and
  `vector-stress.pdf` p95 `6.690 ms` -> `6.911 ms` (~3.3% slower). Some
  spreadsheet fixtures improved, for example `office-table.pdf` p95
  `0.348 ms` -> `0.319 ms` (~8.3% faster), but the focused protection set was
  not neutral.
- Decision: reverted. The candidate trades repeated scans for a larger hot
  struct and is not protection-set-neutral. Revisit only with a narrower bounds
  cache that does not bloat all flattened paths, or with profile evidence that
  path-bound scans dominate a specific family.

Rejected allocation candidate from 2026-06-29:

- Change tested locally but not kept: reuse one `Vec<PreparedStrokeJoin>` scratch
  buffer across path items instead of allocating a fresh prepared-join vector in
  each `stroke_path` call.
- Rationale: this targeted Phase 3 allocation churn in the profiled
  `stroke_path` hot area without adding a dependency or unsafe code.
- Baseline:
  `target/performance-matrix-report-vector-clip-point-bounds-before.json`,
  native hot-render, `report/vector`, `--max-edge 160`, 200 measured
  iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-report-vector-prepared-joins-scratch-after.json`,
  same command and host.
- Result: the protection set regressed: `vector-stress.pdf` p95 `6.488 ms` ->
  `6.712 ms` (~3.5% slower), `technical-hatch-clipping.pdf` p95 `2.896 ms` ->
  `2.954 ms` (~2.0% slower), and `prepress-trim-bleed-marks.pdf` p95
  `1.069 ms` -> `1.101 ms` (~3.0% slower).
- Decision: reverted. The extra mutable scratch plumbing did not reduce the
  dominant runtime on the current fixtures and made the hot path slower.

Rejected candidate from 2026-06-30:

- Change tested locally but not kept: store conservative bounds on prepared
  bevel/miter stroke joins and add a cheap square-bounds check for round joins
  before running join hit tests.
- Rationale: this targeted the remaining `stroke_path` top stack by avoiding
  triangle/circle checks for sample points that are far from each join.
- Baseline:
  `target/performance-matrix-report-vector-join-bounds-before.json`,
  native hot-render, `report/vector`, `--max-edge 160`, 200 measured
  iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-report-vector-join-bounds-after.json`, same
  command and host.
- Result: mixed and not protection-set-neutral. `prepress-trim-bleed-marks.pdf`
  p95 improved `1.088 ms` -> `0.768 ms` (~29.4%) and
  `technical-linework-dimensions.pdf` improved `0.957 ms` -> `0.813 ms`
  (~15.0%), but the primary `vector-stress.pdf` protection fixture regressed
  `6.658 ms` -> `8.061 ms` (~21.1% slower). `technical-hatch-clipping.pdf`
  improved only ~3.8%.
- Decision: reverted. Per-join bounds can help sparse linework, but they add
  enough branch/struct overhead to hurt dense vector stress. Revisit only with
  a split strategy that applies bounds selectively to sparse/simple join sets
  and proves neutrality on `vector-stress`.

Rejected candidate from 2026-06-30:

- Change tested locally but not kept: keep the prepared bevel/miter join
  structure unchanged and add only the cheap square-bounds check before
  round-join `distance_squared`.
- Rationale: this isolated the round-join part of the previous candidate to
  test whether the dense-vector regression came from widening
  `PreparedStrokeJoin` or from the extra branch in join predicates.
- Baseline:
  `target/performance-matrix-report-vector-join-bounds-before.json`,
  native hot-render, `report/vector`, `--max-edge 160`, 200 measured
  iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-report-vector-round-join-bounds-after.json`, same
  command and host.
- Result: rejected by the protection set. `vector-stress.pdf` p95 regressed
  `6.658 ms` -> `8.463 ms` (~27.1% slower), `technical-linework-dimensions.pdf`
  regressed `0.957 ms` -> `1.018 ms` (~6.4% slower), and
  `prepress-trim-bleed-marks.pdf` regressed `1.088 ms` -> `1.149 ms` (~5.6%
  slower). `technical-hatch-clipping.pdf` was effectively neutral.
- Decision: reverted. Round-join bounds are not a good scalar branch tradeoff
  for the current vector workload.

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

Raster compositing fast-path result from 2026-06-30:

- Fresh baselines:
  `target/performance-matrix-current-hot-native.json`,
  `target/performance-matrix-current-technical-native.json`, and
  `target/performance-matrix-current-image-native.json`.
- Phase evidence:
  `target/native-trace-current-vector-stress.json`,
  `target/native-trace-current-engineering-floorplan.json`,
  `target/native-trace-current-technical-hatch.json`, and
  `target/native-trace-current-repeated-xobject.json` again showed path
  rasterization as the dominant phase. `vector-stress` spent `6.481 ms` of
  `7.414 ms` in `raster_paths`; `engineering-floorplan-precision` spent
  `2.980 ms` of `3.937 ms` in `raster_paths`; `technical-hatch-clipping` spent
  `2.690 ms` of `3.680 ms` in `raster_paths`.
- CPU profile evidence:
  `target/sample-current-report-vector.txt` showed `stroke_path` as the
  largest sampled path, but also a meaningful `fill_path -> blend_pixel ->
  source_over` component. Flattening was negligible in this profile, so the
  accepted change targets compositing work, not another flattening cache.
- Change: `blend_pixel` now directly writes fully opaque `BlendMode::Normal`
  pixels when coverage is `1.0`, avoiding the destination read, backdrop blend,
  and source-over calculation.
- Correctness rationale: for normal source-over compositing with source alpha
  `255` and full coverage, the result is exactly the source pixel independent
  of the destination pixel. The implementation stays dependency-free and uses
  only the existing checked raster-device API.
- Results on the starter matrix:
  `browser-chromium-article-print.pdf` p95 `0.713 ms` -> `0.311 ms`
  (~56.4%), `office-report-header-footer-link.pdf` `0.816 ms` -> `0.550 ms`
  (~32.6%), `slide-title-gradient.pdf` `0.963 ms` -> `0.584 ms` (~39.4%),
  `prepress-trim-bleed-marks.pdf` `1.082 ms` -> `0.812 ms` (~25.0%),
  `technical-linework-dimensions.pdf` `0.931 ms` -> `0.744 ms` (~20.1%),
  `technical-hatch-clipping.pdf` `2.893 ms` -> `2.695 ms` (~6.8%), and
  `vector-stress.pdf` `6.638 ms` -> `6.462 ms` (~2.7%).
- Results on the technical matrix:
  `clipped-paths.pdf` p95 improved `0.632 ms` -> `0.407 ms` (~35.6%),
  `schematic-symbol-grid.pdf` `0.507 ms` -> `0.321 ms` (~36.7%),
  `technical-linework-dimensions.pdf` `0.929 ms` -> `0.741 ms` (~20.2%),
  `technical-repeated-symbols.pdf` `0.999 ms` -> `0.823 ms` (~17.6%),
  `engineering-large-transform-detail.pdf` `2.545 ms` -> `2.275 ms`
  (~10.6%), and `technical-large-coordinate-plan.pdf` `2.147 ms` ->
  `1.984 ms` (~7.6%).
- Repeat protection runs:
  `target/performance-matrix-blend-fastpath-image-repeat-after.json` kept the
  image-heavy set neutral to better, with `image-heavy-repeated-xobject-report`
  improving about 33.4% p95 and `image-mask-logo` about 56.2% p95.
  `target/performance-matrix-blend-fastpath-mobile-repeat-after.json` kept the
  same expected three unsupported image-codec fallback records and no errors.
  The only notable watch item was `mobile-rotated-camera-scan.pdf`, which was
  about 4-6% slower in that repeat; it does not exercise the accepted opaque
  normal-blend branch directly, so keep it as a noise/watch fixture in the next
  mobile run instead of treating it as proof of an image-path regression.
- Decision: accept. This is a profile-backed scalar fast path with broad wins
  across path-heavy, text-overlay, and mixed fixtures. It also demonstrates why
  the plan should allow cumulative 5% wins when they are well isolated and
  protection-set-clean.
- Validation:
  `cargo fmt --all --check`, `cargo check --workspace --no-default-features`,
  `cargo test --workspace --no-default-features`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Hairline/clip regression guard from 2026-06-29:

- Change: added `rasterize_paths_should_clip_snapped_axis_aligned_hairlines`
  to protect the interaction between active clipping paths and snapped
  axis-aligned hairlines.
- Purpose: future hairline or simple-stroke fast paths must preserve clip
  bounds and sample coverage. This is a correctness guard for Phase 2, not a
  speed claim.
- Validation: targeted `ferrugo-render` hairline tests passed.

Rect-fill fast-path status from 2026-06-29:

- Existing code already routes simple axis-aligned filled rectangles through
  `axis_aligned_rect_fill_bounds` and `fill_axis_aligned_rect_path`, with
  targeted coverage in `fill_path_should_center_sample_large_axis_aligned_rectangles`.
- Additional change tested locally but not kept: skip the
  `point_in_active_clips` call inside the rect-fill loop when the active clip
  stack is empty.
- Result:
  `target/performance-matrix-report-vector-unclipped-rect-fill-candidate.json`
  vs `target/performance-matrix-report-vector-axis-image-protection.json` showed
  `technical-hatch-clipping` p95 `2.920 ms` -> `2.820 ms` (~3.4%), while
  `vector-stress`, `technical-linework-dimensions`, and
  `prepress-trim-bleed-marks` were effectively neutral.
- Decision: reverted. The candidate is below the 5% threshold and should not be
  treated as a meaningful standalone or cumulative optimization.

Rejected axis-aligned hairline candidate from 2026-06-30:

- Change tested locally but not kept: when snapped hairline strokes had no
  joins, used butt caps, and every segment was axis-aligned, route the stroke
  through a simpler centerline pixel predicate instead of the generic
  `point_in_stroke`/join path.
- Rationale: this targeted the still-open Phase 2 work item for
  axis-aligned hairlines and simple strokes. The candidate was intentionally
  narrow so dashed segments, joins, round/square caps, and clipped pixels would
  continue to use existing semantics.
- Candidate artifacts:
  `target/performance-matrix-axis-hairline-candidate-technical.json` and
  `target/performance-matrix-axis-hairline-candidate-technical-repeat.json`,
  native hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge
  160`.
- Repeat result against
  `target/performance-matrix-blend-fastpath-technical-repeat-after.json`:
  small p95 gains appeared on `clipped-paths.pdf` (~9.3%),
  `dashed-stroke.pdf` (~9.8%), and `engineering-schematic-symbols.pdf`
  (~9.3%), but mean gains were only about 2-4%. The heavier target fixtures
  improved much less: `vector-stress.pdf` p95 ~2.3%, `technical-hatch-clipping`
  ~1.8%, `engineering-floorplan-precision` ~1.3%, and
  `technical-linework-dimensions` ~1.1%.
- Decision: reverted. The branch adds specialized stroke code but does not move
  the actual heavy fixtures enough to justify the extra surface area. Revisit
  simple-stroke work only with a profile that points to a broader stroke
  predicate or bounds strategy, not this narrow centerline branch.

Post-blend stroke profile and rejected bounded-line candidate from 2026-06-30:

- Profile evidence:
  `target/sample-post-blend-report-vector.txt`, captured from a long
  report/vector hot-render run after the opaque normal-blend fast path, showed
  `stroke_path` even more clearly as the dominant CPU target. `stroke_path`
  accounted for about `5750` samples, while `blend_pixel` was about `766`,
  `fill_path` about `187`, `source_over` about `32`, and
  `flatten_path_segments` about `6`.
- Interpretation: allocation and flattening are not the current vector
  bottleneck. The next accepted vector win needs to reduce the amount of stroke
  predicate work per candidate pixel or change the raster algorithm, not add a
  cache around flattening.
- Change tested locally but not kept: precompute per-line pixel bounds for
  strokes with many segments, then check integer pixel bounds before the
  expensive distance-to-line predicate. Variants tested thresholds of `>=8`,
  `>=32`, and `>=64` stroke segments before enabling the precomputed-bounds
  path.
- Candidate artifacts:
  `target/performance-matrix-bounded-stroke-candidate-technical.json`,
  `target/performance-matrix-bounded-stroke-candidate-technical-repeat.json`,
  `target/performance-matrix-bounded-stroke-threshold32-technical.json`,
  `target/performance-matrix-bounded-stroke-threshold32-technical-repeat.json`,
  `target/performance-matrix-bounded-stroke-threshold64-technical.json`, and
  `target/performance-matrix-bounded-stroke-threshold64-technical-repeat.json`.
- Result: the lower thresholds produced useful wins on several large linework
  fixtures, but were not protection-set-neutral. `>=32` improved
  `engineering-floorplan-precision.pdf` by about 13% p95 and
  `technical-large-coordinate-plan.pdf` by about 12% p95 in repeat runs, but
  regressed `technical-hatch-clipping.pdf` by about 5-7% and
  `technical-repeated-symbols.pdf` by about 5%. `>=64` reduced the wins and
  still left repeat mean regressions above 5% on protection fixtures.
- Decision: reverted. The concept is directionally useful, but a per-stroke
  vector of bounded lines is too blunt. Revisit with row buckets, tiling, or a
  stroke-specific spatial index only if the profile remains dominated by
  repeated line predicate scans and the protection fixtures are included from
  the start.

Accepted stroke row-bucket result from 2026-06-30:

- Change: strokes with at least `32` flattened segments now build a temporary
  row-bucket index for the current raster pass. Each bucket stores only the
  stroke lines whose conservative pixel bounds overlap that device row, so the
  inner pixel loop avoids scanning every line segment for every candidate
  sample. Small strokes stay on the existing direct scan to avoid allocation
  overhead.
- Rationale: this is the narrower spatial-index form suggested by the rejected
  bounded-line experiment. It reduces repeated line predicate scans instead of
  adding another check in front of the same full scan. The data structure is
  request-local, dependency-free, and uses safe `Vec`/slice traversal only.
- Candidate artifacts:
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-technical.json`,
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-technical-repeat.json`,
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-starter.json`, and
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-image.json`.
- Correctness note: the row-bucket scan must continue after a candidate line
  misses the current X coordinate because later lines in the same row may still
  cover the pixel. `stroke_row_buckets_should_continue_after_x_miss` protects
  that case.
- Repeat technical result against
  `target/performance-matrix-blend-fastpath-technical-repeat-after.json`:
  `vector-stress.pdf` p95 improved `6.378 ms` -> `5.317 ms` (~16.6%) and mean
  `6.179 ms` -> `5.144 ms` (~16.8%). `dashed-stroke.pdf` improved p95 ~9.8%,
  `clipped-paths.pdf` improved p95 ~9.1%, and
  `engineering-floorplan-precision.pdf` improved mean ~5.4%. Protection
  fixtures stayed inside the acceptance band: `technical-hatch-clipping.pdf`
  was neutral to slightly better, `technical-repeated-symbols.pdf` was about
  3% slower, and `technical-large-coordinate-plan.pdf` was about 4.4% slower
  p95 / 2.7% slower mean.
- Starter protection result:
  `vector-stress.pdf` improved p95 ~17.2% and mean ~18.6%;
  `technical-hatch-clipping.pdf` improved p95 ~1.3%; browser/office fixtures
  improved about 5-7% p95. `technical-linework-dimensions.pdf` was about 4.6%
  slower p95 with mean neutral, still inside the protection threshold.
- Image-heavy protection result:
  all 8 records rendered with no errors or fallback. Image fixtures were
  neutral to better aside from `dct-image.pdf`, which moved by about -4.8% p95
  and is below the noise threshold for this change.
- Rejected follow-up: sorting each row's line indices by `min_x` and breaking
  early was tested in
  `target/performance-matrix-stroke-row-buckets-sorted-technical.json` and
  `target/performance-matrix-stroke-row-buckets-sort128-technical.json`.
  Although some large plan fixtures improved, protection fixtures regressed
  badly (`clipped-paths.pdf` and several engineering fixtures), so the sorted
  variant was reverted.
- Decision: accept. This is the first post-blend stroke-path win that moves the
  dominant `vector-stress` fixture by more than 10% while keeping the previous
  protection regressions out of the repeat run.
- Validation:
  `cargo fmt --all --check`, `git diff --check -- crates/ferrugo-render/src/lib.rs docs/plans/2026-06-29-performance-optimization-working-plan.md`,
  `cargo check --workspace --no-default-features`,
  `cargo test --workspace --no-default-features`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Post-row-bucket profile from 2026-06-30:

- Profile evidence:
  `target/sample-post-row-buckets-xmiss-report-vector.txt`, captured from a
  long native report/vector hot-render run after the corrected row-bucket scan,
  still shows `stroke_path` as the dominant CPU target. Top-of-stack samples
  were: `stroke_path` `5813`, `blend_pixel` `684`, `fill_path` `237`,
  `source_over` `38`, `fill_device_rect` `26`, `draw_text_run` `7`, and
  `flatten_path_segments` `6`.
- Interpretation: the accepted row-bucket index reduced work, but the next
  high-value vector block is still stroke rasterization. Flatten-once path reuse
  and allocation tweaks are not currently supported by this CPU profile for the
  report/vector hot set. The next code candidate should either reduce candidate
  line checks further or change the stroke raster algorithm for dense linework,
  with `technical-repeated-symbols`, `technical-large-coordinate-plan`, and
  `technical-linework-dimensions` kept in the protection set from the first run.

Rejected row/X-tile bucket candidate from 2026-06-30:

- Change tested locally but not kept: split each row bucket into coarse 16px
  X-tiles so a pixel would only scan line indices overlapping its row and tile.
  A second variant enabled the X-tile path only for strokes with at least 128
  bounded lines and low tile-index duplication.
- Candidate artifacts:
  `target/performance-matrix-stroke-row-x-tiles-technical.json` and
  `target/performance-matrix-stroke-row-x-tiles-threshold-technical.json`,
  native hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge
  160`.
- Raw X-tile result against
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-technical-repeat.json`:
  large engineering fixtures improved substantially
  (`engineering-floorplan-precision.pdf` p95 ~31.9%,
  `engineering-large-transform-detail.pdf` p95 ~25.3%, and
  `technical-large-coordinate-plan.pdf` p95 ~35.1%), but protection fixtures
  regressed too much: `clipped-paths.pdf` p95 ~13.3% slower,
  `technical-hatch-clipping.pdf` p95 ~11.6% slower,
  `engineering-schematic-symbols.pdf` p95 ~12.7% slower, and
  `technical-repeated-symbols.pdf` p95 ~6.4% slower.
- Thresholded result: the conservative gating removed the large wins and left
  mostly overhead, with `vector-stress.pdf` p95 ~3.7% slower,
  `technical-linework-dimensions.pdf` p95 ~9.5% slower, and no accepted target
  improvement.
- Decision: reverted. Coarse X-tiling is directionally useful for some large
  engineering drawings, but the current design adds build/query overhead and
  duplicates long-line indices in ways that hurt the protection set. Revisit
  only with fixture-level stroke-shape histograms or a cheaper spatial index
  that can choose between row-only and tiled indexing without extra work on the
  row-only path.

Stroke-shape trace diagnostics from 2026-06-30:

- Change: `trace-native` now includes a numeric `stroke_shape_summary` for the
  rendered page. The summary is request-local, contains no PDF bytes, text,
  image samples, or rendered pixels, and is only collected in the explicit trace
  path. Normal rendering and benchmark paths do not call it.
- Reported fields include stroked item counts, dashed item counts,
  row-bucket-candidate items, flattened line totals, axis-aligned line totals,
  row-index references, max lines per item, max row-index references per item,
  line-count buckets (`<32`, `32-127`, `>=128`), and conservative device-pixel
  X-span buckets (`<=16`, `<=32`, `<=64`, `>64`).
- First fixture evidence:
  `target/trace-native-vector-stress-stroke-shapes.json`, generated with
  `trace-native fixtures/generated/vector-stress.pdf --max-edge 160`, reports
  `66` stroked items, `252` flattened lines, `2` row-bucket candidates,
  `5688` row-index references, max `64` lines per item, `124` axis-aligned
  lines, `64` items below 32 lines, `2` items in the 32-127 line bucket, and no
  `>=128` line-count items. Pixel X-span buckets were `<=16`: `234`, `<=32`:
  `0`, `<=64`: `0`, `>64`: `18`.
- Interpretation: `vector-stress` is dominated by a few medium-sized stroke
  items plus many tiny strokes, not by very large single stroke items. The
  rejected coarse X-tile path was therefore too broad. The next spatial-index
  attempt should first collect the same summary for the protection fixtures and
  choose a shape-specific strategy, for example medium-stroke short-span
  handling, instead of a universal row/X-tile split.

Technical protection-set stroke-shape sweep from 2026-06-30:

- Artifacts: `target/trace-stroke-shapes-*.json`, generated with
  `trace-native <fixture> --max-edge 160 --max-events 1` for all 11 fixtures in
  `fixtures/technical-drawing-manifest.tsv`.
- Aggregate: `648` stroked items, `7330` flattened lines, `70`
  row-bucket-candidate items, and `59392` row-index references. Of those
  flattened lines, `6930` were axis-aligned. Line-count buckets were `578`
  items below 32 lines, `70` items in the 32-127 bucket, and `0` items at
  `>=128`. Pixel X-span buckets were `<=16`: `7080`, `<=32`: `90`, `<=64`:
  `48`, `>64`: `112`.
- Highest row-index-reference fixtures:
  `engineering-floorplan-precision.pdf` (`2174` lines, `24` row-bucket
  candidates, `13278` row refs), `engineering-large-transform-detail.pdf`
  (`1618`, `16`, `9624`), `technical-hatch-clipping.pdf` (`114`, `0`, `9004`),
  `technical-large-coordinate-plan.pdf` (`1380`, `14`, `8314`), and
  `technical-linework-dimensions.pdf` (`1268`, `14`, `8042`).
- Interpretation: the protection set is not dominated by huge single strokes;
  it is dominated by many small/medium strokes and overwhelmingly
  axis-aligned line segments. This made a broad axis-aligned stroke predicate
  inside the existing row-bucket/direct scans worth testing before revisiting
  any heavier spatial index.

Rejected candidate from 2026-06-30:

- Change tested locally but not kept: add a broad horizontal/vertical stroke
  hit-test shortcut inside `point_in_single_stroke_line`, covering butt, round,
  and square caps before falling back to the generic projection-based
  predicate.
- Baseline:
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-technical-repeat.json`,
  native hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge
  160`, 200 measured iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-axis-stroke-predicate-technical.json`, same
  command shape and host.
- Result: some large linework fixtures improved (`engineering-floorplan-precision.pdf`
  p95 ~6.0%, `engineering-large-transform-detail.pdf` ~6.9%, and
  `technical-large-coordinate-plan.pdf` ~5.3%), but the primary
  `vector-stress.pdf` target was effectively neutral (~0.4%), the family
  average was ~0.1%, and `clipped-paths.pdf` regressed on p95 in the local run.
- Decision: reverted. The result is not strong enough for a standalone win, is
  not protection-set-neutral, and overlaps with an earlier rejected simple
  stroke shortcut. The next vector attempt should use a deeper profile of the
  remaining `stroke_path` work instead of another local distance-predicate
  micro-fast-path.

Current profile after rejecting the axis predicate:

- Artifact: `target/sample-vector-stress-after-axis-reject.txt`, a 10-second
  macOS `sample` run against a long release `benchmark-native` process for
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`.
- Top-of-stack summary: `stroke_path` `6238` samples, `fill_path` `539`,
  `blend_pixel` `207`, `source_over` `117`, `RasterDevice::pixel` `25`,
  `flatten_path_segments` `8`.
- Interpretation: parser, flattening, image decode, and generic allocation
  churn are not the dominant next target for this fixture. The next accepted
  vector optimization should reduce work inside `stroke_path` itself, most
  likely by changing candidate-pixel/line reduction or the dense linework
  raster algorithm rather than adding another local predicate shortcut.

Rejected micro candidate from 2026-06-30:

- Change tested locally but not kept: compute `radius * radius` once per
  `stroke_path` call and pass it into direct and row-bucketed stroke predicates
  instead of recomputing it inside each sample hit test.
- Baseline:
  `target/performance-matrix-stroke-row-buckets-xmiss-fix-technical-repeat.json`,
  native hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge
  160`, 200 measured iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-stroke-radius-squared-technical.json`, same
  command shape and host.
- Result: no fixture reached the repeated 5% p95 threshold.
  `vector-stress.pdf` improved only ~0.4% p95 and regressed ~0.8% mean,
  `engineering-large-transform-detail.pdf` regressed ~2.6% p95/mean, and the
  family p95 average moved only ~0.2%.
- Decision: reverted. The compiler and surrounding hot loop already make this
  scalar arithmetic too small to matter. Continue with algorithmic
  candidate-reduction or raster-loop changes, not scalar expression hoisting.

Rejected row-bucket copy candidate from 2026-06-30:

- Change tested locally but not kept: borrow each `BoundedStrokeLine` in
  `point_in_row_bucketed_stroke` until after the X-bounds check, avoiding a
  line/bounds copy for common row-bucket X misses.
- Technical target artifacts:
  `target/performance-matrix-stroke-row-borrow-technical.json` and
  `target/performance-matrix-stroke-row-borrow-technical-repeat.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Technical result: useful but narrow. `technical-large-coordinate-plan.pdf`
  improved p95 ~8.7% then ~7.5% on repeat, and mean ~7.4% then ~6.6%.
  `engineering-large-transform-detail.pdf` improved p95 ~6.5% in the first
  run but only ~4.0% on repeat. `vector-stress.pdf` stayed neutral/slightly
  slower.
- Starter protection artifacts:
  `target/performance-matrix-stroke-row-borrow-starter.json` and
  `target/performance-matrix-stroke-row-borrow-starter-repeat.json`, native
  hot-render, `fixtures/performance-matrix-manifest.tsv`, `--max-edge 160`,
  100 measured iterations after 10 warmups.
- Starter result: not protection-set-neutral. `technical-linework-dimensions.pdf`
  improved p95 ~6-7%, but `browser-chromium-article-print.pdf` regressed p95
  ~6.1% and then ~12.2% on repeat, while mean was only ~2.3% slower.
- Decision: reverted. Borrow-before-copy is directionally reasonable for
  large linework but too small and noisy in the broader starter set. Revisit
  row-bucket internals only with a larger structural reduction in candidate
  checks, not as an isolated copy-order tweak.

Current profile refresh from 2026-06-30:

- Artifact: `target/sample-vector-stress-current-refresh.txt`, a 10-second
  macOS `sample` run against a long release `benchmark-native` process for
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`.
- Top-of-stack summary: `stroke_path` `3866` samples,
  `point_in_row_bucketed_stroke` `948`, `fill_path` `847`, `blend_pixel` `314`,
  `source_over` `200`, and allocator/memmove/free symbols visible but much
  smaller.
- Trace artifacts:
  `target/trace-vector-stress-current-refresh.json` and
  `target/trace-engineering-floorplan-current-refresh.json`.
- Interpretation: the current head still points at row-bucketed stroke
  candidate scanning as the main vector bottleneck. Flattening remains too
  small to justify a flatten-cache change. Allocation churn is visible, but
  should be attacked only when the structural change also preserves the
  protection set.

Rejected compact row-bucket construction candidate from 2026-06-30:

- Change tested locally but not kept: build stroke and join row buckets with a
  two-pass count/offset/index layout instead of `Vec<Vec<usize>>` per raster
  row, while preserving row order and the final flat `rows`/`indices` layout.
- Rationale: the fresh `sample` profile still showed allocator activity under
  `stroke_path`, and the existing bucket builders allocate many small row
  vectors before flattening them.
- Baseline:
  `target/performance-matrix-row-bucket-compact-baseline.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Candidate:
  `target/performance-matrix-row-bucket-compact-candidate.json`, same command
  shape and host.
- Result: not protection-set-neutral and too small overall.
  `technical-large-coordinate-plan.pdf` improved p95 ~3.8%,
  `technical-hatch-clipping.pdf` ~3.4%, and `vector-stress.pdf` ~1.0%, but
  `clipped-paths.pdf` regressed p95 ~6.5% and
  `technical-repeated-symbols.pdf` / `engineering-schematic-symbols.pdf`
  regressed ~3.0%. The family average moved only about -0.7% p95 and +0.5%
  mean.
- Decision: reverted. Row-bucket allocation cleanup alone is not enough; the
  next candidate should reduce `point_in_row_bucketed_stroke` work directly
  rather than only changing the temporary construction strategy.

Stroke row-work trace diagnostics from 2026-06-30:

- Change: `trace-native` stroke summaries now include estimated row-bucket
  sample-line checks, X-bound hits, X-bound misses, and max estimated
  sample-line checks per stroked item. These are derived from conservative
  device line bounds and stroke bounds; they do not inspect rendered pixels,
  text, images, or PDF stream bytes.
- Purpose: the macOS `sample` profile points at `stroke_path`, but not at the
  internal reason. These counters expose whether row-bucket work is dominated
  by useful line predicates or by X-bound rejection before another spatial
  index is attempted.
- Artifacts:
  `target/trace-native-vector-stress-row-work.json`,
  `target/trace-native-technical-large-coordinate-row-work.json`, and
  `target/trace-native-engineering-floorplan-row-work.json`, generated with
  `trace-native <fixture> --max-edge 160 --max-events 1`.
- Observed estimates:
  `vector-stress.pdf` reports `485376` row-bucket sample refs, `25672`
  X-bound hits, and `459704` X-bound misses (~94.7% X-miss);
  `technical-large-coordinate-plan.pdf` reports `2085696` refs, `96832` hits,
  and `1988864` misses (~95.4% X-miss);
  `engineering-floorplan-precision.pdf` reports `2872320` refs, `144960` hits,
  and `2727360` misses (~95.0% X-miss).
- Interpretation: the next substantial stroke win should reduce X-miss-heavy
  row-bucket scans structurally. The previously rejected 16px X-tile path
  proved the direction can help large plans but was too coarse and too costly
  for protection fixtures. A better next candidate should use the new counters
  to gate a narrower row subdivision or line-span grouping only where the
  estimated X-miss ratio and per-item sample refs justify the overhead.

Technical row-work sweep from 2026-06-30:

- Artifacts: `target/trace-row-work-*.json`, generated from every fixture in
  `fixtures/technical-drawing-manifest.tsv`.
- Highest estimated row-bucket sample refs:
  `engineering-floorplan-precision.pdf` `2872320` refs / `95.0%` X-miss,
  `engineering-large-transform-detail.pdf` `2131200` / `95.4%`,
  `technical-large-coordinate-plan.pdf` `2085696` / `95.4%`,
  `vector-stress.pdf` `485376` / `94.7%`, and
  `technical-linework-dimensions.pdf` `388080` / `94.9%`.
- Fixtures with no row-bucket sample refs in this sweep:
  `technical-hatch-clipping.pdf`, `technical-repeated-symbols.pdf`,
  `engineering-schematic-symbols.pdf`, `dashed-stroke.pdf`,
  `clipped-paths.pdf`, and `user-unit-page.pdf`.
- Next threshold to test: only enable any row X-subdivision candidate for
  strokes/items that exceed roughly `1_000_000` estimated row-bucket sample
  refs and have an X-miss ratio above `90%`. That should target the three
  large linework/plan fixtures while leaving `vector-stress`,
  `technical-linework-dimensions`, and the zero-row-bucket protection fixtures
  on the accepted row-bucket path.

Rejected gated sorted-row candidate from 2026-06-30:

- Change tested locally but not kept: when the row-work estimate exceeded
  `1_000_000` sample-line refs and `90%` X-miss, sort each affected row bucket
  by line `min_x` and break the row scan once `min_x > x`.
- Rationale: this tried to keep the previously rejected row-sort idea away
  from `vector-stress`, `technical-linework-dimensions`, and zero-row-bucket
  protection fixtures while reducing the high X-miss work identified by the
  row-work sweep.
- Candidate artifact:
  `target/performance-matrix-stroke-row-sorted-gated-technical.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Result: failed immediately on the technical target set.
  `engineering-floorplan-precision.pdf` regressed p95 ~6.1% and mean ~5.4%;
  `technical-large-coordinate-plan.pdf` regressed p95 ~1.5%; and no target
  fixture reached a 5% p95 improvement. `vector-stress.pdf` stayed neutral, but
  the intended large-plan gains did not materialize.
- Decision: reverted without running the broader starter protection set. Plain
  per-row sort/break is not the right structural reduction. The next attempt
  should avoid per-row sorting overhead and instead test a lower-overhead
  representation, such as per-row span groups, compact x-ranges, or a stroke
  raster algorithm that handles dense horizontal/vertical linework in batches.

Rejected gated X-tile candidate from 2026-06-30:

- Change tested locally but not kept: build optional 16px X-tiles inside
  `StrokeRowBuckets` only when the row-work estimate exceeded `1_000_000`
  sample-line refs and `90%` X-miss. Standard row buckets remained unchanged
  below that gate.
- Rationale: the earlier raw X-tile experiment improved the largest plan
  fixtures but damaged protection fixtures. The new gate used row-work trace
  counters to activate tiles only on the high-work plan/engineering cases.
- Candidate artifact:
  `target/performance-matrix-stroke-row-xtile-gated-technical.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Result: rejected by the first technical run. No fixture reached a 5% p95 win.
  `engineering-floorplan-precision.pdf` regressed p95 ~1.0% and mean ~0.8%,
  `engineering-large-transform-detail.pdf` regressed p95 ~1.9%, and
  `technical-large-coordinate-plan.pdf` regressed p95 ~0.6%. `vector-stress.pdf`
  regressed p95 ~3.7% and mean ~5.0%.
- Decision: reverted. The fixed-width tile index adds allocation and lookup
  overhead without reducing enough work on the gated target set. Future work
  should stop iterating on row-bucket index variants until a different stroke
  raster strategy is scoped, for example batching dense axis-aligned linework
  spans or rasterizing technical-grid strokes as coverage intervals.

Current engineering floorplan profile from 2026-06-30:

- Profile evidence:
  `target/sample-engineering-floorplan-current.txt`, captured from a 10-second
  macOS `sample` run against a long release `benchmark-native` process for
  `fixtures/generated/engineering-floorplan-precision.pdf`, `--max-edge 160`.
- Trace evidence:
  `target/trace-native-engineering-floorplan-current.json` reports total
  `3.223 ms`, with `raster_paths` at `2.762 ms`. The stroke summary reports
  `72` stroked items, `62` dashed items, `24` row-bucket candidates,
  `2174` flattened lines, all `2174` axis-aligned, and `2,872,320`
  row-bucket sample refs with about `95.0%` X-miss.
- Top-of-stack summary: `stroke_path` remains dominant (`6632` symbol samples),
  with `blend_pixel` (`189`), `source_over` (`51`), and `fill_path` (`46`) far
  behind. Allocation frames appear inside `stroke_path` and
  `dashed_subpath_line_segments`, but the first direct allocation candidates
  below did not improve the matrix.
- Interpretation: the large engineering fixtures still need a different
  stroke raster strategy. The profile does not justify more local empty-check
  or `Vec`-reservation micro-optimizations; those have now been tested and
  rejected.

Rejected empty-join hot-loop candidate from 2026-06-30:

- Change tested locally but not kept: compute `has_joins = !joins.is_empty()`
  in `stroke_path` and skip `point_in_join` in the inner sample loop when a
  stroked item has no joins. This targeted dashed floorplan strokes, where the
  dash path passes an empty join slice.
- Baseline artifact:
  `target/performance-matrix-empty-join-baseline-technical.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Candidate artifact:
  `target/performance-matrix-empty-join-candidate-technical.json`, same command
  shape and host.
- Result: rejected. No fixture reached a 5% p95 win.
  `engineering-floorplan-precision.pdf` regressed p95 ~3.5% and mean ~3.8%,
  `technical-large-coordinate-plan.pdf` regressed p95 ~3.8% and mean ~4.8%,
  `dashed-stroke.pdf` regressed p95 ~14.5%, and `clipped-paths.pdf` regressed
  p95 ~50.6%.
- Decision: reverted. The branch avoided an apparently redundant empty join
  call, but the hot loop got worse in protection fixtures. Do not revisit this
  exact check without lower-level evidence from generated assembly or a
  profile that isolates `point_in_join` itself as a measurable cost.

Rejected dashed-line capacity candidate from 2026-06-30:

- Change tested locally but not kept: initialize dashed stroke output with
  `Vec::with_capacity(source_segments * active_dash_segments)`, capped by the
  existing path complexity limit, to reduce reallocations visible in the
  engineering floorplan sample.
- Candidate artifact:
  `target/performance-matrix-dashed-capacity-candidate-technical.json`, compared
  against `target/performance-matrix-empty-join-baseline-technical.json` with
  the same native hot-render technical command.
- Result: rejected. Reserving capacity regressed nearly the whole technical
  set: `engineering-floorplan-precision.pdf` p95 ~7.1% slower,
  `technical-large-coordinate-plan.pdf` ~12.9% slower,
  `technical-linework-dimensions.pdf` ~19.1% slower, and `vector-stress.pdf`
  ~7.7% slower.
- Decision: reverted. The allocation frames in `sample` are not enough by
  themselves to justify pre-reservation. The extra capacity likely increases
  allocation/cache pressure more than it removes growth cost for these short
  dashed stroke buffers.

Span-raster eligibility diagnostics from 2026-06-30:

- Change: `trace-native` stroke summaries now report snapped-hairline item
  counts, all-axis-aligned item counts, joinless-axis-aligned item counts, and
  plausible future span-raster candidate item/line counts. The counters are
  trace-only and do not affect normal rendering or benchmark paths.
- Purpose: before building a new stroke raster path, measure whether the
  obvious narrow version, snapped joinless axis-aligned hairlines, would cover
  enough real linework to matter.
- Artifacts:
  `target/trace-span-eligibility-engineering-floorplan.json`,
  `target/trace-span-eligibility-engineering-large-transform.json`,
  `target/trace-span-eligibility-technical-large-coordinate.json`, and
  `target/trace-span-eligibility-vector-stress.json`, generated with
  `trace-native <fixture> --max-edge 160 --max-events 1`.
- Result: the narrow snapped-hairline span candidate is not broad enough for
  the remaining large-linework bottleneck. `engineering-floorplan-precision.pdf`
  had only `6` span-candidate items and `50/2174` candidate lines;
  `engineering-large-transform-detail.pdf` had `6/1618` candidate lines;
  `technical-large-coordinate-plan.pdf` had `6/1380`; `vector-stress.pdf` had
  no snapped span candidates.
- Decision: keep the diagnostics, but do not implement a snapped-hairline-only
  span rasterizer as the next optimization block. A useful structural stroke
  change must cover general thin axis-aligned linework, including non-snapped
  dashed lines and medium row-bucket candidates, while preserving single-blend
  union coverage semantics.

Accepted axis-aligned span-row result from 2026-06-30:

- Change: joinless axis-aligned strokes with at least `32` flattened lines now
  build temporary per-supersample-row X spans instead of scanning row-bucket
  line indices for every candidate sample. The spans are merged per sample row
  and preserve the existing clip check, coverage counting, and once-per-pixel
  blend behavior. Non-axis-aligned strokes, stroked joins, and small strokes
  still use the existing row-bucket or generic path.
- Rationale: the large engineering and plan fixtures are dominated by
  axis-aligned linework with very high row-bucket X-miss rates. The earlier
  X-tile and sorted-row candidates reduced some of that work but added too much
  index overhead. Span rows remove the X-miss scan for the shape class that
  actually dominates those documents.
- Baseline artifact:
  `target/performance-matrix-empty-join-baseline-technical.json`, native
  hot-render, `fixtures/technical-drawing-manifest.tsv`, `--max-edge 160`, 200
  measured iterations after 10 warmups.
- Candidate artifacts:
  `target/performance-matrix-axis-span-candidate-technical.json`,
  `target/performance-matrix-axis-span-candidate-technical-repeat.json`,
  `target/performance-matrix-axis-span-candidate-starter.json`, and
  `target/performance-matrix-axis-span-candidate-starter-repeat.json`.
- Technical repeat result: `technical-large-coordinate-plan.pdf` p95 improved
  `1.973 ms` -> `1.131 ms` (~42.7%) and mean ~45.4%;
  `engineering-large-transform-detail.pdf` p95 improved ~40.8%;
  `engineering-floorplan-precision.pdf` p95 improved ~40.7%; and
  `technical-linework-dimensions.pdf` p95 improved ~22.6%. `vector-stress.pdf`
  regressed p95 ~2.0%, `technical-hatch-clipping.pdf` regressed p95 ~3.5%,
  and `dashed-stroke.pdf` regressed p95 ~10.9% / mean ~6.0%. The dashed-stroke
  fixture is sub-0.1 ms and does not exercise the target large-linework path,
  so keep it as a watch item in the next technical repeat rather than rejecting
  the block.
- Starter protection result: the first starter run had p95-only regressions on
  `browser-chromium-article-print.pdf` and `scanned-page.pdf`, but traces show
  the span path does not activate for those fixtures (`browser` has only `11`
  stroke lines, below the `32` line gate; `scanned-page` has no strokes). The
  repeat starter run removed those regressions: no fixture regressed by 5% p95,
  while `technical-linework-dimensions.pdf` improved p95 ~25.7% and mean
  ~23.7%.
- Correctness guards: `axis_stroke_spans_should_match_generic_axis_strokes`
  compares span membership against the existing generic stroke predicate over
  a supersampled raster window, and
  `round_axis_stroke_span_should_shrink_beyond_vertical_endpoint` protects
  round-cap span narrowing.
- Decision: accept. This is the first structural stroke raster change after
  row buckets that repeatedly moves the large engineering/plan fixtures by
  more than 40% without broad protection-set regression.
- Validation:
  `cargo fmt --all --check`,
  `cargo check -p ferrugo-render --no-default-features`,
  `cargo test -p ferrugo-render axis_stroke --no-default-features`,
  `cargo test --workspace --no-default-features`,
  `cargo clippy -p ferrugo-render --all-targets --no-default-features -- -D warnings`,
  `cargo clippy -p ferrugo-cli --all-targets --all-features -- -D warnings`,
  and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Post-span image and mobile impact from 2026-06-30:

- Follow-up artifacts:
  `target/performance-matrix-post-span-image-heavy.json` and
  `target/performance-matrix-post-span-mobile.json`, native hot-render,
  `--max-edge 160`.
- Image-heavy result compared with
  `target/performance-matrix-image-row-color-repeat.json`: all 8 image-heavy
  fixtures improved on p95. `image-heavy-repeated-xobject-report.pdf` improved
  ~33.2%, `image-heavy-rotated-mask-sheet.pdf` ~27.6%,
  `image-mask-logo.pdf` ~58.5%, `predictor-image.pdf` ~20.9%, and
  `mobile-mixed-compression-scan.pdf` ~7.8%. This confirms that several
  previously image-labeled slow cases were still paying substantial
  axis-aligned stroke overlay cost.
- Mobile result compared with
  `target/performance-matrix-mobile-row-color-repeat.json`: 8 of 9 rendered
  mobile fixtures improved by at least 5% p95, with no p95 regression above
  5%. `cropped-scan-page.pdf` improved ~65.2%,
  `ocr-invisible-text-layer.pdf` ~68.6%, `rotated-office-export.pdf` ~21.1%,
  and `mobile-mixed-compression-scan.pdf` ~5.2%.
- New slowest rendered fixtures after this pass are
  `mobile-cropped-photo-scan.pdf` (`1.338 ms` p95),
  `mobile-mixed-compression-scan.pdf` (`1.001 ms` p95),
  `rotated-office-export.pdf` (`0.818 ms` p95),
  `image-heavy-rotated-mask-sheet.pdf` (`0.593 ms` p95), and
  `image-heavy-repeated-xobject-report.pdf` (`0.589 ms` p95).
- Decision: count these as secondary wins from the accepted span-row patch, not
  a separate image optimization. The next Phase 4 candidate should use fresh
  post-span attribution on the remaining scan/image top fixtures instead of
  relying on pre-span profiles.

Accepted sparse axis-stroke raster result from 2026-06-30:

- Profiling trigger: fresh post-span traces for the remaining image/mobile top
  fixtures showed that `mobile-cropped-photo-scan.pdf` was still dominated by
  `raster_paths`, not image decode. Its native trace moved from `1.523 ms`
  total / `1.086 ms` `raster_paths` before the change to `0.510 ms` total /
  `0.088 ms` `raster_paths` after the change.
- Change: axis-aligned stroke rasterization now builds sparse per-row X spans
  for the raster loop. Coverage still uses exact line spans plus the existing
  join predicates, so joined rectangle/box strokes avoid scanning their full
  path bounding box without replacing join geometry with an approximation.
- Candidate artifacts:
  `target/performance-matrix-sparse-axis-image-heavy.json`,
  `target/performance-matrix-sparse-axis-mobile.json`, and
  `target/trace-sparse-axis-mobile-cropped-photo-scan.json`.
- Mobile result compared with
  `target/performance-matrix-post-span-mobile.json`:
  `mobile-cropped-photo-scan.pdf` p95 improved `1.338 ms` -> `0.226 ms`
  (~83.1%), `mobile-mixed-compression-scan.pdf` improved `1.001 ms` ->
  `0.228 ms` (~77.2%), and `rotated-office-export.pdf` improved `0.818 ms`
  -> `0.220 ms` (~73.1%). `cropped-scan-page.pdf` improved ~9.3%. Small
  compression/OCR fixtures moved within very small absolute budgets; the
  largest p95 regressions were `dct-image.pdf` ~0.005 ms and
  `predictor-image.pdf` ~0.004 ms.
- Image-heavy result compared with
  `target/performance-matrix-post-span-image-heavy.json`:
  `mobile-mixed-compression-scan.pdf` p95 improved ~76.1%,
  `image-heavy-rotated-mask-sheet.pdf` ~39.3%,
  `image-heavy-repeated-xobject-report.pdf` ~34.0%, and
  `scanner-large-image-budget.pdf` ~31.9%. `soft-mask-image.pdf` regressed
  p95 by ~0.004 ms on a sub-0.1 ms case.
- Correctness guard:
  `axis_stroke_raster_spans_should_cover_joined_axis_strokes` compares sparse
  coverage against the generic stroke-plus-join predicate for a joined
  axis-aligned rectangle.
- Decision: accept. This removes the full-bounding-box raster tax from common
  scan/image frame strokes and produces repeated double-digit wins on the
  profiled top fixtures. Keep the sub-0.1 ms compression/image-mask movements
  as noise/watch items in the next broad matrix rather than blocking this
  block.
- Validation:
  `cargo fmt --all --check`,
  `git diff --check -- crates/ferrugo-render/src/lib.rs docs/plans/2026-06-29-performance-optimization-working-plan.md`,
  `cargo check -p ferrugo-render --no-default-features`,
  `cargo test -p ferrugo-render axis_stroke --no-default-features`,
  `cargo test --workspace --no-default-features`,
  `cargo clippy -p ferrugo-render --all-targets --no-default-features -- -D warnings`,
  and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Accepted empty-join stroke predicate skip from 2026-06-30:

- Profiling trigger: after the sparse axis-stroke win, fresh matrices showed
  `vector-stress.pdf` and `technical-hatch-clipping.pdf` as the remaining
  p95 leaders. `target/sample-vector-repeated-symbols-sparse-axis.txt`
  still showed `stroke_path` as the dominant stack and `point_in_join` as a
  visible inner predicate cost. The focused long run artifact is
  `target/performance-matrix-vector-repeated-symbols-profile-run.json`.
- Change: `stroke_path` now computes whether the flattened/dashed stroke has
  joins once and skips the join predicate entirely when the join slice is
  empty. This keeps the existing stroke and join geometry unchanged while
  avoiding a per-sample always-false branch for dashed/joinless strokes.
- Baseline artifacts:
  `target/performance-matrix-sparse-axis-technical.json` and
  `target/performance-matrix-sparse-axis-starter.json`, native hot-render,
  `--max-edge 160`, 200 measured iterations after 20 warmups.
- Candidate artifacts:
  `target/performance-matrix-join-skip-technical.json`,
  `target/performance-matrix-join-skip-technical-repeat.json`,
  `target/performance-matrix-join-skip-starter.json`, and
  `target/trace-join-skip-technical-hatch-clipping.json`.
- Technical result: `technical-hatch-clipping.pdf` improved p95
  `2.594 ms` -> `2.376 ms` in both candidate runs (~8.4%); mean improved
  ~8.3% in the first run and ~9.7% in the repeat. `clipped-paths.pdf`
  improved p95 ~13.2% in the first run and ~9.0% in the repeat. The top
  `vector-stress.pdf` case improved only ~1-2%, so this is not a
  vector-stress fix.
- Trace confirmation: `technical-hatch-clipping.pdf` moved from `3.450 ms`
  total / `2.465 ms` `raster_paths` in
  `target/trace-sparse-axis-technical-hatch-clipping.json` to `2.576 ms`
  total / `2.163 ms` `raster_paths` after the change.
- Protection result: `target/performance-matrix-join-skip-starter.json`
  rendered all starter fixtures with no fallback-required, missing-tool,
  not-applicable, or error records. Sub-0.1 ms small-text/form movements are
  noise/watch items.
- Decision: accept as a cumulative 5-10% stroke predicate win for the
  hatch/clipping subset. It is too small to claim as a standalone broad vector
  improvement, but it attacks the same path-raster bottleneck with a simple
  zero-dependency branch removal and no protection-set failures.
- Validation:
  `cargo fmt --all --check`,
  `git diff --check -- crates/ferrugo-render/src/lib.rs docs/plans/2026-06-29-performance-optimization-working-plan.md`,
  `cargo check -p ferrugo-render --no-default-features`,
  `cargo test -p ferrugo-render axis_stroke --no-default-features`,
  `cargo test --workspace --no-default-features`,
  and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Accepted join-bucket stroke predicate index from 2026-06-30:

- Profiling trigger:
  `target/sample-vector-stress-profile-refresh.txt`, captured from a long
  release `benchmark-native` run on `vector-stress.pdf`, showed `stroke_path`
  still dominating with about `4358` symbol samples. `fill_path` was about
  `639`, `blend_pixel` about `238`, and `source_over` about `153`. The matching
  trace `target/trace-native-vector-stress-profile-refresh.json` reported
  `4.477 ms` of `4.850 ms` total in `raster_paths`, with `485376`
  row-bucket sample refs and about `95%` X-miss.
- Change: strokes with at least `8` joins now build a conservative per-row
  join index after the axis-span fast path declines the item. The raster loop
  still uses the existing round-join circle and prepared bevel/miter triangle
  predicates; the new index only skips joins whose pixel bounds cannot contain
  the current sample.
- Correctness guard: added focused unit tests comparing the bucketed predicate
  against the existing `point_in_join` predicate for round and miter joins
  across their raster bounds.
- Candidate artifacts:
  `target/performance-matrix-join-buckets-technical.json`,
  `target/performance-matrix-join-buckets-technical-repeat.json`,
  `target/performance-matrix-join-buckets-starter.json`,
  `target/trace-native-vector-stress-join-buckets.json`, and
  `target/trace-native-clipped-paths-join-buckets.json`.
- Technical result against
  `target/performance-matrix-join-skip-technical-repeat.json`:
  `vector-stress.pdf` improved p95 `4.649 ms` -> `3.778 ms` in the repeat
  (~18.7%) and mean `4.488 ms` -> `3.646 ms` (~18.8%). The first candidate run
  measured a similar `~19.6%` p95/mean improvement. `clipped-paths.pdf` had one
  first-run p95 outlier but the repeat was neutral (`0.427 ms` -> `0.430 ms`)
  and its trace confirmed `0` stroked items, so keep it as a watch fixture
  rather than treating it as a code-path regression.
- Protection result:
  `target/performance-matrix-join-buckets-starter.json` rendered all starter
  fixtures with no fallback-required, missing-tool, not-applicable, or error
  records. `vector-stress.pdf` improved p95 `4.696 ms` -> `3.744 ms` (~20.3%)
  and mean `4.559 ms` -> `3.623 ms` (~20.5%). The starter matrix did not show a
  repeated broad regression; small sub-millisecond movements remain watch
  items.
- Decision: accept. This is a profile-backed algorithmic reduction in join
  predicate fanout for the current top vector fixture, with exact geometry
  checks preserved and clean starter protection.
- Validation:
  `cargo fmt --all`,
  `cargo fmt --all --check`,
  `git diff --check -- crates/ferrugo-render/src/lib.rs docs/plans/2026-06-29-performance-optimization-working-plan.md`,
  `cargo check -p ferrugo-render --no-default-features`,
  `cargo test -p ferrugo-render stroke_join_buckets --no-default-features`,
  `cargo test --workspace --no-default-features`,
  and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Accepted simple diagonal stroke span rasterizer from 2026-06-30:

- Profiling trigger:
  `target/sample-technical-hatch-post-join-buckets.txt`, captured from a long
  release `benchmark-native` run on `technical-hatch-clipping.pdf`, showed
  `stroke_path` as the dominant stack after the join-bucket optimization.
  The matching trace `target/trace-native-technical-hatch-post-join-buckets.json`
  reported `2.313 ms` of `3.058 ms` total in `raster_paths`. The stroke shape
  summary showed `92` stroked items, `90` snapped hairline items, `114`
  flattened lines, only `6` axis-aligned items, and no row-bucket candidates.
- Change: single-line non-axis-aligned strokes now build conservative
  sample-row X spans when their clipped pixel bounds cover at least `1024`
  pixels. Rasterization visits only candidate X ranges, but still uses the
  existing exact `point_in_single_stroke_line` predicate, active clip checks,
  alpha, and blend path before writing pixels.
- Scope refinement: an earlier broad candidate also handled axis-aligned
  single-line strokes and used a lower `256` pixel threshold. It kept the hatch
  win but caused repeated p95 regressions on linework-heavy protection
  fixtures. Shape summaries showed those fixtures were almost entirely
  axis-aligned linework already covered by existing axis span and row-bucket
  paths, so the accepted version is limited to non-axis-aligned simple strokes.
- Correctness guard: added
  `simple_line_stroke_raster_spans_should_cover_single_line_strokes`, which
  checks Butt, Round, and Square caps and proves the conservative spans do not
  miss any sample accepted by the generic stroke predicate.
- Baseline control artifacts from the direct predecessor `887184a`:
  `target/performance-matrix-baseline-simple-line-control-technical.json` and
  `target/performance-matrix-baseline-simple-line-control-starter.json`.
- Accepted candidate artifacts:
  `target/performance-matrix-simple-line-spans-diagonal-technical.json`,
  `target/performance-matrix-simple-line-spans-diagonal-technical-repeat.json`,
  `target/performance-matrix-simple-line-spans-diagonal-starter.json`, and
  `target/performance-matrix-simple-line-spans-diagonal-starter-repeat.json`.
- Technical result against the fresh predecessor control:
  `technical-hatch-clipping.pdf` improved p95 `2.466 ms` -> `0.386 ms`
  (~84.3%) in the first accepted run and `2.466 ms` -> `0.376 ms` (~84.8%) in
  the repeat. Mean improved `2.363 ms` -> `0.356 ms` (~84.9%) and then
  `2.363 ms` -> `0.351 ms` (~85.1%). The technical protection fixtures were
  neutral to small-noise after narrowing: the repeat kept
  `technical-linework-dimensions.pdf` to `0.379 ms` -> `0.390 ms` p95 and
  `0.352 ms` -> `0.355 ms` mean, while axis-heavy engineering fixtures were
  neutral.
- Starter protection result:
  `target/performance-matrix-simple-line-spans-diagonal-starter-repeat.json`
  rendered all starter fixtures with no fallback-required, missing-tool,
  not-applicable, or error records. `technical-hatch-clipping.pdf` improved
  p95 `2.482 ms` -> `0.394 ms` (~84.1%) and mean `2.375 ms` -> `0.353 ms`
  (~85.1%). Remaining starter movements were sub-millisecond watch items; the
  only notable p95 percentage was `text-page.pdf` at `0.042 ms` -> `0.046 ms`,
  an absolute `0.004 ms` movement below the meaningful threshold.
- Decision: accept. This is a profile-backed algorithmic reduction for the
  hatch/clipping stroke hot path, preserves exact stroke coverage semantics,
  avoids new dependencies and unsafe code, and keeps the protection set within
  small absolute timing noise.

Accepted row-bucket X-range rasterizer from 2026-06-30:

- Profiling trigger:
  `target/sample-vector-stress-post-diagonal.txt`, captured after the simple
  diagonal stroke span block, still showed `stroke_path` as the dominant stack
  with `5256` symbol samples. The matching trace
  `target/trace-native-vector-stress-post-diagonal.json` reported `3.721 ms`
  of `3.922 ms` total in `raster_paths`. The stroke shape summary showed
  `485376` row-bucket sample refs, `25672` X hits, and `459704` X misses, so
  about `95%` of row-bucket sample checks were known X misses.
- Change: row-bucketed strokes with at least `48` bounded lines now build
  conservative per-row pixel X ranges from line bounds and bucketed join bounds
  before entering the sample loop. The rasterizer still calls the existing
  `point_in_row_bucketed_stroke` and `point_in_join_buckets` predicates before
  blending.
- Scope refinement: an ungated first candidate improved `vector-stress`, but
  small row-bucket fixtures sometimes paid more range/merge overhead than they
  saved. A sample-ref threshold was also tested and proved too high for the
  target fixture. The accepted line-count gate keeps the path on the large
  vector row buckets that motivated the change while leaving smaller linework
  mostly on the previous path.
- Correctness guard: added
  `row_bucket_pixel_x_ranges_should_cover_bucketed_stroke_samples` and
  `join_bucket_pixel_x_ranges_should_cover_bucketed_join_samples` to verify
  that generated candidate ranges cover every sample accepted by the exact
  bucketed stroke and join predicates.
- Candidate artifacts:
  `target/performance-matrix-row-bucket-ranges-linegate-technical.json`,
  `target/performance-matrix-row-bucket-ranges-linegate-technical-repeat.json`,
  `target/performance-matrix-row-bucket-ranges-linegate-starter.json`,
  `target/performance-matrix-row-bucket-ranges-linegate-starter-repeat.json`,
  and `target/trace-native-vector-stress-row-bucket-ranges.json`.
- Technical result against
  `target/performance-matrix-simple-line-spans-diagonal-technical-repeat.json`:
  `vector-stress.pdf` improved p95 `3.862 ms` -> `3.428 ms` (~11.2%) in the
  first accepted run and `3.862 ms` -> `3.439 ms` (~11.0%) in the repeat. Mean
  improved `3.715 ms` -> `3.291 ms` (~11.4%) and then `3.715 ms` -> `3.282 ms`
  (~11.7%). Technical protection fixtures stayed within small absolute timing
  noise; for example `technical-linework-dimensions.pdf` repeat moved p95
  `0.390 ms` -> `0.397 ms` and mean `0.355 ms` -> `0.358 ms`.
- Starter protection result:
  `target/performance-matrix-row-bucket-ranges-linegate-starter-repeat.json`
  rendered all starter fixtures with no fallback-required, missing-tool,
  not-applicable, or error records. `vector-stress.pdf` improved p95
  `3.911 ms` -> `3.423 ms` (~12.5%) and mean `3.761 ms` -> `3.309 ms`
  (~12.0%). Remaining movements were sub-millisecond watch items.
- Decision: accept. This is a profile-backed reduction in candidate pixel
  scanning for the current top vector fixture, with exact geometry predicates
  preserved and no new dependency or unsafe code.

Accepted lower multi-line axis-stroke span threshold from 2026-06-30:

- Profiling trigger:
  `target/sample-vector-stress-post-row-ranges.txt`, captured after the
  row-bucket X-range rasterizer, still showed `stroke_path` as the dominant
  stack with `3844` symbol samples. The matching trace
  `target/trace-native-vector-stress-post-row-ranges-current.json` reported
  `3.266 ms` of `3.477 ms` total in `raster_paths`, while
  `flatten_path_segments` accounted for only `8` samples. That kept the next
  candidate on stroke raster work rather than reusable path flattening.
- Change: lower the axis-aligned stroke span fast-path gate from `32` flattened
  lines to `4`. The existing sparse span rasterizer, exact coverage predicate,
  clipping, alpha, and blend behavior are unchanged. This extends the proven
  axis-aligned span path to medium linework items without routing every
  single-line stroke through the span builder.
- Scope refinement: thresholds `1`, `2`, `4`, and `8` were tested. Threshold
  `1` helped a long single-fixture `vector-stress` run, but caused a repeated
  `technical-repeated-symbols` regression. Threshold `2` kept the large
  technical wins but left more starter p95 movement on office/hatch fixtures.
  Threshold `8` reduced coverage and introduced a double-digit p95 movement on
  `technical-hatch-clipping` in the starter protection set. Threshold `4` was
  the best balance: broad medium-linework gains, neutral `vector-stress`, and
  no double-digit regression in the focused matrices.
- Candidate artifacts:
  `target/performance-matrix-axis-threshold4-technical-repeat.json`,
  `target/performance-matrix-axis-threshold4-starter-repeat.json`,
  plus rejected comparison runs for thresholds `1`, `2`, and `8`.
- Technical result against
  `target/performance-matrix-row-bucket-ranges-linegate-technical-repeat.json`:
  `engineering-floorplan-precision.pdf` improved p95 `1.472 ms` -> `0.726 ms`
  (~50.7%) and mean `1.363 ms` -> `0.650 ms` (~52.3%).
  `engineering-large-transform-detail.pdf` improved p95 `1.071 ms` ->
  `0.572 ms` (~46.6%) and mean `0.979 ms` -> `0.514 ms` (~47.5%).
  `technical-large-coordinate-plan.pdf` improved p95 `0.844 ms` -> `0.495 ms`
  (~41.4%) and mean `0.758 ms` -> `0.447 ms` (~41.0%).
  `technical-linework-dimensions.pdf` improved p95 `0.397 ms` -> `0.320 ms`
  (~19.4%) and mean `0.358 ms` -> `0.299 ms` (~16.5%).
- Protection result:
  `target/performance-matrix-axis-threshold4-starter-repeat.json` rendered all
  starter fixtures with no fallback-required, missing-tool, not-applicable, or
  error records. `technical-linework-dimensions.pdf` improved p95
  `0.404 ms` -> `0.320 ms` (~20.8%) and mean `0.359 ms` -> `0.300 ms`
  (~16.4%). `vector-stress.pdf` stayed neutral (`3.423 ms` -> `3.442 ms` p95,
  `3.309 ms` -> `3.317 ms` mean). Watch items were limited to small absolute
  timing movements, with the largest p95 percentage on
  `office-report-header-footer-link.pdf` at `0.362 ms` -> `0.387 ms`
  (`+0.025 ms`).
- Decision: accept. This is a profile-backed cumulative stroke-rasterization
  improvement for medium axis-aligned linework, preserves the already-tested
  span rasterizer semantics, avoids new dependencies and unsafe code, and keeps
  the protection set within small absolute timing movement.

Post-axis-threshold profile and rejected fill shortcut from 2026-06-30:

- Profile evidence:
  `target/sample-vector-stress-post-axis-threshold.txt`, captured from a long
  release `benchmark-native` run after the axis-threshold commit, still showed
  `stroke_path` as the dominant stack with `3837` samples. The matching
  `trace-native fixtures/generated/vector-stress.pdf --max-edge 160` run
  reported `3.395 ms` of `3.603 ms` total in `raster_paths`.
  `point_in_row_bucketed_stroke` accounted for `949` samples, `fill_path` for
  `843`, `blend_pixel` for `328`, and `flatten_path_segments` for only `6`.
- Interpretation: reusable flattening is still not the next bottleneck for the
  current top vector fixture. The remaining work is mostly stroke rasterization
  plus a visible but secondary fill path.
- Change tested locally but not kept: split fill rasterization into unclipped
  variants so clip-free generic fills skip `point_in_active_clips`, and
  clip-free axis-aligned rectangle fills precompute center-sampled pixel bounds.
- Candidate artifacts:
  `target/performance-matrix-unclipped-rect-fill-technical.json`,
  `target/performance-matrix-unclipped-rect-fill-starter.json`,
  `target/performance-matrix-unclipped-fill-technical.json`, and
  `target/performance-matrix-unclipped-fill-starter.json`.
- Result: rejected. The rect-only variant was mostly noise; the broader
  unclipped-fill variant failed the protection set. Against
  `target/performance-matrix-axis-threshold4-technical-repeat.json`, it
  regressed p95 for `engineering-schematic-symbols.pdf` by ~20.1%,
  `technical-linework-dimensions.pdf` by ~15.0%, and
  `technical-large-coordinate-plan.pdf` by ~10.5%, while `vector-stress.pdf`
  was slightly slower. The starter run showed similar p95 regressions on small
  absolute timings, including `text-page.pdf`, `acroform-text-field.pdf`, and
  `technical-linework-dimensions.pdf`.
- Decision: reverted. Fill-loop branch splitting is not a useful next block
  without a more specific fill-shape profile. Continue with stroke-raster
  structural changes or collect better fill-shape diagnostics before touching
  fill hot paths again.

Rejected row-bucket sample-row span candidate from 2026-06-30:

- Change tested locally but not kept: replace row-bucket X ranges based on full
  per-line pixel bounds with sample-row stroke spans. The candidate built
  tighter candidate X ranges for each supersample row, then still used the
  existing exact `point_in_row_bucketed_stroke` and join predicates before
  blending.
- Rationale: the post-axis-threshold profile still pointed at
  `point_in_row_bucketed_stroke`, and diagonal or slanted lines can have much
  narrower row-local spans than their full device bounds.
- Candidate artifacts:
  `target/performance-matrix-row-bucket-sample-spans-technical.json` and
  `target/performance-matrix-row-bucket-sample-spans-starter.json`.
- Result: rejected. Against
  `target/performance-matrix-axis-threshold4-technical-repeat.json`,
  `vector-stress.pdf` improved only p95 `3.443 ms` -> `3.313 ms` (~3.8%) and
  mean `3.331 ms` -> `3.173 ms` (~4.7%), below the meaningful threshold.
  Protection-set signals were poor: `clipped-paths.pdf` regressed p95
  `0.411 ms` -> `0.538 ms`, and small starter fixtures showed large p95
  percentage movement on tiny absolute timings.
- Decision: reverted. Per-sample span construction adds too much overhead for
  the current row-bucket path. Revisit only if a future shape diagnostic can
  isolate rows with long diagonal bounds where the tighter range construction
  clearly pays for itself.

Rejected opaque normal blend fast path from 2026-06-30:

- Change tested locally but not kept: add a `blend_pixel` fast path for normal
  blend mode when source and destination alpha are both opaque and coverage is
  partial. The specialized path avoided `blend_source_with_backdrop` and the
  full `source_over` alpha calculation while preserving the same floor-based
  channel compositing.
- Rationale: the post-axis-threshold sample showed `blend_pixel` and
  `source_over` together as a secondary stack after `stroke_path`,
  `point_in_row_bucketed_stroke`, and `fill_path`.
- Candidate artifacts:
  `target/performance-matrix-opaque-blend-fastpath-technical.json`,
  `target/performance-matrix-opaque-blend-fastpath-technical-repeat.json`,
  `target/performance-matrix-opaque-blend-fastpath-starter.json`, and
  `target/performance-matrix-opaque-blend-fastpath-starter-repeat.json`.
- Result: rejected as an optimization block. The repeat technical run showed
  some useful movement, for example `engineering-floorplan-precision.pdf` p95
  `0.726 ms` -> `0.687 ms` (~5.4%) and
  `engineering-large-transform-detail.pdf` p95 `0.572 ms` -> `0.543 ms`
  (~5.1%), but the target `vector-stress.pdf` moved only p95 `3.443 ms` ->
  `3.372 ms` (~2.1%) and the starter repeat left `vector-stress.pdf` neutral.
  Small fixtures also showed p95 noise on tiny absolute timings.
- Decision: reverted. Keep blend specialization on the backlog, but do not
  land it as a standalone win until profiling shows blend is primary or the
  fast path can be batched at row/pixel-buffer level with stronger evidence.

Current vector profile and rejected scratch-capacity candidate from 2026-06-30:

- Fresh profile evidence:
  `target/sample-vector-stress-current.txt`, captured from a long release
  `benchmark-native` process for `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, after the accepted row-bucket and axis-threshold commits.
- Top-of-stack summary: `stroke_path` remained dominant with `3823` symbol
  samples. The next visible stacks were `point_in_row_bucketed_stroke` (`929`),
  `fill_path` (`846`), `blend_pixel` (`361`), `point_in_join` (`276`),
  `point_in_join_buckets` (`190`), and `source_over` (`184`).
  `flatten_path_segments` accounted for only `7` samples.
- Trace evidence from the same current tree:
  `trace-native fixtures/generated/vector-stress.pdf --max-edge 160
  --max-events 1` reported `3.234 ms` of `3.499 ms` total in `raster_paths`.
  The stroke shape summary still showed `485376` row-bucket sample refs,
  `25672` X hits, and `459704` X misses.
- Change tested locally but not kept: reserve `x_ranges` scratch vectors from
  known row-bucket/span row lengths and copy axis span rows with exact row
  capacity before appending join raster spans.
- Candidate artifact:
  `target/benchmark-native-vector-stress-scratch-candidate-rebuilt.json`,
  native single-fixture run, `--max-edge 160`, `5000` iterations.
- Result: rejected as a performance candidate. The rebuilt single-fixture run
  measured `3.198 ms` mean, versus `3.205 ms` in the immediately preceding
  old-binary signal run. That is below a meaningful threshold and lacks p95
  matrix evidence. The allocation frames in `sample` are real, but they are
  currently secondary to row-bucket X-miss and stroke predicate work.
- Decision: reverted. Do not land broad `Vec::with_capacity` or scratch-shape
  changes from allocator frames alone. Reopen allocation work only with
  allocation-volume evidence or a candidate that repeats at least a 5-10%
  protection-neutral gain as part of the cumulative stroke-raster track.

Rejected per-row bucket candidate caching from 2026-06-30:

- Change tested locally but not kept: in the row-bucketed stroke range
  rasterizer, resolve stroke and join bucket candidate slices once per raster
  row and pass them into inner predicate helpers. This avoided repeating the
  `y -> row range -> indices` lookup and `radius * radius` calculation for
  every supersample.
- Rationale: the current `vector-stress` sample still showed
  `point_in_row_bucketed_stroke` as the largest visible child stack under
  `stroke_path`, while retaining the existing exact geometry predicates and
  row-bucket data structure.
- Candidate artifact:
  `target/benchmark-native-vector-stress-row-candidates.json`, native
  single-fixture run, `--max-edge 160`, `10000` iterations.
- Result: rejected as a performance candidate. The run measured `3.171 ms`
  mean versus the previous current single-fixture signal at `3.198 ms`; the
  movement is below the 5% threshold and has no p95/protection-matrix support.
- Decision: reverted. The next accepted row-bucket improvement needs to reduce
  the number of candidate line checks or pixels visited, not only cache row
  lookup plumbing around the same X-miss-heavy predicate loop.

Rejected sorted row-bucket early-break candidate from 2026-06-30:

- Fresh profile evidence:
  `target/sample-vector-stress-current-after-downsample.txt`, captured from a
  long release `benchmark-native` process for
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`, after the
  low-memory image decode work. The top visible stacks were `stroke_path`,
  `point_in_row_bucketed_stroke`, `fill_path`, `blend_pixel`,
  `point_in_join`, and `point_in_join_buckets`.
- Trace evidence:
  `target/trace-vector-stress-current-after-downsample.json` reported
  `3.418 ms` of `3.786 ms` total in `raster_paths`. The stroke summary still
  showed `485376` row-bucket sample refs with `459704` X misses, so about
  95% of row-bucket sample checks were rejected by X bounds.
- Change tested locally but not kept: sort each row-bucket and join-bucket row
  by conservative `min_x/max_x`, then break the point predicate loop once the
  current pixel lies before the next sorted candidate. This preserved the
  existing exact geometry predicates and only changed candidate ordering.
- A/B artifacts:
  `target/benchmark-row-bucket-unsorted-report-vector-240.json` and
  `target/benchmark-row-bucket-sorted-report-vector-240.json`, same host,
  report/vector manifest, `--max-edge 160`, `240` iterations.
- Result: positive but below the acceptance threshold. Family mean moved
  `1.094 ms` -> `1.060 ms` (~3.1% faster). `vector-stress.pdf` moved
  `3.243 ms` -> `3.105 ms` (~4.3% faster), while
  `prepress-trim-bleed-marks.pdf` regressed slightly
  `0.510 ms` -> `0.516 ms` (~1.2%). A focused sorted-only
  `vector-stress` run over `500` iterations measured `3.129 ms`.
- Decision: reverted. This is a useful signal that row-bucket X ordering
  matters, but it is still a sub-5% result and not enough to land as a
  performance commit. Revisit only as part of a larger interval-query or
  range-generation change that materially reduces candidate checks.

Accepted gated active row-bucket candidate scan from 2026-06-30:

- Profile basis: the rejected sorted early-break candidate proved that
  X-ordering mattered but did not reduce enough predicate work on its own.
  `vector-stress.pdf` remained dominated by `stroke_path` and
  `point_in_row_bucketed_stroke`, with about 95% row-bucket X misses.
- Change: the row-bucket X-range rasterizer now keeps the existing scan path
  for smaller buckets, but uses an active candidate list for buckets with at
  least `64` bounded lines. For those large rows, candidates are sorted by
  conservative X bounds, activated as X advances, and removed once their
  `max_x` no longer overlaps the current pixel. Exact stroke/join geometry
  predicates are unchanged.
- Gate rationale: `vector-stress.pdf` has two 64-line row-bucket items and is
  X-miss dominated. `technical-linework-dimensions.pdf` has many smaller
  row-bucket items with max `42` lines, so the old path remains better there.
- Correctness guard:
  `active_row_bucket_candidates_should_match_bucketed_stroke_predicate_after_x_misses`
  checks that active candidates still find a later stroke hit after earlier
  X-miss candidates.
- A/B artifacts:
  `target/benchmark-row-bucket-unsorted-report-vector-240.json`,
  `target/benchmark-active-row-candidates-gated-report-vector-240.json`, and
  `target/benchmark-active-row-candidates-gated-report-vector-repeat-240.json`,
  same host, report/vector manifest, `--max-edge 160`, `240` iterations.
- Result: accepted. First run: family mean `1.094 ms` -> `0.990 ms`
  (~9.5% faster), `vector-stress.pdf` `3.243 ms` -> `2.825 ms`
  (~12.9% faster). Repeat run: family mean `1.094 ms` -> `0.994 ms`
  (~9.1% faster), `vector-stress.pdf` `3.243 ms` -> `2.837 ms`
  (~12.5% faster). Protection fixtures were neutral to small noise:
  `prepress-trim-bleed-marks.pdf` about +1.0%, `technical-hatch-clipping.pdf`
  about +0.3%, and `technical-linework-dimensions.pdf` neutral in the repeat.
- Decision: keep as a Phase 2 stroke-raster optimization. This is a
  profile-backed structural reduction in row-bucket predicate checks, not a
  broad sorting or allocation tweak.

Current post-cache vector profile and rejected scratch-capacity candidate from
2026-06-30:

- Baseline artifact:
  `target/performance-matrix-report-vector-current-after-session-cache.json`,
  native hot-render, `fixtures/performance-matrix-manifest.tsv`,
  `report/vector`, `--max-edge 160`, `240` measured iterations and `20`
  warmups. `vector-stress.pdf` remains the top fixture at p95 `3.168 ms` and
  mean `2.970 ms`; the next fixtures are much smaller:
  `prepress-trim-bleed-marks.pdf` p95 `0.584 ms`,
  `technical-hatch-clipping.pdf` p95 `0.403 ms`, and
  `technical-linework-dimensions.pdf` p95 `0.354 ms`.
- Trace evidence:
  `target/trace-vector-stress-after-session-cache.json` reports
  `raster_paths: 3.434 ms` of `3.818 ms` total. The stroke summary still shows
  two row-bucket candidate items, `485376` row-bucket sample refs, `25672`
  X hits, and `459704` X misses.
- CPU profile evidence:
  `target/sample-vector-stress-after-session-cache.txt`, a 10-second macOS
  `sample` run against a long release `benchmark-native` process, still shows
  `stroke_path` as the dominant flat symbol (`4641` samples), followed by
  `fill_path` (`899`), `blend_pixel` (`390`), and `source_over` (`245`).
  Allocator samples remain visible under `stroke_path`, but flattening is still
  negligible (`flatten_path_segments` `8` samples).
- Change tested locally but not kept: pre-size the row-bucket scratch vectors
  from the maximum row reference count before the raster loop. This targeted
  the visible allocator samples without changing row-bucket geometry or output.
- A/B artifacts:
  `target/performance-matrix-report-vector-current-after-session-cache.json`
  versus
  `target/performance-matrix-report-vector-row-scratch-capacity-candidate.json`,
  same host and command shape.
- Result: rejected by the acceptance threshold. `vector-stress.pdf` moved p95
  `3.168 ms` -> `3.067 ms` (`~3.2%`) and mean `2.970 ms` -> `2.952 ms`
  (`~0.6%`). `technical-linework-dimensions.pdf` improved p95
  `0.354 ms` -> `0.330 ms` (`~6.8%`), but its mean moved only `~0.7%`, and
  the primary target remained below the 5% threshold.
- Decision: reverted. Continue the vector track only for candidates that reduce
  candidate samples or visited pixels in `stroke_path`; broad scratch capacity
  tuning remains too weak to land.

Rejected single-active-line row-bucket candidate from 2026-06-30:

- Rationale: the current CPU profile still points at `stroke_path` and
  row-bucket candidate evaluation. The tested change specialized the active
  row-bucket loop when exactly one line candidate and no join candidate were
  active for the current pixel, calling the existing exact single-line
  predicate directly instead of the generic candidate-slice path.
- A/B artifact:
  `target/performance-matrix-report-vector-single-active-line-candidate.json`
  versus
  `target/performance-matrix-report-vector-current-after-session-cache.json`,
  same host and `report/vector` hot-render command shape.
- Result: positive but still below the threshold. `vector-stress.pdf` p95 moved
  `3.168 ms` -> `3.033 ms` (`~4.3%`) and mean `2.970 ms` -> `2.922 ms`
  (`~1.6%`). `technical-linework-dimensions.pdf` p95 moved
  `0.354 ms` -> `0.323 ms` (`~8.8%`), but mean was effectively neutral and the
  primary target stayed below the 5% line.
- Follow-up combination test:
  `target/performance-matrix-report-vector-active-line-plus-capacity-candidate.json`
  combined this candidate with the scratch-capacity candidate. The combination
  did not compound on the primary fixture: `vector-stress.pdf` p95 landed at
  `3.067 ms` (`~3.2%`) and mean at `2.926 ms` (`~1.5%`).
- Decision: reverted. Keep the signal for a future row-bucket interval-query
  or per-row span algorithm, but do not land a local single-candidate branch
  that fails the acceptance threshold on the primary fixture.

Accepted pre-sorted row-bucket rows from 2026-06-30:

- Profile basis: the post-cache vector profile still showed `stroke_path` as
  the dominant flat symbol, and the accepted active row-bucket path still
  needed per-raster-row X-ordered candidate lists. Earlier scratch-capacity and
  single-active-line candidates were positive but below the threshold, so this
  candidate moved the ordering work out of the hot raster loop instead of adding
  another local predicate branch.
- Change: `stroke_row_buckets` and `stroke_join_buckets` now store each row's
  indices sorted by conservative `(min_x, max_x)` bounds when the buckets are
  built. The active row-bucket rasterizer can then copy the already sorted row
  slice and skip per-row sorting in `sorted_row_line_indices` and
  `sorted_row_join_indices`. Exact stroke and join geometry predicates are
  unchanged.
- Correctness guard:
  `stroke_row_buckets_should_sort_row_indices_by_x_bounds` freezes the row
  ordering invariant used by the active scan.
- A/B artifacts:
  `target/performance-matrix-report-vector-current-after-session-cache.json`,
  `target/performance-matrix-report-vector-presorted-row-buckets-candidate.json`,
  and
  `target/performance-matrix-report-vector-presorted-row-buckets-repeat.json`,
  same host, `report/vector` hot-render, `--max-edge 160`, `240` measured
  iterations and `20` warmups.
- Result: accepted as a cumulative row-bucket optimization. First run:
  `vector-stress.pdf` p95 `3.168 ms` -> `2.927 ms` (`~7.6%`) and mean
  `2.970 ms` -> `2.832 ms` (`~4.6%`). Repeat: p95 `3.168 ms` -> `2.958 ms`
  (`~6.6%`) and mean `2.970 ms` -> `2.849 ms` (`~4.1%`).
  `technical-hatch-clipping.pdf` repeated p95 `~9.7%` faster, and
  `technical-linework-dimensions.pdf` repeated p95 `~7.1%` faster.
  `prepress-trim-bleed-marks.pdf` stayed neutral-to-better.
- Decision: keep. This reduces repeated row sorting in the profile-backed
  `stroke_path` track without changing coverage decisions or adding new
  dependencies.

Accepted rectangle clip fill bounds result from 2026-06-30:

- Profile basis:
  `target/sample-vector-stress-current.txt`, captured from a long release
  `benchmark-repeat-native` run on `vector-stress.pdf`, showed `stroke_path`
  still dominant but also showed `fill_path` as the second largest top-stack
  block (`994` samples), ahead of blend and join predicates. The current trace
  `target/trace-native-vector-stress-current-after-presort.json` reported
  `2.876 ms` of `3.091 ms` in `raster_paths`. The fixture uses nested
  rectangular clips and several axis-aligned rectangle fills, so the remaining
  fill work was a profile-backed target.
- Change: active clips now remember when their flattened path is a single
  axis-aligned rectangle. `fill_axis_aligned_rect_path` uses exact
  center-sampled pixel bounds for the filled rectangle and for all active
  rectangular clips, then writes only the intersected pixels without calling
  `point_in_active_clips` inside the per-pixel loop. Non-rectangular clips stay
  on the existing predicate path, and the center-sample edge semantics are
  preserved for fractional rectangle coordinates.
- Correctness guards:
  `center_sampled_rect_pixel_bounds_should_exclude_fractional_edges` protects
  the center-sample edge rule, and
  `center_sampled_axis_aligned_clip_bounds_should_intersect_rect_clips`
  protects multi-clip intersection.
- A/B artifacts:
  `target/performance-matrix-report-vector-profile-current.json`,
  `target/performance-matrix-report-vector-rect-clip-candidate.json`,
  `target/performance-matrix-report-vector-rect-clip-repeat.json`,
  `target/performance-matrix-rect-clip-starter.json`, and
  `target/trace-native-vector-stress-rect-clip.json`.
- Result: accepted as a standalone vector/report improvement. First run:
  `vector-stress.pdf` p95 `3.013 ms` -> `2.564 ms` (`~14.9%`) and mean
  `2.873 ms` -> `2.474 ms` (`~13.9%`). Repeat: p95 `3.013 ms` -> `2.574 ms`
  (`~14.6%`) and mean `2.873 ms` -> `2.486 ms` (`~13.5%`). The trace moved
  `raster_paths` from `2.876 ms` to `2.662 ms`.
- Protection result:
  `target/performance-matrix-rect-clip-starter.json` rendered all `11`
  performance-manifest native hot-render records with no fallback-required,
  missing-tool, not-applicable, or error records.
- Decision: keep. This removes repeated clip path tests from a profile-visible
  rectangle-fill hot path while preserving exact center-sampled rectangle and
  clip semantics, with no new dependency or unsafe code.

Accepted axis-aligned clip predicate fast path from 2026-06-30:

- Profile basis: after the rectangle-fill clip optimization, the fresh
  `target/sample-vector-stress-after-rect-clip.txt` profile showed
  `stroke_path` as the dominant stack and `fill_path` no longer dominated.
  The synthetic `vector-stress.pdf` fixture applies two nested axis-aligned
  rectangle clips before the heavy grid/curve stroke work, so every stroke
  sample still paid the generic clip path predicate even though the active
  clips already carried an `axis_aligned_rect` diagnostic.
- Change: `point_in_active_clips` now checks `ActiveClip::axis_aligned_rect`
  directly with rectangle edge semantics and falls back to the existing
  `point_in_path` predicate for all non-rectangular clips. This keeps the
  general clip path unchanged while removing repeated path winding checks from
  rectangle-clipped stroke and fill hot loops.
- Correctness guard:
  `point_in_active_clips_should_use_axis_aligned_rect_edges` freezes inclusive
  left/top and exclusive right/bottom rectangle containment for the fast path.
- A/B artifacts:
  `target/performance-matrix-report-vector-rect-clip-repeat.json`,
  `target/performance-matrix-report-vector-rect-clip-fast-repeat-1000.json`,
  `target/performance-matrix-rect-clip-fast-starter.json`, and
  `target/trace-native-vector-stress-rect-clip-fast.json`.
- Result: accepted as a standalone vector/report improvement. Against the
  previous rect-clip repeat baseline, `vector-stress.pdf` moved p95
  `2.574 ms` -> `1.442 ms` (`~44.0%`) and mean `2.486 ms` -> `1.320 ms`
  (`~46.9%`). Family p95 moved `2.574 ms` -> `1.442 ms`.
  `technical-linework-dimensions.pdf` also improved p95
  `0.342 ms` -> `0.305 ms` (`~10.8%`). `prepress-trim-bleed-marks.pdf`
  showed a small p95 noise regression (`0.575 ms` -> `0.590 ms`) but improved
  mean (`0.528 ms` -> `0.514 ms`), so it remains protection-neutral.
  The trace moved `raster_paths` from `2.662 ms` to `1.400 ms`.
- Protection result:
  `target/performance-matrix-rect-clip-fast-starter.json` rendered all `11`
  native hot-render records with no fallback-required, missing-tool,
  not-applicable, or error records.
- Decision: keep. This is a profile-backed reduction in per-sample clip
  predicate work for common rectangular PDF clips, preserves generic clip
  handling, and uses an existing field without new allocation, caching, unsafe
  code, or dependency surface.

Accepted axis-stroke join-bucket predicate result from 2026-06-30:

- Profile basis: after the axis-aligned clip predicate fast path,
  `target/sample-vector-stress-after-clip-fast.txt` showed `stroke_path`
  still dominant, with `point_in_join` (`701` samples) nearly as visible as
  `blend_pixel` (`767` samples). The existing join-bucket index already reduced
  generic row-bucketed stroke joins, but axis-aligned strokes still used the
  generic `point_in_join` predicate inside the axis-span raster loop.
- Change: axis-stroke span rasterization now builds a conservative
  `StrokeJoinBuckets` index when the axis-aligned stroked item has joins, then
  uses `point_in_join_buckets` in the inner sample loop. The existing generic
  `point_in_join` path remains the fallback when no bucket can be built. Stroke
  span coverage and exact join geometry predicates are unchanged.
- Correctness guard:
  `axis_stroke_raster_spans_should_cover_joined_axis_strokes` now also checks
  that bucketed join coverage matches the existing generic join predicate over
  the joined axis-stroke raster bounds.
- A/B artifacts:
  `target/performance-matrix-report-vector-rect-clip-fast-repeat-1000.json`,
  `target/performance-matrix-report-vector-axis-join-buckets-candidate.json`,
  `target/performance-matrix-report-vector-axis-join-buckets-repeat.json`,
  `target/performance-matrix-axis-join-buckets-starter.json`, and
  `target/trace-native-vector-stress-axis-join-buckets.json`.
- Result: accepted as a repeated cumulative stroke-raster improvement.
  Against the current fast-clip baseline, `vector-stress.pdf` moved p95
  `1.442 ms` -> `1.356 ms` (`~6.0%`) and mean `1.320 ms` -> `1.239 ms`
  (`~6.1%`) in the repeat. The trace moved `raster_paths` from `1.400 ms` to
  `1.291 ms`. `prepress-trim-bleed-marks.pdf` stayed effectively neutral
  (`0.590 ms` -> `0.582 ms` p95, mean `0.514 ms` -> `0.522 ms`), and
  `technical-linework-dimensions.pdf` showed a small p95 watch regression
  (`0.305 ms` -> `0.314 ms`) with near-neutral mean (`0.279 ms` ->
  `0.281 ms`).
- Protection result:
  `target/performance-matrix-axis-join-buckets-starter.json` rendered all `11`
  native hot-render records with no fallback-required, missing-tool,
  not-applicable, or error records.
- Decision: keep. This reuses the existing bounded join index in a previously
  uncovered axis-stroke path, reduces profile-visible join predicate fanout,
  and avoids new dependencies, unsafe code, or broader cache behavior.

Rejected post-axis-join candidates from 2026-06-30:

- Fresh profile:
  `target/sample-vector-stress-after-axis-join-buckets.txt`, captured from a
  long release `benchmark-repeat-native` run after the axis-join bucket commit,
  still showed `stroke_path` as the dominant symbol. The hottest visible
  secondary buckets were `blend_pixel` / `source_over`, allocation/free samples
  inside `stroke_path`, `fill_path`, and `merge_pixel_ranges`. The paired repeat
  run `target/benchmark-repeat-vector-stress-after-axis-join-buckets-profile-run.json`
  reported repeat mean `1.317 ms` and repeat `raster_paths` mean `1.216 ms`.
- Candidate: re-test an opaque `BlendMode::Normal` partial-coverage fast path
  now that clip and join predicate costs had been reduced. Result:
  `target/performance-matrix-report-vector-opaque-blend-candidate.json` kept
  `vector-stress.pdf` p95 at `1.356 ms` and only moved mean to `1.234 ms`.
  This is below the repeated 5% small-win bar, so the code was reverted.
- Candidate: skip the Vec-of-Vec rebuild in `axis_stroke_raster_spans` when
  axis-aligned strokes have no joins, cloning the compact coverage spans as the
  raster spans. Result:
  `target/performance-matrix-report-vector-joinless-span-reuse-candidate.json`
  regressed `vector-stress.pdf` p95 to `1.416 ms` and mean to `1.307 ms`.
  The code was reverted.
- Candidate: add a narrow axis-aligned rectangle-stroke fast path for solid
  Butt/Miter strokes. Result:
  `target/performance-matrix-report-vector-rect-stroke-candidate.json`
  regressed `vector-stress.pdf` p95 to `1.387 ms` and mean to `1.298 ms`.
  The code was reverted.
- Interpretation: after the accepted clip and join optimizations, the obvious
  micro-specializations are either below noise or negative on the current top
  vector fixture. The next vector pass should use a fresh sample plus a broader
  algorithmic target, not another local branch around `blend_pixel`, joinless
  span copying, or rectangle-stroke detection.

Accepted pixel-aligned rect-clip skip from 2026-06-30:

- Profile basis: after the axis-join bucket result and rejected local
  micro-specializations, the remaining `vector-stress.pdf` hot path still spent
  most time in `stroke_path` with rectangular clips active for the page. The
  earlier clip predicate fast path replaced generic path winding with
  `point_in_rect`, but the inner stroke sample loops still evaluated the two
  rectangle clips for every sample.
- Change: stroke rasterization now computes once whether active clips can be
  skipped inside the sample loops. The skip is enabled only when every active
  clip is an axis-aligned rectangle with pixel-aligned edges. Raster bounds are
  already intersected with active clip bounds before entering the loops, so
  every sample inside those pixel bounds is inside the clip. Fractional
  rectangle edges and all generic clip paths keep the existing
  `point_in_active_clips` predicate.
- Correctness guards:
  `active_clip_checks_should_be_skippable_for_pixel_aligned_rects` and
  `active_clip_checks_should_not_skip_fractional_or_generic_clips` cover the
  gating rule. The fractional-edge rejection is intentional because
  `intersect_active_clip_pixel_bounds` uses conservative floor/ceil bounds that
  can include partially clipped sample positions.
- A/B artifacts:
  `target/performance-matrix-report-vector-axis-join-buckets-repeat.json`,
  `target/performance-matrix-report-vector-rect-clip-skip-candidate.json`,
  `target/performance-matrix-rect-clip-skip-starter.json`, and
  `target/trace-native-vector-stress-rect-clip-skip.json`.
- Result: accepted as a standalone vector/report improvement. Against the
  axis-join focused repeat baseline, `vector-stress.pdf` moved p95
  `1.356 ms` -> `1.153 ms` (`~15.0%`) and mean `1.239 ms` -> `1.038 ms`
  (`~16.2%`). The trace moved `raster_paths` from `1.291 ms` to `1.036 ms`.
- Protection result:
  `target/performance-matrix-rect-clip-skip-starter.json` rendered all `11`
  native hot-render records with no fallback-required, missing-tool,
  not-applicable, or error records. Compared with
  `target/performance-matrix-axis-join-buckets-starter.json`,
  `technical-hatch-clipping.pdf` improved p95 `0.436 ms` -> `0.399 ms`,
  `technical-linework-dimensions.pdf` stayed p95-neutral at `0.326 ms`, and
  `prepress-trim-bleed-marks.pdf` moved only `0.606 ms` -> `0.608 ms` p95
  while mean improved `0.536 ms` -> `0.523 ms`.
- Decision: keep. This removes repeated clip predicate work from common
  pixel-aligned rectangular PDF clips without changing fractional clip
  behavior, generic clip behavior, allocation strategy, dependency surface, or
  unsafe code.

Accepted opaque normal blend shortcut from 2026-06-30:

- Profile basis: the fresh post-rect-clip-skip `sample` run still put nearly
  all repeat-render time inside `stroke_path`, with `blend_pixel` and
  `source_over` now visible as the largest secondary cost after the sample
  rejection work. Earlier opaque blend experiments were rejected before the
  clip-skip win because they did not clear the threshold; the changed profile
  made this narrow retest worthwhile.
- Change: `blend_pixel` now takes a direct source-over path when the blend mode
  is `Normal` and both source and destination pixels are fully opaque. This
  keeps the existing full-coverage direct write, avoids the generic alpha
  normalization path for partial opaque coverage, and does not add dependencies
  or unsafe code.
- Correctness guard:
  `blend_pixel_should_fast_path_opaque_normal_partial_coverage` compares the
  fast path against the existing `source_over` result for partial coverage.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-after-rect-clip-skip-profile-run.json`,
  `target/benchmark-repeat-vector-stress-opaque-blend-post-clip-skip-candidate.json`,
  `target/performance-matrix-report-vector-opaque-blend-post-clip-skip-candidate.json`,
  `target/performance-matrix-opaque-blend-post-clip-skip-starter.json`, and
  `target/trace-native-vector-stress-opaque-blend-post-clip-skip.json`.
- Result: accepted as a repeated 5-10% cumulative vector/report win. Against
  the fresh 30k post-rect-clip-skip repeat baseline, `vector-stress.pdf` moved
  repeat mean `1.051 ms` -> `0.971 ms` (`~7.6%`) and computed p95
  `1.177 ms` -> `1.086 ms` (`~7.7%`). The repeat phase attribution moved
  `raster_paths` from `0.956 ms` to `0.876 ms`.
- Protection result:
  `target/performance-matrix-opaque-blend-post-clip-skip-starter.json`
  rendered all `11` native hot-render records with no fallback-required,
  missing-tool, not-applicable, or error records. The vector family p95 was
  `1.105 ms`; this is below the previous focused p95 `1.153 ms` and consistent
  with the 30k repeat improvement. Single-run traces remain noisy, so the
  acceptance decision is based on the repeat benchmark plus status-neutral
  protection matrix.
- Decision: keep. This is a narrow compositing shortcut on the same
  stroke-raster bottleneck track and preserves the generic blend path for
  non-normal blend modes, transparent source pixels, and transparent
  destination pixels.

Accepted flat row-bucket index build from 2026-06-30:

- Profile basis: the fresh post-blend profile still spent almost all repeat
  time in `stroke_path`, with allocator frames visible around row-bucket setup
  and active row scanning. The stroke shape summary for `vector-stress.pdf`
  showed only `2` row-bucket candidate items but `5688` row-index references,
  making the per-row `Vec<Vec<usize>>` bucket build a plausible allocation
  target.
- Change: `stroke_row_buckets` and `stroke_join_buckets` now build bucket
  indices in two passes. They first collect bounded lines or joins and count
  row entries, then fill one contiguous `indices` buffer and sort each row
  slice in place. This removes the many small per-row vector pushes without
  changing bucket ordering, predicates, dependencies, or unsafe surface.
- Correctness guards:
  existing bucket tests cover row limiting, X-sort order, generic predicate
  equivalence, active row candidates after X misses, and round/miter join
  bucket predicates.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-after-opaque-blend-profile-run.json`,
  `target/benchmark-repeat-vector-stress-flat-buckets-candidate.json`,
  `target/performance-matrix-flat-buckets-starter.json`, and
  `target/trace-native-vector-stress-flat-buckets.json`.
- Result: accepted as another repeated 5-10% cumulative vector/report win.
  Against the fresh post-blend repeat baseline, `vector-stress.pdf` moved
  repeat mean `1.008 ms` -> `0.958 ms` (`~5.0%`) and computed p95
  `1.176 ms` -> `1.071 ms` (`~8.9%`). Repeat `raster_paths` moved
  `0.909 ms` -> `0.863 ms` (`~5.1%`).
- Protection result:
  `target/performance-matrix-flat-buckets-starter.json` rendered all `11`
  native hot-render records with no fallback-required, missing-tool,
  not-applicable, or error records. The vector family p95 was `1.085 ms`;
  the next-slowest vector records remained rendered and the report carried the
  same `rss-unavailable` and `pdfium-hot-reference-not-requested` caveats.
- Decision: keep. This is an allocation-shape improvement on the same
  row-bucket stroke-raster track. It keeps the existing `Vec` storage model,
  uses no `SmallVec`, and avoids adding a performance dependency before a
  length histogram justifies one.

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
- [x] Review `String`, `PathBuf`, and large enum clones inside loops.
- [x] Remove intermediate `.collect()` calls where the consumer can stream.
- [x] Inspect large enum variants if profiles show copy pressure.
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

Stream decode allocation probe from 2026-06-29:

- Profile evidence:
  `target/sample-mobile-mixed-compression-phase-attribution.txt` showed
  allocation/reallocation samples under
  `StreamObject::decode_with_options` / `default_read_to_end` during Flate
  image resource decode.
- Change tested locally but not kept: avoid the initial `raw.to_vec()` copy for
  filtered streams and initialize Flate output capacity from the encoded input
  length.
- Result:
  `target/performance-matrix-image-heavy-stream-decode-candidate.json` vs
  `target/performance-matrix-image-heavy-axis-final.json` regressed the focused
  image-heavy matrix: `mobile-mixed-compression-scan.pdf` p95 `1.046 ms` ->
  `1.139 ms`, `image-heavy-repeated-xobject-report.pdf` p95 `0.833 ms` ->
  `0.898 ms`, `scanner-large-image-budget.pdf` p95 `0.604 ms` -> `0.647 ms`,
  and the smaller image fixtures also worsened.
- Decision: reverted. The allocation samples are real, but this copy/capacity
  shape is not a useful optimization. Any future stream-decode work should use
  a more precise allocation counter or a decoder-level benchmark before touching
  the shared object model.

Clone/collect/large-enum audit from 2026-06-30:

- Command:
  `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::redundant_clone -W clippy::needless_collect -W clippy::large_enum_variant`.
- Finding: Clippy reported one `needless_collect` in
  `render_pages_parallel_partial_with_limits`. The naive streaming rewrite
  would join each scoped worker as it is spawned and risk serializing the
  batch, so the code now uses an explicit `Vec::with_capacity(chunk.len())`
  containing `(page_index, handle)` pairs. This keeps all workers spawned
  before join while removing the misleading collect/zip shape.
- Follow-up Clippy result: the same clone/collect/large-enum command completed
  without warnings after the refactor.
- Focused tests:
  `native_parallel_renderer_should_preserve_requested_page_order`,
  `native_parallel_renderer_should_match_sequential_page_outputs`, and
  `native_parallel_partial_renderer_should_preserve_mixed_page_status`.
- Scheduler smoke:
  `target/benchmark-batch-native-phase3-handle-audit-smoke-clean.json`,
  `benchmark-batch-native`, performance-matrix `office-export`,
  `--pages-per-input 1`, `--max-workers 2`, `--max-edge 120`,
  `--fail-on-budget`: 2/2 native rendered, no fallback, no errors, no budget
  failures.
- Performance claim: none. This is a Phase 3 lint/audit cleanup that preserves
  scheduler semantics and removes one misleading intermediate collect pattern.

## Phase 4: Image And Scan Track

Goal: make scan/image-heavy documents fast without increasing peak memory.

- [x] Identify image-heavy fixtures from matrix and existing image reports.
- [x] Profile decode, color conversion, alpha/soft-mask work, and output encode.
- [x] Add low-memory downsample-aware decode where the source image is much
  larger than the target raster.
- [ ] Extend downsample-aware decode to the default profile only when it is
  speed-neutral or speed-positive on repeated benchmarks.
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

Second image raster optimization result from 2026-06-30:

- Change: the axis-aligned color-image loop now borrows each destination row
  once with `RasterDevice::row_mut` and composites into the row slice directly,
  avoiding repeated per-pixel device offset checks. ImageMask/stencil images
  stay on the existing checked pixel path because the first broader candidate
  regressed `image-mask-logo.pdf`.
- Rationale: this is a safe-slice, no-dependency implementation of the
  hardware-aware rule to make contiguous row writes obvious before considering
  lower-level copy or SIMD work.
- Baselines:
  `target/performance-matrix-image-row-baseline.json`, native hot-render,
  `fixtures/image-heavy-memory-manifest.tsv`, `--max-edge 160`, 300 measured
  iterations after 20 warmups; and
  `target/performance-matrix-mobile-row-baseline.json`, native hot-render,
  `fixtures/mobile-scan-manifest.tsv`, 200 measured iterations after 20
  warmups.
- Accepted candidate:
  `target/performance-matrix-image-row-color-after.json` and
  `target/performance-matrix-mobile-row-color-after.json`, same command shapes
  and host.
- Result on the image-heavy target set: `dct-image.pdf` p95 `0.082 ms` ->
  `0.064 ms` (~22.0%), `predictor-image.pdf` p95 `0.077 ms` -> `0.060 ms`
  (~22.1%), and `soft-mask-image.pdf` p95 `0.095 ms` -> `0.080 ms` (~15.8%).
  `image-heavy-repeated-xobject-report.pdf` improved `0.883 ms` -> `0.843 ms`
  (~4.5%), while `scanner-large-image-budget.pdf` improved `0.615 ms` ->
  `0.593 ms` (~3.6%).
- Protection notes: `image-mask-logo.pdf` stayed neutral after routing stencil
  images back to the checked pixel path (`0.292 ms` -> `0.288 ms`). The
  Mobile-scan protection matrix kept the same expected three
  `image.filter` fallback-required records and no errors. A longer current run
  in `target/performance-matrix-mobile-row-color-repeat.json` showed the
  earlier small-p95 predictor and rotated-camera regressions did not reproduce.
- Visual check: current PNGs in `target/image-row-visual-current/` are
  byte-identical to `target/axis-image-visual-current/` for `dct-image`,
  `predictor-image`, `scanner-large-image-budget`, and `soft-mask-image`.

Accepted Flate image decode capacity result from 2026-06-30:

- Profiling trigger: fresh traces after the stroke-raster wins showed actual
  image/resource work on the remaining image-heavy set. The scanner-large trace
  reported `0.402 ms` in `resource_decode`, `0.095 ms` in `raster_images`, and
  `0.584 ms` total. A 10-second sample,
  `target/sample-scanner-large-current.txt`, showed `decode_image_samples` and
  `StreamObject::decode_with_options` dominated by `miniz_oxide` inflate work,
  with visible `read_to_end` realloc/memmove frames.
- Change: `StreamDecodeOptions` now carries an optional initial output
  capacity. Flate image XObject decoding sets that capacity to the expected
  decoded image byte length, or the PNG-predictor encoded length when a
  predictor is present. Other stream decode call sites keep the default
  `None` capacity.
- Rationale: this is a narrow allocation/traffic optimization for a measured
  Flate image decode path. It does not change image bytes, color conversion,
  sampling, alpha, clipping, or compositing behavior.
- Candidate artifacts:
  `target/benchmark-native-scanner-large-flate-capacity.json`,
  `target/performance-matrix-flate-capacity-image-heavy.json`, and
  `target/performance-matrix-flate-capacity-image-heavy-repeat.json`.
- Repeated result against
  `target/performance-matrix-sparse-axis-image-heavy.json`:
  `scanner-large-image-budget.pdf` improved p95 `0.369 ms` -> `0.339 ms`
  (~8.1%) and mean `0.320 ms` -> `0.307 ms` (~4.1%). The first matrix showed
  the same direction: p95 `0.369 ms` -> `0.344 ms` (~6.8%) and mean
  `0.320 ms` -> `0.309 ms` (~3.4%).
- Protection notes: all image-heavy fixtures still rendered with no fallback or
  error records. P95 movement on tiny fixtures remains noisy:
  `image-mask-logo.pdf` moved `0.124 ms` -> `0.136 ms` in the repeat while
  mean stayed neutral-to-better, and `image-heavy-rotated-mask-sheet.pdf` moved
  p95 `0.360 ms` -> `0.392 ms` with mean `0.333 ms` -> `0.337 ms`. Keep these
  as watch items in the next broad image matrix rather than treating them as
  output-risk, because decoded bytes and raster logic are unchanged.
- Correctness guard:
  `stream_decode_should_decode_flate_with_initial_capacity` verifies that the
  capacity hint preserves decoded Flate bytes.
- Decision: accept as a cumulative 5-10% Phase 4 decode/allocation win for the
  large-scan fixture. This does not close downsample-aware decode or cropped
  decode; it only removes avoidable output-buffer growth on the current Flate
  resource-decode hotspot.

Accepted opaque DeviceRGB image sampling result from 2026-06-30:

- Profiling trigger: after the Flate capacity win, fresh traces still showed
  mixed image work in the remaining image-heavy set. A long sample on
  `mobile-mixed-compression-scan.pdf`,
  `target/sample-mobile-mixed-current-resource-decode.txt`, showed
  `draw_image` and `ImageSampleCache::sample` as the largest render-side image
  stacks, with Flate/JPEG decode still visible in resource loading.
- Change: axis-aligned color images that are plain opaque DeviceRGB now use a
  narrow direct sampler. The fast path skips the per-sample stencil, soft-mask,
  indexed-color, and color-space dispatch while keeping the same source
  coordinate calculation, last-sample cache behavior, row-slice compositing,
  and edge coverage math.
- Scope guard: the fast path is disabled for ImageMask/stencil images,
  Indexed color, and any image with a soft mask. These stay on the existing
  generic sampler.
- Candidate artifacts:
  `target/performance-matrix-opaque-rgb-image-heavy.json` and
  `target/performance-matrix-opaque-rgb-image-heavy-repeat.json`.
- Repeated result against
  `target/performance-matrix-flate-capacity-image-heavy-repeat.json`:
  `image-heavy-repeated-xobject-report.pdf` improved p95 `0.413 ms` ->
  `0.369 ms` (~10.7%) and mean `0.367 ms` -> `0.341 ms` (~7.1%). The first
  candidate matrix showed the same direction: p95 `0.413 ms` -> `0.371 ms`
  (~10.2%) and mean `0.367 ms` -> `0.340 ms` (~7.4%).
- Secondary movement: `image-heavy-rotated-mask-sheet.pdf` improved in both
  runs, with p95 `0.392 ms` -> `0.343 ms` first and `0.392 ms` -> `0.362 ms`
  on repeat. Protected `image-mask-logo.pdf`, `predictor-image.pdf`, and
  `soft-mask-image.pdf` stayed neutral to better on p95 in the repeat matrix.
- Watch item: `mobile-mixed-compression-scan.pdf` moved p95 `0.235 ms` ->
  `0.242 ms` in the repeat while remaining rendered with no fallback/error.
  Keep this in the next mobile-focused matrix; the target of this block is the
  repeated opaque image placement workload.
- Correctness guard:
  `image_fast_path_should_only_accept_opaque_device_rgb_samples` freezes the
  fast-path predicate so soft masks, stencils, and Indexed images cannot enter
  the direct RGB sampler accidentally. Existing image edge antialias coverage
  still exercises the axis-aligned opaque RGB draw path.
- Decision: accept as a Phase 4 repeated opaque image sampling win. It is not a
  downsample-aware decode implementation and does not close the cropped-decode
  backlog.

Accepted opaque DeviceGray image sampling result from 2026-06-30:

- Profiling trigger: after the opaque RGB fast path, a fresh 10-second macOS
  `sample` run on `mobile-mixed-compression-scan.pdf`,
  `target/sample-mobile-mixed-after-opaque-rgb.txt`, still showed image raster
  work at the top of the hot render stack: `ImageSampleCache::sample` (`207`
  top samples), `draw_image` (`180`), and `composite_image_pixel_in_row`
  (`138`). The fixture generator confirmed the dominant scan image is opaque
  DeviceGray with Flate compression.
- Change: axis-aligned opaque DeviceGray images now use a direct gray sampler
  and the existing row-slice compositing path. Opaque RGB and Gray are routed
  through one `opaque_image_sample_kind` classifier so RGB image placements do
  not pay for a second fast-path guard.
- Scope guard: the fast path is disabled for ImageMask/stencil images, Indexed
  color, CMYK, and any image with a soft mask. Those remain on the generic
  sampler.
- Live A/B artifacts: base worktree at `2632944` wrote
  `/private/tmp/pdfrust-gray-base/target/performance-matrix-gray-base-live.json`;
  the accepted candidate wrote
  `target/performance-matrix-opaque-gray-combined-repeat.json`.
- Result against the live base matrix:
  `mobile-mixed-compression-scan.pdf` improved mean `0.220 ms` -> `0.193 ms`
  (~12.3%) and p95 `0.242 ms` -> `0.203 ms` (~16.1%).
  `scanner-large-image-budget.pdf` improved mean `0.321 ms` -> `0.294 ms`
  (~8.4%) and p95 `0.336 ms` -> `0.314 ms` (~6.5%).
- Protection movement: `dct-image.pdf`, `image-heavy-rotated-mask-sheet.pdf`,
  `soft-mask-image.pdf`, and `image-mask-logo.pdf` were neutral to slightly
  better on p95. `image-heavy-repeated-xobject-report.pdf` moved p95
  `0.368 ms` -> `0.376 ms` (~2.2% slower), and `predictor-image.pdf` moved
  `0.049 ms` -> `0.050 ms` (~2.0% slower); both stayed rendered with no
  fallback or error and are treated as noise/watch items for the next image
  matrix.
- Correctness guard:
  `image_fast_path_should_only_accept_opaque_device_gray_samples` freezes the
  Gray predicate beside the existing RGB guard.
- Decision: accept as a profile-backed Phase 4 cumulative image sampling win.
  The scanner fixture is a 5-10% repeated improvement, which is acceptable here
  because it compounds with the mobile win, keeps the protection set effectively
  neutral, and targets the same image sampling bottleneck.

Accepted opaque image interior write result from 2026-06-30:

- Profiling trigger: after the opaque Gray fast path, post-change traces still
  showed `resource_decode` as the largest scanner block, but the renderer-owned
  image work remained visible. A 10-second `sample` on
  `scanner-large-image-budget.pdf`,
  `target/sample-scanner-large-after-gray.txt`, showed
  `miniz_oxide::inflate::core::transfer` as the top external decode stack
  (`2758` top samples), while the local raster side still had `draw_image`
  (`1269`) and `composite_image_pixel_in_row` (`560`).
- Change: axis-aligned opaque Gray/RGB images now split fully-covered interior
  pixels from edge pixels. Interior pixels skip per-pixel coverage calculation
  and alpha/compositing checks and write the opaque RGBA row bytes directly.
  Edge pixels keep the existing coverage and compositing path, preserving
  subpixel antialiasing.
- Scope guard: only the opaque Gray/RGB fast path uses the direct interior row
  writer. Soft masks, ImageMask/stencil images, Indexed color, CMYK, and
  rotated/non-axis-aligned images remain on existing paths.
- Live A/B artifacts:
  `/private/tmp/pdfrust-opaque-interior-base/target/performance-matrix-opaque-interior-base.json`,
  `/private/tmp/pdfrust-opaque-interior-base/target/performance-matrix-opaque-interior-base-2.json`,
  `target/performance-matrix-opaque-interior-candidate.json`, and
  `target/performance-matrix-opaque-interior-candidate-2.json`. Final
  post-Clippy-fix verification artifact:
  `target/performance-matrix-opaque-interior-final.json`.
- Aggregated result across the two live A/B runs:
  `mobile-mixed-compression-scan.pdf` improved p95 `0.2155 ms` -> `0.163 ms`
  (~24.4%) and mean `0.1975 ms` -> `0.152 ms` (~23.0%);
  `scanner-large-image-budget.pdf` improved p95 `0.3325 ms` -> `0.269 ms`
  (~19.1%) and mean `0.297 ms` -> `0.252 ms` (~15.2%);
  `predictor-image.pdf` improved p95 `0.051 ms` -> `0.0345 ms` (~32.4%).
- Protection movement: `image-heavy-repeated-xobject-report.pdf` improved p95
  ~5.8%, `image-mask-logo.pdf` was neutral on p95, `soft-mask-image.pdf`
  moved p95 ~1.3% slower, and `image-heavy-rotated-mask-sheet.pdf` moved p95
  ~4.2% slower. The latter two are under the 5% noise threshold and remain
  watch items because they do not use the opaque interior writer.
- Final nearest-base remeasure after the Clippy cleanup confirmed the win:
  `dct-basic.jpg.pdf` p95 `0.067 ms` -> `0.040 ms` (~40.3%),
  `image-heavy-repeated-xobject-report.pdf` p95 `0.413 ms` -> `0.360 ms`
  (~12.8%), `mobile-mixed-compression-scan.pdf` p95 `0.218 ms` ->
  `0.160 ms` (~26.6%), `scanner-large-image-budget.pdf` p95 `0.333 ms` ->
  `0.264 ms` (~20.7%), and `soft-mask-image.pdf` p95 `0.079 ms` ->
  `0.072 ms` (~8.9%).
- Correctness guards: existing axis-aligned image antialias coverage remained
  unchanged, and `full_coverage_pixel_range_should_only_include_interior_pixels`
  pins the interior-only range calculation.
- Decision: accept as a profile-backed Phase 4 opaque image raster win. This
  does not solve Flate decode dominance, but it removes avoidable renderer work
  from the same image/scan family and compounds with the Gray/RGB sampler wins.

Rejected stream raw-copy candidate from 2026-06-30:

- Profiling trigger: after the opaque interior write win, a fresh long
  `scanner-large-image-budget.pdf` benchmark was sampled in
  `target/sample-scanner-large-post-interior.txt`. The remaining CPU profile
  was dominated by `decode_image_samples`,
  `std::io::default_read_to_end`, `flate2::read::ZlibDecoder`, and
  `miniz_oxide::inflate`; `draw_image` was now materially smaller than the
  Flate stack. `_platform_memmove` also remained visible in the sample.
- Change tested locally but not kept: avoid the unconditional `raw.to_vec()`
  in `decode_stream_bytes` for filtered streams by applying the first stream
  filter directly to the borrowed raw byte slice, then continuing later filters
  from owned intermediate buffers.
- Rationale: this targeted a plausible compressed-byte copy before Flate
  decode, not the actual miniz transfer/decompression cost.
- A/B artifacts:
  `/private/tmp/pdfrust-stream-copy-base/target/benchmark-native-scanner-large-stream-copy-base.json`,
  `target/benchmark-native-scanner-large-stream-copy-candidate.json`,
  `target/benchmark-native-scanner-large-post-interior-long.json`, and
  `target/benchmark-native-scanner-large-skip-raw-copy.json`.
- Result: rejected as a performance candidate. The clean base worktree measured
  `0.244 ms` mean, while the candidate measured `0.246 ms` mean on the same
  100000-iteration scanner benchmark. Earlier candidate-only reruns moved from
  `0.246 ms` to `0.255 ms`, so the apparent first-run `0.252 ms` -> `0.246 ms`
  improvement was noise.
- Decision: keep `decode_stream_bytes` unchanged for now. The next Flate work
  should target actual decoder cost, predictor application, or avoiding full
  decode for downsample/crop cases rather than only removing the compressed raw
  staging copy.

Rejected page XObject form-skip candidate from 2026-06-30:

- Profiling trigger: after adding resource subphase attribution, release traces
  showed image-heavy fixtures dominated by `resource_images`, with smaller but
  visible `resource_forms` time on image-only pages. For example,
  `target/trace-scanner-large-resource-subphases-release.json` reported
  `resource_decode` `0.365 ms`, `resource_images` `0.327 ms`, and
  `resource_forms` `0.037 ms`.
- Change tested locally but not kept: resolve page-level image resources before
  form resources, then skip page-level XObject names already known to be images
  when building `FormResources`. The candidate preserved those names as known
  non-form invocations so image `Do` operators would still be ignored by the
  form display-list builder.
- Rationale: this removed duplicate page-level image stream lookup and subtype
  checks in the form-resource pass, without changing image decoding itself.
- A/B artifacts:
  `/private/tmp/pdfrust-xobject-form-base/target/performance-matrix-xobject-form-base.json`,
  `target/performance-matrix-xobject-form-candidate.json`,
  `/private/tmp/pdfrust-xobject-form-base/target/performance-matrix-xobject-form-base-repeat.json`,
  `target/performance-matrix-xobject-form-candidate-repeat.json`,
  `/private/tmp/pdfrust-xobject-form-base/target/performance-matrix-scanner-xobject-form-base-100.json`,
  `target/performance-matrix-scanner-xobject-form-candidate-100.json`,
  `/private/tmp/pdfrust-xobject-form-base/target/performance-matrix-repeated-xobject-form-base-100.json`,
  and `target/performance-matrix-repeated-xobject-form-candidate-100.json`.
- Result: rejected as a performance candidate. The initial 20-iteration runs
  were noisy and only showed a plausible p95 win on some fixtures. The
  stabilizing 100-iteration runs did not confirm it:
  `scanner-large-image-budget.pdf` mean `0.269 ms` -> `0.266 ms` (~1.1%) and
  p95 `0.332 ms` -> `0.320 ms` (~3.6%);
  `image-heavy-repeated-xobject-report.pdf` mean `0.356 ms` -> `0.357 ms`
  (~0.3% slower) and p95 `0.437 ms` -> `0.449 ms` (~2.8% slower).
- Decision: reverted. The duplicate page-level form/image subtype check is real
  but too small to matter versus image decode. Continue Phase 4 with
  `resource_images` work that reduces decoded bytes, predictor work, or image
  sample processing.

Image resource summary instrumentation from 2026-06-30:

- Profiling trigger: after the opaque image raster wins and the rejected
  stream-copy/form-skip candidates, `scanner-large-image-budget.pdf` remained
  dominated by `resource_images` / Flate decode. The next optimization decision
  needed byte-shape evidence rather than another generic stream-copy guess.
- Change: `ImageResources` now records aggregate resource counters while image
  XObjects are decoded: image count, raw/Flate/DCT/Predictor counts, color vs
  stencil count, soft-mask count, encoded bytes, decoded sample bytes,
  soft-mask bytes, Indexed lookup bytes, resident bytes, and max source
  dimensions/pixels. `trace-native` exposes the counters as
  `image_resource_summary`.
- Trace artifacts:
  `target/trace-scanner-large-image-resource-summary.json`,
  `target/trace-mobile-mixed-image-resource-summary.json`, and
  `target/trace-predictor-image-resource-summary.json`.
- Initial readings:
  `scanner-large-image-budget.pdf` has one Flate image with `4,653` encoded
  bytes expanding to `563,200` decoded/resident bytes (`640 x 880` pixels);
  `mobile-mixed-compression-scan.pdf` has one Flate image and one DCT image
  with `1,345` encoded bytes expanding to `70,448` resident bytes; and
  `predictor-image.pdf` correctly reports one Flate Predictor image.
- Regression guard: this is instrumentation, not a speed claim. A sequential
  100-iteration native hot-render repeat compared
  `/private/tmp/pdfrust-image-summary-base/target/performance-matrix-image-summary-base-repeat.json`
  with `target/performance-matrix-image-summary-candidate-repeat.json`.
  `scanner-large-image-budget.pdf` stayed effectively neutral (`0.251 ms` ->
  `0.252 ms` mean, `0.267 ms` -> `0.265 ms` p95), while
  `mobile-mixed-compression-scan.pdf` improved within noise (`0.150 ms` ->
  `0.147 ms` mean). Small fixture p95 values remained noisy; the largest watch
  item is `image-heavy-rotated-mask-sheet.pdf` p95 (`0.332 ms` -> `0.375 ms`)
  with only a smaller mean move (`0.319 ms` -> `0.329 ms`).
- Decision: accept as Phase 4 profiling infrastructure. The scanner byte ratio
  supports prioritizing downsample/crop-aware image decode or avoiding full
  decoded sample materialization for oversized Flate scans over more
  compressed-stream copy micro-optimizations.

Image placement footprint instrumentation from 2026-06-30:

- Profiling trigger: `image_resource_summary` showed source image bytes and
  decoded sample size, but did not show whether the decoded source pixels were
  much larger than the final raster footprint. Downsample/crop-aware decode
  needs source-vs-device placement evidence before changing decode strategy.
- Change: `trace-native` now emits `image_placement_summary` with placement
  count, summed source pixels, conservative clipped device pixels, max source
  and device footprint, downsample-candidate count, off-device count,
  axis-aligned/transformed placement counts, and max
  `source_pixels / device_pixels * 100` ratio. The summary is request-local
  and only collected in the explicit trace path.
- Trace artifacts:
  `target/trace-scanner-large-image-placement-summary.json` and
  `target/trace-mobile-mixed-image-placement-summary.json`.
- Initial readings: `scanner-large-image-budget.pdf` has one visible
  axis-aligned placement with `563,200` source pixels and only `18,560`
  conservative device pixels at `--max-edge 160`, for a ratio of `30.34x`;
  it is counted as one downsample candidate. `mobile-mixed-compression-scan.pdf`
  has two visible axis-aligned placements with `70,416` source pixels and
  `18,916` device pixels, max ratio `3.96x`, so it stays just below the
  current 4x downsample-candidate threshold.
- Decision: accept as Phase 4 profiling infrastructure. The scanner fixture is
  now a concrete downsample-aware decode target; the mobile mixed fixture is a
  watch item where a lower threshold would need stronger quality and speed
  evidence.

Accepted low-memory downsample decode result from 2026-06-30:

- Profiling trigger: the placement trace showed
  `scanner-large-image-budget.pdf` rendering `563,200` source pixels into
  `18,560` clipped device pixels at `--max-edge 160`, a `30.34x` source/device
  pixel ratio.
- Change: native low-memory render limits now enable request-local
  placement-aware image decode hints. The first downsample path is deliberately
  conservative: top-level, visible, axis-aligned Image XObject placements only;
  Raw/Flate images only; DeviceGray/DeviceRGB only; no Predictor, DCT, Decode
  array, ImageMask, Indexed color, SoftMask, or Form XObject image cases. The
  default profile passes empty hints and keeps the previous decode path.
- Trace artifacts:
  `target/trace-scanner-large-default-after-gate.json` and
  `target/trace-scanner-large-low-memory-downsample.json`.
- Memory result: default keeps the scanner image at `640 x 880`,
  `563,200` decoded/resident bytes, and still reports a downsample candidate.
  Low-memory stores the same placement as `116 x 160`, `18,560`
  decoded/resident bytes, and the placement summary drops to a `1.00x`
  source/device ratio. This is a `96.7%` resident image-sample reduction for
  the targeted fixture.
- Benchmark result, 1000 native hot-render iterations:
  default remained neutral against the baseline worktree
  (`0.259 ms` -> `0.260 ms` mean). Low-memory traded speed for memory
  (`0.246 ms` -> `0.304 ms` mean), so this is accepted only as a low-memory
  memory optimization, not as a default performance win.
- Regression guard: targeted tests cover the decode-hint sample reduction,
  low-memory scanner trace resource counters, and `trace-native
  --native-profile low-memory`. Existing masks, Predictor images, DCT images,
  Indexed color, and SoftMask cases remain outside this fast path.

Rejected default-profile downsample retest from 2026-06-30:

- Profiling trigger: after the `zlib-rs` Flate backend and opaque image raster
  wins, the earlier low-memory downsample result needed a fresh default-profile
  speed check. The goal was to see whether placement-aware downsample decode
  had become speed-neutral or speed-positive enough to enable by default.
- Change tested locally but not kept: enable `downsample_image_decode` in
  `NativeRenderLimits::default()` while leaving the downsample algorithm and
  low-memory profile unchanged.
- A/B artifacts:
  `target/benchmark-native-scanner-default-downsample-current-base.json` and
  `target/benchmark-native-scanner-default-downsample-candidate.json`, both
  `benchmark-native`, `fixtures/generated/scanner-large-image-budget.pdf`,
  `--max-edge 160`, 100,000 iterations.
- Result: default-profile downsample was still slower on the focused scanner
  fixture: mean `0.133 ms` -> `0.184 ms` (~38.3% slower). Output dimensions and
  bytes stayed unchanged, and the fixture rendered without fallback or errors.
- Decision: reverted. Keep downsample-aware decode limited to the low-memory
  profile for now. Reopen default-profile downsample only if the implementation
  avoids full source decode or a broader scan workload proves a repeated
  speed-neutral result.

Current scanner profile after default-downsample retest from 2026-06-30:

- Artifacts:
  `target/benchmark-native-scanner-current-profile-run.json`,
  `target/sample-scanner-current-post-default-retest.txt`, and
  `target/trace-scanner-current-post-default-retest.json`.
- Focused benchmark result:
  `scanner-large-image-budget.pdf`, `benchmark-native`, `--max-edge 160`,
  500,000 iterations, mean `0.135 ms`, rendered with no fallback or error.
- Trace attribution: total `0.383 ms`, with `resource_decode` `0.203 ms`,
  `resource_images` `0.197 ms`, `raster_paths` `0.036 ms`, and
  `raster_images` `0.043 ms`.
- CPU sample: the dominant stack is now
  `decode_image_samples` -> `StreamObject::decode_with_options` ->
  `flate2`/`zlib-rs` inflate. Raster image work and stroke overlay work are
  materially smaller on this fixture after the accepted image-raster and
  row-bucket wins.
- Next direction: do not spend the next scanner block on image raster loops or
  generic stream-copy staging. A credible next candidate must either reduce
  actual Flate decoder work, avoid full source decode for downsample/crop
  cases, or prove via a broader scan fixture that another phase has become
  dominant.

Rejected streaming Flate downsample candidate from 2026-06-30:

- Change tested locally but not kept: add a `ferrugo-object` API that decodes a
  single Flate stream into a caller-provided writer, then use a render-side
  row writer to materialize only nearest-neighbor target rows/columns for
  conservative placement-aware image downsample hints.
- Scope: the candidate stayed narrow: Flate-only, no Predictor, no Decode
  array, no ImageMask, no DCT, no SoftMask, and only the existing
  DeviceGray/DeviceRGB downsample-hint cases.
- A/B artifacts:
  `target/benchmark-native-scanner-low-memory-stream-downsample-base.json` and
  `target/benchmark-native-scanner-low-memory-stream-downsample-candidate.json`,
  both `benchmark-native`, `fixtures/generated/scanner-large-image-budget.pdf`,
  `--native-profile low-memory`, `--max-edge 160`, 100,000 iterations. Candidate
  trace artifact:
  `target/trace-scanner-low-memory-stream-downsample-candidate.json`.
- Result: rejected. Low-memory mean regressed `0.180 ms` -> `0.205 ms`
  (~13.9% slower). The trace still showed `resource_images` as dominant
  (`0.207 ms`), and final resident image bytes stayed at `18,560`, matching
  the existing low-memory downsample result.
- Decision: reverted. Streaming through `Write` avoided the full source output
  vector conceptually, but the per-row writer overhead was not worth it on the
  current scanner fixture. Reopen only with allocation high-water evidence or a
  decoder-level design that reduces inflate work instead of just changing the
  decoded-byte sink.

Accepted shared image sample Vec result from 2026-06-30:

- Profiling trigger: `target/sample-scanner-large-post-interior.txt` showed a
  visible `Arc::from(Vec<u8>)` copy after Flate image decode, and the image
  resource summary showed large decoded sample buffers relative to encoded
  bytes. This targeted renderer-owned allocation/copy traffic after decode,
  not the `miniz_oxide` transfer loop itself.
- Change: decoded image samples and decoded soft masks are now stored as
  `Arc<Vec<u8>>` instead of `Arc<[u8]>`. This keeps cheap sharing across
  repeated image placements while preserving the original decode `Vec`
  allocation, avoiding the large copy required to move decoded samples into an
  Arc slice.
- Scope guard: indexed lookup tables stay as `Arc<[u8]>`; raster sampling,
  soft-mask alpha, filter decode, color conversion, image bounds, and output
  pixels are unchanged. `resident_bytes` now uses sample `Vec::capacity()` so
  trace memory accounting remains honest for the new storage shape.
- A/B artifacts:
  `/private/tmp/pdfrust-arc-vec-base/target/performance-matrix-arc-vec-base.json`,
  `target/performance-matrix-arc-vec-candidate.json`,
  `/private/tmp/pdfrust-arc-vec-base/target/performance-matrix-arc-vec-base-repeat.json`,
  `target/performance-matrix-arc-vec-candidate-repeat.json`,
  `/private/tmp/pdfrust-arc-vec-base/target/benchmark-native-arc-vec-scanner-base.json`,
  and `target/benchmark-native-arc-vec-scanner-candidate.json`.
- Repeated image-heavy matrix result: `scanner-large-image-budget.pdf` improved
  mean `0.248 ms` -> `0.237 ms` (~4.4%) and p95 `0.274 ms` -> `0.256 ms`
  (~6.6%) across the two 200-iteration A/B runs. The focused 50,000-iteration
  scanner benchmark confirmed mean movement `0.247 ms` -> `0.237 ms` (~4.0%).
- Protection movement: `mobile-mixed-compression-scan.pdf` improved mean
  ~2.0% and stayed roughly neutral on p95. `image-heavy-rotated-mask-sheet.pdf`,
  `image-mask-logo.pdf`, and `soft-mask-image.pdf` stayed neutral to better.
  `dct-image.pdf` and `predictor-image.pdf` had small noisy p95 regressions,
  with mean movement under 5%; keep them as watch items in the next image
  matrix.
- Decision: accept as a cumulative Phase 4 resource-decode allocation win. It
  does not replace downsample-aware decode, but it removes one measured
  renderer-owned copy from the large decoded-image path and compounds with the
  earlier Flate capacity and opaque image raster wins.

Accepted zlib-rs Flate backend result from 2026-06-30:

- Profiling trigger: after the row-bucket vector win, a fresh 10-second
  `sample` run on `scanner-large-image-budget.pdf` showed the remaining
  scanner workload dominated by Flate image resource decode:
  `decode_image_samples` / `StreamObject::decode_with_options` accounted for
  `4974` of `7314` samples, with `miniz_oxide` inflate/transfer as the largest
  stack. `draw_image` was much smaller (`957` samples) and `stroke_path` was no
  longer the next meaningful target (`173` samples).
- Change: `ferrugo-object` now builds `flate2` with default features disabled
  and the Rust-native `zlib-rs` backend enabled. This keeps the decoder in Rust
  while replacing the previous `miniz_oxide` backend for Flate streams.
- Focused result: two 100,000-iteration native hot-render runs on
  `scanner-large-image-budget.pdf`, `--max-edge 160`, measured `0.126 ms` and
  `0.131 ms` mean, compared with the current pre-change image-heavy matrix
  value of `0.236 ms` mean and the earlier 50,000-iteration scanner baseline of
  about `0.237 ms`. This is roughly a `44-47%` scanner improvement.
- Phase trace: `target/trace-scanner-large-zlib-rs-candidate.json` reduced
  `resource_decode` from `0.412 ms` to `0.219 ms`, and `resource_images` from
  `0.296 ms` to `0.172 ms`, while preserving image resource and placement
  counters.
- Image-heavy protection: repeated 240-iteration image-heavy runs kept all
  target fixtures rendered with no fallback/error records. The repeat improved
  `scanner-large-image-budget.pdf` mean `0.236 ms` -> `0.127 ms`
  (`~46.2%`), `mobile-mixed-compression-scan.pdf` `0.146 ms` -> `0.131 ms`
  (`~10.3%`), `image-mask-logo.pdf` `0.121 ms` -> `0.110 ms` (`~9.1%`),
  `soft-mask-image.pdf` `0.074 ms` -> `0.067 ms` (`~9.5%`), and
  `predictor-image.pdf` `0.035 ms` -> `0.031 ms` (`~11.4%`).
- Vector protection: the 240-iteration `report/vector` protection run stayed
  neutral overall against the accepted row-bucket baseline. `vector-stress.pdf`
  improved `2.837 ms` -> `2.797 ms`, while the small vector fixtures moved only
  within noise (`prepress` `+1.7%`, `technical-hatch` `+3.2%`,
  `technical-linework` `-1.8%`).
- Decision: accept as a Phase 4 decoder-backend optimization. This is not a
  downsample-aware default decode change; it directly addresses the profiled
  Flate hotspot and leaves the lower-memory downsample/crop backlog open.

Rejected image raster candidate from 2026-06-30:

- Change tested locally but not kept: apply the same row-slice compositing
  strategy to non-axis-aligned color images, leaving stencil images on the
  checked pixel path.
- Rationale: this targeted rotated image placements and soft-mask sheets that
  still used the generic per-pixel device accessor route.
- Baselines:
  `target/performance-matrix-generic-image-row-baseline.json` and
  `target/performance-matrix-generic-image-row-transparency-baseline.json`.
- Candidate:
  `target/performance-matrix-generic-image-row-after.json` and
  `target/performance-matrix-generic-image-row-transparency-after.json`.
- Result: mixed and not protection-set-neutral. `image-heavy-rotated-mask-sheet.pdf`
  improved only about 3-4% p95, but `dct-image.pdf` regressed `0.066 ms` ->
  `0.117 ms` (~77% slower) and
  `browser-print-raster-vector-mix.pdf` regressed `0.446 ms` -> `0.716 ms`
  (~60% slower). All records still rendered with no fallback or error.
- Decision: reverted. Keep row-slice compositing scoped to the axis-aligned
  image loop until a rotated-image-specific profile shows a cleaner branch
  shape.

Rejected stroke row-blend candidate from 2026-06-30:

- Change tested locally but not kept: route the generic, row-bucketed, simple
  line-span, and axis-span stroke raster loops through a new row-slice
  `blend_pixel_in_row` helper instead of calling `RasterDevice::pixel` and
  `set_pixel` per covered pixel.
- Rationale: the long `mobile-mixed-compression-scan.pdf` sample showed
  `stroke_path`, `blend_pixel`, and small allocation frames in the remaining
  path overlay work, and previous image wins came from making contiguous row
  writes explicit.
- Candidate artifacts:
  `target/performance-matrix-row-blend-image-heavy.json` and
  `target/performance-matrix-row-blend-image-heavy-repeat.json`, compared
  against `target/performance-matrix-opaque-rgb-image-heavy-repeat.json`.
- Result: not repeatable and not protection-set-neutral. The first matrix
  showed promising p95 movement on `mobile-mixed-compression-scan.pdf`
  (`0.242 ms` -> `0.229 ms`) and `image-heavy-rotated-mask-sheet.pdf`
  (`0.362 ms` -> `0.327 ms`), but the repeat only kept a small mobile p95 win
  (`0.242 ms` -> `0.237 ms`) and regressed rotated-mask p95
  (`0.362 ms` -> `0.402 ms`). `scanner-large-image-budget.pdf` and
  `soft-mask-image.pdf` were also slightly worse in both candidate runs.
- Decision: reverted. Do not apply a broad row-slice stroke blend helper
  without a narrower target or lower-level evidence; the additional row helper
  and call shape are not a stable win across the image-heavy protection set.

## Phase 5: Session Cache, But Bounded

Goal: improve batch and multi-page workloads without introducing hidden global
state.

- [x] Keep global caches out of the renderer path.
- [x] Define an explicit request/session cache object for batch or multi-page
  rendering.
- [x] Enforce cache-entry budgets by bytes and item count.
- [x] Report session-retained bytes and item counts.
- [x] Cache parsed document/page tree data only inside the request/session.
- [x] Cache decoded image resources only when identity and budget are clear.
- [ ] Add font/form decoded-resource caches only after a profile names them as
  repeat bottlenecks.
- [x] Make cache use visible in benchmark output.

Acceptance:

- [ ] Repeat/batch benchmark shows a standalone or cumulative accepted
  improvement.
- [ ] Low-memory profile remains bounded.
- [ ] Cache invalidation is tied to document identity and render options.

Initial document-session result from 2026-06-29:

- Change: added `NativeDocumentSession<'a>` as an explicit request-local native
  session that borrows caller-owned PDF bytes and retains the parsed
  `ClassicDocument` plus `PageTree` without global state or disk persistence.
- Benchmark visibility: `benchmark-repeat-native` now reports
  `document-session` cache policy and per-record session stats:
  `input_bytes`, `loaded_objects`, `loaded_object_bytes`, `page_count`, and
  `first_page_only`.
- Baseline:
  `target/benchmark-repeat-report-vector-session-baseline.json`, native
  repeat benchmark, `fixtures/performance-matrix-manifest.tsv`,
  `--include-family report/vector`, `--max-edge 160`, 30 repetitions.
- After:
  `target/benchmark-repeat-report-vector-session-after.json`, same command and
  host.
- Result on one-page `report/vector` fixtures: family first-render mean
  improved `3.087 ms` -> `2.883 ms` (~6.6%), family repeat mean improved
  `2.834 ms` -> `2.787 ms` (~1.7%). Fixture-level repeat gains were mixed:
  `technical-hatch-clipping.pdf` repeat mean improved ~5.1%, while
  `vector-stress.pdf` was effectively neutral.
- Decision: keep the session boundary because it completes the required
  explicit, request-local cache shape and makes retained state visible in
  reports. Do not treat the current one-page repeat result as a standalone
  speed claim. The next Phase 5 optimization should target multi-page/batch
  reuse or decoded shared resources, where repeated document loading and shared
  resource decode are larger parts of total time.

Session budget result from 2026-06-29:

- Change: `NativeRenderLimits` now includes explicit document-session caps for
  retained indirect-object count and retained parsed object bytes. Default
  limits are `65,536` objects and `256 MiB`; low-memory limits are `16,384`
  objects and `32 MiB`.
- Enforcement: `NativeDocumentSession` rejects documents whose retained parsed
  object table exceeds those caps and maps the failure to the existing typed
  `renderer.memory-budget` unsupported bucket.
- Benchmark visibility: `benchmark-repeat-native` now writes
  `max_loaded_objects` and `max_loaded_object_bytes` beside the observed
  `loaded_objects` and `loaded_object_bytes`.
- Verification report:
  `target/benchmark-repeat-report-vector-session-budget.json`, same focused
  `report/vector` repeat benchmark as the first session run, completed with
  4/4 native rendered records, no fallback/error records, and visible session
  budget fields.
- Decision: this completes the bounded-cache part of the initial session
  contract. It is a low-memory/safety improvement, not a speed claim.

Rejected session cache candidate from 2026-06-30:

- Change tested locally but not kept: eagerly cache filtered page content and
  XObject invocation names inside `NativeDocumentSession`, with a new
  request-local page-content byte budget and JSON-visible session stats.
- Rationale: this was the smallest possible Phase 5 cache step before decoded
  image/font/form resource maps. It avoids repeated stream decode and repeated
  XObject invocation scans while keeping the cache request-local, bounded, and
  independent of output size/background options.
- Baseline:
  `target/benchmark-repeat-shared-session-prep-before.json`,
  `benchmark-repeat-native`, `fixtures/generated`,
  `fixtures/shared-resource-cache-manifest.tsv`, `--max-edge 160`, 30
  repetitions.
- Focused candidate:
  `target/benchmark-repeat-shared-session-prep-focused-after.json`, same
  command shape with explicit shared-resource family filters.
- Result: no accepted repeat/batch win. `repeated-image-xobject` repeat mean
  regressed `0.722 ms` -> `0.770 ms` (~6.6% slower), `long-document-shared`
  repeat mean regressed `0.383 ms` -> `0.386 ms`, and first render became
  slower for all five focused fixtures because eager prep moved content decode
  into session creation. Small wins on `repeated-font-image` and
  `repeated-font-program` were below the threshold and did not offset the
  protection regression.
- Decision: reverted. The next Phase 5 cache candidate should skip eager
  content-only caching and target decoded shared resources directly, or first
  add phase timing to `benchmark-repeat-native` so repeat-time resource decode
  can be isolated per fixture.

Repeat benchmark phase-timing instrumentation from 2026-06-30:

- Change: `NativeDocumentSession` now exposes `render_page_with_timings`, and
  `benchmark-repeat-native` records `phase_timings_ms.first` plus
  `phase_timings_ms.repeat_mean` for every native-rendered record.
- Purpose: Phase 5 cache work can now see whether repeat time is dominated by
  stream decode, content tokenization, resource decode, display-list build, or
  raster work before adding another cache.
- Smoke artifact:
  `target/benchmark-repeat-repeated-image-phase-timings.json`,
  `repeated-image-xobject`, `--max-edge 160`, five repetitions.
- Observed example: `image-heavy-repeated-xobject-report.pdf` repeat mean
  reported `resource_decode: 0.070 ms`, `raster_paths: 0.512 ms`,
  `raster_images: 0.072 ms`, and `total: 0.740 ms`. This points away from
  content-only caching and toward either path-raster work or a more targeted
  decoded-resource cache.
- Validation: targeted native session test, repeat benchmark JSON test,
  `cargo fmt --all --check`, `cargo check --workspace --no-default-features`,
  `cargo test --workspace --no-default-features`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.

Focused shared-resource phase evidence from 2026-06-30:

- Artifact:
  `target/benchmark-repeat-shared-phase-timings-current.json`,
  `benchmark-repeat-native`, `fixtures/generated`,
  `fixtures/shared-resource-cache-manifest.tsv`, five shared-resource
  families, `--max-edge 160`, 30 repetitions.
- Result: repeat-time `resource_decode` is not the dominant shared-resource
  cost in the focused set. `icc-rgb-image.pdf`, `subset-type3-repeated-charprocs.pdf`,
  and `longform-repeated-resources.pdf` show near-zero repeat `resource_decode`;
  `long-document-navigation-deck.pdf` and `longform-repeated-resources.pdf`
  are dominated by `raster_paths`; `image-heavy-repeated-xobject-report.pdf`
  is also dominated by `raster_paths` on repeat (`0.726 ms` of `1.046 ms`
  total).
- Decision: do not add another content/session cache from this evidence. The
  next cache step still needs a fixture where decoded shared resources are a
  named repeat bottleneck, or the work should return to path rasterization.

Accepted decoded image-resource session cache from 2026-06-30:

- Profiling trigger: after the zlib-rs Flate backend change, a fresh focused
  `benchmark-repeat-native` run on `fixtures/shared-resource-cache-manifest.tsv`
  showed only one credible decoded-resource repeat bottleneck:
  `image-heavy-repeated-xobject-report.pdf` spent `0.073 ms` of `0.354 ms`
  repeat mean in `resource_decode` / `resource_images`. Other shared-resource
  fixtures were dominated by raster/text work or had near-zero repeat resource
  decode.
- Change: `NativeDocumentSession` now keeps a request-local decoded
  `ImageResources` cache keyed by page index, `max_edge`, and native profile.
  The cache is bounded by entry count and resident decoded image-resource bytes,
  uses no global state or disk persistence, and is visible in repeat benchmark
  session stats as cached image-resource entries and bytes.
- Guardrail: resource maps smaller than `4 KiB` are not cached. The first
  ungated candidate proved this mattered: tiny 12-192 byte resource maps added
  overhead to small protection fixtures without meaningful reuse benefit.
- Repeated result against
  `target/benchmark-repeat-shared-phase-post-zlib-rs-focused-30.json`:
  `image-heavy-repeated-xobject-report.pdf` repeat mean improved
  `0.354 ms` -> `0.259 ms` (`~26.8%`) and then `0.257 ms` (`~27.4%`) on the
  repeat. The final post-refactor confirmation artifact
  `target/benchmark-repeat-shared-image-cache-final-30.json` measured
  `0.261 ms` (`~26.3%`). Repeat `resource_decode` moved from `0.073 ms` to
  `0.001 ms`, and the benchmark reported one cached image-resource entry
  retaining `20,736` bytes.
- Protection movement: `long-document-navigation-deck.pdf`,
  `longform-repeated-resources.pdf`, `subset-type3-repeated-charprocs.pdf`,
  and `icc-rgb-image.pdf` did not cache their tiny image-resource maps. Their
  repeat movement stayed at tiny absolute values around one micro-benchmark
  tick (`-3.4%` to `+5.9%`, at `0.001 ms` absolute scale) with no fallback,
  error, or budget records.
- Correctness guards: `native_document_session_should_cache_decoded_image_resources_with_budget`
  verifies repeated session renders keep identical output while retaining a
  bounded decoded image-resource entry. Memory diagnostics now expose the new
  session image-resource entry and byte budgets.
- Decision: accept as the Phase 5 decoded shared-resource cache step. This
  closes the open "cache decoded shared resources only when identity and budget
  are clear" item for decoded image resources; font/form resource caches still
  require their own profile-backed fixture before being added.

Repeat family phase-summary instrumentation from 2026-06-30:

- Change: `benchmark-repeat-native` now aggregates record-level
  `phase_timings_ms` into each family summary as `first_mean` and
  `repeat_mean`.
- Purpose: future cache and image/vector decisions can compare phase dominance
  directly from the family summary instead of requiring ad-hoc JSON inspection.
- Performance claim: none. This is benchmark instrumentation and should not be
  counted as a renderer speed win.

## Phase 6: Benchmark Gates And Claims

Goal: turn stable evidence into guardrails, not premature marketing.

- [x] Promote a stable fixture subset into a budget-free local smoke gate before
  adding CI timing budgets.
- [x] Keep the full matrix as a local maintainer tool until tool availability is
  reliable on CI.
- [x] Add a "performance claim update" checklist before changing README copy.
- [x] Keep MuPDF as v2 comparison backlog, not a blocker for the first
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
2. broader stroke raster candidate reduction for dense linework;
3. fixture-level stroke-shape histograms before another spatial-index variant;
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

Benchmark report platform metadata from 2026-06-30:

- Change: `PlatformMetadata` now records `rustc_version`, `logical_cpus`,
  `cpu_brand`, and `memory_bytes` in addition to OS, architecture, family,
  endian, and pointer width.
- Scope: all reports that already serialize `platform` get the expanded fields,
  including native benchmark, batch benchmark, repeat benchmark, matrix, visual
  diff, and comparison reports.
- Privacy: the added fields are host/tool descriptors only. They do not include
  PDF bytes, rendered pixels, private file paths, environment variables, or
  PDFium library paths.
- Best-effort policy: CPU brand and memory size are `null` when the host or
  sandbox does not expose them. The field presence is stable even when values
  are unavailable.
- Smoke artifact:
  `target/performance-matrix-platform-metadata-smoke.json`, native hot-render,
  `small-text`, `--max-edge 120`, 3 measured iterations after one warmup.
  The smoke reported `rustc 1.95.0-nightly`, 20 logical CPUs, and `null`
  CPU/memory fields in this sandbox.

Matrix timing reliability flags from 2026-06-30:

- Change: `benchmark-matrix` now emits a top-level `timing_reliability` object
  in JSON and a matching Markdown section.
- Signals: RSS sample availability, PDFium/Poppler request and availability,
  hot PDFium comparison availability, cold reference availability, and caveats.
- Caveat policy: `not-applicable`, `not-requested`, and `missing-tool` are
  distinct. Poppler hot-render records stay `not-applicable` because Poppler is
  an external process reference, not an in-process hot renderer.
- Acceptance impact: 5-10% wins can now be judged against explicit report
  caveats instead of relying on local memory of which reference tools or RSS
  fields were available.

Performance claim guardrail from 2026-06-30:

- Change: added `docs/policies/performance-claims.md` and
  `scripts/check_performance_claims.sh`.
- Checklist: public speed or memory copy now requires two stable matrix runs,
  host/tool/version context, timing reliability review, named workload family,
  named metric, local artifacts, and wording that avoids broad renderer parity.
- CI stance: the full matrix remains local maintainer tooling until variance and
  tool availability are understood; only focused subsets should become CI gates
  after budgets are documented.
- MuPDF stance: MuPDF remains v2 comparison backlog and must not block the
  first optimization wave.

Performance matrix smoke gate from 2026-06-30:

- Change: added `scripts/check_performance_matrix_smoke.sh` as a focused
  native hot-render smoke for one manifest family, defaulting to `small-text`,
  `--max-edge 120`, three measured iterations, and one warmup.
- Guardrail: the script validates JSON status only: non-empty records,
  `timing_reliability` presence, native/hot-render mode, all records rendered,
  no fallback, no missing tool, no errors, and numeric p95 values.
- CI stance: no p95 or wall-time budget is encoded yet. This is a safe first
  subset gate while variance and tool availability remain under observation.
- Docs: `docs/benchmarks.md` and `README.md` now list the smoke beside the
  existing performance-claim and native-only gates.
- Validation: `bash scripts/check_performance_matrix_smoke.sh` passed and wrote
  `target/performance-matrix-smoke.json` plus
  `target/performance-matrix-smoke.md`.

## Questions Closed For The Next Wave

- [x] What family-specific standalone and cumulative thresholds should replace
  the default 10% / repeatable 5-10% rule after we understand variance?

  Keep the default rule for now, but apply it by workload family instead of
  globally. `report/vector` and image-heavy work need a 10% p95 standalone win
  on the named target fixture, or repeated 5-10% wins on the same bottleneck
  track with a neutral protection set. For sub-millisecond image fixtures, p95
  must be supported by mean movement and a repeat run because tiny p95 values
  can swing sharply.

- [x] Should any focused performance subset become CI-gated, or should all
  benchmark budgets remain maintainer-local for now?

  Keep timing budgets maintainer-local. CI should run only budget-free smoke
  checks that prove the benchmark harness emits valid records and that selected
  native fixtures render without fallback or errors. Promote timing budgets only
  after repeated CI-host variance is documented.

- [x] Which `smallvec` inline capacities are justified by real path/token/clip
  histograms?

  None yet. Current evidence does not justify `SmallVec` in persistent renderer
  structures. Any future `SmallVec` candidate must first record p50, p95, p99,
  and max lengths for the specific collection, then prove that the inline
  capacity does not widen hot structs enough to hurt cache locality.

- [x] Which memory tool should be the default for allocation evidence on macOS:
  Instruments Allocations, heaptrack-equivalent tooling, or targeted counters in
  the renderer?

  Use Instruments Allocations when allocation stack detail is needed on macOS.
  For repeatable local gates and CI-friendly evidence, prefer targeted renderer
  counters in JSON reports. `sample` remains the first CPU profiler, but it is
  not sufficient allocation evidence by itself.
