# Packaging

Status: accepted.
Date: 2026-06-24.

`pdfrust-cli` builds native-only by default. The PDFium backend remains in the
workspace for differential testing and fallback workflows, but it is not part of
the default CLI dependency graph.

## Native-Only Build

Use the default feature set for normal Rust-native renderer work:

```sh
cargo build -p pdfrust-cli
cargo test --no-default-features
```

The native-only CLI includes:

- `render` / `render-auto` for Rust-native first rendering.
- `render-native` to force the Rust-native backend.
- `summarize-fallbacks` and `extract-corpus-metadata` for corpus work that does
  not load PDFium.
- `benchmark-native` for Rust-native benchmark reports.

PDFium-specific commands remain visible but fail with a usage error in
native-only builds. This keeps scripts diagnosable while making accidental
PDFium packaging obvious.

## PDFium-Enabled Build

Enable PDFium explicitly when fallback, oracle comparison, or PDFium benchmark
work is needed:

```sh
cargo build -p pdfrust-cli --features pdfium
cargo test -p pdfrust-cli --features pdfium
```

Then provide the local dynamic library at runtime:

```sh
export PDFRUST_PDFIUM_LIBRARY="/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib"
export DYLD_LIBRARY_PATH="/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib"
```

The PDFium-enabled CLI adds:

- `render-pdfium`
- `render-isolated`
- `compare-metadata`
- `benchmark-pdfium`
- PDFium fallback from `render` / `render-auto` when Rust-native returns
  `unsupported` and the caller passes `--allow-pdfium-fallback`

## Workspace Defaults

The workspace `default-members` exclude `crates/pdfrust-pdfium`, so root-level
`cargo build`, `cargo check`, and `cargo test` focus on the native-only stack.
Run `cargo test -p pdfrust-pdfium` or `cargo clippy --workspace --all-features`
when the PDFium crate itself must be checked.

The dependency graph difference is visible with:

```sh
cargo tree -p pdfrust-cli --no-default-features
cargo tree -p pdfrust-cli --features pdfium
```

The native-only graph has no `pdfrust-pdfium` edge. The PDFium-enabled graph
adds only the optional `pdfrust-pdfium` crate and its shared
`pdfrust-thumbnail` facade dependency.
