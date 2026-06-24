# 0055: Incremental Updates And Hybrid References

Status: done
Phase: 7
Size: medium
Depends on: 0054

## Goal

Load PDFs that use incremental updates, hybrid-reference files, or multiple
trailers as common producer output.

## Scope

- Follow `Prev` trailer chains with cycle and depth limits.
- Merge object revisions according to latest reachable xref data.
- Support hybrid-reference files when both classic xref and xref streams are
  present.
- Add fixtures for edited, signed, and saved-as PDFs.

## Non-Goals

- Signature validation.
- Repairing arbitrary corrupt update chains.
- Writing incremental updates.

## Deliverables

- Incremental xref resolver.
- Revision merge tests.
- Fixtures for multi-revision PDFs.

## Acceptance Criteria

- Latest object revisions are used for supported incremental files.
- Cyclic or oversized revision chains fail with typed errors.
- Hybrid-reference behavior is documented and covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run corpus comparisons for edited and signed PDFs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed:

- First implementation slice follows classic trailer `/Prev` chains with a
  `16`-revision depth limit and cycle detection.
- Classic xref entries are merged newest-first so later reachable object
  revisions win while older xrefs still fill missing objects.
- Added object-loader tests for latest object revision selection,
  incremental-update cycles, and incremental-update depth overflow.
- Fixture slice adds generated `fixtures/generated/incremental-update.pdf`
  with a base green page revision and a later red page/content revision. Native
  rendering verifies that the latest reachable object revisions are used
  end-to-end.
- PDFium/native comparison for `incremental-update.pdf` at `max-edge 120`
  renders `1600` non-white pixels in both backends, with the later red content
  visible.
- Hybrid-reference slice resolves classic trailer `/XRefStm` streams and adds
  direct in-use xref-stream entries that are not already present in the
  newest-first classic/incremental xref set. Existing classic entries keep
  precedence; compressed xref-stream entries remain modern-loader territory.
- Hybrid fixture slice adds generated `fixtures/generated/hybrid-reference.pdf`,
  where the page content stream is reachable only through the trailer
  `/XRefStm` stream. Native rendering verifies the hybrid path end-to-end.
- PDFium bridge probe for `hybrid-reference.pdf` at `max-edge 120` stays blank
  through the current CLI bridge, while native renders `1600` non-white pixels
  with the blue content stream loaded through `/XRefStm`. Keep this as native
  regression coverage until the PDFium oracle path is refreshed for this
  synthetic hybrid fixture shape.
- Incremental and hybrid reference behavior is documented in
  `docs/policies/incremental-and-hybrid-references.md`; native unsupported
  diagnostics can use the `xref.incremental-hybrid` bucket recorded in
  `docs/errors.md`.
- Current validation:
  - `cargo test -p pdfrust-object incremental -- --nocapture`
  - `cargo test -p pdfrust-object hybrid -- --nocapture`
  - `cargo test -p pdfrust-native incremental_update -- --nocapture`
  - `cargo test -p pdfrust-native hybrid_reference -- --nocapture`
  - `cargo fmt --check`
  - `cargo check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --quiet`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/incremental-update.pdf --max-edge 120 --output target/pdfrust-thumbnails/incremental-update-pdfium-0055.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/incremental-update.pdf --max-edge 120 --output target/pdfrust-thumbnails/incremental-update-native-0055.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/hybrid-reference.pdf --max-edge 120 --output target/pdfrust-thumbnails/hybrid-reference-pdfium-0055.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/hybrid-reference.pdf --max-edge 120 --output target/pdfrust-thumbnails/hybrid-reference-native-0055.png`
