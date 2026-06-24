# PDFium Build Measurement Baseline

Status: blocked pending local PDFium toolchain.
Date: 2026-06-24.

This report records the Phase 0 measurement protocol and the current local
state. It should be updated in place after `depot_tools`, the pinned PDFium
checkout, and the GN configuration are available locally.

## Inputs

- PDFium revision: `573758fe2dd928279cd52b5a4bc955a6938aab39`
- Checkout recipe: `docs/build/pdfium-checkout.md`
- GN args: `docs/build/pdfium-gn-args.md`
- Intended output directory: `../pdfium-work/pdfium/out/pdfrust-thumb`

## Local Environment

- OS: macOS 26.5.1, build 25F80
- Architecture: arm64
- CPU: not recorded; sandboxed `sysctl` access was denied
- Memory: not recorded; sandboxed `sysctl` access was denied

## Measurement Commands

Run from `../pdfium-work/pdfium` after the checkout and GN generation steps:

```sh
/usr/bin/time -l ninja -C out/pdfrust-thumb pdfium
du -sh out/pdfrust-thumb
find out/pdfrust-thumb -type f -name '*pdfium*' -maxdepth 2 -print -exec ls -lh {} \;
```

For render measurements, use the Phase 0 CLI once the PDFium backend is built:

```sh
/usr/bin/time -l cargo run -p pdfrust-cli -- \
  render fixtures/generated/text-page.pdf \
  --output target/pdfrust-measurements/text-page.png \
  --max-edge 256
```

Repeat for `max-edge` values `256`, `512`, and `1024`. Record:

- wall-clock time
- user and system CPU time
- maximum resident set size
- output dimensions
- output file size
- backend identity
- error class, if rendering fails

## Current Result

The build was not run in this environment because `gclient`, `gn`, and `ninja`
are not installed. The blocking condition is local toolchain setup, not a known
PDFium configuration failure.

## Plausibility Conclusion

No operational conclusion can be drawn yet. The source revision and GN flags are
now pinned, so the next useful data point is a local `gn gen` plus `ninja`
build. If that build succeeds, Phase 0 can measure binary size and first-page
thumbnail behavior. If it fails, update this report with the exact GN/Ninja
failure before changing the Rust API shape.
