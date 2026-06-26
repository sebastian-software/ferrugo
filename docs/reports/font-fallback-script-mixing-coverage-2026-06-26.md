# Font Fallback Script Mixing Coverage 2026-06-26

Milestone: 0169

## Summary

Added a focused mixed-script/font-fallback manifest and one new chat-export
emoji boundary fixture. The supported slice covers CJK, pre-positioned RTL,
ligature expansion, combining marks, missing-font fallback, and symbol-heavy
documents. It renders natively without fallback.

Emoji/color-font rendering remains an explicit boundary. The new chat fixture
decodes an emoji through ToUnicode and verifies that native rendering returns a
typed `text.font-program` unsupported bucket instead of silently substituting an
incorrect glyph.

## Fixture Coverage

Added `fixtures/font-fallback-script-mixing-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `cjk` | 3 | Identity-H, Identity-V, and vertical CJK text coverage. |
| `rtl` | 2 | Pre-positioned RTL and Arabic presentation-form coverage. |
| `ligature-combining` | 2 | ToUnicode ligature expansion and combining mark layout coverage. |
| `missing-font` | 3 | Deterministic fallback selection for subset, office, and browser missing-font exports. |
| `symbols` | 2 | Equation/symbol placement and Type3 symbol-like glyph coverage. |
| `emoji-boundary` | 1 | Chat-export emoji boundary returning typed unsupported behavior. |

The new generated `chat-emoji-fallback-boundary.pdf` is also included in the
main corpus manifest with `expected:typed-unsupported`.

## Renderer Coverage

Added:

- `text_display_list_should_classify_emoji_as_unsupported_layout_boundary`
  verifies ToUnicode emoji decoding and layout classification as unsupported
  complex shaping.
- `native_backend_should_report_generated_chat_emoji_boundary_fixture` verifies
  the native backend returns `text.font-program` for the generated boundary PDF.

No color-font renderer was added in this milestone. That is intentional: emoji
glyphs need platform/font-stack decisions and should not be approximated by an
incorrect monochrome fallback.

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/font-fallback-script-mixing-manifest.tsv \
  --include-family cjk \
  --include-family rtl \
  --include-family ligature-combining \
  --include-family missing-font \
  --include-family symbols \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/font-fallback-0169-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 12 | 12 | 0 | 0 |

## Boundary Summary

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/font-fallback-script-mixing-manifest.tsv \
  --include-family cjk \
  --include-family rtl \
  --include-family ligature-combining \
  --include-family missing-font \
  --include-family symbols \
  --include-family emoji-boundary \
  --max-edge 160 \
  --output target/font-fallback-0169-all-summary.json
```

Result:

| Family | Total | Native rendered | Fallbacks | Bucket |
| --- | ---: | ---: | ---: | --- |
| `emoji-boundary` | 1 | 0 | 1 | `text.font-program` |

All other families rendered natively.

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/font-fallback-script-mixing-manifest.tsv \
  --include-family cjk \
  --include-family rtl \
  --include-family ligature-combining \
  --include-family missing-font \
  --include-family symbols \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/font-fallback-0169-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `cjk` | 3 | 3 | 0 | 0 | 0 | 0.452 | 0.520 | 193920 |
| `ligature-combining` | 2 | 2 | 0 | 0 | 0 | 0.392 | 0.399 | 113920 |
| `missing-font` | 3 | 3 | 0 | 0 | 0 | 0.407 | 0.417 | 145920 |
| `rtl` | 2 | 2 | 0 | 0 | 0 | 0.521 | 0.573 | 113920 |
| `symbols` | 2 | 2 | 0 | 0 | 0 | 29.756 | 58.083 | 133760 |

## Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/font-fallback-script-mixing-manifest.tsv \
  --include-family cjk \
  --include-family rtl \
  --include-family ligature-combining \
  --include-family missing-font \
  --include-family symbols \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/font-fallback-0169-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 12 | 0 | 1 | 11 | 0 | 0 |

Blockers are fidelity work, not native support failures:

| Subsystem | Blockers |
| --- | ---: |
| `text-fonts` | 10 |
| `page-geometry` | 1 |

## Validation

- `cargo test -p pdfrust-render emoji -- --nocapture`
- `cargo test -p pdfrust-native emoji -- --nocapture`
- `cargo test -p pdfrust-render glyph_bitmap_cache -- --nocapture`
- `cargo test -p pdfrust-render font_resources_should_bound_fallback_resolution_cache -- --nocapture`
- `cargo test -p pdfrust-render font_resources_should_resolve_missing_embedded_font_deterministically -- --nocapture`
- Native supported gate, boundary summary, benchmark, and visual comparison
  commands listed above.
