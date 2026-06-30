# Packaging

Status: accepted.
Date: 2026-06-24.

`ferrugo` builds native-only by default. The PDFium backend remains in the
workspace for maintainer comparison workflows, but it is not part of the
default CLI dependency graph or normal runtime rendering path.

## Native-Only Build

Use the default feature set for normal Rust-native renderer work:

```sh
cargo build -p ferrugo
cargo test --no-default-features
```

For library consumers that depend on the native backend directly:

```toml
[dependencies]
ferrugo-native = "0.1.0"
ferrugo-thumbnail = "0.1.0"
```

For CLI consumers that install from this workspace or a git revision, keep the
default feature set empty:

```sh
cargo install --path crates/ferrugo-cli --no-default-features
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

This check fails if native-only `ferrugo` regains a `ferrugo-pdfium`
dependency edge or if runtime crates grow forbidden PDFium integration symbols.

Run the plugin-free distribution check before release packaging or install
workflow changes:

```sh
bash scripts/check_plugin_free_distribution.sh
```

This check confirms that the native-only CLI dependency graph contains neither
`ferrugo-pdfium` nor network/TLS download crates, that runtime sources do not
contain hidden fetch or plugin hooks, and that no native binary artifacts are
checked in under `crates/`.

For release-candidate validation, run the full native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

This local CI-equivalent gate runs native-only check/test, plugin-free and
PDFium quarantine scans, `ferrugo` package file inspection, leaf package
artifact dry-runs, and all-features clippy. It writes the inspected CLI package
file list to
`target/native-only-release-ferrugo-package-files.txt`.

## Serverless Profile

Short-lived server rendering can use the explicit Cargo `serverless` profile:

```sh
cargo build --profile serverless -p ferrugo --no-default-features
```

The profile inherits release mode, strips symbols, uses ThinLTO, keeps one code
generation unit, optimizes for size, and uses `panic = "abort"`. It is intended
for native-only thumbnail workers where PDFium is not packaged and process
startup matters.

Measure the profile before changing renderer dependencies, feature flags, or
startup-sensitive code:

```sh
bash scripts/measure_serverless_profile.sh
```

The script builds `target/serverless/ferrugo`, inspects the CLI package file
list for PDFium/native runtime assets, and records binary size, process startup,
and first-render latency in `target/serverless-profile-0197.json`.

Default local budgets:

- binary size: 8 MiB;
- startup p95: 500 ms;
- first-render p95: 250 ms;
- render output: 1 MiB.

## Plugin-Free Install

The native CLI can be installed from the workspace without PDFium binaries,
platform plugins, or runtime downloads:

```sh
cargo install --path crates/ferrugo-cli --no-default-features --locked
ferrugo render fixtures/generated/text-page.pdf \
  --max-edge 96 \
  --output target/plugin-free-smoke/text-page.png
```

No `FERRUGO_PDFIUM_LIBRARY`, `DYLD_LIBRARY_PATH`, or system PDF renderer is
required for the native-only path. The Rust crates used by the default CLI are
pure Rust except for the Rust standard library and normal Cargo build tooling.

## Consumer Migration Checklist

- Remove `FERRUGO_PDFIUM_LIBRARY` and platform dynamic-library packaging from
  normal deployment images.
- Build `ferrugo` without `--features pdfium` for production native-only
  usage.
- Use `render` / `render-auto` for normal native-only runtime rendering.
- Use `render-native` when scripts must make the native backend explicit.
- Remove `--allow-pdfium-fallback`; runtime PDFium fallback has been removed.
- Treat `unsupported` feature buckets as native renderer backlog, not as a
  packaging signal to bundle PDFium again.
- Follow `docs/guides/native-only-consumer-migration.md` and
  `docs/policies/unsupported-feature-sla.md` for class/bucket routing.

## PDFium-Enabled Build

Enable PDFium explicitly when oracle comparison, direct PDFium probes, or PDFium
benchmark work is needed:

```sh
cargo build -p ferrugo --features pdfium
cargo test -p ferrugo --features pdfium
```

Then provide the local dynamic library at runtime:

```sh
export FERRUGO_PDFIUM_LIBRARY="/path/to/pdfium/out/ferrugo-dylib/libpdfium.dylib"
export DYLD_LIBRARY_PATH="/path/to/pdfium/out/ferrugo-dylib"
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

The workspace `default-members` exclude `crates/ferrugo-pdfium`, so root-level
`cargo build`, `cargo check`, and `cargo test` focus on the native-only stack.
Run `cargo test -p ferrugo-pdfium` or `cargo clippy --workspace --all-features`
when the PDFium crate itself must be checked.

The dependency graph difference is visible with:

```sh
cargo tree -p ferrugo --no-default-features
cargo tree -p ferrugo --features pdfium
```

The native-only graph has no `ferrugo-pdfium` edge. The PDFium-enabled graph
adds only the optional `ferrugo-pdfium` crate and its shared
`ferrugo-thumbnail` facade dependency.

## Native-Only Maintenance Gate

The 0120 maintenance gate confirmed that:

- `cargo tree -p ferrugo --no-default-features` has no
  `ferrugo-pdfium` dependency edge.
- `cargo tree -p ferrugo --features pdfium` adds the optional
  `ferrugo-pdfium` dependency only under the explicit feature.
- `cargo package -p ferrugo --allow-dirty --no-verify --list` contains only
  CLI package files and Cargo metadata.
- `cargo package -p ferrugo --allow-dirty --no-verify` is blocked until
  internal dependencies such as `ferrugo-native` are available from the
  registry; this is a release-order blocker, not a PDFium dependency leak.
- `ferrugo-syntax` and `ferrugo-thumbnail` package dry-runs pass as the first
  release-train leaf crates.

The 0142 quarantine gate adds `scripts/check_pdfium_quarantine.sh` as a
regression check for this boundary.

## Package Release Order

Cargo package validation for `ferrugo` expects versioned internal
dependencies to be available from the registry. Publish or otherwise provide
the crates in dependency order:

1. `ferrugo-syntax` and `ferrugo-thumbnail`
2. `ferrugo-object`
3. `ferrugo-content`
4. `ferrugo-render`
5. `ferrugo-native`
6. `ferrugo-pdfium` when maintainer PDFium workflows are distributed
7. `ferrugo`

Run the local publish-readiness gate before starting the release train:

```sh
bash scripts/check_crate_publish_ready.sh
```

The gate writes package-file lists for every publishable crate to
`target/publish-ready/`, then builds package archives for the two leaf crates
that have no internal registry dependency: `ferrugo-syntax` and
`ferrugo-thumbnail`.

Cargo verifies dependency-chain crates such as `ferrugo-object`,
`ferrugo-render`, and `ferrugo` against crates.io. Their full `cargo package`
dry-runs are expected to fail until the lower-level internal crates have already
been published and are visible in the index. After publishing each lower-level
crate, rerun the gate with registry-backed package checks enabled:

```sh
FERRUGO_VERIFY_REGISTRY_PACKAGES=1 bash scripts/check_crate_publish_ready.sh
```

Local package dry-runs can validate leaf crates before the full release train:

```sh
cargo package -p ferrugo-syntax --allow-dirty --no-verify
cargo package -p ferrugo-thumbnail --allow-dirty --no-verify
```

Once the previous crates in the sequence are visible on crates.io, publish the
next crate from the same checkout:

```sh
cargo publish -p ferrugo-syntax --locked
cargo publish -p ferrugo-thumbnail --locked
cargo publish -p ferrugo-object --locked
cargo publish -p ferrugo-content --locked
cargo publish -p ferrugo-render --locked
cargo publish -p ferrugo-native --locked
cargo publish -p ferrugo-pdfium --locked
cargo publish -p ferrugo --locked
```

The workspace package dry-run validates the full local release train through
Cargo's temporary registry when the dependency chain is locally resolvable:

```sh
cargo package --workspace --allow-dirty
```

The release hardening gate wraps native-only build and artifact checks. It keeps
the default path offline-capable. Set `FERRUGO_NATIVE_RELEASE_VERIFY_REGISTRY=1`
to also run the full registry-backed workspace package verification when
crates.io access is available:

```sh
bash scripts/check_native_only_release.sh
```
