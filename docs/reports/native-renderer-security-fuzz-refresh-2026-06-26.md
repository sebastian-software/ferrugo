# Native Renderer Security And Fuzz Refresh

Date: 2026-06-26
Milestone: 0139

## Summary

Milestone 0139 refreshed the native renderer's adversarial-input coverage after
the recent font, image, transparency, and document-family expansion. The focused
code hardening in this slice rejects oversized declared image sample dimensions
before copying or decoding stream data, preserving the stable
`renderer.memory-budget` unsupported bucket for excessive allocation attempts.

No high-confidence exploitable security finding was identified in the reviewed
parser, font, image, and raster budget boundaries. The remaining untrusted-input
stance is still defensive: malformed content may fail, unsupported PDF features
may return typed fallback buckets, and native rendering is intentionally bounded
instead of attempting unbounded repair.

## Hardening Changes

- Added `fixtures/adversarial/huge-image-dimensions.pdf`, a minimized image
  XObject case with a tiny page and a huge declared image sample plane.
- Added the adversarial PDF to the `render_setup` fuzz-smoke seed set.
- Added a native backend regression that verifies the input returns
  `renderer.memory-budget` before allocating declared image samples.
- Added a renderer unit regression for declared image sample budgets independent
  of actual stream length.
- Moved image byte-budget enforcement ahead of XObject and inline-image decode
  length validation, so hostile dimensions are rejected before decode work.

## Budget Boundary Review

| Area | Current boundary | 0139 result |
| --- | --- | --- |
| Primitive parser | Nested arrays/dictionaries capped by parser nesting budget. | Existing adversarial nesting regression remains green. |
| Xref and object loading | Fuzz smoke covers classic and modern xref setup paths. | No new unbounded setup path found. |
| Stream decode | Decode paths use bounded decoded-length options and smoke coverage. | Existing stream smoke remains green. |
| Content tokenizer | Inline-image EOF and content tokenization are smoke-tested. | Unterminated inline-image regression remains green. |
| Font programs | Font bytes, CMaps, glyph outlines, and caches have explicit limits. | No new font budget regression added in this slice. |
| Image XObjects | Raw, Flate, predictor, DCT, soft-mask, ICC, and total image budgets are explicit. | Declared sample length is now checked before decode and length mismatch checks. |
| Raster surfaces | Page pixels, transparency groups, patterns, shadings, and vector segments are budgeted. | No new raster allocation path found in the refreshed corpus. |

## Fuzz Smoke Results

Commands completed without panics:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke
```

Observed smoke case counts:

| Target | Cases |
| --- | ---: |
| `primitive_parse` | 165 |
| `xref_load` | 154 |
| `stream_decode` | 154 |
| `content_tokenize` | 165 |
| `render_setup` | 176 |

`render_setup` grew from the earlier 165-case baseline to 176 cases because
the huge-dimensions image PDF is now part of the deterministic seed set.

## Adversarial Corpus

| Input | Expected behavior |
| --- | --- |
| `truncated-header.pdf` | Native metadata/render setup returns `malformed`. |
| `huge-image-dimensions.pdf` | Native rendering returns `renderer.memory-budget` before sample allocation. |
| `deep-primitive-array.input` | Primitive parsing rejects excessive nesting. |
| `unterminated-inline-image.content` | Content tokenization returns `UnexpectedEof`. |

Targeted corpus checks passed:

```sh
cargo test -p pdfrust-syntax excessive_nesting -- --nocapture
cargo test -p pdfrust-content adversarial_unterminated_inline_image -- --nocapture
cargo test -p pdfrust-native adversarial_truncated_header -- --nocapture
cargo test -p pdfrust-native huge_image_dimensions -- --nocapture
```

## Untrusted-Input Assumptions

- Native rendering treats PDFs as untrusted byte input and favors typed failure
  over best-effort recovery when a path would exceed configured budgets.
- Budget failures intentionally map to public `unsupported` errors with stable
  diagnostic buckets instead of exposing internal allocation details.
- Optional fuzz smoke remains a local hardening tool; release confidence still
  depends on targeted regression tests, corpus gates, and visual comparison.
- This milestone does not claim full coverage for external image/font decoder
  internals; it verifies the native call sites preserve explicit byte and
  output-size limits before invoking them.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p pdfrust-render image_resources_should_enforce_declared_image_byte_budget -- --nocapture
cargo test -p pdfrust-render image_resources_should_enforce_image_byte_budget -- --nocapture
cargo test -p pdfrust-native huge_image_dimensions -- --nocapture
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke
cargo check --workspace
cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke
cargo test -p pdfrust-syntax excessive_nesting -- --nocapture
cargo test -p pdfrust-content adversarial_unterminated_inline_image -- --nocapture
cargo test -p pdfrust-native adversarial_truncated_header -- --nocapture
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
```
