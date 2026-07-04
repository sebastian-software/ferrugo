# PNG row-streaming encoder slice

Issue: #52

## Summary

This PR is a narrow encoder-memory slice toward full band-to-PNG streaming.
The CLI PNG encoder no longer builds a full filtered-row buffer before zlib
storage. It now feeds each PNG filter byte and RGBA row into a small streaming
zlib-store writer that:

- keeps at most one pending uncompressed deflate block (`65,535` bytes);
- updates Adler-32 incrementally;
- preserves the existing zlib-store block layout, including final-block
  handling when the filtered stream length lands exactly on a block boundary.

This does **not** complete the full #52 acceptance criteria. The native backend
still renders into a full `Thumbnail` before CLI encoding, so the next slice
must move the same row writer behind a serial native band callback.

## Parity

The new streaming zlib-store path is byte-identical to the old full-filtered
reference path in unit coverage, including a multi-block image.

Branch PNG output is byte-identical to `main` for:

| Fixture | max edge | SHA-256 |
| --- | ---: | --- |
| `scanner-large-image-budget.pdf` | 440 | `f584263302476f4d92cefe538d47b5620b0032c9d6e55b58555828a4a1332c60` |
| `text-page.pdf` | 160 | `6479d74dbfc7c294c8ab15a3e31247a1a4c09e5bff138cdb45a501916fc56a08` |

## Memory effect

Before this slice, PNG encoding held both:

- the rendered RGBA thumbnail buffer; and
- a full filtered-row buffer of `height * (width * 4 + 1)` bytes before zlib
  storage.

After this slice, the full filtered-row buffer is replaced by a single pending
zlib-store block. For the `scanner-large-image-budget.pdf` control at
`max-edge 440`, the avoided filtered-row allocation is approximately
`440 * (320 * 4 + 1) = 563,640` bytes.

The final PNG byte vector is still accumulated in memory, and the native RGBA
thumbnail buffer is still present. Full #52 closure should stream serial raster
bands directly into this encoder and then write IDAT bytes to the output file
instead of collecting the final PNG vector.

## Validation

```sh
cargo fmt --check
cargo test -p ferrugo encode_rgba_png_should_write_png_signature --no-default-features -- --nocapture
cargo test -p ferrugo streaming_png_zlib_store_should_match_full_filtered_reference --no-default-features -- --nocapture
cargo clippy -p ferrugo --all-targets --all-features -- -D warnings
cargo run --release -p ferrugo --no-default-features -- render-native fixtures/generated/scanner-large-image-budget.pdf --max-edge 440 --output target/issue52-scanner-branch.png
cargo run --release -p ferrugo --no-default-features -- render-native fixtures/generated/text-page.pdf --max-edge 160 --output target/issue52-text-branch.png
```
