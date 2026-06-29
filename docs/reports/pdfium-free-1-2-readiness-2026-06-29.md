# PDFium-Free 1.2 Readiness 2026-06-29

Milestone 0200 makes the 1.2 release decision from the current native-only
server/runtime evidence.

## Decision

Stabilize the scoped PDFium-free server/runtime path, but defer a broad
PDFium-replacement claim for 1.2.

The runtime and packaging story is healthy: PDFium is absent from the supported
native package path, server batch gates are bounded, diagnostics are
privacy-safe, serverless startup/binary budgets pass, and fuzz smoke passes.

The broader corpus is not ready for a broad replacement claim. The expanded
primary families still contain 12 typed unsupported rows across scan, report,
presentation, mixed-layout, and office-export. These are explicit and
reproducible, but they block broad scan/fax/archive, report/dashboard, and
office-export claims until the owner milestones land.

## Native Runtime Coverage

Dashboard command:

```sh
bash scripts/generate_corpus_dashboard.sh target/readiness-0200-dashboard
```

Primary family support artifact:

- `target/readiness-0200-dashboard/support.json`

| Scope | Total | Native rendered | Typed unsupported | Errors |
| --- | ---: | ---: | ---: | ---: |
| Primary families | 203 | 190 | 12 | 1 encrypted |

Family summary:

| Family | Total | Native rendered | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| `form` | 24 | 24 | 0 | 0 |
| `mixed-layout` | 27 | 24 | 2 | 1 encrypted |
| `office-export` | 63 | 62 | 1 | 0 |
| `presentation` | 12 | 10 | 2 | 0 |
| `report` | 50 | 46 | 4 | 0 |
| `scan` | 27 | 24 | 3 | 0 |

Unsupported categories match the 0199 burn-down:

| Category | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.transparency` | 2 |
| `graphics.optional-content` | 2 |
| `annotation.appearance` | 1 |
| `form.xfa-dynamic` | 1 |
| `graphics.color-management` | 1 |
| `graphics.pattern-shading` | 1 |
| `text.font-program` | 1 |

## Performance, Batch, And Serverless

Performance artifact:

- `target/readiness-0200-dashboard/performance.json`

Report/presentation sample:

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 62 | 56 | 6 | 0 | 6 |

The 6 budget failures are the expected typed unsupported rows in that sample.

Batch artifact:

- `target/readiness-0200-dashboard/batch.json`

| Jobs | Native rendered | Fallbacks | Errors | Budget failures | Throughput/sec | P95 ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 0 | 0 | 0 | 44.188 | 136.729 |

Serverless artifact:

- `target/serverless-profile-0197.json`

| Binary bytes | Startup p95 ms | First-render p95 ms | Budget failures |
| ---: | ---: | ---: | ---: |
| 1,017,344 | 286.426 | 4.726 | 0 |

## Packaging And Security

Native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

Result: passed.

This covered native-only check/test, plugin-free distribution, PDFium
quarantine, CLI package file inspection, leaf package dry-runs, and all-features
Clippy. Registry-backed workspace verification remains optional and was skipped
because `PDFRUST_NATIVE_RELEASE_VERIFY_REGISTRY` was not set.

The plugin-free scan initially produced a false positive by matching
`hyperlink` as the `hyper` crate. The script now uses word boundaries for
network/TLS/package-hook terms and still rejects actual dependency or source
hooks.

Fuzz smoke:

```sh
bash scripts/check_fuzz_smoke.sh
```

Result: passed after updating the `render_setup` target to include the current
`ThumbnailOptions` fields.

| Target | Cases | Result |
| --- | ---: | --- |
| `primitive_parse` | 165 | passed |
| `xref_load` | 154 | passed |
| `stream_decode` | 154 | passed |
| `content_tokenize` | 165 | passed |
| `render_setup` | 176 | passed |

## Visual Validation

The PDFium-free oracle strategy remains unchanged: runtime readiness must be
native-only, while visual comparisons are maintainer evidence. 1.2 did not add a
new PDFium runtime dependency and did not use visual-diff results to hide typed
unsupported outcomes.

Current 0200 decision relies on:

- native-only support and benchmark artifacts from the dashboard;
- 0199 fixture-level unsupported burn-down;
- existing Poppler/PDFium maintainer visual reports for subsystem backlog
  context.

## Post-1.2 Backlog

1. `image.filter`: choose safe Rust-native codec policy for CCITT/JBIG2/JPX
   before broad scan/fax/archive claims.
2. `graphics.transparency`: reduce unsupported soft-mask/blend boundaries for
   report/dashboard claims.
3. `text.font-program`: triage the office-export emoji/font-program boundary.
4. `annotation.appearance`: decide FreeText synthesis/appearance policy.
5. `graphics.optional-content`: keep OCMD and usage-application behavior typed
   until the layer semantics are fully covered.
6. `graphics.pattern-shading`: reduce vector/chart pattern and mesh gaps.
7. `graphics.color-management`: keep black point compensation typed unless
   real-corpus frequency rises.
8. `form.xfa-dynamic`: keep dynamic XFA as an accepted unsupported boundary.

## Validation Commands

```text
bash scripts/generate_corpus_dashboard.sh target/readiness-0200-dashboard
bash scripts/check_native_only_release.sh
bash scripts/check_fuzz_smoke.sh
scripts/measure_serverless_profile.sh
cargo fmt --check
git diff --check -- scripts/check_plugin_free_distribution.sh fuzz/fuzz_targets/render_setup.rs docs/milestones/0200-pdfium-free-1-2-readiness-gate.md docs/milestones/README.md docs/reports/pdfium-free-1-2-readiness-2026-06-29.md
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
