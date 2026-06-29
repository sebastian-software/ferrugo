# Native Renderer GA2 Coverage Gate

Date: 2026-06-26
Milestone: 0140

## Decision

The Rust-native renderer is ready for PDFium-free runtime execution on the
current core supported families: `browser-print`, `office-export`, and `form`.
Those 67 fixtures render natively with zero fallback and zero errors in the
native-only gate.

The broader typical-document corpus is not ready for a visual GA claim or for
deleting PDFium comparison tooling. The full current corpus has strong native
execution coverage, but PDFium visual comparison still reports material
blockers concentrated in text/font fidelity, form appearance parity, rendering
core details, image/color parity, and page geometry.

Recommendation: proceed to the next stabilization/deletion cycle with PDFium
out of normal native-only runtime paths, keep PDFium as maintainer oracle
tooling, and split the remaining blocker work into targeted follow-up
milestones before runtime deletion.

## Native Core Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/ga2-0140-core-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 0 | 0 |
| `office-export` | 44 | 44 | 0 | 0 |
| `form` | 15 | 15 | 0 | 0 |
| **Core total** | **67** | **67** | **0** | **0** |

This is the strongest PDFium-free runtime claim: supported core families no
longer require PDFium fallback for technical execution.

## Typical Corpus Coverage

Full corpus fallback summary:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 0 |
| `browser-print` | 8 | 8 | 0 | 0 |
| `form` | 15 | 15 | 0 | 0 |
| `mixed-layout` | 22 | 20 | 1 | 1 encrypted |
| `office-export` | 44 | 44 | 0 | 0 |
| `presentation` | 9 | 8 | 1 | 0 |
| `report` | 34 | 31 | 3 | 0 |
| `scan` | 22 | 19 | 3 | 0 |
| **Total** | **155** | **146** | **8** | **1 encrypted** |

Typed fallback categories:

| Bucket | Count | Boundary |
| --- | ---: | --- |
| `image.filter` | 3 | CCITT, JBIG2, JPX policy/support boundary |
| `graphics.transparency` | 2 | unsupported blend/soft-mask boundaries |
| `form.xfa-dynamic` | 1 | dynamic XFA without static appearance |
| `graphics.optional-content` | 1 | OCMD membership policy |
| `graphics.pattern-shading` | 1 | unsupported mesh shading boundary |

A broad fail-on-fallback run over all typical families intentionally failed
with these 8 typed fallbacks. That failure is useful evidence: unsupported
categories remain explicit and are not hidden behind ambiguous success states.

## Native Benchmark

Artifact: `target/ga2-0140-benchmark-native.json`

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 0 | 0 | 5.166 | 5.166 |
| `browser-print` | 8 | 8 | 0 | 0 | 0 | 20.902 | 45.867 |
| `form` | 15 | 15 | 0 | 0 | 0 | 10.159 | 44.707 |
| `mixed-layout` | 22 | 20 | 1 | 1 | 2 | 12.350 | 45.441 |
| `office-export` | 44 | 44 | 0 | 0 | 0 | 11.168 | 40.384 |
| `presentation` | 9 | 8 | 1 | 0 | 1 | 12.660 | 25.064 |
| `report` | 34 | 31 | 3 | 0 | 3 | 45.816 | 295.745 |
| `scan` | 22 | 19 | 3 | 0 | 3 | 11.580 | 49.362 |
| **Total** | **155** | **146** | **8** | **1** | **9** | n/a | n/a |

The 9 budget failures correspond to the 8 typed fallback rows plus the encrypted
fixture error, not to native-rendered core-family performance failures.

## PDFium Visual Oracle

Artifact: `target/ga2-0140-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | Both errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `adversarial` | 1 | 1 | 0 | 0 | 0 | 0 |
| `browser-print` | 8 | 2 | 4 | 2 | 0 | 0 |
| `form` | 15 | 0 | 1 | 14 | 0 | 0 |
| `mixed-layout` | 22 | 8 | 6 | 6 | 1 | 1 |
| `office-export` | 44 | 0 | 3 | 41 | 0 | 0 |
| `presentation` | 9 | 3 | 0 | 5 | 1 | 0 |
| `report` | 34 | 8 | 8 | 15 | 3 | 0 |
| `scan` | 22 | 10 | 1 | 8 | 3 | 0 |
| **Total** | **155** | **32** | **23** | **91** | **8** | **1** |

Subsystem blocker clusters:

| Subsystem | Blockers | Native errors | Main implication |
| --- | ---: | ---: | --- |
| `rendering-core` | 34 | 1 | dense tables, dashboards, technical drawings, and some generated layout details need parity work. |
| `text-fonts` | 24 | 0 | visible text/font fidelity is still the largest GA blocker. |
| `annotations-forms` | 13 | 0 | form appearance parity remains a product-visible gap despite native technical rendering. |
| `page-geometry` | 9 | 0 | several rotated, user-unit, and longform geometry cases still drift. |
| `images-color` | 6 | 3 | unsupported codecs and CMYK/scan resampling parity remain split concerns. |
| `vector-graphics` | 3 | 1 | some vector stress and presentation drawing cases still drift. |
| `transparency` | 1 | 2 | transparent alpha edge drift plus typed unsupported soft-mask/blend boundaries remain. |
| `document-structure` | 1 | 0 | structure/xref visual parity has one blocker. |

The visual result is a no-go for broad GA. It is still compatible with a
runtime PDFium-free core execution strategy because the visual oracle is
maintainer evidence, not a normal runtime dependency.

## Packaging And Profile Evidence

- `cargo tree -p ferrugo-cli --no-default-features` has no `ferrugo-pdfium`
  dependency edge.
- `cargo package -p ferrugo-syntax --allow-dirty --no-verify` passed:
  27.1 KiB raw, 6.2 KiB compressed.
- `cargo package -p ferrugo-thumbnail --allow-dirty --no-verify` passed:
  16.9 KiB raw, 4.9 KiB compressed.
- `cargo package -p ferrugo-cli --allow-dirty --no-verify --list` contains only
  CLI package files and Cargo metadata.
- Full `ferrugo-cli` package preparation remains release-order blocked until
  internal crates such as `ferrugo-native` are available from the registry.

WASM and low-memory host checks remain compatibility signals. They passed as
part of the workspace tests, but they do not define the server-side GA2 decision.

## Backlog

Recommended next stabilization/deletion cycle:

1. Keep 0141 scoped to removing PDFium from normal runtime paths, not deleting
   maintainer comparison tooling.
2. Keep 0142 scoped to quarantining PDFium oracle tooling behind explicit
   maintainer commands and documentation.
3. Use 0143 and 0144 to triage the 91 visual blockers by subsystem and operator
   surface before adding broad new features.
4. Prioritize `text-fonts`, `annotations-forms`, `rendering-core`, and
   `images-color` work because those are the highest product-impact clusters.
5. Treat `image.filter`, `graphics.transparency`, `graphics.optional-content`,
   `graphics.pattern-shading`, and `form.xfa-dynamic` as typed unsupported
   boundaries until their follow-up milestones implement them.

## Validation

Commands run:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --include-family scan --include-family report --include-family presentation --include-family mixed-layout --fail-on-fallback --max-edge 160 --output target/ga2-0140-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/ga2-0140-core-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/ga2-0140-full-fallback-summary.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/ga2-0140-benchmark-native.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/ga2-0140-visual-diff.json
cargo tree -p ferrugo-cli --no-default-features
cargo package -p ferrugo-syntax --allow-dirty --no-verify
cargo package -p ferrugo-thumbnail --allow-dirty --no-verify
cargo package -p ferrugo-cli --allow-dirty --no-verify --list
cargo package -p ferrugo-cli --allow-dirty --no-verify
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected non-zero validation:

- The broad typical fail-on-fallback command found 8 typed fallbacks.
- Full `ferrugo-cli` package preparation failed because `ferrugo-native` is not
  available from the crates.io index yet.
