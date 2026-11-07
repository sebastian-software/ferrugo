---
title: Getting started
---

Ferrugo renders one page of a PDF into a small image, under limits you set for
size, time and memory. When it cannot, it returns a named outcome instead of a
picture. Use it from the command line or from Rust.

Ferrugo is early. The current release previews a useful part of everyday
documents, and many others still come back unsupported. Try it on files that
look like yours before you depend on it.

## Install the command line

You need Rust 1.81 or newer and a normal Cargo toolchain. There is no PDFium
library or other external renderer to install.

```sh
cargo install ferrugo --locked
```

Render the first page of a PDF into a PNG of at most 256 pixels on its longer
side:

```sh
ferrugo render input.pdf --max-edge 256 --output preview.png
```

If the file cannot be previewed, the command fails and names the class:

```text
render error [malformed]: PDF is malformed
```

Useful options:

| Option                 | Default | Meaning                                         |
| ---------------------- | ------: | ----------------------------------------------- |
| `--page-index N`       |     `0` | Zero-based page to render.                      |
| `--max-edge N`         |  `1024` | Largest width or height of the image in pixels. |
| `--timeout SECONDS`    |     `5` | Time limit for one render.                      |
| `--background #RRGGBB` |   white | Background behind transparent pages.            |

## Use the Rust API

Applications depend on the facade and the Rust-native backend:

```sh
cargo add ferrugo-thumbnail ferrugo-native
```

Render one page and branch on the outcome:

```rust
use std::path::Path;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    OutputFormat, PdfSource, ThumbnailBackend, ThumbnailErrorClass, ThumbnailOptions,
};

let backend = NativeBackend::low_memory();
let options = ThumbnailOptions {
    max_edge: 256,
    output_format: OutputFormat::Png,
    ..ThumbnailOptions::default()
};

match backend.render(PdfSource::from_path(Path::new("input.pdf")), &options) {
    Ok(thumbnail) => std::fs::write("preview.png", &thumbnail.bytes)?,
    Err(error) => match error.class() {
        ThumbnailErrorClass::Unsupported => {
            let bucket = error.unsupported_feature_bucket().unwrap_or("native.unsupported");
            eprintln!("no preview: unsupported ({bucket})");
        }
        class => eprintln!("no preview: {class}"),
    },
}
# Ok::<(), std::io::Error>(())
```

`NativeBackend::low_memory()` keeps tight budgets for servers and batch jobs.
`NativeBackend::new()` uses larger budgets: it accepts larger pages and images,
and uses more memory to do so.

## Handle the outcome

Branch on the class, never on the message text. An unsupported outcome can be a
valid PDF that uses a feature Ferrugo does not render yet; the bucket says
which.

| Class         | A typical response                                                   |
| ------------- | -------------------------------------------------------------------- |
| `encrypted`   | Ask for an unlocked file, or skip the preview.                       |
| `malformed`   | Reject the upload, or hold it for review.                            |
| `unsupported` | Store the bucket and show a "preview unavailable" state.             |
| `timeout`     | Retry with a smaller `max_edge`, or mark the preview as over budget. |
| `cancelled`   | Your own code stopped the render; nothing to report.                 |
| `internal`    | Treat it as a defect and keep the diagnostics.                       |

[Errors and outcomes](/reference/errors/) lists every class and bucket.

## Know the limits

- Ferrugo renders one page at a time into an image. It is not a viewer, an
  editor, a signer or a JavaScript or dynamic XFA engine.
- Its budgets cover the work inside parsing and rendering. Process isolation
  and operating-system limits are still yours to set when you accept files
  from strangers.
- The Node.js binding lives in the repository and is not published on npm yet.
- WebAssembly is tested as a compatibility check, not offered as a product.

The [Rust-native backend](/reference/native-backend/) page records what the
renderer supports and where it stops.

## API reference

- [ferrugo-thumbnail on docs.rs](https://docs.rs/ferrugo-thumbnail)
- [ferrugo-native on docs.rs](https://docs.rs/ferrugo-native)
- [The repository on GitHub](https://github.com/sebastian-software/ferrugo)
