# Color Management Coverage 2026-06-24

This report records milestone 0075 coverage for the native renderer color
management and OutputIntent policy.

## Implemented Slice

- Added `fixtures/generated/output-intent-rgb.pdf`, a DeviceRGB page with a
  catalog `/OutputIntents` entry and `/DestOutputProfile` stream.
- Added the fixture to `fixtures/corpus-manifest.tsv` and `docs/fixtures.md`.
- Added native-backend coverage proving the OutputIntent fixture renders through
  the Rust-native path.
- Accepted `docs/decisions/0005-color-management-and-output-intent-policy.md`.

## Policy Result

OutputIntent dictionaries are metadata-only for the current thumbnail renderer.
They do not trigger ICC parsing or PDFium fallback when the page content uses a
supported process color space. DeviceGray, DeviceRGB, DeviceCMYK, Indexed
DeviceGray/RGB, CalGray, and CalRGB remain the accepted common thumbnail color
surface. ICCBased, Lab, Separation, DeviceN, spot-color proofing, and print
simulation remain explicit unsupported workflows.

This keeps memory behavior bounded: the native renderer does not retain or
expand ICC profile streams during page rendering, and existing image/page byte
budgets continue to guard decoded samples and raster output.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/color-summary-0075.json
cargo run -p ferrugo-cli -- render-native fixtures/generated/output-intent-rgb.pdf --max-edge 120 --output target/ferrugo-thumbnails/output-intent-rgb-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/output-intent-rgb.pdf --max-edge 120 --output target/ferrugo-thumbnails/output-intent-rgb-pdfium.png
```

All commands completed successfully.

The generated corpus summary reported 51 fixtures total, 49 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `report` family rendered 11 of 11 fixtures natively after
adding the OutputIntent fixture.

Native and PDFium both rendered `output-intent-rgb.pdf` at `120x90`. Local RGBA
comparison reported an exact match:

| Fixture | Mean Abs Delta | P95 Delta | Max Delta |
| --- | ---: | ---: | ---: |
| `output-intent-rgb.pdf` | `0.000` | `0` | `0` |

Sample pixel `(40, 40)` matched in both outputs as `(26, 115, 217, 255)`.

## Remaining Limits

- ICCBased image color spaces remain explicit unsupported errors.
- Lab, Separation, DeviceN, spot colors, and overprint-style print proofing
  remain out of scope for thumbnails.
- OutputIntent profile streams are not parsed or applied to raster colors.
- Future ICC support needs a separate dependency, memory, and corpus decision.
