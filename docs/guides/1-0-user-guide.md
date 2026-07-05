# Ferrugo 1.0 User Guide

Status: release-candidate user guide.
Date: 2026-07-04.

Ferrugo 1.0 is scoped to bounded server-side PDF previews. The normal runtime
path is Rust-native and PDFium-free: render one selected page into a thumbnail,
apply explicit size and timeout budgets, and report stable error classes when a
document is outside the supported boundary.

Use Ferrugo for:

- first-page or selected-page thumbnails in Rust services and automation;
- generated fixtures, office exports, browser print PDFs, invoices, reports,
  scans, forms, presentations, and mixed text-image documents that fit the
  supported native renderer boundary;
- explicit native-only deployments where a large external PDF renderer should
  not be part of the normal runtime.

Do not use Ferrugo 1.0 as:

- a full interactive PDF viewer;
- a PDF editor, signer, JavaScript engine, or dynamic XFA processor;
- a guaranteed pixel-perfect replacement for every PDFium-supported document;
- a runtime that silently falls back to PDFium or Poppler.

## Install Or Build The CLI

For a published release:

```sh
cargo install ferrugo --locked
```

From a repository checkout:

```sh
cargo install --path crates/ferrugo-cli --no-default-features --locked
```

For local development without installing:

```sh
cargo run -p ferrugo --no-default-features -- --version
```

The native-only CLI does not require a PDFium library.

## Render A Thumbnail

Render a generated fixture to PNG:

```sh
cargo run -p ferrugo --no-default-features -- \
  render fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page.png
```

With an installed binary:

```sh
ferrugo render fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page.png
```

`render` and `render-auto` use the Rust-native backend. `render-native` is
available when you want the command name to make that explicit:

```sh
ferrugo render-native fixtures/generated/text-page.pdf \
  --page-index 0 \
  --max-edge 512 \
  --timeout 5 \
  --annotation-mode screen \
  --output target/text-page-native.png
```

Useful options:

| Option | Default | Meaning |
| --- | ---: | --- |
| `--page-index N` | `0` | Zero-based page index to render. |
| `--max-edge N` | `1024` | Maximum thumbnail width or height in pixels. |
| `--timeout SECONDS` | `5` | Per-render timeout. |
| `--background #RRGGBB` | white | Background used for transparent pages. |
| `--annotation-mode screen\|print` | `screen` | Screen preview or print-preview annotation visibility. |
| `--output PATH` | required | PNG output path for CLI renders. |

## Use The Rust API

Applications normally depend on `ferrugo-thumbnail` for the public facade and
`ferrugo-native` for the Rust-native backend:

```toml
[dependencies]
ferrugo-native = "0.1"
ferrugo-thumbnail = "0.1"
```

When testing from a checkout before publishing, use path dependencies that point
at the matching local crates:

```toml
[dependencies]
ferrugo-native = { path = "../ferrugo/crates/ferrugo-native" }
ferrugo-thumbnail = { path = "../ferrugo/crates/ferrugo-thumbnail" }
```

Minimal render example:

```rust
use std::fs;
use std::path::Path;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailErrorClass,
    ThumbnailOptions,
};

fn render_preview(path: &Path) -> Result<Thumbnail, ThumbnailError> {
    let backend = NativeBackend::low_memory();
    let options = ThumbnailOptions {
        max_edge: 256,
        ..ThumbnailOptions::default()
    };

    backend.render(PdfSource::from_path(path), &options)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match render_preview(Path::new("fixtures/generated/text-page.pdf")) {
        Ok(thumbnail) => {
            println!(
                "rendered {}x{} RGBA thumbnail, stride {}",
                thumbnail.width, thumbnail.height, thumbnail.stride
            );
            fs::write("target/text-page.rgba", &thumbnail.bytes)?;
            Ok(())
        }
        Err(error) => {
            match error.class() {
                ThumbnailErrorClass::Encrypted => eprintln!("encrypted PDF"),
                ThumbnailErrorClass::Malformed => eprintln!("malformed PDF"),
                ThumbnailErrorClass::Unsupported => {
                    let bucket = error
                        .unsupported_feature_bucket()
                        .unwrap_or("native.unsupported");
                    eprintln!("unsupported PDF feature bucket: {bucket}");
                }
                ThumbnailErrorClass::Timeout => eprintln!("render timed out"),
                ThumbnailErrorClass::Cancelled => eprintln!("render cancelled"),
                ThumbnailErrorClass::Internal => eprintln!("internal render failure"),
            }
            Err(Box::new(error))
        }
    }
}
```

The library returns raw RGBA bytes when `output_format` is `Rgba`, or PNG bytes
when `output_format` is `Png`. `Thumbnail::pixel_format` and
`Thumbnail::output_format` describe the payload, so PNG thumbnails have
`pixel_format = Png`, `output_format = Png`, and no fixed row stride.

Use `NativeBackend::low_memory()` or
`NativeBackend::low_memory_parallel_with_workers(max_workers)` for server and
batch-thumbnail workloads that need tighter memory and concurrency budgets. Use
`NativeBackend::new()` for the default desktop-oriented render budget profile.
Single-page native renders enforce `ThumbnailOptions::timeout`; callers that
need to abort work explicitly can use `render_with_cancellation` or session
`render_page_with_cancellation` with `RenderCancellation`.

## Handle Unsupported Documents

Unsupported documents are part of the public contract, not exceptional string
messages to scrape. Always branch on `ThumbnailError::class()` first, then use
`unsupported_feature_bucket()` when the class is `Unsupported`.

Stable unsupported buckets include:

- `renderer.memory-budget`
- `renderer.form-xobject-composition`
- `graphics.optional-content`
- `graphics.color-management`
- `graphics.pattern-shading`
- `graphics.stroke-clip`
- `graphics.transparency`
- `annotation.appearance`
- `image.color-space`
- `image.filter`
- `form.xfa-dynamic`
- `form.appearance-mutation`
- `text.cmap-tounicode`
- `text.font-program`
- `text.glyph-outline`
- `native.unsupported`

Recommended service behavior:

| Error class | Typical response |
| --- | --- |
| `encrypted` | Ask for an unlocked PDF or skip preview generation; permissions-only empty-user-password PDFs open natively. |
| `malformed` | Reject the upload or quarantine it for manual review. |
| `unsupported` | Store the bucket and show a generic preview-unavailable state. |
| `timeout` | Retry with a smaller `max_edge` or mark the preview as budget-exceeded. |
| `internal` | Treat as a defect signal and capture diagnostics. |

Do not parse `Display` text for routing. The stable API surface is the error
class plus the optional unsupported bucket.

## Troubleshooting

If a render fails with `missing input PDF` or `missing --output path`, check the
CLI argument order and include exactly one input PDF.

If `--max-edge` is rejected, make sure it is greater than zero. For server
previews, start with `256` or `512` and increase only when the downstream UI
needs larger images.

If a document is unsupported, keep the bucket in logs or telemetry and avoid
claiming the preview failed because of PDF corruption. Unsupported means the
input can be valid PDF while still outside the current native renderer boundary.

If a render times out, lower `--max-edge`, keep one selected page per request,
or use a lower-memory backend profile in Rust. Ferrugo 1.0 is designed for
bounded previews, not unbounded full-document rasterization.

If a PNG is needed from Rust, encode the returned RGBA buffer in the
application layer. The facade intentionally keeps the runtime API backend-neutral
and byte-oriented.

## Maintainer Workflows

Reference renderers are maintainer tools, not runtime dependencies. Use them to
compare behavior, refresh baselines, and investigate fidelity gaps.

Read:

- [Renderer benchmarks](../benchmarks.md)
- [Native renderer conformance backlog](../backlogs/native-renderer-conformance-backlog.md)
- [Renderer performance optimization backlog](../backlogs/renderer-performance-optimization-backlog.md)
- [Reference oracle strategy](../policies/reference-oracle-strategy.md)
- [PDFium checkout recipe](../build/pdfium-checkout.md)

Build explicit PDFium comparison tooling only when doing maintainer work:

```sh
cargo build -p ferrugo --features pdfium
cargo test -p ferrugo --features pdfium
```

Then point the CLI at a local PDFium library as described in the PDFium checkout
recipe. Keep these commands out of the native-only runtime path.
