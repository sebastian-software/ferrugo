# Streaming Parse Coverage 2026-06-24

This report records milestone 0076 coverage for page-targeted stream decoding
and incremental memory behavior in the Rust-native thumbnail renderer.

## Implemented Slice

- Added `fixtures/generated/page-targeted-stream.pdf`, a two-page fixture whose
  first page renders normally while:
  - page 0 contains an unused malformed Image XObject resource;
  - page 1 contains a malformed content stream using an unsupported filter.
- Added native-backend tests proving page 0 renders without decoding the unused
  XObject or page 1 content stream.
- Added native-backend coverage proving page 1 fails deterministically when it
  is explicitly requested.
- Filtered page-level Image/Form XObject resource decoding to names actually
  invoked by `Do` operators in the optional-content-filtered page content.

## Memory And Decode Behavior

The native renderer still loads the classic object table before page rendering;
this milestone does not implement true network streaming or a seekable lazy
object store. The improvement is narrower and measurable: expensive XObject
stream decoding is now page-content-targeted. Unused page XObject resources do
not allocate decoded image/form buffers and do not fail a page render before
their resource name is invoked.

The page-targeted fixture rendered page 0 at `120x80` while ignoring both the
unused malformed Image XObject on page 0 and the malformed page 1 content
stream. Rendering page 1 directly failed with:

```text
render error [malformed]: PDF is malformed
```

The final page-0 CLI render measurement reported:

```text
0.28 real
3325952 maximum resident set size
1491280 peak memory footprint
```

## Validation

```text
cargo test -p pdfrust-native page_streams -- --nocapture
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/streaming-summary-0076.json
/usr/bin/time -l cargo run -p pdfrust-cli -- render-native fixtures/generated/page-targeted-stream.pdf --page-index 0 --max-edge 120 --output target/pdfrust-thumbnails/page-targeted-stream-page0-native.png
cargo run -p pdfrust-cli -- render-native fixtures/generated/page-targeted-stream.pdf --page-index 1 --max-edge 120 --output target/pdfrust-thumbnails/page-targeted-stream-page1-native.png
```

All success-path commands completed successfully. The page-1 render command
failed intentionally with `render error [malformed]: PDF is malformed`.

The generated corpus summary reported 52 fixtures total, 50 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `report` family rendered 12 of 12 fixtures natively after
adding the page-targeted fixture.

## Remaining Limits

- Classic document loading still builds a full object table before page render.
- Content streams for the requested page are still materialized as a `Vec<u8>`
  before display-list construction.
- Font, shading, pattern, and annotation resources can still be tightened in
  later resource-targeting slices.
- True seekable incremental parsing remains a future milestone, not part of this
  page-render decode reduction.
