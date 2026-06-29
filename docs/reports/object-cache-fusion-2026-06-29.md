# Incremental Parser And Object Cache Fusion

Milestone: 0214
Date: 2026-06-29

## Summary

Xref-stream documents now build a document-local compressed-object lookup index
when loading `ModernDocument`. The index maps compressed object ids to the
loaded object stream and object position. `ModernDocument::get_object` can
therefore resolve compressed objects without scanning every loaded object stream
and every object entry on each lookup.

The change is intentionally bounded: decoded object stream bytes were already
retained for validated compressed entries, and the new index stores only object
ids plus `usize` positions. Compressed object values are still parsed on demand
from the decoded stream slices, so the loader does not materialize every
compressed indirect object as an owned value.

## Cache Invalidation

The lookup index is scoped to one `ModernDocument` instance. Loading a new input,
retrying recovery, or falling back from a linearized first-page load to the full
loader creates a new document instance and a new index. Nothing is shared across
documents, tenants, render jobs, or byte slices.

## Coverage

Added `fixtures/object-cache-fusion-manifest.tsv` with seven generated fixtures:

| Family | Count | Purpose |
| --- | ---: | --- |
| `linearized-navigation` | 1 | Fast first-page object retention. |
| `linearized-recovery` | 1 | Malformed linearization hints with full-loader fallback. |
| `incremental-navigation` | 2 | Latest incremental revision and deleted object tombstone behavior. |
| `hybrid-navigation` | 1 | Hybrid classic xref plus xref stream. |
| `recovery-navigation` | 1 | Bounded xref offset drift recovery. |
| `long-document-navigation` | 1 | Long-document page navigation/resource access. |

## Native Gate

Artifact: `target/object-cache-0214-supported-gate.json`

Result:

- Total: 7
- Native rendered: 7
- Fallback required: 0
- Errors: 0

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/object-cache-fusion-manifest.tsv --include-family linearized-navigation --include-family linearized-recovery --include-family incremental-navigation --include-family hybrid-navigation --include-family recovery-navigation --include-family long-document-navigation --fail-on-fallback --max-edge 160 --output target/object-cache-0214-supported-gate.json
```

## Navigation Benchmark

Artifact: `target/object-cache-0214-benchmark.json`

Result:

- Total: 7
- Native rendered: 7
- Fallback required: 0
- Errors: 0
- Budget failures: 0
- Slowest family mean: `long-document-navigation` at `5.208ms`
- Largest family output bytes: `long-document-navigation` at `72960`

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/object-cache-fusion-manifest.tsv --include-family linearized-navigation --include-family linearized-recovery --include-family incremental-navigation --include-family hybrid-navigation --include-family recovery-navigation --include-family long-document-navigation --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/object-cache-0214-benchmark.json
```

## Focused Tests

```bash
cargo test -p ferrugo-object load_modern_document_should_load_xref_stream_and_object_stream -- --nocapture
cargo test -p ferrugo-object incremental -- --nocapture
```

All focused tests passed locally.

## Workspace Validation

```bash
cargo fmt --check
git diff --check -- crates/ferrugo-object/src/lib.rs docs/policies/incremental-and-hybrid-references.md docs/milestones/README.md docs/milestones/0214-incremental-parser-and-object-cache-fusion.md docs/reports/object-cache-fusion-2026-06-29.md fixtures/object-cache-fusion-manifest.tsv
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All workspace validation commands passed locally. The pre-existing unstaged
`.gitignore` whitespace change remains unrelated and was not modified here.
