# PDFium-Free 1.0 Readiness 2026-06-26

Milestone: 0160.

## Decision

Defer a broad PDFium-free 1.0 GA claim.

The Rust-native renderer is ready for PDFium-free runtime execution on the
current core supported families: `browser-print`, `office-export`, and `form`.
Those 87 fixtures render natively with 0 fallbacks and 0 errors in the
native-only gate.

The project should not claim that the Rust renderer is a broad visual
replacement for PDFium yet. The PDFium visual oracle still reports 77 blockers
inside the same 87-fixture core set. PDFium remains necessary as maintainer
oracle tooling for visual diff, benchmarks, metadata comparison, and targeted
triage, but it is not a runtime dependency for the supported native path.

Recommendation: ship/stabilize the PDFium-free supported runtime slice, keep
the 1.0 GA language scoped to explicit supported families and typed unsupported
boundaries, and defer any broad "PDFium replacement" claim until the visual
blocker backlog is materially reduced.

## Native Runtime Coverage

Artifact: `target/readiness-0160-core-supported-gate.json`

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/readiness-0160-core-supported-gate.json
```

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 11 | 11 | 0 | 0 |
| `form` | 22 | 22 | 0 | 0 |
| `office-export` | 54 | 54 | 0 | 0 |
| **Core total** | **87** | **87** | **0** | **0** |

This is the valid runtime readiness claim: the supported core families do not
need PDFium to render through the native thumbnail path.

## Full Corpus Boundaries

Artifact: `target/readiness-0160-full-fallback-summary.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 0 |
| `browser-print` | 11 | 11 | 0 | 0 |
| `form` | 22 | 22 | 0 | 0 |
| `mixed-layout` | 22 | 20 | 1 | 1 encrypted |
| `office-export` | 54 | 54 | 0 | 0 |
| `presentation` | 9 | 8 | 1 | 0 |
| `report` | 42 | 39 | 3 | 0 |
| `scan` | 25 | 22 | 3 | 0 |
| **Total** | **186** | **177** | **8** | **1 encrypted** |

Typed unsupported boundaries:

| Bucket | Count | Boundary |
| --- | ---: | --- |
| `image.filter` | 3 | CCITT, JBIG2, and JPX codec policy/support boundary. |
| `graphics.transparency` | 2 | Unsupported blend and soft-mask cases. |
| `form.xfa-dynamic` | 1 | Dynamic XFA without a supported static appearance path. |
| `graphics.optional-content` | 1 | OCMD membership-policy gap. |
| `graphics.pattern-shading` | 1 | Mesh/pattern shading gap. |

The full corpus result is acceptable only because the unsupported cases remain
typed and explicit. It is not evidence for broad PDF specification coverage.

## Performance And Memory Evidence

Native benchmark artifact: `target/readiness-0160-benchmark-native.json`

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 0 | 0 | 6.030 | 6.030 |
| `browser-print` | 11 | 11 | 0 | 0 | 0 | 26.951 | 54.675 |
| `form` | 22 | 22 | 0 | 0 | 0 | 22.707 | 88.488 |
| `mixed-layout` | 22 | 20 | 1 | 1 | 2 | 13.045 | 47.211 |
| `office-export` | 54 | 54 | 0 | 0 | 0 | 16.041 | 50.134 |
| `presentation` | 9 | 8 | 1 | 0 | 1 | 12.971 | 26.067 |
| `report` | 42 | 39 | 3 | 0 | 3 | 54.871 | 308.727 |
| `scan` | 25 | 22 | 3 | 0 | 3 | 14.075 | 52.056 |
| **Total** | **186** | **177** | **8** | **1** | **9** | n/a | n/a |

The 9 budget failures match the 8 typed unsupported rows plus the encrypted
fixture error. The core supported families have 0 benchmark budget failures.

Batch memory/throughput artifact: `target/readiness-0160-batch-memory.json`

| Metric | Value |
| --- | ---: |
| Total jobs | 16 |
| Native rendered | 16 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |
| Throughput jobs/sec | 26.226 |
| Mean latency ms | 46.786 |
| P95 latency ms | 191.782 |
| Max in-flight pixels | 51200 |
| Max output bytes | 78720 |

RSS sampling returned `null` in this restricted run, but the gate records the
hard memory scheduling bound and output high-water. The earlier low-memory
profile and scratch-buffer audit remain the dedicated constrained-memory
evidence for thumbnail execution.

## PDFium Visual Oracle

Artifact: `target/readiness-0160-core-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 11 | 2 | 4 | 5 | 0 | 0 |
| `form` | 22 | 0 | 1 | 21 | 0 | 0 |
| `office-export` | 54 | 0 | 3 | 51 | 0 | 0 |
| **Core total** | **87** | **2** | **8** | **77** | **0** | **0** |

Subsystem blocker clusters:

| Subsystem | Blockers | Main implication |
| --- | ---: | --- |
| `rendering-core` | 29 | Tables, dense layouts, and generated layout details still need parity work. |
| `text-fonts` | 24 | Text metrics and font fidelity remain product-visible blockers. |
| `annotations-forms` | 16 | Form appearance parity is not ready for a broad GA claim. |
| `page-geometry` | 7 | Rotation, box, and generated layout transforms still drift. |
| `vector-graphics` | 1 | A small vector subset still needs focused parity work. |

This result is the reason for the defer decision. PDFium is still required as a
maintainer oracle even though the runtime supported path no longer depends on
PDFium.

## Packaging And Security Evidence

The PDFium-free installation gate passed:

- `scripts/check_plugin_free_distribution.sh` confirms the native CLI graph has
  no `ferrugo-pdfium` dependency edge, no hidden fetch/plugin hooks, and no
  committed binary artifacts under `crates/`.
- `scripts/check_pdfium_quarantine.sh` confirms PDFium remains quarantined in
  explicit maintainer-only paths.
- `cargo package --workspace --allow-dirty` passed.

Fuzz smoke suite passed:

| Target | Result |
| --- | --- |
| `primitive_parse` | 165 smoke cases completed |
| `xref_load` | 154 smoke cases completed |
| `stream_decode` | 154 smoke cases completed |
| `content_tokenize` | 165 smoke cases completed |
| `render_setup` | 176 smoke cases completed |

## Stabilization Backlog

Use `docs/backlogs/native-renderer-conformance-backlog.md` as the primary
visual-fidelity backlog and `docs/backlogs/pdfium-free-maintenance-backlog.md`
as the runtime/tooling separation backlog.

Highest-priority stabilization slices:

1. Reduce `office-export` text/font and dense table blockers.
2. Reduce form/widget appearance blockers before broad form-facing claims.
3. Split `rendering-core` blockers into table rules, clipping, generated
   layout, and vector composition reductions.
4. Keep specialized image codecs, transparency gaps, OCMD policy, mesh/pattern
   shading, and dynamic XFA as typed unsupported boundaries until their
   follow-up milestones land.
5. Keep PDFium oracle commands behind `--features pdfium`; do not move them
   back into runtime or package-install requirements.

## Validation Commands

```sh
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo package --workspace --allow-dirty
bash scripts/check_plugin_free_distribution.sh
bash scripts/check_pdfium_quarantine.sh
cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/readiness-0160-core-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/readiness-0160-full-fallback-summary.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/readiness-0160-benchmark-native.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/readiness-0160-batch-memory.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/readiness-0160-core-visual-diff.json
cargo fmt --check
```
