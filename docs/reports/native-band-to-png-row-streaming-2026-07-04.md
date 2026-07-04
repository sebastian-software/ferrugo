# Native band-to-PNG row streaming

Issue: #52

## Summary

This slice completes the gap left by `png-row-streaming-encoder-2026-07-03.md`
for the native CLI render path. `render-native` and native `render-auto` can now
send completed native RGBA rows directly into the PNG row-stream encoder instead
of first materializing a full `Thumbnail` in the CLI.

The new boundary is deliberately narrow:

- `NativeBackend::render_rgba_rows` streams top-to-bottom RGBA rows into a
  caller-owned `NativeRgbaRowSink`;
- serial banded raster replay writes each completed band to the sink and drops
  the band target before rendering the next band;
- the scanned-page direct-image route writes sampled rows directly without
  building a full RGBA thumbnail first;
- the existing `ThumbnailBackend::render` path remains the compatibility and
  byte-parity oracle.

Parallel band-to-row ordering remains out of scope. Parallel profiles still use
the existing deterministic full-thumbnail path.

## Output Formats

PNG benefits directly because the CLI has a row-oriented PNG adapter. Raw RGBA
callers can use `render_rgba_rows` directly if they want to consume rows or
write their own stream. Other future formats, such as JPEG/WebP/AVIF, do not
benefit automatically; they need encoder-specific row or scanline adapters.

The final PNG byte vector is still accumulated before `fs::write`. This PR
removes the full native RGBA output buffer from the native-to-PNG path, but it
does not yet implement seeked/file-streamed IDAT chunk output.

## Memory Evidence

`trace-native fixtures/generated/high-dpi-preview-fidelity.pdf --max-edge 1024`
reports:

| Metric | Bytes |
| --- | ---: |
| Full output buffer | 691,200 |
| Active serial band target | 122,880 |
| Previous banded RGBA peak (`output + active band`) | 814,080 |
| Row-streaming RGBA peak | 122,880 |

That is an explicit RGBA raster-buffer reduction of 849 per mille for the
streamed path on this fixture. The trace JSON now includes
`estimated_peak_streaming_raster_bytes` and
`streaming_raster_byte_reduction_per_mille` next to the existing full-buffer
peak fields.

The generated CLI PNG smoke artifact for the same fixture was:

| Fixture | max edge | PNG bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `high-dpi-preview-fidelity.pdf` | 1024 | 691,678 | `c40a138bc376cc39094f8ca2715e947d85544d473ef094ce48b25bf46465fc8b` |

## Parity

The row stream is covered at both boundaries:

- native row streaming matches full-thumbnail bytes for a banded
  `high-dpi-preview-fidelity.pdf` render;
- native row streaming matches full-thumbnail bytes for the scanned-page
  direct-image route;
- the CLI `PngRowStreamSink` emits byte-identical PNG output to
  `encode_rgba_png` for the same rows.

## Validation

```sh
cargo fmt --check
cargo check -p ferrugo-native --no-default-features
cargo check -p ferrugo --no-default-features
cargo test -p ferrugo-native native_rgba_row_stream_should_match --no-default-features -- --nocapture
cargo test -p ferrugo-native low_memory_trace_should_report_large_scanner_image_bands --no-default-features -- --nocapture
cargo test -p ferrugo png_row_stream_sink_should_match_full_thumbnail_encoder --no-default-features -- --nocapture
cargo run -p ferrugo --no-default-features -- render-native fixtures/generated/high-dpi-preview-fidelity.pdf --max-edge 1024 --output target/issue52-high-dpi-stream.png
cargo run -p ferrugo --no-default-features -- trace-native fixtures/generated/high-dpi-preview-fidelity.pdf --max-edge 1024 --output target/issue52-high-dpi-trace.json
```
