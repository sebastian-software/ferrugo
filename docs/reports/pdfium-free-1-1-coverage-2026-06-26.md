# PDFium-Free 1.1 Coverage 2026-06-26

Milestone: 0180

## Decision

Stabilize the PDFium-free server/runtime path, but defer a broad PDFium-free
1.1 replacement claim.

The native-only release gate passed, server batch isolation remains clean, and
the expanded dashboard keeps unsupported boundaries visible. However, the
expanded 1.1 core support slice is no longer fallback-free because
`office-export` now contains one typed `text.font-program` fallback. The PDFium
maintainer visual oracle also still reports heavy visual blocker counts across
browser print, forms, and office exports.

Recommendation: treat 1.1 as a scoped stabilization release for the supported
native runtime and diagnostics, not as a broad PDFium replacement release.

## Native Runtime Coverage

Core support artifact: `target/readiness-0180-core-supported-gate.json`

The strict `--fail-on-fallback` run failed as expected after detecting one
typed fallback in the expanded `office-export` set. The same classification was
rerun without `--fail-on-fallback` to capture the exact boundary:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 15 | 15 | 0 | 0 |
| `form` | 22 | 22 | 0 | 0 |
| `office-export` | 61 | 60 | 1 | 0 |
| **Core total** | **98** | **97** | **1** | **0** |

Fallback bucket:

| Bucket | Count | Implication |
| --- | ---: | --- |
| `text.font-program` | 1 | The expanded office-export set is not yet a fallback-free release slice. |

## Expanded Dashboard

Dashboard artifact: `target/readiness-0180-dashboard/dashboard.json`

Primary generated families:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 187 | 176 | 10 | 1 encrypted |

Fallback categories:

| Category | Count |
| --- | ---: |
| `form.xfa-dynamic` | 1 |
| `graphics.color-management` | 1 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |
| `graphics.transparency` | 2 |
| `image.filter` | 3 |
| `text.font-program` | 1 |

Operator coverage:

| Total | Scanned | Errors | Operators | Inline images |
| ---: | ---: | ---: | ---: | ---: |
| 187 | 186 | 1 | 9652 | 0 |

## Performance And Memory

Performance artifact: `target/readiness-0180-dashboard/performance.json`

Report and presentation sample:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 53 | 48 | 5 | 0 | 5 |

| Family | Total | Native rendered | Fallback required | Mean ms | Max ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| `presentation` | 9 | 8 | 1 | 11.608 | 24.057 |
| `report` | 44 | 40 | 4 | 50.653 | 285.294 |

Batch artifact: `target/readiness-0180-dashboard/batch.json`

| Jobs | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 0 | 0 | 0 |

| Metric | Value |
| --- | ---: |
| Throughput jobs/sec | 33.998 |
| Mean latency ms | 48.432 |
| P95 latency ms | 184.055 |
| Max in-flight pixels | 102400 |
| Max output bytes | 78720 |

RSS sampling returned `null` in this restricted run. The gate still records
hard in-flight pixel bounds and output high-water.

## Visual Oracle Evidence

Visual artifact: `target/readiness-0180-core-visual-diff.json`

This was an explicit maintainer comparison run with PDFium available locally.
It is comparison evidence, not a release runtime dependency.

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 15 | 2 | 6 | 7 | 0 | 0 |
| `form` | 22 | 0 | 1 | 21 | 0 | 0 |
| `office-export` | 61 | 0 | 3 | 57 | 1 | 0 |
| **Core total** | **98** | **2** | **10** | **85** | **1** | **0** |

Subsystem blockers:

| Subsystem | Blockers | Native errors |
| --- | ---: | ---: |
| `rendering-core` | 33 | 1 |
| `text-fonts` | 24 | 0 |
| `annotations-forms` | 16 | 0 |
| `page-geometry` | 7 | 0 |
| `vector-graphics` | 4 | 0 |
| `transparency` | 1 | 0 |

The native error is the same `text.font-program` boundary surfaced by the
native support gate.

## Native-Only Release Gate

Command:

```sh
bash scripts/check_native_only_release.sh
```

Result: passed.

Covered:

- native-only `cargo check`;
- native-only `cargo test`;
- plugin-free distribution check;
- PDFium quarantine check;
- `pdfrust-cli` package file inspection;
- leaf package dry-runs for `pdfrust-syntax` and `pdfrust-thumbnail`;
- all-features Clippy.

Registry-backed workspace package verification was intentionally skipped because
`PDFRUST_NATIVE_RELEASE_VERIFY_REGISTRY` was not set.

## Security

Command:

```sh
bash scripts/check_fuzz_smoke.sh
```

Result:

| Target | Cases | Result |
| --- | ---: | --- |
| `primitive_parse` | 165 | passed |
| `xref_load` | 154 | passed |
| `stream_decode` | 154 | passed |
| `content_tokenize` | 165 | passed |
| `render_setup` | 176 | passed |

## Post-1.1 Backlog

Ranked follow-up:

1. Restore a fallback-free core gate by addressing or explicitly moving the
   `text.font-program` office-export boundary.
2. Reduce office-export `rendering-core` and `text-fonts` visual blockers.
3. Reduce form/widget visual blockers before broad form-facing claims.
4. Keep advanced image filters, dynamic XFA, optional content, color
   management, pattern/mesh shading, and transparency as typed unsupported
   boundaries until their implementation milestones land.
5. Continue using PDFium only as explicit maintainer comparison tooling, not as
   a runtime or release dependency.

## Validation

- `bash scripts/generate_corpus_dashboard.sh target/readiness-0180-dashboard`
- strict core supported gate with `--fail-on-fallback` confirmed the new typed
  fallback boundary
- core classification artifact without `--fail-on-fallback`
- `bash scripts/check_native_only_release.sh`
- `bash scripts/check_fuzz_smoke.sh`
- PDFium maintainer visual-diff on the expanded core slice
