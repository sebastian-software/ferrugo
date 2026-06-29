# Renderer Operator Coverage Audit

Date: 2026-06-26
Milestone: 0144

## Summary

Milestone 0144 adds a native operator coverage scan so renderer fidelity work
can start from measured content-stream usage instead of broad visual-diff
clusters alone. The scan uses the native document loading and stream decoding
boundary, tokenizes page and annotation appearance streams, and emits a stable
JSON report grouped by operator, fixture family, status, and typed fallback
bucket.

Artifact:

- `target/operator-coverage-0144.json`

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/corpus-manifest.tsv --page-index 0 --output target/operator-coverage-0144.json
```

## Coverage Result

| Total fixtures | Scanned | Errors | Operators | Inline images |
| ---: | ---: | ---: | ---: | ---: |
| 155 | 154 | 1 | 5,565 | 1 |

The only scan error is `fixtures/generated/encrypted-placeholder.pdf`, where
the native loader returns the expected `encrypted` class. This is a document
security policy boundary, not an operator coverage gap.

Status counts:

| Status | Count | Meaning |
| --- | ---: | --- |
| `implemented` | 5,472 | Native renderer has common-case behavior for the operator. |
| `partial` | 85 | Native renderer has bounded or policy-dependent behavior. |
| `unsupported` | 0 | No currently scanned corpus operator uses a fully unsupported operator. |
| `ignored` | 8 | Non-visual marked-content operators intentionally ignored for thumbnails. |

## Family Matrix

| Family | Total | Scanned | Errors | Operators | Implemented | Partial | Unsupported | Ignored |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 3 | 3 | 0 | 0 | 0 |
| `browser-print` | 8 | 8 | 0 | 57 | 56 | 1 | 0 | 0 |
| `form` | 15 | 15 | 0 | 245 | 236 | 9 | 0 | 0 |
| `mixed-layout` | 22 | 21 | 1 | 327 | 318 | 9 | 0 | 0 |
| `office-export` | 44 | 44 | 0 | 2,611 | 2,584 | 25 | 0 | 2 |
| `presentation` | 9 | 9 | 0 | 189 | 172 | 11 | 0 | 6 |
| `report` | 34 | 34 | 0 | 1,959 | 1,929 | 30 | 0 | 0 |
| `scan` | 22 | 22 | 0 | 174 | 174 | 0 | 0 | 0 |

## Highest Frequency Operators

| Operator | Count | Status | Notes |
| --- | ---: | --- | --- |
| `l` | 582 | `implemented` | Path line segments. |
| `S` | 509 | `implemented` | Stroke painting. |
| `Td` | 502 | `implemented` | Text positioning. |
| `Tj` | 501 | `implemented` | Simple text show. |
| `m` | 474 | `implemented` | Path move. |
| `re` | 472 | `implemented` | Rectangle path construction. |
| `q` / `Q` | 400 each | `implemented` | Graphics state save/restore. |
| `f` | 395 | `implemented` | Nonzero fill. |
| `rg` | 321 | `implemented` | RGB fill color. |
| `Tf` | 178 | `implemented` | Font selection. |
| `BT` / `ET` | 167 each | `implemented` | Text object boundaries. |

These counts explain why 0143 prioritized text/font and dense table fidelity:
the common corpus is dominated by line, text, rectangle, fill, and graphics
state operators rather than exotic PDF operators.

## Partial Operators

| Operator | Count | Bucket | Reason |
| --- | ---: | --- | --- |
| `gs` | 33 | `graphics.transparency` | External graphics state support is bounded to the currently implemented alpha, blend, and overprint subset. |
| `W` | 28 | `graphics.stroke-clip` | Clip paths are represented as bounded placeholders and still need fuller clipping parity. |
| `cs` | 8 | `image.color-space` | Nonstroking color-space changes are implemented for common spaces but still policy-dependent. |
| `scn` | 8 | `image.color-space` | Pattern and spot-color operands are partial by resource support. |
| `sh` | 5 | `graphics.pattern-shading` | Shading support covers current axial/radial/Type4 slices, not all shading semantics. |
| `CS` | 1 | `image.color-space` | Stroking color-space changes are partial. |
| `SCN` | 1 | `image.color-space` | Stroking pattern/spot-color operands are partial. |
| `W*` | 1 | `graphics.stroke-clip` | Even-odd clipping is partial. |

These are the high-impact operator candidates for the next fidelity work. They
line up with the 0143 blocker clusters in rendering-core, images/color,
page-geometry, vector-graphics, and transparency.

## Unsupported Operator Taxonomy

The current generated corpus does not exercise fully unsupported operators, but
the scanner now assigns typed buckets when they appear:

| Operators | Status | Bucket |
| --- | --- | --- |
| `v`, `y`, `b`, `b*` | `unsupported` | `graphics.stroke-clip` |
| `T*`, `TD`, `TL`, `Ts`, `'`, `"` | `unsupported` | `text.font-program` |
| `K`, `k` | `unsupported` | `image.color-space` |
| Unknown operators | `unsupported` | `native.unsupported` |

Marked-content operators `MP`, `DP`, `BMC`, `BDC`, `EMC`, `BX`, and `EX` are
reported as `ignored` because they are non-visual for current thumbnail output,
except where optional-content policy already filters content before rendering.

## Implementation Notes

- Added `ferrugo_native::scan_operator_coverage`.
- Added public report types:
  - `OperatorCoverageOptions`
  - `OperatorSupportStatus`
  - `OperatorCoverageEntry`
  - `OperatorCoverageReport`
- Added `ferrugo-cli operator-coverage`.
- The scanner records inline images as `BI` and counts them separately.
- Annotation appearance and synthesized fallback appearance streams are
  included by default; `--no-annotations` disables that part of the scan.
- The first slice intentionally does not recursively expand Form XObject
  streams. That keeps the audit bounded and avoids turning the scanner into a
  second renderer. Form expansion can be added later if a follow-up milestone
  needs nested operator accounting.

## Follow-Up Candidates

1. Expand clipping parity around `W` / `W*` and the `graphics.stroke-clip`
   bucket.
2. Audit `gs` rows against transparency/overprint visual blockers.
3. Split color-space partial operators (`cs`, `CS`, `scn`, `SCN`) from image
   decode/resampling work.
4. Keep unsupported shorthand curve/text-leading operators as typed audit
   rows until corpus evidence justifies implementation.
5. Consider nested Form XObject operator accounting only after 0145/0146
   corpus refreshes show it would change prioritization.

## Validation

Commands run:

```sh
cargo fmt --check
cargo check -p ferrugo-native
cargo test -p ferrugo-native operator_coverage -- --nocapture
cargo check -p ferrugo-cli --no-default-features
cargo test -p ferrugo-cli operator_coverage -- --nocapture
cargo clippy -p ferrugo-native -p ferrugo-cli --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/corpus-manifest.tsv --page-index 0 --output target/operator-coverage-0144.json
```

All commands passed.
