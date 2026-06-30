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

Accepted opaque rectangle row-fill shortcut from 2026-06-30:

- Profiling trigger: after the image-resource cache work, the fresh
  `sample` run on `image-heavy-repeated-xobject-report.pdf`,
  `target/sample-repeated-xobject-current-refresh.txt`, still showed hot time
  dominated by `fill_path` / `blend_pixel` inside path rasterization rather
  than resource decode. The fixture generator confirms that the workload
  contains large opaque normal rectangle fills plus repeated image placements.
- Change: axis-aligned rectangle fills now write whole row slices directly when
  the source is fully opaque, the blend mode is `Normal`, and the effective
  fill alpha is full coverage. Fractional/antialiased fills, non-normal blend
  modes, transparent sources, and non-rectangular clips continue through the
  existing pixel blend path.
- Correctness guard:
  `fill_pixel_bounds_opaque_should_write_only_selected_rows` verifies the row
  helper writes only the requested pixel bounds and leaves surrounding pixels
  unchanged.
- A/B artifacts:
  `target/benchmark-native-repeated-xobject-rect-fill-row-base.json`,
  `target/benchmark-native-repeated-xobject-rect-fill-row-candidate.json`,
  `target/performance-matrix-image-heavy-rect-fill-row-candidate.json`, and
  `target/performance-matrix-report-vector-rect-fill-row-protection.json`.
- Focused result: `image-heavy-repeated-xobject-report.pdf` moved mean
  `0.354 ms` -> `0.274 ms` over 300,000 iterations (`~22.6%`). The image-heavy
  matrix moved the same fixture p95 `0.410 ms` -> `0.347 ms` versus
  `target/performance-matrix-image-heavy-current-refresh.json`.
- Protection result: the image-heavy matrix rendered all `8` records with no
  fallback or errors. The report/vector protection matrix rendered all `4`
  records with no fallback or errors; `vector-stress.pdf` p95 was `1.054 ms`,
  consistent with the prior vector range after the opaque blend shortcut.
- Decision: keep as a profile-backed path-raster optimization. It removes
  repeated per-pixel blend work for the common opaque-rectangle case without
  changing generic fill semantics, allocation strategy, dependency surface, or
  unsafe-code policy.

Accepted tiling-pattern rectangle fill shortcut from 2026-06-30:

- Profiling trigger: the broader corpus matrix
  `target/performance-matrix-corpus-post-row-incremental-axial-current.json`
  showed `tiling-pattern.pdf` and `uncolored-tiling-pattern.pdf` as the top two
  rendered fixtures at p95 `1.709 ms` and `1.413 ms`. Focused repeat artifacts
  `target/benchmark-repeat-tiling-pattern-current.json` and
  `target/benchmark-repeat-uncolored-tiling-pattern-current.json` measured
  repeat means `1.570 ms` and `1.304 ms`, almost entirely in `raster_paths`.
  The traces showed one filled rectangle with a tiling pattern color and no
  stroke or image work.
- Change: `fill_path_with_tiling_pattern` now reuses the existing
  center-sampled axis-aligned rectangle logic when the filled path is a rect
  and active clips are axis-aligned rectangles. It skips per-pixel supersample
  `point_in_path` checks for the rectangle interior, still resolves the tiling
  pattern color per pixel, and falls back to the previous generic path for
  fractional, non-rectangular, or non-axis-aligned cases.
- Correctness guard: existing tiling pattern render tests continue to exercise
  colored and uncolored pattern fills:
  `rasterize_paths_should_repeat_tiling_pattern_fill` and
  `rasterize_paths_should_apply_uncolored_tiling_pattern_fill_color`.
- Focused result:
  `target/benchmark-repeat-tiling-pattern-rect-fastpath-noinline-candidate.json`
  improved repeat mean `1.570 ms` -> `0.648 ms`; uncolored tiling improved
  `1.304 ms` -> `0.452 ms`.
- Protection result:
  `target/performance-matrix-corpus-tiling-rect-fastpath-candidate.json`
  preserved corpus status exactly (`217` rendered, `12` fallback-required,
  `4` errors). In that corpus matrix, `tiling-pattern.pdf` moved p95
  `1.709 ms` -> `0.725 ms`, mean `1.592 ms` -> `0.666 ms`, and
  `uncolored-tiling-pattern.pdf` moved p95 `1.413 ms` -> `0.533 ms`, mean
  `1.321 ms` -> `0.471 ms`. A direct same-host starter A/B
  (`target/performance-matrix-temp-baseline-after-tiling-check-starter.json`
  vs `target/performance-matrix-tiling-rect-fastpath-noinline-starter.json`)
  kept non-pattern mean movement in the `0.0%` to `3.5%` range.
- Decision: keep. This is an algorithmic path-raster reduction for a top corpus
  workload and keeps generic pattern fill semantics as the fallback path.

Rejected row-bucket range-capacity candidate from 2026-06-30:

- Profiling trigger: the fresh post-opaque-rect matrix,
  `target/performance-matrix-current-post-rect-row.json`, still showed
  `vector-stress.pdf` as the top native hot-render fixture at p95 `1.035 ms`.
  The paired long benchmark/sample artifacts,
  `target/benchmark-native-vector-stress-post-rect-row-profile-run.json`,
  `target/sample-vector-stress-post-rect-row.txt`, and
  `target/trace-vector-stress-post-rect-row.json`, reported mean `0.898 ms`,
  `raster_paths: 0.861 ms` of `1.073 ms`, and hot stacks in
  `stroke_path` / `rasterize_row_bucketed_stroke_ranges`.
- Change tested locally but not kept: store each stroke row/join bucket's
  maximum row occupancy and use that to preallocate temporary `x_ranges`,
  sorted row indices, and active row indices inside the row-bucket stroke
  rasterizer.
- Rationale: this targeted visible `Vec` growth, sort, and allocator frames in
  the sample without changing geometry, clipping, blend semantics, or adding
  dependencies.
- A/B artifacts:
  `target/benchmark-native-vector-stress-row-bucket-capacity-base.json` and
  `target/benchmark-native-vector-stress-row-bucket-capacity-candidate.json`,
  both `benchmark-native`, `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 150,000 iterations.
- Result: rejected as noise. Mean moved only `0.905 ms` -> `0.893 ms`
  (`~1.3%`), below the 5% floor for a cumulative performance commit.
- Decision: reverted. Row-bucket temporary allocation is visible but not large
  enough by itself. The next vector pass should reduce per-pixel stroke hit
  testing, range sorting/merging work, or the number of row-bucket candidate
  samples rather than only preallocating scratch vectors.

Rejected row-bucket sorted-merge candidate from 2026-06-30:

- Profiling trigger: the same `vector-stress.pdf` sample showed
  `merge_pixel_ranges` sort frames below
  `rasterize_row_bucketed_stroke_ranges`. Row-bucket line indices are already
  sorted by X bounds per row, so no-join row-bucket strokes can theoretically
  merge their X ranges without sorting again.
- Change tested locally but not kept: add a `merge_sorted_pixel_ranges` helper
  and use it only for row-bucket range rasterization when no join bucket ranges
  are appended. The generic `merge_pixel_ranges` path still sorted whenever
  join ranges or unsorted callers were involved.
- A/B artifacts:
  `target/benchmark-native-vector-stress-row-bucket-capacity-base.json` and
  `target/benchmark-native-vector-stress-row-bucket-sorted-merge-candidate.json`,
  both `benchmark-native`, `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 150,000 iterations.
- Result: rejected as below threshold. Mean moved `0.905 ms` -> `0.871 ms`
  (`~3.8%`). The direction is useful, but not enough to carry a standalone or
  cumulative commit under the current rules.
- Decision: reverted. Sorting contributes some cost, but the larger vector
  blocker remains per-pixel stroke coverage work. The next vector candidate
  should target active candidate reduction or a range-fill shortcut that
  eliminates sample tests for fully covered stroke interiors.

Accepted axis-aligned simple-line span routing from 2026-06-30:

- Profiling trigger: after the row-bucket capacity and sorted-merge rejections,
  the same `vector-stress.pdf` profile still pointed at per-sample stroke
  coverage work rather than allocation or sorting alone. The fixture contains
  many single-operation grid strokes such as `x 18 m x 104 l S` and
  `18 y m 144 y l S`, which were not eligible for the simple-line span
  rasterizer because only non-axis-aligned single lines used that route.
- Change: single-line strokes may now use `simple_line_stroke_raster_spans`
  regardless of axis alignment. Axis-aligned single lines get a lower pixel
  area threshold (`128` pixels) while non-axis-aligned lines keep the existing
  `1024` threshold. The final per-sample coverage check still goes through
  `point_in_single_stroke_line`, so the span route only narrows candidate
  pixels; it does not replace stroke semantics.
- Correctness guard:
  `simple_line_stroke_raster_spans_should_accept_axis_aligned_lines` verifies
  that the new axis-aligned span route covers all samples that the generic
  stroke predicate would cover for a representative long grid line.
- A/B artifacts:
  `target/benchmark-native-vector-stress-axis-simple-line-base.json`,
  `target/benchmark-native-vector-stress-axis-simple-line-candidate.json`,
  `target/benchmark-native-vector-stress-axis-simple-line-candidate-repeat.json`,
  `target/performance-matrix-report-vector-axis-simple-line-candidate.json`,
  and `target/performance-matrix-axis-simple-line-candidate.json`.
- Focused result: `vector-stress.pdf` moved mean `0.856 ms` -> `0.815 ms`
  (`~4.8%`) on the first 150,000-iteration candidate run, then `0.812 ms`
  (`~5.1%`) on the repeat. This lands as a cumulative 5% vector-track
  improvement, not a broad standalone claim.
- Protection result: the report/vector matrix rendered all `4` records with no
  fallback or errors, and the full starter matrix rendered all `11` records
  with no fallback or errors. Compared with
  `target/performance-matrix-current-post-rect-row.json`, the full matrix moved
  `vector-stress.pdf` p95 `1.035 ms` -> `0.898 ms` and kept the remaining
  fixture families rendered.
- Decision: keep. This reduces candidate stroke samples for common
  axis-aligned single-line grid strokes without adding dependencies, unsafe
  code, global state, or alternate stroke geometry.

Rejected lower axis-aligned simple-line span threshold from 2026-06-30:

- Profiling trigger: after the row-incremental axial shading win, the starter
  matrix still showed technical linework fixtures with visible `raster_paths`
  cost: `technical-hatch-clipping.pdf` repeat mean `0.266 ms` with
  `raster_paths` `0.168 ms`, and `technical-linework-dimensions.pdf` repeat
  mean `0.206 ms` with `raster_paths` `0.126 ms`. Their traces showed many
  snapped hairline items, so a narrower axis-aligned single-line threshold was
  tested before touching broader row-bucket logic.
- Change tested locally but not kept: lower
  `STROKE_AXIS_SIMPLE_LINE_SPAN_MIN_PIXELS` from `128` to `16`, so smaller
  axis-aligned single-line strokes route through `simple_line_stroke_raster_spans`.
- A/B artifacts:
  `target/benchmark-repeat-technical-hatch-post-row-incremental-axial-current.json`,
  `target/benchmark-repeat-technical-hatch-axis-simple-threshold16-candidate.json`,
  `target/benchmark-repeat-technical-linework-post-row-incremental-axial-current.json`,
  `target/benchmark-repeat-technical-linework-axis-simple-threshold16-candidate.json`,
  `target/benchmark-repeat-vector-stress-post-join-bounds-current.json`,
  and `target/benchmark-repeat-vector-stress-axis-simple-threshold16-candidate.json`.
- Result: rejected as a regression. Hatch repeat mean moved `0.266 ms` ->
  `0.274 ms`, linework `0.206 ms` -> `0.211 ms`, and vector-stress
  `0.706 ms` -> `0.723 ms`. The matching `raster_paths` phase moved about
  `+2.4%` on the three technical fixtures.
- Decision: reverted. The current `128` pixel threshold remains the better
  tradeoff; below that, span setup and range merging cost more than the reduced
  sample checks.

Rejected sampled-blend helper candidate from 2026-06-30:

- Profile basis: after the axis-aligned simple-line span routing win,
  `target/sample-vector-stress-post-axis-line-spans.txt` still showed
  `stroke_path -> blend_pixel` and `RasterDevice::pixel` as visible costs
  inside the remaining path raster hot section.
- Change tested locally but not kept: route sampled fill/stroke pixels through
  a small `SampledPixelBlend` helper so the supersample scale is computed once
  per raster pass and full-coverage opaque normal pixels skip the generic
  `blend_pixel` coverage calculation.
- A/B artifacts:
  `target/benchmark-native-vector-stress-post-axis-line-profile-run.json` and
  `target/benchmark-native-vector-stress-sampled-blend-candidate.json`, both
  `benchmark-native`, `fixtures/generated/vector-stress.pdf`, `--max-edge 160`,
  150,000 iterations.
- Result: rejected as below threshold. Mean moved `0.807 ms` -> `0.784 ms`
  (`~2.9%`). The direction confirms some blend overhead, but not enough for a
  5-10% cumulative vector-track commit.
- Decision: reverted. Do not retest this helper shape unless a later profile
  shows `blend_pixel` dominating more heavily or the candidate also eliminates
  per-pixel sample tests for fully covered stroke interiors.

Accepted exact axis-span raster result from 2026-06-30:

- Profile basis: `target/sample-vector-stress-post-axis-line-spans.txt` still
  showed the remaining vector hot section in `stroke_path`, including
  `blend_pixel`, `RasterDevice::pixel`, and span/point predicate work after the
  axis-aligned simple-line span routing commit.
- Change: add a narrow exact-span raster route for axis-aligned simple-line
  strokes and joinless axis-stroke span items when active clip checks can be
  skipped. The route keeps the existing sample coverage math, but tests sample
  X positions directly against the precomputed span row instead of constructing
  a `Point` and re-entering the generic stroke predicate. Fully covered opaque
  normal samples use the same direct-write condition as existing blend paths.
- Correctness guard:
  `exact_axis_line_span_raster_should_match_sampled_stroke_raster` renders the
  same axis-aligned square-capped line through the exact route and through the
  sampled fallback route, then compares the raw RGBA pixels byte-for-byte.
- A/B artifacts:
  `target/benchmark-native-vector-stress-post-axis-line-profile-run.json`,
  `target/benchmark-native-vector-stress-exact-span-candidate.json`,
  `target/performance-matrix-exact-span-candidate.json`, and
  `target/performance-matrix-exact-span-candidate-repeat.json`.
- Focused result: long `benchmark-native` mean moved `0.807 ms` -> `0.716 ms`
  (`~11.3%`) on `fixtures/generated/vector-stress.pdf`, `--max-edge 160`,
  150,000 iterations.
- Protection result: the repeated starter matrix rendered all `11` fixtures
  with no fallback or errors. In the `report/vector` family, repeat p95 moved
  `vector-stress.pdf` `0.912 ms` -> `0.827 ms`, `prepress-trim-bleed-marks.pdf`
  `0.487 ms` -> `0.466 ms`, `technical-hatch-clipping.pdf` `0.313 ms` ->
  `0.311 ms`, and `technical-linework-dimensions.pdf` `0.247 ms` -> `0.239 ms`.
- Decision: keep. This is an algorithmic culling win inside the same vector
  stroke bottleneck track; it adds no dependency, unsafe code, global cache, or
  alternate stroke geometry.

Rejected exact-span full-coverage row candidate from 2026-06-30:

- Profile basis: after the exact axis-span raster commit,
  `target/sample-vector-stress-post-exact-span.txt` showed
  `rasterize_span_covered_stroke_ranges` and `blend_pixel` as visible costs
  inside `stroke_path`. The current exact-span route still computes sample
  coverage for every candidate pixel, including interior pixels that are fully
  covered by all supersample rows.
- Change tested locally but not kept: compute full-coverage X ranges by
  intersecting all supersample span rows for each raster row, write those ranges
  directly when the blend is opaque normal, and keep the existing sampled loop
  only for the remaining partial edge ranges.
- A/B artifacts:
  `target/benchmark-native-vector-stress-post-exact-span-profile-run.json` and
  `target/benchmark-native-vector-stress-exact-span-full-ranges-candidate.json`,
  both `benchmark-native`, `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 150,000 iterations.
- Result: rejected as below threshold. Mean moved `0.764 ms` -> `0.738 ms`
  (`~3.4%`). The direction is technically coherent but too small for the
  amount of extra range-intersection and split-loop code.
- Decision: reverted. Revisit only if a later sample shows exact-span
  full-coverage pixels dominating more heavily, or if a simpler row-fill shape
  can avoid most of the added range plumbing.

Rejected borrowed row-bucket slice candidate from 2026-06-30:

- Profile basis: `target/sample-vector-stress-post-exact-span.txt` still showed
  allocator/copy-like frames around `stroke_path` and active row-bucket scans
  after the exact-span win. The active row-bucket rasterizer still copied each
  pre-sorted bucket row into scratch `Vec`s before scanning.
- Change tested locally but not kept: replace `sorted_row_line_indices` and
  `sorted_row_join_indices` scratch copies with borrowed row slices from the
  already sorted flat bucket index.
- Initial signal:
  `target/benchmark-native-vector-stress-borrowed-row-slices-candidate.json`
  appeared to move mean `0.764 ms` -> `0.714 ms`, but the matrix p95 movement
  was small and mixed.
- Fresh A/B artifacts:
  `target/benchmark-native-vector-stress-borrowed-row-slices-fresh-base.json`
  and
  `target/benchmark-native-vector-stress-borrowed-row-slices-fresh-candidate.json`,
  both `benchmark-native`, `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 150,000 iterations.
- Result: rejected as noise. The direct fresh A/B moved mean `0.721 ms` ->
  `0.721 ms`. Starter matrices still rendered with no fallback or errors, but
  the performance effect did not repeat.
- Decision: reverted. Borrowing the sorted row slices is cleaner on paper, but
  it does not measurably move the current hot fixture. Keep future row-bucket
  work focused on reducing visited candidate pixels or predicate calls, not
  local slice-copy cleanup.

Accepted axial shading raster fast path from 2026-06-30:

- Profile basis: after the exact axis-span work, the next starter top fixture
  outside the vector-stroke track was
  `fixtures/generated/slide-title-gradient.pdf`. `trace-native` reported total
  `0.763 ms`, with `raster_paths` at `0.439 ms`, `resource_decode` at
  `0.105 ms`, `display_list_build` at `0.094 ms`, and an operator summary with
  one `sh` axial-shading operation. The target is therefore shading raster
  work, not text, image, or output encoding.
- Change: axial shading rasterization now converts start/end colors to `Rgba`
  once per shading item, shares the projection/extension calculation through a
  small helper, skips `powf` for the common exponent `1.0` case, and writes
  opaque `BlendMode::Normal` pixels directly through the raster row API.
  Non-normal blend modes continue through `blend_pixel`.
- Correctness rationale: sampled axial shading colors are opaque in the current
  implementation, so full-coverage normal source-over is equivalent to writing
  the source pixel. The extension and interpolation behavior is preserved in
  the helper path, and the existing axial sampling test now covers the
  precomputed-RGBA color interpolation helper.
- A/B artifacts:
  `target/trace-slide-title-gradient-current.json`,
  `target/benchmark-native-slide-title-gradient-current.json`,
  `target/benchmark-native-slide-title-gradient-axial-fastpath-candidate.json`,
  `target/benchmark-native-slide-title-gradient-axial-fastpath-final.json`,
  and `target/performance-matrix-axial-shading-fastpath-candidate.json`.
- Focused result: long `benchmark-native` mean moved `0.504 ms` -> `0.304 ms`
  (`~39.7%`) on `fixtures/generated/slide-title-gradient.pdf`, `--max-edge
  160`, 150,000 iterations. The pre-refactor candidate run measured
  `0.310 ms`, so the final Clippy cleanup did not remove the speedup.
- Protection result: the starter native hot-render matrix rendered all `11`
  fixtures with no fallback or errors. In that matrix,
  `slide-title-gradient.pdf` moved from the previous repeat p95 `0.550 ms` and
  mean `0.504 ms` to p95 `0.349 ms` and mean `0.312 ms`, with unchanged
  output dimensions and `57600` output bytes.
- Decision: keep. This is a profile-backed scalar raster fast path with no new
  dependency, unsafe code, global cache, or alternate shading semantics.

Accepted row-incremental axial shading projection from 2026-06-30:

- Profile basis: after the prepared join-bounds optimization,
  `target/performance-matrix-post-prepared-join-bounds-current.json` still
  showed `slide-title-gradient.pdf` as a top starter fixture at p95
  `0.329 ms`, mean `0.301 ms`. The focused repeat artifact
  `target/benchmark-repeat-slide-post-join-bounds-current.json` measured repeat
  mean `0.294 ms`, with `raster_paths` at `0.245 ms`. The matching trace
  `target/trace-slide-post-join-bounds-current.json` showed one axial `sh`
  operation and no stroke work, so the hot path was axial shading rasterization.
- Change: axial shading now computes the first projected `t` value for each
  raster row plus a constant per-pixel step, then advances `t` by addition
  inside the row loop. The existing extend/clip behavior remains centralized in
  `AxialShadingProjection::clamp_t`, and exponent handling is preclassified so
  the common linear exponent path avoids a per-pixel epsilon comparison.
- Correctness guard:
  `axial_shading_projection_should_advance_linearly_across_row` checks the
  incremental row projection, and
  `axial_shading_sampling_should_interpolate_colors` continues to cover linear
  color interpolation.
- Focused result:
  `target/benchmark-repeat-slide-row-incremental-axial-candidate.json` improved
  repeat mean `0.294 ms` -> `0.248 ms`, with `raster_paths` moving
  `0.245 ms` -> `0.202 ms`. The repeat
  `target/benchmark-repeat-slide-row-incremental-axial-candidate-repeat.json`
  measured repeat mean `0.244 ms`, with `raster_paths` at `0.199 ms`.
- Protection result:
  `target/performance-matrix-row-incremental-axial-candidate.json` rendered all
  `11` starter records with no fallback or errors. In that matrix,
  `slide-title-gradient.pdf` moved p95 `0.329 ms` -> `0.305 ms` and mean
  `0.301 ms` -> `0.264 ms`. `vector-stress.pdf` stayed neutral on mean
  (`0.724 ms` -> `0.720 ms`), and other fixture movements were either positive
  or small absolute timing noise.
- Decision: keep. This is a narrow arithmetic reduction in the already sampled
  axial shading bottleneck, with no dependency, unsafe code, cache, or visual
  behavior change.

Accepted radial shading raster fast path from 2026-06-30:

- Profile basis: after the row-incremental axial shading work, the focused
  radial fixture `fixtures/generated/radial-gradient.pdf` measured repeat mean
  `0.925 ms`; `trace-native` reported one shading operation, no text or image
  work, and `raster_paths` at `0.933 ms` of `1.036 ms` total. The target was
  therefore radial shading rasterization.
- Change: radial shading now computes center distance, start/end `Rgba`
  colors, and the linear-exponent flag once per shading item. The row loop
  hoists the `y` sample and delta, uses the shared shading interpolation
  helper, and writes opaque `BlendMode::Normal` pixels directly through the
  raster row API. Non-normal blend modes still use `blend_pixel`.
- Correctness guard:
  `radial_shading_sampling_should_interpolate_colors` continues to cover radial
  interpolation through a test-only wrapper around the shared color sampler.
- Focused result:
  `target/benchmark-repeat-radial-gradient-fastpath-candidate.json` improved
  repeat mean `0.925 ms` -> `0.527 ms` and p95 `0.997 ms` -> `0.558 ms`.
  The matching trace
  `target/trace-radial-gradient-fastpath-candidate.json` moved `raster_paths`
  `0.933 ms` -> `0.518 ms`.
- Protection result:
  `target/benchmark-repeat-slide-radial-fastpath-candidate.json` kept the
  axial slide fixture within noise after the previous shading optimizations:
  repeat mean `0.244 ms` -> `0.252 ms`, p95 `0.264 ms` -> `0.270 ms`.
  `target/benchmark-repeat-vector-stress-radial-fastpath-candidate.json` stayed
  neutral on p95 (`0.798 ms` -> `0.796 ms`) with no errors.
- Decision: keep. This is a profile-backed scalar raster fast path with a
  large isolated radial-gradient win and no dependency, unsafe code, cache, or
  clipping/blend behavior change for the supported normal opaque path.

Rejected prepared row-bucket line-metrics candidate from 2026-06-30:

- Profile basis: after the axial-shading fast path,
  `target/performance-matrix-current-post-axial-shading.json` still showed
  `vector-stress.pdf` as the slowest starter fixture at p95 `0.792 ms`.
  `target/trace-current-post-axial-vector-stress.json` reported total
  `1.462 ms`, with `raster_paths` at `0.878 ms`. A fresh CPU sample,
  `target/sample-vector-stress-current-post-axial.txt`, showed the dominant
  remaining stacks in `stroke_path`, especially
  `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `blend_pixel`, and
  `point_in_single_stroke_line`.
- Change tested locally but not kept: store precomputed `dx`, `dy`, and
  `len_squared` on `BoundedStrokeLine`, then route row-bucket stroke candidate
  checks through distance helpers that reuse those prepared values instead of
  recalculating line metrics for every sample.
- A/B artifacts:
  `target/benchmark-native-vector-stress-current-post-axial-profile-run.json`
  and
  `target/benchmark-native-vector-stress-prepared-line-metrics-candidate.json`,
  both `benchmark-native`, `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 160,000 iterations.
- Result: rejected as below threshold. Mean moved `0.728 ms` -> `0.716 ms`
  (`~1.6%`). The direction is positive, but too small for a standalone or
  cumulative vector-track commit.
- Decision: reverted. Prepared line metrics avoid some arithmetic inside the
  row-bucket predicate, but the current blocker is broader span/range
  rasterization and blending work. Revisit only if a future profile shows line
  metric recomputation dominating independently.

Rejected span row-write and sorted-range merge candidate from 2026-06-30:

- Profile basis: the same `target/sample-vector-stress-current-post-axial.txt`
  run showed `blend_pixel`, `rasterize_span_covered_stroke_ranges`, and
  `merge_pixel_ranges` below `stroke_path`. This suggested a cumulative
  vector-raster candidate: avoid per-pixel `set_pixel`/`pixel` dispatch for
  opaque normal span samples, and avoid sorting row-bucket X ranges that are
  already appended in row-bucket X order when no join ranges are present.
- Changes tested locally but not kept:
  - add a normal/opaque row-based variant of
    `rasterize_span_covered_stroke_ranges`, using `row_mut` plus
    `composite_image_pixel_in_row` for covered samples;
  - add `merge_sorted_pixel_ranges` and route no-join row-bucket raster ranges
    through it, keeping the generic sorting merge for join ranges and other
    callers.
- A/B artifacts:
  `target/benchmark-native-vector-stress-current-post-axial-profile-run.json`,
  `target/benchmark-native-vector-stress-row-opaque-span-candidate.json`, and
  `target/benchmark-native-vector-stress-row-opaque-sorted-ranges-candidate.json`,
  all `benchmark-native`, `fixtures/generated/vector-stress.pdf`, `--max-edge
  160`, 160,000 iterations.
- Result: rejected as below threshold. The row-write candidate moved mean
  `0.728 ms` -> `0.708 ms` (`~2.7%`). The combined row-write plus sorted-range
  candidate moved mean only `0.728 ms` -> `0.711 ms` (`~2.3%`).
- Decision: reverted. These local reductions touch real sampled stacks, but
  they do not move the current top fixture enough to justify additional raster
  branches. The next vector attempt needs a broader algorithmic reduction in
  sampled stroke/range work, or it should switch to a different top family.

Rejected lower join-bucket threshold candidate from 2026-06-30:

- Profile basis: after repeated local `vector-stress` rejections,
  `prepress-trim-bleed-marks.pdf` was the next slowest starter fixture.
  `target/trace-current-post-axial-prepress-trim-bleed-marks.json` showed a
  mixed profile with `raster_paths` `0.428 ms`, `resource_decode` `0.176 ms`,
  and `display_list_build` `0.156 ms`. A longer sample,
  `target/sample-prepress-current-post-axial.txt`, showed the raster part
  dominated by `stroke_path` and especially `point_in_join`.
- Change tested locally but not kept: lower `STROKE_JOIN_BUCKET_MIN_JOINS` from
  `8` to `4`, so the small joined prepress stroke items could use join buckets
  instead of scanning joins directly.
- A/B artifacts:
  `target/benchmark-native-prepress-current-post-axial-profile-run.json` and
  `target/benchmark-native-prepress-join-bucket4-candidate.json`, both
  `benchmark-native`, `fixtures/generated/prepress-trim-bleed-marks.pdf`,
  `--max-edge 160`, 220,000 iterations.
- Result: rejected as a regression. Mean moved `0.421 ms` -> `0.432 ms`
  (`~2.6%` slower).
- Decision: reverted. Even where `point_in_join` is prominent, the bucket
  setup and query overhead is too high for these small joined strokes. Keep the
  join-bucket threshold at `8` until a profile identifies larger joined stroke
  items or a cheaper join-index representation.

Accepted streamed fallback text rasterization from 2026-06-30:

- Profile basis: after the vector and axial-shading work, the next useful
  non-vector sample targeted `fixtures/generated/office-report-header-footer-link.pdf`.
  `target/trace-office-report-current.json` showed `raster_text` as the largest
  single phase (`0.104 ms`), followed by `display_list_build` (`0.086 ms`) and
  `content_tokenize` (`0.045 ms`). The matching macOS sample,
  `target/sample-office-report-current.txt`, was dominated by `draw_text_run`
  and showed allocation/grow samples around fallback text scratch and glyph
  bitmap cache storage.
- Change: fallback text rasterization now streams decoded glyph characters
  directly from each `TextDisplayItem` instead of first building a temporary
  `TextRasterScratch` atom vector. Combining-mark placement and ligature
  expansion remain covered by raster behavior tests. The glyph bitmap cache also
  lazily reserves its bounded entry capacity on the first cached glyph insert,
  avoiding repeated growth without allocating on pages that never paint fallback
  text.
- Focused A/B artifacts:
  `target/benchmark-native-office-report-text-cache-base.json`,
  `target/benchmark-native-office-report-stream-text-candidate.json`,
  `target/benchmark-native-ebook-stream-text-base.json`,
  `target/benchmark-native-ebook-stream-text-candidate.json`, and
  `target/benchmark-native-ebook-stream-text-candidate-repeat.json`, all
  `benchmark-native`, `--max-edge 160`, 30,000 iterations.
- Result: office text improved mean `0.230 ms` -> `0.222 ms` (`~3.5%`), which
  was useful but below threshold on its own. The text-heavy
  `ebook-narrow-longform.pdf` fixture improved mean `0.275 ms` -> `0.161 ms`
  (`~41.5%`) and repeated at `0.161 ms`.
- Protection: `target/performance-matrix-stream-text-candidate.json` ran the
  generated fixture directory with performance-manifest classifications,
  rendered `217` native records, preserved the existing fallback/error buckets,
  and reported starter-family means including `office-export` `0.226 ms`,
  `small-text` `0.040 ms`, `presentation` `0.321 ms`, and `report/vector`
  `0.433 ms`.
- Decision: accepted as a text/office-export performance win. The change removes
  a per-run temporary allocation path rather than adding a new raster branch, so
  it is low risk for non-text pages and directly addresses the sampled text
  bottleneck.

Accepted row-slice text rectangle fill from 2026-06-30:

- Profiling basis: after the streamed fallback text work,
  `target/benchmark-repeat-office-report-head-sample-run.json` measured
  `fixtures/generated/office-report-header-footer-link.pdf` at repeat mean
  `0.206 ms`, with `raster_text` still the largest repeat phase at `0.089 ms`.
  The matching CPU sample, `target/sample-office-report-head-sample-run.txt`,
  was dominated by `draw_text_run`; the largest child stacks were
  `fill_device_rect` and `source_over`, with additional samples in
  per-pixel `RasterDevice::pixel` reads.
- Change: `fill_device_rect` now takes one mutable row slice per affected row,
  computes y coverage once, reuses the existing 1D coverage helper for x
  coverage, and writes/blends directly inside the row. Opaque source-over on
  opaque destination pixels uses the existing `source_over_opaque` helper,
  avoiding the general alpha-normalization path for the common fallback text
  case.
- Focused A/B artifacts:
  `target/benchmark-repeat-office-report-head-sample-run.json`,
  `target/benchmark-repeat-office-report-row-rect-candidate.json`, and
  `target/benchmark-repeat-office-report-row-rect-candidate-repeat.json`, all
  `benchmark-repeat-native`, `--max-edge 160`, 160,000 iterations.
- Result: office report repeat mean improved `0.206 ms` -> `0.175 ms`
  (`~15.0%`) on the first candidate run and repeated at `0.181 ms`
  (`~12.1%`). The text phase improved `0.089 ms` -> `0.062 ms`
  (`~30.3%`) and repeated at `0.062 ms`.
- Protection: `target/performance-matrix-row-rect-candidate.json` and
  `target/performance-matrix-row-rect-candidate-repeat.json` both rendered the
  11-fixture starter matrix without fallback or errors. Short-run p95 was noisy
  on unrelated fixtures, so focused repeats were used for the flagged vector
  cases: `target/benchmark-repeat-vector-stress-row-rect-candidate.json`
  measured repeat mean `0.671 ms` versus the current baseline artifact
  `target/benchmark-repeat-vector-stress-profile-run-current.json` at
  `0.681 ms`, and `target/benchmark-repeat-prepress-row-rect-candidate.json`
  measured repeat mean `0.307 ms`, below the short-matrix baseline mean
  `0.325 ms`.
- Decision: accepted as a text/office-export raster win. The change removes
  repeated device-level bounds checks from rectangle fills and keeps the
  existing coverage model, so visual semantics stay covered by the subpixel
  rectangle raster test.

Rejected inline snapped-stroke scratch candidate from 2026-06-30:

- Profiling basis: after the row-slice text rectangle fill, the next non-text
  profile targeted `fixtures/generated/technical-hatch-clipping.pdf`.
  `target/trace-technical-hatch-head-after-row-rect.json` measured
  `raster_paths` as the dominant phase (`0.201 ms` first render). The focused
  repeat artifact `target/benchmark-repeat-technical-hatch-head-sample-run.json`
  measured repeat mean `0.262 ms`, with `raster_paths` `0.162 ms`; the matching
  CPU sample `target/sample-technical-hatch-head-sample-run.txt` was dominated
  by `stroke_path` and showed allocator samples around snapped hairline
  `Vec<LineSegment>` / join scratch construction.
- Change tested locally but not kept: use fixed inline arrays for up to eight
  snapped hairline lines and joins before falling back to the existing
  `Vec`-based path. This targeted the hatch fixture shape, where
  `stroke_shape_summary` reported 92 stroked items, 90 snapped hairline items,
  and at most six lines per item.
- A/B artifacts:
  `target/benchmark-repeat-technical-hatch-head-sample-run.json` and
  `target/benchmark-repeat-technical-hatch-inline-snap-candidate.json`, both
  `benchmark-repeat-native`, `--max-edge 160`, 140,000 iterations.
- Result: rejected as noise. Repeat mean moved only `0.262 ms` -> `0.261 ms`;
  `raster_paths` stayed `0.162 ms` -> `0.162 ms`.
- Decision: reverted. The allocator samples are real, but inline scratch for
  snapped hairline line/join vectors is too small a standalone win. Do not
  retry this exact small-array replacement unless allocation counters show a
  larger contribution or it is folded into a broader stroke scratch reuse
  design.

Repeat family phase-summary instrumentation from 2026-06-30:

- Change: `benchmark-repeat-native` now aggregates record-level
  `phase_timings_ms` into each family summary as `first_mean` and
  `repeat_mean`.
- Purpose: future cache and image/vector decisions can compare phase dominance
  directly from the family summary instead of requiring ad-hoc JSON inspection.
- Performance claim: none. This is benchmark instrumentation and should not be
  counted as a renderer speed win.

Current vector profile and rejected joinless axis-span reuse candidate from
2026-06-30:

- Profiling basis:
  `target/performance-matrix-current-profile-selection.json` rendered all `11`
  starter records with no fallback or errors. `vector-stress.pdf` remained the
  slowest fixture at mean `0.750 ms`, p95 `0.859 ms`. The focused baseline
  `target/benchmark-repeat-vector-stress-current-profile-selection.json`
  measured repeat mean `0.728 ms`, with `raster_paths` at `0.634 ms`.
- CPU sample:
  `target/sample-vector-stress-current-profile-selection.txt` sampled a longer
  `benchmark-repeat-native` run. The dominant top-of-stack buckets were
  `stroke_path` (`1191` samples), `blend_pixel` (`859`),
  `rasterize_row_bucketed_stroke_ranges` (`739`), and
  `point_in_single_stroke_line` (`430`). Allocation/free frames were still
  visible inside `stroke_path`, but the top predicate and blend work remained
  larger than the specific allocation hypothesis.
- Candidate: avoid materializing a duplicate `AxisStrokeSpans` raster structure
  for joinless axis-aligned strokes by making `AxisStrokeRasterSpans::raster`
  optional and reusing `coverage` when `joins.is_empty()`.
- Result: rejected. The focused candidate
  `target/benchmark-repeat-vector-stress-axis-span-reuse-candidate.json`
  regressed repeat mean `0.728 ms` -> `0.820 ms`, with `raster_paths` moving
  `0.634 ms` -> `0.729 ms`. The repeat
  `target/benchmark-repeat-vector-stress-axis-span-reuse-candidate-repeat.json`
  confirmed repeat mean `0.821 ms`, with `raster_paths` at `0.730 ms`.
- Decision: reverted. The shape looked allocation-friendly, but it worsened the
  current compiler/codegen/runtime path reproducibly. Do not reopen this exact
  `Option<AxisStrokeSpans>` form unless a lower-level allocation profile shows
  a different construction strategy can avoid the branch/shape regression.

Rejected flat axis-span clone candidate from 2026-06-30:

- Profile basis: after the radial shading fast path, the fresh starter matrix
  `target/performance-matrix-current-post-radial-starter.json` still showed
  `vector-stress.pdf` as the slowest fixture at mean `0.744 ms`, p95
  `0.863 ms`. The focused baseline
  `target/benchmark-repeat-vector-stress-current-post-radial.json` measured
  repeat mean `0.711 ms`, p95 `0.813 ms`, with `raster_paths` at `0.620 ms`.
  A long `sample` artifact at `target/sample-vector-stress-post-radial.txt`
  showed `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `blend_pixel`, `merge_pixel_ranges`,
  and stroke allocation/free frames as the visible CPU work.
- Change tested locally but not kept: when `axis_stroke_raster_spans` had no
  joins, return `AxisStrokeRasterSpans { raster: coverage.clone(), coverage }`
  instead of converting coverage back through `Vec<Vec<AxisStrokeSpan>>` and
  rebuilding a merged raster span set.
- Rationale: this tested a narrower construction strategy than the previously
  rejected `Option<AxisStrokeSpans>` shape while preserving the existing
  `AxisStrokeRasterSpans` fields and raster call sites.
- Result: rejected. The primary target did not clear the repeated 5% bar:
  `target/benchmark-repeat-vector-stress-flat-axis-clone-candidate.json`
  measured repeat mean `0.715 ms` and p95 `0.806 ms` versus the fresh baseline
  repeat mean `0.711 ms` and p95 `0.813 ms`. Against the matching accepted
  prepress join-bounds artifact,
  `target/benchmark-repeat-prepress-flat-axis-clone-candidate.json` moved mean
  `0.326 ms` -> `0.320 ms` and p95 `0.364 ms` -> `0.348 ms`, still below 5%.
  `technical-hatch-clipping.pdf` moved mean `0.266 ms` -> `0.272 ms` and p95
  `0.296 ms` -> `0.302 ms`.
- Decision: reverted. The flat clone avoids one obvious reconstruction path,
  but the measured effect is below the acceptance threshold and slightly noisy
  across protection fixtures. Do not reopen this local construction-only axis
  span variant unless a later allocation profile shows a larger isolated cost.

Rejected full-coverage axis-span raster candidate from 2026-06-30:

- Profile basis: the same post-radial CPU sample
  `target/sample-vector-stress-post-radial.txt` showed
  `rasterize_span_covered_stroke_ranges`, `blend_pixel`, and per-pixel stroke
  sample checks in the remaining vector hot path. This retested the open idea
  from the earlier sampled-blend rejection: only revisit blend-adjacent work if
  the candidate also eliminates per-pixel sample tests for fully covered stroke
  interiors.
- Change tested locally but not kept: compute full-coverage pixel X ranges for
  joinless axis-span strokes by intersecting the per-sample-row span interiors.
  Full-coverage opaque normal pixels were written directly, while edge pixels
  stayed on the existing sampled coverage path.
- Result: rejected. The primary target regressed on mean:
  `target/benchmark-repeat-vector-stress-full-span-candidate.json` measured
  repeat mean `0.732 ms` and p95 `0.810 ms` versus the fresh baseline
  `0.711 ms` and p95 `0.813 ms`. `prepress-trim-bleed-marks.pdf` stayed
  effectively flat against the accepted join-bounds artifact at mean
  `0.326 ms` -> `0.326 ms`, p95 `0.364 ms` -> `0.353 ms`.
  `technical-hatch-clipping.pdf` moved mean `0.266 ms` -> `0.264 ms`, p95
  `0.296 ms` -> `0.292 ms`, not enough to offset the primary regression.
- Decision: reverted. The extra full-range intersection work costs more than
  it saves on the current `vector-stress` shape. Reopen only with row-level
  coverage histograms showing many wide fully-covered interiors and a cheaper
  way to split full and edge pixels.

Rejected flat axis-span builder candidate from 2026-06-30:

- Profile basis: `target/sample-vector-stress-post-radial.txt` showed visible
  allocator frames under `stroke_path`, including `Vec` growth around axis-span
  construction. The candidate targeted the `axis_stroke_spans` builder, not the
  already rejected raster-time clone/full-coverage variants.
- Change tested locally but not kept: replace the per-sample-row
  `Vec<Vec<AxisStrokeSpan>>` construction in `axis_stroke_spans` with one flat
  `Vec<AxisStrokeRowSpan>`, sorted by row and `min_x`, then merged into the
  same final `AxisStrokeSpans` shape.
- Result: rejected. The primary target regressed:
  `target/benchmark-repeat-vector-stress-flat-axis-spans-candidate.json`
  measured repeat mean `0.733 ms` and p95 `0.830 ms` versus the fresh baseline
  `0.711 ms` and p95 `0.813 ms`. `prepress-trim-bleed-marks.pdf` was neutral
  to slightly slower on mean (`0.326 ms` -> `0.329 ms`), and
  `technical-hatch-clipping.pdf` regressed mean `0.266 ms` -> `0.278 ms` and
  p95 `0.296 ms` -> `0.308 ms`.
- Decision: reverted. The single flat allocation removes many tiny row vectors,
  but the global sort and row-tagged records cost more than they save on the
  current vector fixtures. Reopen only with allocation counters showing row-Vec
  growth as a larger standalone cost or with a two-pass flat builder that
  avoids global sorting.

Accepted two-pass axis-span builder from 2026-06-30:

- Profile basis: `target/sample-vector-stress-post-radial.txt` showed allocator
  work under `stroke_path` around axis-span construction, while the flat
  globally-sorted builder above regressed. The follow-up target was therefore a
  flatter builder that keeps per-row sorting local and avoids the many small
  `Vec` allocations from `Vec<Vec<AxisStrokeSpan>>`.
- Change: `axis_stroke_spans` now counts span entries per sample row, allocates
  one flat span buffer, fills it in a second pass, sorts each row slice in
  place, and merges into the existing final `AxisStrokeSpans` representation.
  Raster-time structures, coverage semantics, joins, clips, blend behavior, and
  public APIs are unchanged.
- Correctness guard:
  `axis_stroke_spans_should_match_generic_axis_strokes`,
  `axis_stroke_raster_spans_should_cover_joined_axis_strokes`, and
  `round_axis_stroke_span_should_shrink_beyond_vertical_endpoint` passed
  against the new builder.
- Focused result:
  `target/benchmark-repeat-vector-stress-two-pass-axis-spans-candidate.json`
  improved repeat mean `0.711 ms` -> `0.676 ms` and p95 `0.813 ms` ->
  `0.749 ms`, with `raster_paths` moving `0.620 ms` -> `0.585 ms`. The repeat
  `target/benchmark-repeat-vector-stress-two-pass-axis-spans-repeat.json`
  measured repeat mean `0.673 ms`, p95 `0.752 ms`, and `raster_paths`
  `0.583 ms`.
- Protection result:
  `target/benchmark-repeat-prepress-two-pass-axis-spans-candidate.json` moved
  mean `0.326 ms` -> `0.315 ms`, p95 `0.364 ms` -> `0.340 ms`.
  `target/benchmark-repeat-technical-hatch-two-pass-axis-spans-candidate.json`
  moved mean `0.266 ms` -> `0.262 ms`, p95 `0.296 ms` -> `0.288 ms`.
  `target/performance-matrix-two-pass-axis-spans-starter.json` rendered all
  `11` starter records with no fallback, missing-tool, not-applicable, or
  errors; report/vector p95 moved `0.863 ms` -> `0.794 ms`.
- Caveat: single-run trace artifacts remained noisy
  (`target/trace-vector-stress-two-pass-axis-spans.json` did not show a
  reliable phase reduction), so this acceptance is based on repeated hot-render
  timings and status-neutral starter protection.
- Decision: keep. This is a cumulative 5-10% vector/report win on the same
  profiled `stroke_path` allocation/build track, without a dependency, unsafe
  code, cache, or raster semantics change.

Rejected row-slice stroke blend candidate from 2026-06-30:

- Profile basis:
  `target/performance-matrix-current-after-axis-spans-native-hot.json` kept
  `vector-stress.pdf` as the slowest starter fixture at p95 `0.854 ms`.
  `target/sample-vector-stress-after-two-pass-axis-spans.txt`, captured from a
  long release `benchmark-repeat-native` run, showed `stroke_path` still
  dominating. The visible subpaths were
  `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `blend_pixel`,
  `merge_pixel_ranges`, and `point_in_single_stroke_line`.
- Candidate:
  route the hot stroke range loops through a borrowed row slice and a local
  row-blend helper, avoiding repeated `RasterDevice::pixel` /
  `RasterDevice::set_pixel` offset checks for every covered pixel. Coverage
  math, clipping decisions, blend semantics, and output dimensions were
  unchanged.
- Focused result:
  `target/benchmark-repeat-vector-stress-current-after-two-pass-axis-spans-1000.json`
  measured repeat mean `0.685 ms`, p95 `0.771 ms`, and `raster_paths`
  `0.594 ms`. The candidate
  `target/benchmark-repeat-vector-stress-row-blend-candidate-1000.json`
  measured repeat mean `0.653 ms`, p95 `0.737 ms`, and `raster_paths`
  `0.561 ms`.
- Protection result:
  `target/benchmark-repeat-prepress-row-blend-candidate-1000.json` measured
  repeat mean `0.324 ms` versus the recent two-pass baseline `0.315 ms`, and
  `target/benchmark-repeat-technical-hatch-row-blend-candidate-1000.json`
  measured `0.264 ms` versus `0.262 ms`. Both remained rendered without
  fallback or errors, but the candidate was not protection-set neutral.
- Decision:
  reverted. The primary fixture moved only about `4.7%` on mean and `4.4%` on
  p95, below the repeated 5% threshold, and the protection set showed small
  regressions. The useful signal is that remaining `blend_pixel` samples are
  real, but a broad row-slice helper is not the next accepted shape. Reopen only
  with a narrower `span_covered` or full-coverage range path that clears the
  threshold without hurting prepress.

Accepted common shading exponent ratio fast paths from 2026-06-30:

- Profile basis:
  after the vector-focused wins, the current native hot-render matrix
  `target/performance-matrix-current-after-axis-spans-native-hot.json` showed
  `slide-title-gradient.pdf` as the next non-vector hotspot at p95 `0.304 ms`.
  The focused baseline
  `target/benchmark-repeat-slide-current-after-axis-spans-1000.json` measured
  repeat mean `0.247 ms`, p95 `0.269 ms`, and `raster_paths` `0.201 ms`.
  The long CPU profile
  `target/sample-slide-title-gradient-after-axis-spans.txt` put
  `rasterize_shading_item -> pow` at the top of the sample tree.
- Change:
  `sample_shading_color_from_rgba` now routes through `shading_sample_ratio`,
  which keeps the existing linear result for exponent `1.0` and adds safe
  scalar fast paths for common type-2-function exponents `2.0` and `0.5`
  before falling back to `powf`. The change is local to shading color ratio
  calculation; color interpolation, alpha, clipping, transforms, output
  dimensions, and fallback behavior are unchanged.
- Correctness guard:
  `shading_sample_ratio_should_fast_path_common_exponents`,
  `axial_shading_sampling_should_interpolate_colors`,
  `radial_shading_sampling_should_interpolate_colors`,
  and the shading resource/display-list tests passed.
- Focused result:
  `target/benchmark-repeat-slide-shading-ratio-candidate-1000.json` improved
  repeat mean `0.247 ms` -> `0.114 ms`, p95 `0.269 ms` -> `0.138 ms`, and
  `raster_paths` `0.201 ms` -> `0.066 ms`. The repeat artifact
  `target/benchmark-repeat-slide-shading-ratio-candidate-repeat-1000.json`
  measured repeat mean `0.112 ms`, p95 `0.129 ms`, and `raster_paths`
  `0.065 ms`.
- Protection result:
  `target/benchmark-repeat-vector-stress-shading-ratio-protection-1000.json`
  stayed on the established vector track at repeat mean `0.680 ms`, p95
  `0.783 ms`, and no fallback or errors. The starter protection matrix
  `target/performance-matrix-shading-ratio-candidate-starter.json` rendered all
  `11` records with no fallback, missing tool, not-applicable, or errors.
  Focused axial/radial smoke artifacts
  `target/benchmark-repeat-axial-gradient-shading-ratio-candidate-1000.json`
  and `target/benchmark-repeat-radial-gradient-shading-ratio-candidate-1000.json`
  both rendered without fallback or errors.
- Decision:
  keep. This is a profile-backed presentation/shading win that removes a hot
  `powf` path for common PDF Type 2 function exponents without adding a
  dependency, unsafe code, cache state, or public API surface.

Rejected off-bounds axis-span skip candidate from 2026-06-30:

- Profile basis:
  after the shading ratio fast path, the current starter matrix
  `target/performance-matrix-current-after-shading-ratio-native-hot.json`
  still showed `vector-stress.pdf` as the top fixture at p95 `0.830 ms`.
  The refreshed repeat baseline
  `target/benchmark-repeat-vector-stress-current-after-shading-ratio-1000.json`
  measured repeat mean `0.705 ms`, p95 `0.801 ms`, and `raster_paths`
  `0.610 ms`. The CPU sample
  `target/sample-vector-stress-current-after-shading-ratio.txt` again pointed
  at `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `blend_pixel`, and
  `point_in_single_stroke_line`.
- Technical-family check:
  `target/performance-matrix-technical-current-after-shading-ratio.json`
  showed `engineering-floorplan-precision.pdf` p95 `0.635 ms`,
  `engineering-large-transform-detail.pdf` p95 `0.496 ms`, and
  `technical-large-coordinate-plan.pdf` p95 `0.443 ms`. Their traces
  (`target/trace-engineering-floorplan-current-after-shading-ratio.json`,
  `target/trace-engineering-large-transform-current-after-shading-ratio.json`,
  and `target/trace-technical-large-coordinate-current-after-shading-ratio.json`)
  still showed row-bucket X-miss ratios around `95%`.
- Change tested locally but not kept:
  make `axis_stroke_spans` and axis-join raster span construction skip
  off-device / off-clip lines and joins instead of rejecting the entire
  axis-span route. The intent was to keep all-axis linework fixtures on the
  span path when a few segments did not intersect the active stroke bounds.
- A/B artifacts:
  `target/performance-matrix-axis-span-skip-offbounds-candidate-technical.json`,
  `target/benchmark-repeat-engineering-floorplan-axis-span-skip-offbounds-candidate-1000.json`,
  `target/benchmark-repeat-technical-large-coordinate-axis-span-skip-offbounds-candidate-1000.json`,
  and
  `target/benchmark-repeat-vector-stress-axis-span-skip-offbounds-protection-1000.json`.
- Result:
  rejected by the technical protection matrix. `engineering-floorplan` p95
  regressed `0.635 ms` -> `0.652 ms`, `engineering-large-transform-detail`
  regressed `0.496 ms` -> `0.512 ms`, `technical-large-coordinate-plan`
  regressed `0.443 ms` -> `0.478 ms`, and `technical-linework-dimensions`
  regressed `0.257 ms` -> `0.290 ms`. `vector-stress` protection stayed close
  but did not improve, moving p95 `0.768 ms` -> `0.783 ms`.
- Decision:
  reverted. The all-or-nothing axis-span rejection looked suspicious, but the
  current large-linework cost is not solved by skipping off-bounds segments in
  this builder. Future X-miss work should reduce candidate scans directly,
  not broaden axis-span routing through this shape.

Rejected sampled-blend routing candidate from 2026-06-30:

- Profile basis:
  `target/trace-technical-repeated-symbols-current-after-axis-skip-rejection.json`
  and
  `target/benchmark-repeat-technical-repeated-symbols-current-after-axis-skip-rejection-1000.json`
  showed `technical-repeated-symbols.pdf` as a small-stroke workload: repeat
  mean `0.462 ms`, p95 `0.491 ms`, with `raster_paths` `0.232 ms`, `212`
  stroked items, `212` snapped hairline items, no row-bucket candidates, and
  no images. This suggested the generic stroke loops might benefit from the
  existing `SampledPixelBlend` direct-write shortcut for opaque normal
  full-coverage pixels.
- Change tested locally but not kept:
  route the generic stroke fallback, row-bucket stroke ranges, active
  row-bucket stroke ranges, simple-line stroke spans, and joined axis-stroke
  spans through `blend_sampled_pixel` instead of manually calling
  `blend_pixel` when `covered > 0`.
- A/B artifacts:
  `target/benchmark-repeat-technical-repeated-symbols-sampled-blend-routing-candidate-1000.json`,
  `target/benchmark-repeat-vector-stress-sampled-blend-routing-candidate-1000.json`,
  `target/benchmark-repeat-technical-hatch-sampled-blend-routing-candidate-1000.json`,
  and
  `target/performance-matrix-sampled-blend-routing-candidate-technical.json`.
- Result:
  rejected by repeat and matrix evidence. `technical-repeated-symbols` repeat
  mean regressed `0.462 ms` -> `0.469 ms` and p95 regressed `0.491 ms` ->
  `0.560 ms`. `vector-stress` repeat p95 regressed `0.801 ms` -> `0.848 ms`;
  the technical matrix showed `vector-stress` p95 `0.768 ms` -> `0.959 ms`
  and `clipped-paths` p95 `0.406 ms` -> `0.717 ms`.
- Decision:
  reverted. The direct-write branch is useful in the existing span-covered
  route, but adding it to broader sampled stroke loops worsens branch behavior
  and tail latency on important protection fixtures. Do not reintroduce this
  shape unless a future profile isolates fully covered opaque sampled strokes
  as a larger standalone cost.

Current prepress join-bounds optimization from 2026-06-30:

- Profiling basis:
  `target/benchmark-repeat-prepress-current-profile-selection.json` measured
  repeat mean `0.409 ms`, with `raster_paths` at `0.363 ms`.
  `target/trace-prepress-current-profile-selection.json` showed all 32
  flattened stroke lines were axis-aligned, with 20 stroked items and 16
  joinless axis-aligned span candidates. The long CPU sample
  `target/sample-prepress-current-profile-selection.txt` put `point_in_join`
  at the top of the stack (`4293` samples), ahead of `stroke_path` (`2554`).
- Change:
  `PreparedJoinSide` now stores bounds for bevel triangles and wraps miter
  triangles in `PreparedJoinTriangle`, so join containment rejects points
  outside the prepared triangle bounds before running `point_in_triangle`.
  Prepared bucket entries keep only an index plus pixel bounds; the heavier
  prepared join data stays in a separate vector to avoid widening the hot
  bucket enum.
- Target result:
  `target/benchmark-repeat-prepress-prepared-join-bounds-indexed-candidate.json`
  improved repeat mean `0.409 ms` -> `0.326 ms`, with `raster_paths` moving
  `0.363 ms` -> `0.279 ms`. Earlier focused runs of the same bounds check
  measured repeat means between `0.325 ms` and `0.340 ms`.
- Protection result:
  `target/benchmark-repeat-vector-stress-prepared-join-bounds-indexed-candidate.json`
  was neutral on the current slowest fixture: repeat mean `0.728 ms` ->
  `0.723 ms`. Focused protection runs for
  `office-report-header-footer-link.pdf`,
  `browser-chromium-article-print.pdf`, and `acroform-text-field.pdf` did not
  show path-raster pressure from the change (`raster_paths` at `0.020 ms`,
  `0.008 ms`, and `0.007 ms` respectively).
- Matrix caveat:
  `target/performance-matrix-prepared-join-bounds-candidate-repeat.json`
  had broad p95 and mean spikes, including fixtures unrelated to joins. Treat
  that run as noisy host evidence, not as a standalone regression signal. The
  optimization is accepted on focused before/after repeat evidence, neutral
  vector-stress protection, and unchanged fallback/error status.
- Decision:
  accepted. This is the first profile-proven renderer speedup in the current
  performance wave. It is narrow, local, and keeps the next work focused on
  path-raster predicates rather than allocation guesses.

Current vector-stress profiling pass from 2026-06-30:

- Profiling basis:
  `target/benchmark-repeat-vector-stress-profile-run-current.json` measured
  `fixtures/generated/vector-stress.pdf` with 120000 repetitions at
  `--max-edge 160`: repeat mean `0.681 ms`, repeat min `0.639 ms`, repeat max
  `6.037 ms`. Repeat phase attribution was `raster_paths` `0.592 ms`,
  `display_list_build` `0.061 ms`, and `content_tokenize` `0.023 ms`.
  `target/sample-vector-stress-profile-run-current.txt` again put the CPU
  inside path rasterization, led by `stroke_path`, `blend_pixel`,
  `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `point_in_single_stroke_line`, and
  `axis_stroke_span_for_sample_y`. The sample also showed allocator activity
  under stroke/path span construction, but not enough by itself to justify a
  dependency or arena rewrite.
- Change tested locally but not kept:
  pre-size the per-row vectors in `axis_stroke_span_rows` from the already
  known row slice length instead of creating empty vectors and extending them.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-profile-run-current.json`,
  `target/sample-vector-stress-profile-run-current.txt`, and
  `target/benchmark-repeat-vector-stress-row-capacity-candidate-120k.json`.
- Result:
  rejected as noise. Repeat mean stayed `0.681 ms` -> `0.681 ms`, and
  `raster_paths` moved only `0.592 ms` -> `0.591 ms`. This is far below the
  5% minimum signal threshold and does not justify keeping the code change.
- Decision:
  reverted. Do not retry simple per-row `Vec` capacity tuning for
  `axis_stroke_span_rows` unless a future allocation profile shows this exact
  allocation as a larger standalone cost. The next vector-stress work should
  stay with profile-backed stroke predicate reduction, row/span algorithm
  changes, or a targeted allocation counter before changing allocation
  strategy again.

Rejected span-row blend candidate from 2026-06-30:

- Profiling basis:
  `target/sample-vector-stress-profile-run-current.txt` still showed
  `blend_pixel` and `rasterize_span_covered_stroke_ranges` as prominent
  sampled stacks under `stroke_path`, while
  `target/benchmark-repeat-vector-stress-profile-run-current.json` measured
  repeat mean `0.681 ms`, computed p95 `0.774 ms`, and `raster_paths`
  `0.592 ms`.
- Change tested locally but not kept:
  route the existing `rasterize_span_covered_stroke_ranges` path through a
  row-slice blend helper instead of calling `RasterDevice::pixel` /
  `set_pixel` through `blend_sampled_pixel` for every covered pixel.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-span-row-blend-candidate-120k.json`,
  `target/performance-matrix-report-vector-span-row-blend-candidate.json`,
  `target/performance-matrix-report-vector-span-row-blend-candidate-repeat.json`,
  `target/benchmark-repeat-technical-repeated-symbols-span-row-blend-candidate-120k.json`,
  and
  `target/benchmark-repeat-vector-stress-long-span-row-blend-candidate-120k.json`.
- Result:
  rejected. The broad row-slice variant improved the target repeat run
  (`vector-stress` mean `0.681 ms` -> `0.641 ms`, computed p95 `0.774 ms` ->
  `0.728 ms`, `raster_paths` `0.592 ms` -> `0.551 ms`), but it was not
  protection-set neutral. The report/vector matrix showed unstable or worse
  p95s on `prepress-trim-bleed-marks.pdf` and
  `technical-hatch-clipping.pdf`; a second matrix repeated broad p95 spikes
  and showed `prepress` mean `0.342 ms` -> `0.363 ms`.
  `technical-repeated-symbols` repeat mean improved only `0.462 ms` ->
  `0.449 ms`, while computed p95 regressed `0.491 ms` -> `0.511 ms`.
  A narrower long-span-only variant reduced the target effect below the
  threshold (`vector-stress` mean `0.681 ms` -> `0.663 ms`, p95 `0.774 ms` ->
  `0.749 ms`, `raster_paths` `0.592 ms` -> `0.572 ms`).
- Decision:
  reverted. Do not revive row-slice stroke blending in the span-covered path
  without a shape discriminator that preserves small snapped-hairline
  workloads. The target win is real, but the current branch shape trades away
  protection-set stability and the guarded variant is below the 5% threshold.

Rejected single-sample axis-span range candidate from 2026-06-30:

- Profiling basis:
  after the row-slice text rectangle win, the next Prepress refresh targeted
  `fixtures/generated/prepress-trim-bleed-marks.pdf` at `--max-edge 160`.
  `target/trace-prepress-head-after-row-rect.json` reported `0.474 ms` total
  with `0.282 ms` in `raster_paths`; all 32 flattened stroke lines were
  axis-aligned, 20 stroke items were snapped hairlines, and 16 items were
  joinless axis-span candidates. The focused repeat artifact
  `target/benchmark-repeat-prepress-head-sample-after-row-rect.json` measured
  repeat mean `0.312 ms`, with `raster_paths` at `0.269 ms`. The CPU sample
  `target/sample-prepress-head-after-row-rect.txt` still put almost all time
  under `stroke_path`, with visible allocator samples but only small direct
  `axis_stroke_span_for_sample_y` and `blend_pixel` samples.
- Change tested locally but not kept:
  add a `samples == 1` axis-span raster fast path that derived center-sampled
  pixel ranges from each `AxisStrokeSpan` and skipped the existing per-pixel
  single-sample `x_in_axis_stroke_span_row` test. The helper used inclusive
  stroke-span max-edge semantics rather than the half-open rectangle-fill edge
  rule.
- A/B artifacts:
  `target/benchmark-repeat-prepress-head-sample-after-row-rect.json` and
  `target/benchmark-repeat-prepress-single-sample-axis-span-candidate.json`.
- Result:
  rejected. The candidate moved repeat mean `0.312 ms` -> `0.316 ms`; output
  dimensions and bytes stayed unchanged, but the target did not improve. This
  confirms that the remaining Prepress cost is not the one-sample span
  membership check alone.
- Decision:
  reverted. Do not retry a one-sample axis-span range shortcut unless a deeper
  profile or counter shows per-pixel span membership as a standalone cost. The
  next Prepress pass should use a debug-symbol profiling build, Instruments, or
  focused counters inside `stroke_path` before another micro-optimization.

Profiling build profile from 2026-06-30:

- Trigger:
  the normal release-size-oriented profiling loop still left the hottest
  Prepress stacks as large `stroke_path + offset` blocks. A local
  `CARGO_PROFILE_RELEASE_STRIP=none` / `DEBUG=line-tables-only` build improved
  top-level symbols but did not provide enough line-level detail for inlined
  stroke raster work.
- Change:
  add a dedicated Cargo `profiling` profile that inherits release
  optimizations, keeps symbols, emits full debug info, disables LTO, and uses
  one codegen unit. It is intended for `sample`, Instruments, Samply, and
  `atos` runs, not for shipped binaries or public speed claims.
- Usage:
  `cargo build --profile profiling -p ferrugo-cli --no-default-features`, then
  run `target/profiling/ferrugo-cli benchmark-repeat-native ...` and attach
  `sample` or Instruments to that process. If the local Rust toolchain supports
  it, add `RUSTFLAGS="-C force-frame-pointers=yes"` for more stable native
  call stacks.
- Acceptance impact:
  this is profiling infrastructure, not a renderer speed claim. Optimization
  commits still need release-mode before/after benchmarks and protection-set
  checks.

Stroke routing counters from 2026-06-30:

- Trigger:
  the `target/sample-prepress-cargo-profiling.txt` run from the new profiling
  profile mapped the hottest Prepress stacks to `stroke_path` lines around
  axis-span setup/raster and the generic stroke loop. The previous
  one-sample span-membership candidate did not help, so the trace needed to
  show which runtime route each stroked item actually takes.
- Change:
  extend `StrokeShapeSummary` and `trace-native` JSON with
  `axis_span_routed_items`, `simple_line_span_routed_items`,
  `simple_line_span_below_threshold_items`,
  `simple_line_span_below_threshold_pixels`, and
  `generic_stroke_fallback_items`. These are route-shape counters only; they do
  not inspect rendered pixels or document contents.
- Fresh Prepress result:
  `target/trace-prepress-routing-counters.json` reports `20` stroked items:
  `4` route through axis spans, `0` route through simple-line spans, and `16`
  short single-line strokes are below the simple-line span threshold and fall
  through to the generic loop. Their combined conservative pixel area is `912`.
- Optimization impact:
  do not lower `STROKE_AXIS_SIMPLE_LINE_SPAN_MIN_PIXELS` broadly; threshold `16`
  was already rejected on technical protection fixtures. The next candidate
  should either target the generic small-hairline loop directly or add a
  discriminator that protects the technical fixtures that regressed in the
  earlier threshold test.

Rejected single-line hairline shortcut from 2026-06-30:

- Profiling basis:
  `target/trace-prepress-routing-counters.json` showed `16` short single-line
  snapped-hairline strokes below the simple-line span threshold and falling
  through to the generic stroke loop. This made a narrower route worth testing,
  without repeating the previously rejected broad threshold-16 change.
- Change tested locally but not kept:
  before axis-span setup, route only `snap_hairline && samples == 1`
  single-line, axis-aligned, butt-cap, no-join strokes with skippable clips
  through a direct center-sampled axis range raster path. The helper used the
  existing sampled pixel blend path and had a byte-for-byte unit test against
  the generic butt-stroke predicate for horizontal and vertical lines.
- A/B artifacts:
  `target/benchmark-repeat-prepress-head-sample-after-row-rect.json` and
  `target/benchmark-repeat-prepress-single-line-hairline-candidate.json`.
- Result:
  rejected. The Prepress repeat mean moved `0.312 ms` -> `0.319 ms`, with
  identical output dimensions and bytes. Even the narrow direct route did not
  beat the current generic path on this fixture.
- Decision:
  reverted. Do not retry direct center-sampled single-line hairline drawing
  unless a future profile shows the generic line-distance predicate itself as
  the dominant standalone cost. The next attempt should inspect allocation/drop
  or display-list lifetime costs visible around the remaining `stroke_path`
  sample, not another small line raster shortcut.

Row-bucket merged sample-point counters from 2026-06-30:

- Profiling basis:
  the current `vector-stress` refresh still shows `raster_paths` as the
  dominant phase. `target/benchmark-repeat-vector-stress-current-next-20k.json`
  measured repeat mean `0.835 ms`, p95 `0.915 ms`, and repeat mean
  `raster_paths` `0.744 ms`. The matching trace
  `target/trace-vector-stress-current-next.json` still reported `485376`
  conservative row-bucket sample refs, `25672` X hits, and `459704` X misses.
- Change:
  `StrokeShapeSummary` now also reports
  `row_bucket_merged_sample_points` and
  `max_row_bucket_merged_sample_points_per_item`. These estimate the sample
  points actually visited after row-bucket X ranges are merged, separate from
  the line-check count that passes X-bounds filtering.
- Fresh trace:
  `target/trace-vector-stress-row-bucket-merged-points.json` reports
  `row_bucket_merged_sample_points` `13488` and
  `max_row_bucket_merged_sample_points_per_item` `6744`, while
  `row_bucket_sample_x_hits` remains `25672`.
- Optimization impact:
  the current `vector-stress` shape is not primarily wasting time by visiting
  huge merged pixel ranges after the accepted range-culling work. It still does
  about `1.9` line-hit checks per visited sample point, so the next vector
  attempt should target candidate grouping/predicate reduction inside visited
  ranges rather than another broad X-range merge or blend-only variant.

Accepted span-row cursor result from 2026-06-30:

- Profile basis:
  the same current `vector-stress` repeat and profile still had visible time in
  `rasterize_span_covered_stroke_ranges`. The existing span membership helper
  scanned each sorted coverage row from the beginning for every pixel/sample,
  even though raster X positions advance monotonically across merged ranges.
- Change:
  large span-covered stroke rows now keep one cursor per supersample Y row and
  advance it through the sorted `AxisStrokeSpan` row as X increases. Smaller
  span sets stay on the original from-start membership helper; the cursor route
  is gated by `STROKE_SPAN_CURSOR_MIN_SPANS = 512` to avoid adding branch and
  cursor overhead to small snapped/prepress strokes.
- Correctness guard:
  `exact_axis_line_span_raster_should_match_sampled_stroke_raster` still
  compares the exact span route against the sampled fallback byte-for-byte.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-current-next-20k.json`,
  `target/benchmark-repeat-vector-stress-span-cursor-gate512-candidate-20k.json`,
  `target/benchmark-repeat-prepress-current-next-20k.json`,
  `target/benchmark-repeat-prepress-span-cursor-gate512-candidate-20k.json`,
  `target/benchmark-repeat-technical-hatch-span-cursor-gate512-candidate-20k.json`,
  and `target/performance-matrix-span-cursor-gate512-report-vector.json`.
- Focused result:
  `vector-stress.pdf` repeat mean improved `0.835 ms` -> `0.685 ms`, p95
  improved `0.915 ms` -> `0.782 ms`, and repeat mean `raster_paths` moved
  `0.744 ms` -> `0.594 ms`.
- Protection result:
  Prepress repeat mean moved only `0.311 ms` -> `0.314 ms`; p95 was noisy
  (`0.338 ms` -> `0.355 ms`) and should be watched in the next matrix run.
  `technical-hatch-clipping.pdf` measured `0.264 ms` mean and `0.299 ms` p95
  in the candidate repeat. The report/vector matrix rendered all four records
  with no fallback, missing-tool, not-applicable, or error records.
- Decision:
  keep. This is a profile-backed algorithmic reduction in the remaining
  span-covered raster loop. It adds no dependency, unsafe code, cache, or
  alternate stroke geometry, and the gate keeps the changed loop on large span
  workloads where the cursor can amortize its setup cost.

Post-span-cursor profile and span-route diagnostics from 2026-06-30:

- Current matrix artifact:
  `target/performance-matrix-current-after-span-cursor-native-hot.json`, native
  hot-render, `--max-edge 160`, 5 measured iterations after one warmup.
  `vector-stress.pdf` is still the slowest native hot fixture at mean
  `0.726 ms`, p95 `0.800 ms`; `technical-hatch-clipping.pdf` follows at mean
  `0.315 ms`, p95 `0.400 ms`.
- CPU sample:
  `target/sample-vector-stress-post-span-cursor.txt`, captured from a profiling
  build repeat run, still puts the work under `stroke_path`. The visible
  buckets are `blend_pixel`, `rasterize_row_bucketed_stroke_ranges`,
  `rasterize_span_covered_stroke_ranges`, `point_in_single_stroke_line`,
  `RasterDevice::pixel`, `axis_stroke_span_for_sample_y`, `merge_pixel_ranges`,
  and allocator/free frames. Parser/tokenizer stacks are present but smaller
  than raster work.
- Change:
  `trace-native` now exposes item-level span-route counters in
  `stroke_shape_summary`: axis-span cursor candidates, axis-span coverage/raster
  span counts, simple-line span cursor candidates, and simple-line coverage
  span counts. The counters are collected only in the explicit trace path and
  do not change normal render or benchmark output.
- Smoke artifact:
  `target/trace-vector-stress-span-route-counters.json` reports `20`
  axis-span routed items, `4016` total axis coverage spans, max `312` coverage
  spans per axis item, `44` simple-line span routed items, `4508` total
  simple-line coverage spans, and max `172` coverage spans per simple-line
  item. Both cursor-candidate counters are `0` at this item-summary level.
- Interpretation:
  the post-cursor profile still justifies stroke raster work, but the new
  counters show that a simple item-level span count is not enough to explain
  the accepted cursor win or pick the next threshold. Do not tune
  `STROKE_SPAN_CURSOR_MIN_SPANS` again from these counters alone. The next
  implementation candidate should either instrument actual
  `rasterize_span_covered_stroke_ranges` call counts/cursor-route decisions or
  target the still-visible row-bucket predicate/blend work with a fresh
  protection run.

Runtime span-route counters from 2026-06-30:

- Change:
  `trace-native` now also includes `stroke_raster_route_summary`, collected
  from the actual path-raster callsites. Unlike `stroke_shape_summary`, these
  counters record the runtime `rasterize_span_covered_stroke_ranges` decisions:
  total span-covered calls, cursor-route calls, from-start calls, coverage span
  totals, max coverage spans per call, raster span totals, and max raster spans
  per call.
- Scope:
  the counters are request-local and only active in explicit native trace
  renders. Normal renders, hot benchmarks, and timing-only repeat benchmarks
  stay on the existing raster path and do not pay the counter overhead.
- Smoke artifact:
  `target/trace-vector-stress-runtime-span-routes.json` reports `44`
  span-covered calls, `0` cursor calls, `44` from-start calls, `4508` total
  coverage spans, max `172` coverage spans per call, `4508` raster spans, and
  max `172` raster spans per call.
- Interpretation:
  current `vector-stress` at `--max-edge 160` is not using the accepted
  `STROKE_SPAN_CURSOR_MIN_SPANS = 512` route. The next code candidate should
  not lower that threshold blindly; it needs a focused A/B on the from-start
  path or a row-bucket/blend target with `prepress` and
  `technical-hatch-clipping` in the protection set from the first run.

Rejected span-cursor threshold 128 candidate from 2026-06-30:

- Profiling basis:
  runtime route counters showed `44` span-covered calls on `vector-stress`, all
  using the from-start route, with max `172` coverage spans per call. This made
  a threshold-lowering A/B worth testing, but only with protection fixtures.
- Change tested locally but not kept:
  lower `STROKE_SPAN_CURSOR_MIN_SPANS` from `512` to `128`.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-current-span-gate-baseline-20k.json`,
  `target/benchmark-repeat-vector-stress-span-cursor-gate128-candidate-20k.json`,
  `target/benchmark-repeat-prepress-current-span-gate-baseline-20k.json`,
  `target/benchmark-repeat-prepress-span-cursor-gate128-candidate-20k.json`,
  `target/benchmark-repeat-technical-hatch-current-span-gate-baseline-20k.json`,
  and
  `target/benchmark-repeat-technical-hatch-span-cursor-gate128-candidate-20k.json`.
- Result:
  rejected. `vector-stress.pdf` regressed mean `0.666 ms` -> `0.675 ms`, p95
  `0.772 ms` -> `0.786 ms`, and `raster_paths` `0.578 ms` -> `0.586 ms`.
  `prepress-trim-bleed-marks.pdf` regressed mean `0.307 ms` -> `0.310 ms` and
  p95 `0.335 ms` -> `0.344 ms`. `technical-hatch-clipping.pdf` was neutral
  on mean and raster paths, with p95 `0.280 ms` -> `0.279 ms`.
- Decision:
  reverted. The from-start span route is visible, but lowering the cursor
  threshold adds overhead before it removes enough scan work. Do not retest
  simple threshold lowering for this shape; the next candidate should change
  the from-start algorithm itself or move to row-bucket/blend work.

Rejected joinless raster-span reuse and from-start row precompute candidates
from 2026-06-30:

- Profiling basis:
  `target/sample-vector-stress-current-continuation.txt`, captured from a
  10-second macOS `sample` run against a profiling build repeat process for
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`, still showed
  `stroke_path` as the dominant path. The visible sampled buckets included
  `blend_pixel` (~1020 symbol samples),
  `rasterize_span_covered_stroke_ranges` (~948),
  `rasterize_row_bucketed_stroke_ranges` (~942), `stroke_path` (~889),
  `point_in_single_stroke_line` (~451), allocator/free frames,
  `axis_stroke_span_for_sample_y`, and `merge_pixel_ranges`.
- Change tested locally but not kept:
  represent joinless `AxisStrokeRasterSpans` with only the coverage span set,
  using that same span set as the raster span source when `joins.is_empty()`.
  This targeted the sampled `axis_stroke_raster_spans` allocation/free work.
- A/B artifacts for the joinless reuse candidate:
  `target/benchmark-repeat-vector-stress-current-span-gate-baseline-20k.json`,
  `target/benchmark-repeat-vector-stress-reuse-coverage-candidate-20k.json`,
  `target/benchmark-repeat-prepress-current-span-gate-baseline-20k.json`,
  `target/benchmark-repeat-prepress-reuse-coverage-candidate-20k.json`,
  `target/benchmark-repeat-technical-hatch-current-span-gate-baseline-20k.json`,
  and
  `target/benchmark-repeat-technical-hatch-reuse-coverage-candidate-20k.json`.
- Result:
  rejected. `vector-stress.pdf` improved p95 only `0.772 ms` -> `0.751 ms`
  (~2.7%) and repeat mean only `0.666 ms` -> `0.663 ms` (~0.5%), while
  `technical-hatch-clipping.pdf` regressed mean `0.255 ms` -> `0.260 ms`
  and raster paths `0.159 ms` -> `0.162 ms`. `prepress-trim-bleed-marks.pdf`
  stayed effectively neutral on p95 and raster paths.
- Follow-up change tested locally but not kept:
  precompute the `coverage_spans.rows` lookups once per output row in
  `rasterize_span_covered_stroke_ranges_from_start`, instead of recomputing
  the sample-row index for every pixel. This was tested together with the
  joinless reuse candidate.
- A/B artifacts for the row-precompute candidate:
  `target/benchmark-repeat-vector-stress-row-precompute-candidate-20k.json`,
  `target/benchmark-repeat-prepress-row-precompute-candidate-20k.json`, and
  `target/benchmark-repeat-technical-hatch-row-precompute-candidate-20k.json`,
  compared against the same current span-gate baseline artifacts above.
- Result:
  rejected. The combined candidate regressed `vector-stress.pdf` mean
  `0.666 ms` -> `0.672 ms`, `prepress-trim-bleed-marks.pdf` mean
  `0.307 ms` -> `0.313 ms`, and `technical-hatch-clipping.pdf` mean
  `0.255 ms` -> `0.264 ms`. Output dimensions, output bytes, native-render
  status, and error count stayed unchanged in all runs.
- Decision:
  reverted. The sample exposed real allocation and row-lookup work, but these
  local restructurings do not move enough wall/raster time and hurt at least
  one protection fixture. Do not retry joinless raster-span ownership changes
  or per-row from-start lookup precomputation unless a lower-level allocation
  profile shows the specific allocation as a larger standalone cost. The next
  vector pass should target the remaining `blend_pixel`, row-bucket predicate,
  or `point_in_single_stroke_line` samples with a protection run from the first
  A/B.

Rejected sampled opaque integer blend candidate from 2026-06-30:

- Profiling basis:
  the same `target/sample-vector-stress-current-continuation.txt` run showed
  `blend_pixel` as the largest visible symbol bucket after the latest accepted
  stroke optimizations. The candidate targeted only sampled stroke pixels that
  were not already full-coverage direct writes.
- Change tested locally but not kept:
  add a `blend_sampled_pixel` fast path for `Normal` blend mode, opaque source,
  `alpha >= 1.0`, and opaque destination pixels, computing partial sample
  coverage with integer channel math instead of calling the generic
  `blend_pixel` path.
- Correctness smoke:
  a focused unit test compared the integer sampled blend against the existing
  `source_over` result for quarter-sample coverages.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-sampled-opaque-blend-candidate-20k.json`,
  `target/benchmark-repeat-prepress-sampled-opaque-blend-candidate-20k.json`,
  and
  `target/benchmark-repeat-technical-hatch-sampled-opaque-blend-candidate-20k.json`,
  compared against the same current span-gate baseline artifacts.
- Result:
  rejected. `vector-stress.pdf` improved p95 only `0.772 ms` -> `0.752 ms`
  (~2.6%) and raster paths `0.578 ms` -> `0.569 ms` (~1.6%), but
  `prepress-trim-bleed-marks.pdf` regressed p95 `0.335 ms` -> `0.346 ms` and
  `technical-hatch-clipping.pdf` regressed p95 `0.280 ms` -> `0.292 ms`.
  Output bytes, native-render status, and error count stayed unchanged.
- Decision:
  reverted. The extra branch/destination-read path is not protection-set
  neutral, which matches the earlier sampled-blend routing rejection. Do not
  retry broad sampled opaque blend routing unless a profile isolates a larger
  fully covered or partial opaque stroke subset and the protection run starts
  with Prepress and Technical Hatch.

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

1. more join/stroke predicate bounding and early rejection, guided by `sample`;
2. device-bounds culling before raster work;
3. broader stroke raster candidate reduction for dense linework;
4. fixture-level stroke-shape histograms before another spatial-index variant;
5. clip-before-loop checks.

If deeper profiler samples point elsewhere inside path rasterization, this
section should be edited before code changes start.

Row-bucket predicate runtime counters from 2026-06-30:

- Change: `trace-native` stroke raster route summaries now include runtime
  counters for row-bucket range calls, active-row range calls, rows, merged
  X-ranges, visited pixels, visited sample points, line candidate checks,
  line X hits, line geometry hits, join candidate checks, join X hits, join
  geometry hits, and covered pixels. The counters are only collected through
  the trace route; normal render and benchmark paths keep the existing
  non-counting hot loops.
- Smoke artifact:
  `target/trace-vector-stress-row-predicate-counters.json`, generated with
  native `trace-native`, `fixtures/generated/vector-stress.pdf`, `--max-edge
  160`.
- Current `vector-stress.pdf` trace signal: `2` row-bucket range calls, both
  active-row calls, `56` visited rows, `296` merged X-ranges, `3324` visited
  pixels, `13296` visited sample points, `21822` line candidate checks,
  `21822` line X hits, `3424` line geometry hits, `1096` join candidate checks,
  `1096` join X hits, `0` join hits, and `1126` covered pixels.
- Interpretation: after the accepted row-bucket X-range and active-candidate
  work, this fixture is no longer dominated by broad X misses in the traced
  active path. The next vector attempt should focus on reducing line predicate
  work inside visited ranges, span/coverage loop cost, or blend/write cost.
  Join-specific optimization is not supported by this trace because the current
  target pays join checks but records no join coverage hits.

Accepted single-offset blend write from 2026-06-30:

- Profiling basis:
  `target/sample-vector-stress-row-predicate-counters-base.txt`, captured from
  a long profiling-build repeat run for `fixtures/generated/vector-stress.pdf`,
  still showed `rasterize_span_covered_stroke_ranges`,
  `rasterize_row_bucketed_stroke_ranges`, `point_in_single_stroke_line`,
  `merge_pixel_ranges`, and `blend_pixel` as visible path-raster costs.
- Rejected local candidates:
  replacing active candidate `Vec::retain` with manual write-index compaction
  improved `vector-stress.pdf` only from repeat mean `0.703 ms` to `0.694 ms`
  and was not kept. Moving `LineCap` dispatch out of the row-bucket candidate
  line loop regressed the focused target (`0.703 ms` -> `0.705 ms`). A small
  `merge_pixel_ranges` sorted/insertion-sort fast path also regressed the
  target (`0.703 ms` -> `0.704 ms`). All three were reverted.
- Change:
  `blend_pixel` now computes the raster pixel offset once for the partial and
  non-normal blend paths, reads the destination pixel from that offset, and
  writes the result back through the same offset. The full-coverage opaque
  normal direct-write path remains unchanged.
- Focused result:
  `target/benchmark-repeat-vector-stress-active-retain-baseline-20k.json`
  versus
  `target/benchmark-repeat-vector-stress-single-offset-blend-candidate-20k.json`.
  `vector-stress.pdf` repeat mean improved `0.703 ms` -> `0.667 ms`, and
  repeat mean `raster_paths` improved `0.611 ms` -> `0.576 ms`.
- Protection result:
  Fresh Prepress baseline/candidate artifacts
  `target/benchmark-repeat-prepress-single-offset-blend-baseline-20k.json` and
  `target/benchmark-repeat-prepress-single-offset-blend-candidate-20k.json`
  moved repeat mean `0.327 ms` -> `0.328 ms`. Technical Hatch artifacts
  `target/benchmark-repeat-technical-hatch-single-offset-blend-baseline-20k.json`
  and
  `target/benchmark-repeat-technical-hatch-single-offset-blend-candidate-20k.json`
  moved repeat mean `0.264 ms` -> `0.263 ms`.
- Decision:
  keep. This is a profile-backed reduction in a shared blend hot path. It
  avoids new dependencies and unsafe code, preserves existing blend math, and
  keeps the protection set neutral within small absolute timing noise.

Post-offset-blend profile and rejected axis row-copy candidate from 2026-06-30:

- Fresh profile:
  `target/sample-vector-stress-post-offset-blend.txt`, captured after the
  single-offset blend write, still showed path rasterization as the hot phase.
  Visible buckets included `rasterize_span_covered_stroke_ranges`,
  `blend_pixel`, `rasterize_row_bucketed_stroke_ranges`,
  `point_in_single_stroke_line`, `advance_active_line_indices`, and allocator
  frames under `axis_stroke_raster_spans`.
- Change tested locally but not kept:
  replace `axis_stroke_span_rows`'s `vec![Vec::new(); rows]` plus
  `extend_from_slice` loop with a direct iterator using
  `spans.spans[row.clone()].to_vec()` per row. This targeted row-vector growth
  in the existing raster-span construction path without reopening the already
  rejected `coverage.clone()` or `Option<AxisStrokeSpans>` variants.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-single-offset-blend-candidate-20k.json`
  versus
  `target/benchmark-repeat-vector-stress-axis-row-tovec-candidate-20k.json`.
- Result:
  rejected. `vector-stress.pdf` repeat mean regressed `0.667 ms` -> `0.675 ms`
  and repeat mean `raster_paths` regressed `0.576 ms` -> `0.582 ms`.
- Decision:
  reverted. The allocation frame is real, but this local row-copy spelling does
  not improve the current target. Keep future axis-span allocation work tied to
  a more structural reduction, not another local copy-shape rewrite.

Rejected sampled float blend dispatch candidate from 2026-06-30:

- Rationale:
  `target/sample-vector-stress-post-offset-blend.txt` still showed
  `blend_pixel` under both span-covered and row-bucket stroke rasterization.
  The previously rejected sampled opaque integer blend changed arithmetic; this
  follow-up kept the existing floating-point `source_over_opaque` math but
  tried to route sampled opaque normal pixels around generic `blend_pixel`
  dispatch.
- Change tested locally but not kept:
  add an `opaque_normal` flag to `SampledPixelBlend` and route non-full sampled
  opaque-normal pixels through a sampled-specific helper that computed coverage,
  read the destination pixel once, and wrote the existing `source_over_opaque`
  or `source_over` result directly.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-single-offset-blend-candidate-20k.json`
  versus
  `target/benchmark-repeat-vector-stress-sampled-float-blend-candidate-20k.json`.
- Result:
  rejected. `vector-stress.pdf` repeat mean regressed `0.667 ms` -> `0.701 ms`
  and repeat mean `raster_paths` regressed `0.576 ms` -> `0.604 ms`.
- Decision:
  reverted. Even without changing blend arithmetic, the extra sampled-specific
  branch and helper shape lose on the current hot fixture. Do not retry sampled
  opaque-normal blend routing unless a future profile isolates a narrower
  subcase than the whole sampled stroke path.

### Accepted span-raster work counters from 2026-06-30

- Change:
  `trace-native` now reports span-covered raster work in addition to route
  counts: raster rows, merged X ranges, visited pixels, sample points, covered
  pixels, full-coverage pixels, and partial-coverage pixels.
- Normal-render overhead:
  the detailed counters run only when stroke-route tracing is requested. The
  non-trace render path keeps the previous loops.
- Focus fixture:
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`.
- Trace artifact:
  `target/trace-vector-stress-span-work.json`.
- Result:
  `span_covered_calls=44`, `span_from_start_calls=44`, `span_pixels=9008`,
  `span_sample_points=36032`, `span_covered_pixels=9008`,
  `span_full_coverage_pixels=0`, and `span_partial_coverage_pixels=9008`.
  Row-bucket raster work on the same trace reported
  `row_bucket_pixels=3324` and `row_bucket_sample_points=13296`.
- Neutrality check:
  a normal hot-render canary with 20k repetitions reported repeat mean
  `0.676 ms` and repeat mean `raster_paths=0.582 ms`, close to the prior
  accepted single-offset blend result (`0.667 ms` / `0.576 ms`) and not treated
  as a speed claim.
- Decision:
  accepted as profiling infrastructure. The current `vector-stress` bottleneck
  does not expose a full-coverage fast path in span rastering; every span pixel
  is partially covered. The next optimization candidate should target
  coverage/sample work or route selection before trying another sampled
  blend/write micro-optimization.

### Current span profile and rejected unstable sort candidate from 2026-06-30

- Profile artifact:
  `target/sample-vector-stress-span-work-current.txt`, a 10-second macOS
  `sample` run against the profiling build repeat process for
  `fixtures/generated/vector-stress.pdf`, `--max-edge 160`.
- CPU sample:
  the largest flat symbols were `rasterize_span_covered_stroke_ranges` (`888`
  samples), `blend_pixel` (`884`), `stroke_path` (`819`),
  `point_in_single_stroke_line` (`483`),
  `axis_stroke_span_for_sample_y` (`219`), and `merge_pixel_ranges` (`147`).
- Change tested locally but not kept:
  switch `merge_pixel_ranges` from `sort_by_key` to `sort_unstable_by_key`.
  Equal-start ranges do not need stable ordering for the current merge
  semantics, so this was a valid isolated candidate.
- Candidate artifact:
  `target/benchmark-repeat-vector-stress-sort-unstable-candidate-20k.json`.
- Result:
  slightly positive but below the acceptance threshold. The focused 20k run
  moved repeat mean `0.676 ms` -> `0.668 ms` and repeat mean `raster_paths`
  `0.582 ms` -> `0.576 ms`.
- Decision:
  reverted. This is not enough signal for a renderer-wide sort behavior change.
  Keep the evidence, but spend the next pass on reducing actual span/sample
  work or row-bucket predicate work rather than changing sort flavor.

### Rejected small span-cursor candidate from 2026-06-30

- Profile basis:
  the span-work counters showed that `vector-stress.pdf` still spends visible
  time in span-covered rastering, while all `44` span-covered calls stay on the
  from-start route. The current CPU sample also kept
  `rasterize_span_covered_stroke_ranges`, `point_in_single_stroke_line`,
  `axis_stroke_span_for_sample_y`, and `merge_pixel_ranges` visible.
- Change tested locally but not kept:
  add a small stack cursor route for `samples <= 4`, using fixed
  `[Range<usize>; 4]` and cursor arrays instead of the existing from-start
  membership helper for below-threshold span sets. This was a narrower variant
  than the previously rejected simple cursor-threshold reduction.
- Correctness check while the candidate was present:
  `small_cursor_span_raster_should_match_from_start_raster` compared the new
  route against the existing from-start route byte-for-byte, and the focused
  `ferrugo-render` tests passed.
- Candidate artifact:
  `target/benchmark-repeat-vector-stress-small-cursor-candidate-20k.json`.
- Result:
  rejected. The focused run regressed the current canary from repeat mean
  `0.676 ms` to `0.681 ms`, and repeat mean `raster_paths` from `0.582 ms` to
  `0.587 ms`.
- Decision:
  reverted. The from-start route remains better for these small span-covered
  calls; the stack cursor adds branch/cursor bookkeeping without reducing enough
  membership work. Do not retry small-cursor routing unless a lower-level
  profile isolates from-start membership scanning as a larger standalone cost
  than the cursor setup overhead.

### Accepted flat joined-axis raster span builder from 2026-06-30

- Profile basis:
  `target/sample-vector-stress-profile-refresh.txt`, captured from a long
  profiling-build `benchmark-repeat-native` run for
  `fixtures/generated/vector-stress.pdf`, still showed `stroke_path` as the
  dominant render path. The largest flat symbols were
  `rasterize_span_covered_stroke_ranges` (`947` samples), `blend_pixel`
  (`929`), `stroke_path` (`906`), `point_in_single_stroke_line` (`487`),
  allocator/free frames, `axis_stroke_span_for_sample_y` (`238`), and
  `merge_pixel_ranges` (`156`). Inside `stroke_path`, the sample showed a
  visible allocation/free block under `axis_stroke_raster_spans` for joined
  axis-aligned strokes.
- Change:
  `axis_stroke_raster_spans` now keeps the old row-Vec builder only for
  joinless raster spans and uses a two-pass flat builder when axis-aligned
  joins are present. The new path counts join spans per sample row, allocates
  one flat span buffer, copies existing coverage spans by row, appends join
  spans into the same flat storage, then sorts and merges each row slice into
  the existing `AxisStrokeSpans` representation. It does not change coverage
  semantics, clipping, blend math, public APIs, dependencies, or unsafe usage.
- Correctness guard:
  `axis_stroke_raster_spans_with_axis_joins_should_match_row_builder` compares
  the new flat joined-axis builder against the previous row-Vec builder, and
  `axis_stroke_raster_spans_should_cover_joined_axis_strokes` still verifies
  conservative raster coverage for joined axis strokes.
- Focused artifacts:
  `target/benchmark-repeat-vector-stress-profile-refresh-200k.json`,
  `target/benchmark-repeat-vector-stress-flat-join-raster-candidate-200k.json`,
  and `target/trace-vector-stress-flat-join-raster-candidate.json`.
- Focused result:
  accepted. `vector-stress.pdf` repeat mean improved `0.689 ms` -> `0.631 ms`
  (~8.4%), p95 improved `0.828 ms` -> `0.743 ms` (~10.3%), and repeat mean
  `raster_paths` improved `0.593 ms` -> `0.538 ms` (~9.3%). Output status,
  dimensions, and bytes stayed unchanged.
- Protection result:
  `target/benchmark-repeat-prepress-single-offset-blend-candidate-20k.json`
  versus
  `target/benchmark-repeat-prepress-flat-join-raster-candidate-20k.json`
  improved mean `0.328 ms` -> `0.322 ms`, p95 `0.379 ms` -> `0.367 ms`, and
  raster paths `0.283 ms` -> `0.278 ms`.
  `target/benchmark-repeat-technical-hatch-single-offset-blend-candidate-20k.json`
  versus
  `target/benchmark-repeat-technical-hatch-flat-join-raster-candidate-20k.json`
  stayed neutral to slightly positive: mean `0.263 ms` -> `0.262 ms`, p95
  `0.316 ms` -> `0.308 ms`, and raster paths `0.164 ms` -> `0.162 ms`.
  Both protection fixtures kept native-rendered status, output dimensions, and
  output bytes unchanged.
- Decision:
  keep. This is a profile-backed structural allocation reduction on the same
  vector/report stroke path. It deliberately avoids reopening the previously
  rejected joinless coverage-reuse and row-copy variants, because the accepted
  change is scoped to joined axis spans where the row-Vec rebuild was visible
  in the fresh profile.

### Post flat joined-axis profile from 2026-06-30

- Current post-commit artifacts:
  `target/sample-vector-stress-after-flat-join-raster.txt` and
  `target/benchmark-repeat-vector-stress-after-flat-join-raster-200k.json`.
- Current focused baseline:
  `vector-stress.pdf` repeat mean `0.642 ms`, p95 `0.779 ms`, p99 `0.868 ms`,
  and repeat mean `raster_paths=0.548 ms`. The run had one large max outlier
  (`133.433 ms`), so use p95/p99 and repeat mean rather than max for the next
  local decision.
- CPU sample:
  the largest flat symbols are now `blend_pixel` (`1015` samples),
  `stroke_path` (`967`), `rasterize_span_covered_stroke_ranges` (`918`),
  `rasterize_row_bucketed_stroke_ranges` (`795`),
  `axis_stroke_raster_spans` (`559`), `point_in_single_stroke_line` (`499`),
  and `axis_stroke_span_for_sample_y` (`227`). Allocator/free top-level samples
  are lower than before the flat joined-axis builder, but `axis_stroke_raster_spans`
  still has visible copy/sort/build work.
- Decision:
  use this as the next vector/report baseline. The next candidate should be
  based on one of three still-visible classes: row-bucket predicate work,
  remaining span-covered sample/blend work, or a narrower axis-span build
  reduction. Do not repeat broad row-slice blend, sampled blend dispatch,
  simple cursor threshold, joinless raster-span reuse, or row-copy spelling
  variants already rejected above.

### Accepted row-bucket line metrics from 2026-06-30

- Profile basis:
  `target/sample-vector-stress-live-status.txt`,
  `target/benchmark-repeat-vector-stress-live-status-200k.json`, and
  `target/trace-vector-stress-live-status.json` showed the current hot section
  still inside path rasterization. The largest sampled buckets included
  `rasterize_span_covered_stroke_ranges`, `blend_pixel`,
  `rasterize_row_bucketed_stroke_ranges`, `axis_stroke_raster_spans`, and
  `point_in_single_stroke_line`. Runtime counters reported `2` active
  row-bucket range calls, `13296` row-bucket sample points, `21822` line
  candidate checks, and `3424` line hits.
- Change:
  `BoundedStrokeLine` now stores `StrokeLineMetrics` with precomputed `dx`,
  `dy`, `len_squared`, and reciprocal length squared. Row-bucket stroke
  predicates use those metrics for Butt and Round caps instead of recomputing
  line deltas and dividing by length squared for every candidate sample. The
  generic stroke predicate and Square-cap fallback semantics stay unchanged.
- Correctness guard:
  `bounded_stroke_line_predicate_should_match_single_line_predicate` compares
  the bounded-metric predicate against the existing single-line predicate for
  Butt, Round, and Square caps over a small point grid.
- Focused result:
  `target/benchmark-repeat-vector-stress-live-status-200k.json` versus
  `target/benchmark-repeat-vector-stress-bounded-line-reciprocal-candidate-200k.json`.
  `vector-stress.pdf` repeat mean improved `0.604 ms` -> `0.572 ms` (~5.3%),
  p95 improved `0.637 ms` -> `0.612 ms` (~3.9%), and repeat mean
  `raster_paths` improved `0.518 ms` -> `0.486 ms` (~6.2%). Output status,
  dimensions, and bytes stayed unchanged.
- Protection result:
  `target/benchmark-repeat-prepress-flat-join-raster-candidate-20k.json`
  versus
  `target/benchmark-repeat-prepress-bounded-line-reciprocal-candidate-20k.json`
  improved mean `0.322 ms` -> `0.301 ms`, p95 `0.367 ms` -> `0.316 ms`, and
  raster paths `0.278 ms` -> `0.259 ms`.
  `target/benchmark-repeat-technical-hatch-flat-join-raster-candidate-20k.json`
  versus
  `target/benchmark-repeat-technical-hatch-bounded-line-reciprocal-candidate-20k.json`
  improved mean `0.262 ms` -> `0.250 ms`, p95 `0.308 ms` -> `0.261 ms`, and
  raster paths `0.162 ms` -> `0.154 ms`.
  Both protection fixtures kept native-rendered status, output dimensions, and
  output bytes unchanged.
- Decision:
  keep. This is a narrow row-bucket predicate optimization backed by current
  profile and trace data. It accepts a small per-line bucket memory increase to
  remove repeated hotpath arithmetic in documents with larger stroke linework,
  without adding dependencies, unsafe code, global state, or alternate
  geometry.

### Post row-bucket line metrics profile from 2026-06-30

- Current post-commit artifacts:
  `target/sample-vector-stress-after-row-bucket-metrics.txt` and
  `target/benchmark-repeat-vector-stress-after-row-bucket-metrics-200k.json`.
- Current focused baseline:
  `vector-stress.pdf` repeat mean `0.579 ms`, p95 `0.637 ms`, p99 `0.713 ms`,
  and repeat mean `raster_paths=0.491 ms`. The timing run is slightly noisier
  than the accepted candidate repeat (`0.572 ms` mean, p95 `0.612 ms`), so use
  repeated before/after evidence for acceptance decisions and this run mainly
  for the next hotspot ordering.
- CPU sample:
  the largest flat symbols are now
  `rasterize_row_bucketed_stroke_ranges` (`413` samples), `blend_pixel`
  (`404`), `stroke_path` (`384`),
  `rasterize_span_covered_stroke_ranges` (`362`),
  `axis_stroke_raster_spans` (`213`), and `merge_pixel_ranges` (`57`).
  `point_in_single_stroke_line` is no longer a top visible bucket after the
  bounded-line metric route.
- Decision:
  use this as the next vector/report baseline. The next candidate should
  target row-bucket raster loop structure, remaining blend/write work,
  span-covered from-start work, or a narrower axis-span build reduction. Do
  not repeat row-bucket metric precomputation, broad row-slice blend, sampled
  blend dispatch, cursor-threshold lowering, joinless raster-span reuse, or
  local row-copy variants already tested above.

Rejected hoisted radius-squared candidate from 2026-06-30:

- Rationale:
  after row-bucket line metrics, `rasterize_row_bucketed_stroke_ranges` was the
  largest sampled flat symbol. The candidate tried to compute `radius * radius`
  once in the row/axis raster loops and pass the squared value through
  row-bucket and join-bucket predicates instead of recomputing it per sample
  predicate call.
- Change tested locally but not kept:
  thread `radius_squared` through row-bucket line predicates and join-bucket
  predicates, including traced variants and tests. No geometry or blend math
  was changed.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-after-row-bucket-metrics-200k.json`
  versus
  `target/benchmark-repeat-vector-stress-hoisted-radius-squared-candidate-200k.json`.
- Result:
  rejected. `vector-stress.pdf` repeat mean regressed `0.579 ms` -> `0.607 ms`,
  p95 regressed `0.637 ms` -> `0.693 ms`, and repeat mean `raster_paths`
  regressed `0.491 ms` -> `0.516 ms`. Output status, dimensions, and bytes
  stayed unchanged.
- Decision:
  reverted. The extra argument threading appears to hurt more than the removed
  multiplication helps in the optimized build. Do not retry this local
  hoisting shape unless a future profile isolates `radius * radius` itself,
  not just the broader row-bucket raster loop.

Fresh post-row-bucket profiling pass from 2026-06-30:

- Artifacts:
  `target/sample-vector-stress-fresh-after-row-bucket-metrics.txt` and
  `target/benchmark-repeat-vector-stress-fresh-profile-200k.json`, captured
  from the profiling build on `fixtures/generated/vector-stress.pdf`,
  `--max-edge 160`, 200,000 repetitions.
- Current focused baseline:
  repeat mean `0.607 ms`, p50 `0.601 ms`, p95 `0.693 ms`, p99 `0.736 ms`,
  repeat mean `raster_paths=0.514 ms`, output status `native_rendered`,
  dimensions `160x120`, and output bytes `76800`.
- CPU sample:
  the largest flat symbols were `rasterize_row_bucketed_stroke_ranges`
  (`1042` samples), `rasterize_span_covered_stroke_ranges` (`985`),
  `blend_pixel` (`972`), `stroke_path` (`927`),
  `axis_stroke_raster_spans` (`556`), and `merge_pixel_ranges` (`142`).
  `merge_pixel_ranges` still shows `sort_by_key` frames, but the earlier
  sorted/unstable merge candidates were too small or regressed protection
  fixtures, so the next accepted attempt should reduce span/sample work or
  row-bucket predicate work rather than change sort flavor again.

Rejected sample-scale candidate from 2026-06-30:

- Rationale:
  the fresh profile still showed heavy per-sample stroke raster work. The
  candidate tried to remove repeated `1.0 / samples` divisions from the
  stroke sample loops by computing a per-raster-pass scale and using
  multiplication in `sample_point_scaled`.
- Change tested locally but not kept:
  add `sample_point_scaled` and route row-bucket, active row-bucket,
  simple-line, and axis-span stroke loops through a precomputed sample scale.
  Geometry predicates, blend math, clipping, output format, dependencies, and
  unsafe usage were unchanged.
- Correctness checks while the candidate was present:
  `cargo fmt --all --check`,
  `cargo test -p ferrugo-render --no-default-features stroke_row_buckets`, and
  `cargo test -p ferrugo-render --no-default-features axis_stroke`.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-fresh-profile-200k.json` versus
  `target/benchmark-repeat-vector-stress-sample-scale-candidate-200k.json`.
- Result:
  rejected as below the repeated 5% acceptance floor. The focused repeat moved
  mean `0.607 ms` -> `0.593 ms` (~2.3%) and repeat mean `raster_paths`
  `0.514 ms` -> `0.503 ms` (~2.1%). p95 moved `0.693 ms` -> `0.650 ms`
  (~6.2%), but that was not enough by itself because the mean and phase timing
  stayed below threshold and the baseline p95 was noisier than the prior
  accepted row-bucket metric run. Output status, dimensions, and bytes stayed
  unchanged.
- Decision:
  reverted. Do not retry this sample-scale shape unless a future lower-level
  profile isolates floating-point sample-coordinate division as a standalone
  cost, not just the broader span or row-bucket raster loops.

Rejected span sample-scale candidate from 2026-06-30:

- Rationale:
  `rasterize_span_covered_stroke_ranges` was the second-largest flat symbol in
  the fresh profile. The prior sample-scale candidate only touched Point-based
  stroke loops, so this follow-up tested the same arithmetic idea in the
  span-coverage X membership loops.
- Change tested locally but not kept:
  compute `sample_scale = 1.0 / samples` once in the span-covered cursor and
  from-start routes, including traced variants, then compute sample X
  coordinates with multiplication instead of division. No row-bucket code,
  geometry predicates, blend math, clipping, dependencies, or unsafe usage
  changed.
- Correctness checks while the candidate was present:
  `cargo fmt --all --check` and
  `cargo test -p ferrugo-render --no-default-features axis_stroke`.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-fresh-profile-200k.json` versus
  `target/benchmark-repeat-vector-stress-span-sample-scale-candidate-200k.json`.
- Result:
  rejected as below the repeated 5% acceptance floor for mean and phase timing.
  The focused repeat moved mean `0.607 ms` -> `0.588 ms` (~3.1%) and repeat
  mean `raster_paths` `0.514 ms` -> `0.499 ms` (~2.9%). p95 moved
  `0.693 ms` -> `0.637 ms` (~8.1%), but the p95-only movement is not enough
  because the fresh baseline p95 was noisier than the previous accepted
  post-row-bucket metric run. Output status, dimensions, and bytes stayed
  unchanged.
- Decision:
  reverted. Avoid repeating scalar sample-coordinate rewrites in the current
  vector track; the next accepted change needs to remove actual span or
  row-bucket work, not just replace division syntax inside the same loops.

Rejected span single-pass membership candidate from 2026-06-30:

- Rationale:
  a fresh trace artifact,
  `target/trace-vector-stress-current-work-reduction.json`, confirmed that
  current `vector-stress.pdf` still spends visible work in the from-start span
  path: `44` span-covered calls, `44` from-start calls, `9008` span pixels,
  `36032` span sample points, `9008` covered pixels, and no full-coverage
  pixels. The candidate tried to reduce actual membership work rather than
  repeating the rejected sample-coordinate arithmetic changes.
- Change tested locally but not kept:
  replace the from-start span membership loop with
  `covered_samples_in_axis_stroke_span_row`, which scans each sorted coverage
  span row once per output pixel and counts all sample-X positions for that
  pixel. The cursor-route threshold, row-bucket code, blend math, clipping,
  output format, dependencies, and unsafe usage stayed unchanged.
- Correctness checks while the candidate was present:
  `cargo fmt --all --check`,
  `cargo test -p ferrugo-render --no-default-features covered_samples_in_axis_stroke_span_row_should_match_membership_scan`,
  and
  `cargo test -p ferrugo-render --no-default-features stroke_raster_route_summary_should_count_span_from_start_work`.
- A/B artifacts:
  `target/benchmark-repeat-vector-stress-fresh-profile-200k.json` versus
  `target/benchmark-repeat-vector-stress-span-single-pass-candidate-200k.json`.
- Result:
  rejected as below the repeated 5% acceptance floor. The focused repeat moved
  mean `0.607 ms` -> `0.597 ms` (~1.6%) and repeat mean `raster_paths`
  `0.514 ms` -> `0.512 ms` (~0.4%). p95 moved `0.693 ms` -> `0.639 ms`
  (~7.8%), but the phase timing shows the structural work did not actually
  move the dominant raster cost. Output status, dimensions, and bytes stayed
  unchanged.
- Decision:
  reverted. The from-start span path is visible, but local membership-loop
  spelling is not the next useful lever. The next vector/report attempt should
  move to row-bucket predicate reduction, blend/write reduction backed by a
  narrower profile, or a larger route-selection change that reduces visited
  pixels/sample points.

Rejected direct pixel write candidate from 2026-06-30:

- Rationale:
  the fresh CPU sample still showed `blend_pixel` as a top flat symbol after
  the accepted single-offset blend write. The candidate targeted the remaining
  per-pixel write helper without changing blend routing, source-over math, or
  sampled blend behavior.
- Change tested locally but not kept:
  replace `write_pixel_at_offset`'s four-byte `copy_from_slice` from a
  temporary array with direct channel assignments into the raster buffer.
  Public APIs, blend math, geometry, clipping, dependencies, and unsafe usage
  stayed unchanged.
- Correctness checks while the candidate was present:
  `cargo fmt --all --check`,
  `cargo test -p ferrugo-render --no-default-features blend_pixel`, and
  `cargo test -p ferrugo-render --no-default-features source_over`.
- Profiling-build A/B artifacts:
  `target/benchmark-repeat-vector-stress-fresh-profile-200k.json`,
  `target/benchmark-repeat-vector-stress-direct-write-candidate-200k.json`,
  and
  `target/benchmark-repeat-vector-stress-direct-write-candidate-repeat-200k.json`.
  The profiling build looked promising: repeat mean `0.607 ms` -> `0.576 ms`
  and `0.574 ms` on repeat, with p95 `0.693 ms` -> `0.623 ms` and
  `0.614 ms`.
- Protection artifacts:
  `target/benchmark-repeat-prepress-direct-write-candidate-20k.json` and
  `target/benchmark-repeat-technical-hatch-direct-write-candidate-20k.json`
  showed small protection regressions versus the accepted bounded-line metric
  artifacts, but all were below the 5% regression threshold and outputs stayed
  unchanged.
- Release-mode A/B artifacts:
  `target/benchmark-repeat-vector-stress-direct-write-baseline-release-100k.json`
  versus
  `target/benchmark-repeat-vector-stress-direct-write-candidate-release-100k.json`.
- Result:
  rejected because release mode did not confirm the profiling-build win.
  Release repeat mean moved only `0.569 ms` -> `0.564 ms` (~0.9%),
  repeat mean `raster_paths` moved `0.484 ms` -> `0.478 ms` (~1.2%), and p95
  was neutral to slightly worse (`0.610 ms` -> `0.612 ms`). Output status,
  dimensions, and bytes stayed unchanged.
- Decision:
  reverted. Treat direct byte-write spelling as a profiling-build artifact, not
  a renderer performance win. Future blend work needs a release-confirmed
  profile that isolates arithmetic or memory traffic inside `blend_pixel`
  more narrowly than the write helper.

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
