# Rust-Native Font Cache Compaction

Milestone: 0212
Date: 2026-06-29

## Summary

The native renderer now has a resident total decoded embedded-font-program
budget per page resource map. This closes the gap where many unique embedded
subset fonts could each stay under the per-program limit while still growing the
resource map without an aggregate font-program cap.

The cache keeps the existing sharing behavior: repeated references to the same
embedded font stream return the cached `Arc<[u8]>` and count once against the
total. Unique font streams count independently. Budget failures return
`FontProgramResourceBytesOverflow` and surface through native diagnostics
without including document text, font names, glyph strings, or decoded font
bytes.

## Coverage

Added `fixtures/font-cache-compaction-manifest.tsv` with nine existing generated
fixtures:

| Family | Count | Purpose |
| --- | ---: | --- |
| `office-font-cache` | 1 | Office-export missing-font fallback selection. |
| `browser-font-cache` | 1 | Browser-print missing-font fallback selection. |
| `report-font-cache` | 1 | Report-style table and text workload. |
| `longform-font-cache` | 1 | Multipage repeated font/resource workload. |
| `subset-font-cache` | 2 | TrueType and CFF subset font programs. |
| `cjk-font-cache` | 2 | CID/Identity-H CJK mapping and width coverage. |
| `type3-font-cache` | 1 | Repeated Type3 CharProc glyph workload. |

## Native Gate

Artifact: `target/font-cache-0212-supported-gate.json`

Result:

- Total: 9
- Native rendered: 9
- Fallback required: 0
- Errors: 0

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/font-cache-compaction-manifest.tsv --include-family office-font-cache --include-family browser-font-cache --include-family report-font-cache --include-family longform-font-cache --include-family subset-font-cache --include-family cjk-font-cache --include-family type3-font-cache --fail-on-fallback --max-edge 160 --diagnostics-dir target/font-cache-0212-diagnostics-filtered --output target/font-cache-0212-supported-gate.json
```

## Low-Memory Repeat Gate

Artifact: `target/font-cache-0212-low-memory-repeat.json`

Result:

- Total: 9
- Native rendered: 9
- Fallback required: 0
- Errors: 0
- Budget failures: 0
- Slowest family first-render mean: `report-font-cache` at `10.346ms`
- Slowest family repeat mean: `report-font-cache` at `10.252ms`

Command:

```bash
cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/font-cache-compaction-manifest.tsv --include-family office-font-cache --include-family browser-font-cache --include-family report-font-cache --include-family longform-font-cache --include-family subset-font-cache --include-family cjk-font-cache --include-family type3-font-cache --native-profile low-memory --repetitions 2 --max-edge 160 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/font-cache-0212-low-memory-repeat.json
```

## Cache Invariants

- `max_font_program_bytes` remains the per embedded-font-program decode cap.
- `max_total_font_program_bytes` is the resident aggregate cap for decoded
  embedded-font programs in one page resource map.
- Cache hits are checked before aggregate accounting, so repeated references to
  the same font object do not consume the total budget again.
- ToUnicode CMaps, Type3 CharProcs, fallback face resolution, glyph bitmaps,
  glyph outlines, and text raster scratch storage remain covered by their
  existing bounded byte, entry, segment, or retained-capacity limits.

## Focused Tests

```bash
cargo test -p ferrugo-render font_resources_should_share_program_cache_for_repeated_references -- --nocapture
cargo test -p ferrugo-render font_resources_should_enforce_total_program_byte_budget -- --nocapture
cargo test -p ferrugo-native native_backend_should_expose_memory_diagnostics -- --nocapture
cargo test -p ferrugo-native native_low_memory_profile_should_expose_tighter_memory_diagnostics -- --nocapture
```

All focused tests passed locally.

## Workspace Validation

```bash
cargo fmt --check
git diff --check -- crates/ferrugo-render/src/lib.rs crates/ferrugo-native/src/lib.rs crates/ferrugo-cli/src/main.rs fixtures/font-cache-compaction-manifest.tsv docs/backend/native.md docs/baselines.md docs/milestones/README.md docs/milestones/0212-rust-native-font-cache-compaction.md docs/reports/font-cache-compaction-2026-06-29.md
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All workspace validation commands passed locally. A full unscoped
`git diff --check` still reports the pre-existing `.gitignore` trailing
whitespace change that was left untouched by this milestone.
