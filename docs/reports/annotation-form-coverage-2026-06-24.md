# Annotation Form Coverage 2026-06-24

This report records milestone 0074 coverage for static annotation and form
appearance states in the Rust-native thumbnail renderer.

## Implemented Slice

- Added `fixtures/generated/acroform-radio.pdf` for a selected radio widget
  appearance state.
- Added `fixtures/generated/acroform-radio-off.pdf` for an unchecked radio widget
  appearance state.
- Added native-backend pixel tests that verify the selected marker, Off-state
  border, and empty Off-state center render through `/AP /N` state dictionaries
  selected by `/AS`.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p ferrugo-native acroform_radio -- --nocapture
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/form-summary-0074.json
cargo run -p ferrugo-cli -- render-native fixtures/generated/acroform-radio.pdf --max-edge 100 --output target/ferrugo-thumbnails/acroform-radio-native.png
cargo run -p ferrugo-cli -- render-native fixtures/generated/acroform-radio-off.pdf --max-edge 100 --output target/ferrugo-thumbnails/acroform-radio-off-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/acroform-radio.pdf --max-edge 100 --output target/ferrugo-thumbnails/acroform-radio-pdfium.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/acroform-radio-off.pdf --max-edge 100 --output target/ferrugo-thumbnails/acroform-radio-off-pdfium.png
```

All commands completed successfully.

The generated corpus summary reported 50 fixtures total, 48 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `form` family, including text field, checkbox, radio, and
signature-placeholder fixtures, rendered 6 of 6 fixtures natively.

Native and PDFium both rendered the radio fixtures at `100x80`. Local RGBA
comparison reported:

| Fixture | Mean Abs Delta | P95 Delta | Max Delta |
| --- | ---: | ---: | ---: |
| `acroform-radio.pdf` | `2.666` | `0` | `255` |
| `acroform-radio-off.pdf` | `1.805` | `0` | `255` |

The max deltas are localized edge/appearance placement differences; selected
and Off-state visual semantics are present in native output.

## Remaining Limits

- Multiple annotation appearance forms in one synthetic annotation content pass
  can still be clipped by the current form-BBox clip placeholder model. This
  should be fixed with explicit scoped clip save/restore semantics before
  densely annotated forms are treated as high-confidence.
- Dynamic appearances, JavaScript actions, form filling, and editing remain
  unsupported by policy.
- The current fixture set covers static widget appearances, not full
  interaction data preservation beyond metadata needed for future layers.
