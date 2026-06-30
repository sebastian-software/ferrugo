# PDFium Build Measurement Baseline

Status: measured locally.
Date: 2026-06-24.

This report records the Phase 0 PDFium source-build and thumbnail-render
baseline for the pinned checkout.

## Inputs

- PDFium revision: `573758fe2dd928279cd52b5a4bc955a6938aab39`
- Checkout recipe: `docs/build/pdfium-checkout.md`
- GN args: `docs/build/pdfium-gn-args.md`
- Static output directory:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-thumb`
- Runtime dylib output directory:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib`
- Runtime library:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`

## Local Environment

- OS: macOS 26.5.1, build 25F80
- Architecture: arm64
- CPU: Apple M1 Ultra, 20 logical CPUs
- Memory: 64 GiB
- GN: `2425 (d31e02004d86)`
- Ninja: `1.13.2`

## Build Commands

Run from `/private/tmp/ferrugo-tools/pdfium-work/pdfium` after the checkout and
`gclient sync --nohooks && gclient runhooks`.

```sh
gn gen out/ferrugo-thumb --args='is_debug = false is_component_build = false pdf_enable_v8 = false pdf_enable_xfa = false pdf_use_skia = false pdf_use_agg = true pdf_is_standalone = false pdf_is_complete_lib = true clang_use_chrome_plugins = false use_remoteexec = false treat_warnings_as_errors = false'
ninja -C out/ferrugo-thumb pdfium

gn gen out/ferrugo-dylib --args='is_debug = false is_component_build = true pdf_enable_v8 = false pdf_enable_xfa = false pdf_use_skia = false pdf_use_agg = true pdf_is_standalone = false pdf_is_complete_lib = false clang_use_chrome_plugins = false use_remoteexec = false treat_warnings_as_errors = false'
ninja -C out/ferrugo-dylib pdfium
```

The static complete-library build succeeded and produced:

- `out/ferrugo-thumb/obj/libpdfium.a`: 264M

The runtime component build succeeded and produced:

- `out/ferrugo-dylib/libpdfium.dylib`: 5.4M
- Additional `@rpath` dylib dependencies in the same output directory:
  Abseil, ICU, partition allocator, `chrome_zlib`, HarfBuzz, and
  `libc++_chrome`.

The complete static-library configuration is useful for size plausibility. The
runtime component build is the one used by the Rust backend because Phase 0
loads PDFium through `dlopen`.

## Runtime Configuration

```sh
export FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib
export DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib
```

Smoke probe:

```sh
cargo run -p ferrugo-pdfium --example smoke
```

Result:

```text
initialized=true last_error=0 library=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib
```

## Render Measurements

The release CLI was used for render measurements:

```sh
/usr/bin/time -l target/release/ferrugo \
  render fixtures/generated/text-page.pdf \
  --output target/ferrugo-thumbnails/text-page-256.png \
  --max-edge 256
```

The same command shape was repeated for `max-edge` values `512` and `1024`.

| max edge | output | dimensions | PNG size | PNG SHA-256 | decoded RGBA SHA-256 | time | max RSS |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 256 | `target/ferrugo-thumbnails/text-page-256.png` | 256x137 | 137K | `1711931704d73467a89f35f4ff523dabecd3b1bf4f4716924e350c4dfc957593` | `2cd4dbfeb05110c7c67e5ae7bf6f2f6c0a9cc240bf69aa5f0acd902426ff31b8` | 0.04s real, 0.01s user, 0.02s sys | 24,313,856 bytes |
| 512 | `target/ferrugo-thumbnails/text-page-512.png` | 300x160 | 188K | `f4a6974feabbd0beb894ca40733498da3a623518bc5f26557df2cf83cee923e0` | `2d01856844de307026a2f727d900314810b7df746463e991909ec148900a8897` | 0.03s real, 0.01s user, 0.02s sys | 24,674,304 bytes |
| 1024 | `target/ferrugo-thumbnails/text-page-1024.png` | 300x160 | 188K | `f4a6974feabbd0beb894ca40733498da3a623518bc5f26557df2cf83cee923e0` | `2d01856844de307026a2f727d900314810b7df746463e991909ec148900a8897` | 0.03s real, 0.01s user, 0.02s sys | 24,625,152 bytes |

The fixture page renders natively to 300x160 pixels. Therefore `max-edge` 512
and 1024 do not upscale and produce identical output.

## Plausibility Conclusion

The pinned PDFium revision builds locally with V8, XFA, and Skia disabled, and
the Phase 0 Rust backend can load the component dylib, initialize PDFium, render
the generated text fixture, and write valid PNG thumbnails. The first useful
runtime baseline is small for the generated fixture: roughly 24 MiB max RSS and
0.03-0.04s wall time after using the release CLI.

The next decision is timeout and isolation policy. The live run proves the
backend path works, but hostile PDFs still need process or worker isolation
before the API should promise robust cancellation.
