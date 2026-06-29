# Minimal PDFium GN Configuration

Status: accepted Phase 0 configuration.
Date: 2026-06-24.

This configuration is for the source tree from
[`docs/build/pdfium-checkout.md`](pdfium-checkout.md). It is intentionally a
local build recipe, not repository automation.

## Static Output Directory

From `../pdfium-work/pdfium`:

```sh
gn gen out/ferrugo-thumb --args="$(cat <<'EOF'
is_debug = false
is_component_build = false
pdf_enable_v8 = false
pdf_enable_xfa = false
pdf_use_skia = false
pdf_use_agg = true
pdf_is_standalone = false
pdf_is_complete_lib = true
clang_use_chrome_plugins = false
use_remoteexec = false
treat_warnings_as_errors = false
EOF
)"
```

## Runtime Dylib Output Directory

The Rust backend loads PDFium at runtime, so the measured local probe also uses
a component build that emits `libpdfium.dylib` plus its colocated `@rpath`
dependencies:

```sh
gn gen out/ferrugo-dylib --args="$(cat <<'EOF'
is_debug = false
is_component_build = true
pdf_enable_v8 = false
pdf_enable_xfa = false
pdf_use_skia = false
pdf_use_agg = true
pdf_is_standalone = false
pdf_is_complete_lib = false
clang_use_chrome_plugins = false
use_remoteexec = false
treat_warnings_as_errors = false
EOF
)"
```

`pdf_is_complete_lib = true` is only valid with
`is_component_build = false`, so the runtime dylib build uses
`pdf_is_complete_lib = false`.

## Flag Rationale

| Flag | Value | Rationale |
| --- | --- | --- |
| `is_debug` | `false` | Measure optimized behavior and size. |
| `is_component_build` | `false` or `true` | Use `false` for the complete static artifact and `true` for the runtime dylib probe. |
| `pdf_enable_v8` | `false` | JavaScript is out of Phase 0 thumbnail scope. |
| `pdf_enable_xfa` | `false` | XFA is out of Phase 0 thumbnail scope. |
| `pdf_use_skia` | `false` | Avoid selecting the Skia render path for this probe. |
| `pdf_use_agg` | `true` | Use the smaller AGG raster path for thumbnails. |
| `pdf_is_standalone` | `false` | Keep the normal PDFium checkout/build defaults unless measurements require changing this. |
| `pdf_is_complete_lib` | `true` or `false` | Use `true` for static size plausibility and `false` for the component dylib required by the runtime probe. |
| `clang_use_chrome_plugins` | `false` | Avoid requiring Chromium-specific clang plugins in the local probe. |
| `use_remoteexec` | `false` | Keep the build local and reproducible outside Google infrastructure. |
| `treat_warnings_as_errors` | `false` | Avoid failing local exploratory builds on toolchain warning drift. |

## Build Command

```sh
ninja -C out/ferrugo-thumb pdfium
ninja -C out/ferrugo-dylib pdfium
```

If the `pdfium` target is unavailable for the pinned revision, inspect targets:

```sh
gn ls out/ferrugo-thumb '*pdfium*'
```

and record the target used in the measurement report.

## Local Validation Notes

In this environment on 2026-06-24, both configurations built successfully for
PDFium revision `573758fe2dd928279cd52b5a4bc955a6938aab39`.

- `out/ferrugo-thumb/obj/libpdfium.a`: 264M
- `out/ferrugo-dylib/libpdfium.dylib`: 5.4M
