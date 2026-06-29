# Native-Only Packaging Validation

Date: 2026-06-25
Milestone: 0098

## Scope

This pass prepared Cargo package metadata and consumer migration notes for the
Rust-native renderer stack while preserving explicit PDFium-enabled maintainer
workflows.

## Metadata Changes

- Added workspace package `description` metadata and inherited it in all Rust
  crates.
- Added explicit `version = "0.1.0"` to internal path dependencies so packaged
  manifests have versioned dependency metadata.
- Kept `ferrugo-pdfium` optional behind the `ferrugo-cli/pdfium` feature.
- Kept root workspace `default-members` focused on the native-only stack.

## Dependency Graph Comparison

Command:

```sh
cargo tree -p ferrugo-cli --no-default-features --prefix none
cargo tree -p ferrugo-cli --features pdfium --prefix none
```

Observed locally:

| Build | Dependency lines | Includes `ferrugo-pdfium` |
| --- | ---: | --- |
| Native-only | 24 | no |
| PDFium-enabled | 26 | yes |

## Platform Validation

Host:

```text
host: aarch64-apple-darwin
rustc: 1.95.0-nightly (842bd5be2 2026-01-29)
```

Native-only commands completed:

```sh
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
```

PDFium maintainer smoke completed:

```sh
cargo test -p ferrugo-cli --features pdfium
```

## Package Dry-Run

Leaf package dry-runs completed:

```sh
cargo package -p ferrugo-syntax --allow-dirty --no-verify
cargo package -p ferrugo-thumbnail --allow-dirty --no-verify
```

Results:

| Package | Raw size | Compressed size |
| --- | ---: | ---: |
| `ferrugo-syntax` | 27.1 KiB | 6.2 KiB |
| `ferrugo-thumbnail` | 15.6 KiB | 4.5 KiB |

`ferrugo-cli` package preparation is intentionally blocked until internal
crates are available in release order. With registry access, Cargo reports no
matching published `ferrugo-native` package for the versioned dependency. See
`docs/packaging.md` for the release order.

## Consumer Migration Notes

See `docs/packaging.md` for the native-only install pattern, feature examples,
PDFium-enabled maintainer commands, and migration checklist.
