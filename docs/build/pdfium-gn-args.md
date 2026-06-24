# Minimal PDFium GN Configuration

Status: accepted Phase 0 configuration.
Date: 2026-06-24.

This configuration is for the source tree from
[`docs/build/pdfium-checkout.md`](pdfium-checkout.md). It is intentionally a
local build recipe, not repository automation.

## Output Directory

From `../pdfium-work/pdfium`:

```sh
gn gen out/pdfrust-thumb --args="$(cat <<'EOF'
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

## Flag Rationale

| Flag | Value | Rationale |
| --- | --- | --- |
| `is_debug` | `false` | Measure optimized behavior and size. |
| `is_component_build` | `false` | Prefer a single static-style build output. |
| `pdf_enable_v8` | `false` | JavaScript is out of Phase 0 thumbnail scope. |
| `pdf_enable_xfa` | `false` | XFA is out of Phase 0 thumbnail scope. |
| `pdf_use_skia` | `false` | Avoid selecting the Skia render path for this probe. |
| `pdf_use_agg` | `true` | Use the smaller AGG raster path for thumbnails. |
| `pdf_is_standalone` | `false` | Keep the normal PDFium checkout/build defaults unless measurements require changing this. |
| `pdf_is_complete_lib` | `true` | Prefer a complete library artifact for local FFI linkage. |
| `clang_use_chrome_plugins` | `false` | Avoid requiring Chromium-specific clang plugins in the local probe. |
| `use_remoteexec` | `false` | Keep the build local and reproducible outside Google infrastructure. |
| `treat_warnings_as_errors` | `false` | Avoid failing local exploratory builds on toolchain warning drift. |

## Build Command

```sh
ninja -C out/pdfrust-thumb pdfium
```

If the `pdfium` target is unavailable for the pinned revision, inspect targets:

```sh
gn ls out/pdfrust-thumb '*pdfium*'
```

and record the target used in the measurement report.

## Local Validation Notes

In this environment on 2026-06-24, `gn` and `ninja` were not installed because
`depot_tools` is not present. The args above match the Phase 0 decision
baseline and are ready to validate after the PDFium checkout recipe has been
run on a machine with `depot_tools`.
