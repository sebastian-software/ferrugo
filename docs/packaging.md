# Packaging

Status: accepted.
Date: 2026-06-24.

`pdfrust-cli` builds native-only by default. The PDFium backend remains in the
workspace for maintainer comparison workflows, but it is not part of the
default CLI dependency graph or normal runtime rendering path.

## Native-Only Build

Use the default feature set for normal Rust-native renderer work:

```sh
cargo build -p pdfrust-cli
cargo test --no-default-features
```

For library consumers that depend on the native backend directly:

```toml
[dependencies]
pdfrust-native = "0.1.0"
pdfrust-thumbnail = "0.1.0"
```

For CLI consumers that install from this workspace or a git revision, keep the
default feature set empty:

```sh
cargo install --path crates/pdfrust-cli --no-default-features
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

Before changing CLI features or package dependencies, run:

```sh
bash scripts/check_pdfium_quarantine.sh
```

This check fails if native-only `pdfrust-cli` regains a `pdfrust-pdfium`
dependency edge or if runtime crates grow forbidden PDFium integration symbols.

Run the plugin-free distribution check before release packaging or install
workflow changes:

```sh
bash scripts/check_plugin_free_distribution.sh
```

This check confirms that the native-only CLI dependency graph contains neither
`pdfrust-pdfium` nor network/TLS download crates, that runtime sources do not
contain hidden fetch or plugin hooks, and that no native binary artifacts are
checked in under `crates/`.

## Plugin-Free Install

The native CLI can be installed from the workspace without PDFium binaries,
platform plugins, or runtime downloads:

```sh
cargo install --path crates/pdfrust-cli --no-default-features --locked
pdfrust-cli render fixtures/generated/text-page.pdf \
  --max-edge 96 \
  --output target/plugin-free-smoke/text-page.png
```

No `PDFRUST_PDFIUM_LIBRARY`, `DYLD_LIBRARY_PATH`, or system PDF renderer is
required for the native-only path. The Rust crates used by the default CLI are
pure Rust except for the Rust standard library and normal Cargo build tooling.

## Consumer Migration Checklist

- Remove `PDFRUST_PDFIUM_LIBRARY` and platform dynamic-library packaging from
  normal deployment images.
- Build `pdfrust-cli` without `--features pdfium` for production native-only
  usage.
- Use `render` / `render-auto` for normal native-only runtime rendering.
- Use `render-native` when scripts must make the native backend explicit.
- Remove `--allow-pdfium-fallback`; runtime PDFium fallback has been removed.
- Treat `unsupported` feature buckets as native renderer backlog, not as a
  packaging signal to bundle PDFium again.

## PDFium-Enabled Build

Enable PDFium explicitly when oracle comparison, direct PDFium probes, or PDFium
benchmark work is needed:

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
- `visual-diff`

It does not add runtime fallback to `render` / `render-auto`.
The internal `render-worker` entry point is private child-process plumbing for
`render-isolated`; direct CLI invocation is rejected.

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

## Native-Only Maintenance Gate

The 0120 maintenance gate confirmed that:

- `cargo tree -p pdfrust-cli --no-default-features` has no
  `pdfrust-pdfium` dependency edge.
- `cargo tree -p pdfrust-cli --features pdfium` adds the optional
  `pdfrust-pdfium` dependency only under the explicit feature.
- `cargo package -p pdfrust-cli --allow-dirty --no-verify --list` contains only
  CLI package files and Cargo metadata.
- `cargo package -p pdfrust-cli --allow-dirty --no-verify` is blocked until
  internal dependencies such as `pdfrust-native` are available from the
  registry; this is a release-order blocker, not a PDFium dependency leak.
- `pdfrust-syntax` and `pdfrust-thumbnail` package dry-runs pass as the first
  release-train leaf crates.

The 0142 quarantine gate adds `scripts/check_pdfium_quarantine.sh` as a
regression check for this boundary.

## Package Release Order

Cargo package validation for `pdfrust-cli` expects versioned internal
dependencies to be available from the registry. Publish or otherwise provide
the crates in dependency order:

1. `pdfrust-syntax` and `pdfrust-thumbnail`
2. `pdfrust-object`
3. `pdfrust-content`
4. `pdfrust-render`
5. `pdfrust-native`
6. `pdfrust-pdfium` when maintainer PDFium workflows are distributed
7. `pdfrust-cli`

Local package dry-runs can validate leaf crates before the full release train:

```sh
cargo package -p pdfrust-syntax --allow-dirty --no-verify
cargo package -p pdfrust-thumbnail --allow-dirty --no-verify
```

The workspace package dry-run validates the full local release train through
Cargo's temporary registry:

```sh
cargo package --workspace --allow-dirty
```
