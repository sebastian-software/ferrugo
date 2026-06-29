# 0017: Run Local PDFium Build

Status: done
Phase: 1
Size: medium
Depends on: 0016

## Goal

Run the pinned PDFium checkout and minimal GN/Ninja build on a machine with
`depot_tools`.

## Scope

- Install or point to `depot_tools`.
- Run the checkout recipe.
- Run GN generation with the Phase 0 args.
- Build the PDFium target needed by `ferrugo-pdfium`.
- Record the exact dynamic-library path.

## Non-Goals

- Optimize binary size.
- Package PDFium for distribution.
- Add Node-API bindings.

## Deliverables

- Updated `docs/measurements/pdfium-build-baseline.md`.
- Confirmed `FERRUGO_PDFIUM_LIBRARY` value.
- Build failure log if the build does not complete.

## Acceptance Criteria

- `gn gen` and `ninja` either succeed or have exact failure output recorded.
- The produced PDFium library path is documented.

## Validation

- Run the backend smoke probe against the built library.

## Completion Notes

Completed on 2026-06-24.

- Installed local toolchain pieces outside the repo:
  `depot_tools`, GN `2425 (d31e02004d86)`, and Ninja `1.13.2`.
- Checked out and synced PDFium revision
  `573758fe2dd928279cd52b5a4bc955a6938aab39`.
- Built the complete static target:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-thumb/obj/libpdfium.a`
  at 264M.
- Built the runtime component target:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`
  at 5.4M plus colocated `@rpath` dylibs.
- Confirmed runtime config:
  `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`
  and
  `DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib`.
- Added and ran `cargo run -p ferrugo-pdfium --example smoke`, which reported
  `initialized=true last_error=0`.
