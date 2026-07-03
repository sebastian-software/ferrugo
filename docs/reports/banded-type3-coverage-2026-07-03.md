# Banded Type3 Coverage

Date: 2026-07-03
Issue: #28

## Summary

This slice adds Type3 fixtures to the native banded-replay byte-parity matrix.
It does not promote banding to the default profile and does not complete the
generic-surface rollout. The goal is to cover the previously untested Type3
banded replay path with the same single-target vs. banded byte-parity harness
used for the existing text, image, vector, shading, pattern, and transparency
fixtures.

## Byte-Parity Matrix Addition

The `native_banded_raster_should_match_single_target_output` matrix now includes
these Type3 fixtures:

| Fixture | Coverage role |
| --- | --- |
| `subset-type3-repeated-charprocs.pdf` | repeated Type3 CharProc/template reuse |
| `type3-vector-text.pdf` | simple filled vector glyph CharProcs |
| `type3-symbol-font.pdf` | symbol-like Type3 CharProc glyphs |
| `type3-barcode-font.pdf` | barcode-like Type3 glyph geometry |

Each case renders once with the single-target default profile and once with a
17-row banded profile, then asserts matching dimensions, stride, and exact RGBA
bytes. The same matrix also asserts that the banded render used more than one
band and reduced active raster target bytes relative to the full-page target.

Focused command:

```bash
cargo test -p ferrugo-native native_banded_raster_should_match_single_target_output -- --nocapture
```

Result:

```text
test tests::native_banded_raster_should_match_single_target_output ... ok
```

## Trace Evidence

The repeated Type3 fixture was traced through the low-memory native profile:

```bash
cargo run -p ferrugo --no-default-features -- trace-native fixtures/generated/subset-type3-repeated-charprocs.pdf --native-profile low-memory --max-edge 260 --output target/issue-28-type3-low-memory-trace.json
```

Artifact:

- `target/issue-28-type3-low-memory-trace.json`

Raster band summary:

| Metric | Value |
| --- | ---: |
| Width | 260 |
| Height | 120 |
| Full-page bytes | 124,800 |
| Bands | 2 |
| Workers | 1 |
| Max band rows | 64 |
| Active target peak bytes | 66,560 |
| Active target reduction | 466 permille |
| Estimated peak raster bytes | 191,360 |

Type3 template summary:

| Metric | Value |
| --- | ---: |
| Entries | 2 |
| Bytes | 4,145 |
| Hits | 10 |
| Misses | 2 |
| Inserts | 2 |
| Evictions | 0 |

## Remaining #28 Work

The full #28 acceptance still requires:

- corpus-wide banded coverage reporting;
- support for the remaining generic surfaces that still fall back to
  single-target replay;
- default-profile rollout criteria and default-profile validation;
- process-RSS before/after evidence on genuinely large pages.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-native native_banded_raster_should_match_single_target_output -- --nocapture
cargo test -p ferrugo-native
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
