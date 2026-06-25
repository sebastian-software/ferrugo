# Font Fallback Policy

Date: 2026-06-25
Milestone: 0101

## Scope

This pass made missing and substituted font handling deterministic in the
Rust-native renderer without adding operating-system font resolution or runtime
font downloads.

## Implementation

- Added explicit built-in fallback faces: `Sans`, `Serif`, `Monospace`, and
  `Symbol`.
- Added fallback source classification for embedded-program text, PDF standard
  base fonts, missing embedded font programs, and unspecified font metadata.
- Added subset-prefix stripping before fallback classification, so names like
  `ABCDEE+InvoiceSerif` resolve the same way across platforms.
- Added a bounded fallback resolution cache with a default limit of 128 entries.
  Cache keys store only classified fallback metadata, not document resource
  names or raw font names.
- Added the fallback face to the glyph bitmap cache key, preparing the current
  built-in rasterizer for distinct faces without cross-face cache aliasing.
- Exposed `max_font_fallback_cache_entries` through native memory diagnostics
  and CLI comparison JSON.

## Deterministic Policy

| Input family signal | Built-in face | Source |
| --- | --- | --- |
| Helvetica, Arial, unknown, or no base font | `Sans` | standard, missing, or unspecified |
| Times, Georgia, or `serif` names | `Serif` | standard or missing |
| Courier, Consolas, Monaco, or `mono` names | `Monospace` | standard or missing |
| Symbol or ZapfDingbats names | `Symbol` | standard or missing |
| Type 3 fonts | none | rendered through CharProc paths |

The policy is intentionally platform-independent. It does not ask the host
system for installed fonts and does not download replacement fonts.

## Fixture Coverage

Added generated missing-font fixtures:

| Fixture | Family | Expected behavior |
| --- | --- | --- |
| `fixtures/generated/missing-font-browser-print.pdf` | `browser-print` | Missing TrueType program renders via deterministic `Monospace` fallback. |
| `fixtures/generated/missing-font-invoice.pdf` | `form` | Missing TrueType program renders via deterministic `Sans` fallback. |
| `fixtures/generated/missing-font-office-export.pdf` | `office-export` | Missing TrueType program renders via deterministic `Serif` fallback. |

Native smoke tests assert that all three render successfully and produce visible
non-white output.

## Corpus Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/font-fallback-0101-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 6 | 6 | 0 | 0 |
| `office-export` | 15 | 15 | 0 | 0 |
| `form` | 12 | 12 | 0 | 0 |
| **Supported gate total** | **33** | **33** | **0** | **0** |

## PDFium Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/font-fallback-0101-visual-diff.json
```

Full-corpus summary after adding the fixtures:

| Metric | Count |
| --- | ---: |
| Total fixtures | 78 |
| Exact | 26 |
| Accepted drift | 13 |
| Blockers | 33 |
| Native errors | 5 |
| PDFium errors | 0 |
| Both errors | 1 |

Missing-font fixture comparison:

| Fixture | Status | Mean absolute error | Changed ratio | P95 delta |
| --- | --- | ---: | ---: | ---: |
| `missing-font-browser-print.pdf` | blocker | 23.784 | 0.117576 | 255 |
| `missing-font-invoice.pdf` | blocker | 24.334 | 0.123718 | 255 |
| `missing-font-office-export.pdf` | blocker | 22.200 | 0.110606 | 255 |

The deterministic fallback policy satisfies stable native behavior, but the
current built-in bitmap text rasterizer does not match PDFium closely enough to
accept visual drift for missing-font text. That fidelity work remains in the
post-0101 text/font backlog.

## Follow-Ups

- Use 0102 and 0103 to improve embedded font outline and OpenType layout
  fidelity before reopening missing-font visual acceptance.
- Keep the fallback resolver platform-independent; any future system-font
  adapter should feed into the same deterministic policy and bounded metrics.
- Add visual acceptance thresholds per font class only after real font raster
  output is available.

## Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/font-fallback-0101-supported-gate.json`
- `cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --output target/font-fallback-0101-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
