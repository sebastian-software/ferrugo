# ferrugo

`ferrugo` is the command-line interface for the Ferrugo Rust-native PDF
preview engine.

It renders bounded PDF thumbnails with the Rust-native backend by default and
keeps PDFium-backed comparison tooling behind the explicit `pdfium` feature for
maintainers.

## Install

```sh
cargo install ferrugo --locked
```

Until all internal crates are published in order, install from a checkout:

```sh
cargo install --path crates/ferrugo-cli --no-default-features --locked
```

## Render A Thumbnail

```sh
ferrugo render input.pdf --max-edge 256 --output thumbnail.png
```

## Maintainer Comparison Builds

PDFium comparison commands are optional and are not part of the default runtime
path:

```sh
cargo install ferrugo --features pdfium --locked
```

See the repository README for current support boundaries and release notes.
