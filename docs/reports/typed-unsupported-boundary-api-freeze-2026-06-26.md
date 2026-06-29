# Typed Unsupported Boundary API Freeze 2026-06-26

Milestone: 0174

## Summary

Promoted the native unsupported-feature buckets into the backend-neutral
`ferrugo-thumbnail` facade as stable diagnostic constants. The high-level public
error class remains `unsupported`; the bucket refines it for telemetry,
support decisions, and explicit alternate-renderer routing.

The native backend now uses the same facade constants that consumers can import,
so runtime behavior and documentation share one source of truth.

## Public API

Added:

- `ferrugo_thumbnail::unsupported_feature_buckets`
- `ferrugo_thumbnail::STABLE_UNSUPPORTED_FEATURE_BUCKETS`

The stable bucket set covers:

| Bucket |
| --- |
| `native.unsupported` |
| `renderer.memory-budget` |
| `renderer.form-xobject-composition` |
| `graphics.optional-content` |
| `graphics.color-management` |
| `graphics.pattern-shading` |
| `graphics.stroke-clip` |
| `graphics.transparency` |
| `image.color-space` |
| `image.filter` |
| `form.xfa-dynamic` |
| `text.cmap-tounicode` |
| `text.font-program` |
| `text.glyph-outline` |

## Consumer Guidance

Updated `docs/errors.md` and `docs/policies/native-renderer-api-semver.md`:

- branch on `ThumbnailError::class()` for coarse fallback behavior;
- use `unsupported_feature_bucket()` only when feature-specific handling is
  needed;
- do not treat `malformed`, `encrypted`, `timeout`, or `internal` as
  unsupported feature buckets;
- do not parse display strings for control flow.

## Regression Coverage

Added facade tests for stable unsupported bucket strings and native tests for
representative unsupported boundaries:

| Fixture or condition | Expected bucket |
| --- | --- |
| `unsupported-ccitt-image.pdf` | `image.filter` |
| `optional-content-ocmd.pdf` | `graphics.optional-content` |
| `xfa-dynamic-no-static-appearance.pdf` | `form.xfa-dynamic` |
| `chat-emoji-fallback-boundary.pdf` | `text.font-program` |
| Tight native page pixel budget | `renderer.memory-budget` |

## Unsupported Classification

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family scan \
  --include-family mixed-layout \
  --include-family office-export \
  --include-family form \
  --include-family report \
  --include-family presentation \
  --max-edge 160 \
  --output target/unsupported-0174-classification.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 187 | 176 | 10 | 1 encrypted |

Fallback buckets:

| Bucket | Count |
| --- | ---: |
| `form.xfa-dynamic` | 1 |
| `graphics.color-management` | 1 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |
| `graphics.transparency` | 2 |
| `image.filter` | 3 |
| `text.font-program` | 1 |

## Validation

- `cargo test -p ferrugo-thumbnail unsupported_feature_buckets -- --nocapture`
- `cargo test -p ferrugo-native typed_unsupported_boundary -- --nocapture`
- Unsupported corpus classification with `summarize-fallbacks`.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
