# 0052: Annotation Appearance Rendering

Status: in-progress
Phase: 7
Size: medium
Depends on: 0051

## Goal

Render annotation appearances that are visible in common reviewed, signed, or
commented PDFs.

## Scope

- Resolve annotation dictionaries from page objects.
- Render normal appearance streams for supported annotation types.
- Support link, text markup, stamp, and widget appearance handling as driven by
  fixtures.
- Define fallback behavior for annotations without usable appearances.

## Non-Goals

- Interactive annotation editing.
- JavaScript actions.
- Full PDF form behavior.

## Deliverables

- Annotation appearance discovery and render path.
- Fixtures for visible annotation appearances.
- Unsupported annotation diagnostics.

## Acceptance Criteria

- PDFs with appearance streams show visible annotations in thumbnails.
- Missing or unsupported appearances do not abort otherwise renderable pages.
- Action dictionaries are ignored or reported without execution.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for annotation fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice resolves page `/Annots` entries, extracts normal
  `/AP /N` Form XObject appearance streams, and renders their path content after
  the base page content. Missing annotation dictionaries, missing `/AP`, missing
  `/N`, and missing `/Rect` are skipped without aborting page rendering.
- The PDFium bridge now renders with `FPDF_ANNOT` so differential annotation
  fixtures can use PDFium as an oracle.
- First fixture slice adds generated
  `fixtures/generated/annotation-appearance.pdf` through
  `scripts/generate_fixtures.py`, covering a `/Subtype /Stamp` annotation with
  a normal appearance stream.
- Second implementation slice maps each appearance Form XObject `/BBox` onto
  the annotation `/Rect` before invocation, so appearances with smaller local
  coordinate systems scale to their page annotation bounds.
- Third implementation slice resolves normal appearance state dictionaries by
  selecting the `/AP /N` entry named by `/AS`; annotations without `/AS` fall
  back to the first referenced normal appearance.
- Fourth fallback slice adds generated
  `fixtures/generated/annotation-missing-appearance.pdf` and verifies that an
  annotation without usable `/AP` does not abort otherwise renderable page
  content.
- Fifth fixture slice adds generated
  `fixtures/generated/link-annotation-appearance.pdf`, covering a `/Subtype
  /Link` annotation with a normal appearance stream and inert border/action
  handling.
- PDFium/native comparison for `link-annotation-appearance.pdf` at
  `max-edge 120`: `120x120`, changed RGB pixels `14`, RGB MAE `0.1065`, p95
  RGB delta `0`, max channel delta `255`, native non-white pixels `322`. Border
  and interior sample pixels match PDFium exactly; differences are confined to
  antialiasing along the stroked rectangle.
- PDFium/native comparison for `annotation-appearance.pdf` at `max-edge 120`:
  `120x120`, changed RGB pixels `0`, RGB MAE `0.0000`, p95 RGB delta `0`,
  max channel delta `0`, native non-white pixels `800`. Filled and outside
  sample pixels match PDFium exactly.
- Current validation:
  - `cargo test -p pdfrust-native annotation_appearance -- --nocapture`
  - `cargo test -p pdfrust-native annotation_without_appearance -- --nocapture`
  - `cargo test -p pdfrust-native link_annotation_appearance -- --nocapture`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/annotation-appearance.pdf --max-edge 120 --output target/pdfrust-thumbnails/annotation-appearance-pdfium-0052.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/annotation-appearance.pdf --max-edge 120 --output target/pdfrust-thumbnails/annotation-appearance-native-0052.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/link-annotation-appearance.pdf --max-edge 120 --output target/pdfrust-thumbnails/link-annotation-appearance-pdfium-0052.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/link-annotation-appearance.pdf --max-edge 120 --output target/pdfrust-thumbnails/link-annotation-appearance-native-0052.png`
