# Page Cache Reuse Policy 2026-06-25

## Decision

The native renderer keeps page artifact reuse scoped to one render pass by
default. The exposed policy is `isolated-render`, and it does not permit disk
persistence of document-derived artifacts.

Longer-lived reuse should remain caller-owned until a future document-session
cache can account for memory, tenant lifetime, invalidation, and eviction. The
prototype key shape is `NativePageCacheKey`:

- `document_identity`: caller-provided content hash or tenant-scoped document
  version id.
- `page_index`: zero-based page.
- `max_edge`: requested thumbnail edge budget.
- `background`: RGBA background bytes.
- `renderer_version`: native backend package version.
- `native_profile`: `default` or `low-memory`.

## Cacheable Artifacts

The current renderer already uses bounded pass-local reuse for selected hot
paths:

| Artifact | Current scope | Persistence decision |
| --- | --- | --- |
| Parsed objects and page tree | Single render/document load | Do not persist by default |
| Font programs and glyph outlines | Bounded resource/display-list work | Revisit through document-session cache |
| Fallback glyph bitmaps | Raster pass | Keep pass-local |
| Decoded images and ICC transform metadata | Resource build / caller-owned ICC cache | Keep caller-owned for now |
| Tiling pattern cells | Rasterization pass | Keep pass-local |
| Display lists | Single page render | Revisit only with bounded memory accounting |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/page-cache-policy-manifest.tsv --include-family small --include-family business --include-family repeated-resources --include-family vector-stress --repetitions 3 --max-edge 160 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/page-cache-0134-repeat-benchmark.json
```

Summary:

| Metric | Value |
| --- | ---: |
| Fixtures | 4 |
| Repetitions | 3 |
| Native rendered | 4 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |
| Cache policy | `isolated-render` |
| Disk persistence | disabled |

Per-family timing:

| Family | First mean ms | Repeat mean ms | Repeat / first |
| --- | ---: | ---: | ---: |
| `business` | 32.164 | 29.801 | 0.927 |
| `repeated-resources` | 15.687 | 15.859 | 1.011 |
| `small` | 0.483 | 0.478 | 0.990 |
| `vector-stress` | 191.372 | 190.049 | 0.993 |

The repeated runs are close to first-render timings. That is useful evidence
against introducing a global persistent page cache now: it would add privacy,
invalidation, and memory-retention risk without a demonstrated default win on
the representative thumbnail workload.

## Isolation And Privacy

Cache keys include document identity and render options, so two different
documents with identical page and output options cannot share an artifact
unless the caller intentionally gives them the same document identity. The
benchmark JSON records a distinct content-hash identity for each fixture.

No default policy writes document content, decoded resources, display lists, or
rendered pixels to disk. Applications that need cross-request reuse should keep
the cache in their own trust boundary, apply an explicit byte budget, and evict
by document version or tenant lifetime.

## Validation

- `cargo fmt --check`
- `cargo test -p pdfrust-native native_page_cache -- --nocapture`
- `cargo test -p pdfrust-cli repeat_benchmark -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native ...`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
