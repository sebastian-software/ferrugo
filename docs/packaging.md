# Packaging

Status: accepted.
Date: 2026-06-24.

`ferrugo` builds and ships as a Rust-native product. The workspace no longer
contains a PDFium binding crate or a `pdfium` Cargo feature. PDFium can appear
only as an external process oracle for maintainer benchmark and visual
comparison workflows.

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

The CLI includes:

- `render` / `render-auto` for Rust-native first rendering.
- `render-native` to force the Rust-native backend.
- `summarize-fallbacks` and `extract-corpus-metadata` for corpus work.
- `benchmark-native` for Rust-native benchmark reports.

Before changing CLI dispatch or package dependencies, run:

```sh
bash scripts/check_pdfium_quarantine.sh
```

This check fails if the workspace regains `crates/ferrugo-pdfium`, a
`ferrugo-pdfium` dependency edge, PDFium binding symbols, PDFium dynamic-library
configuration, or packaged native PDFium assets.

Run the plugin-free distribution check before release packaging or install
workflow changes:

```sh
bash scripts/check_plugin_free_distribution.sh
```

This check confirms that the CLI dependency graph contains neither
`ferrugo-pdfium` nor network/TLS download crates, that runtime sources do not
contain hidden fetch or plugin hooks, and that no native binary artifacts are
checked in under `crates/`.

For release-candidate validation, run the full native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

This local CI-equivalent gate runs native-only check/test, fuzz smoke,
benchmark smoke, native golden image comparison, plugin-free and PDFium
quarantine scans, `ferrugo` package file inspection, leaf package artifact
dry-runs, and all-features clippy. It writes the inspected CLI package file list
to
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

No PDFium dynamic library, PDFium environment variable, or system PDF renderer
is required for the native path. The Rust crates used by the CLI are pure Rust
except for the Rust standard library and normal Cargo build tooling.

## Node-API Package

`crates/ferrugo-node` builds the npm package named `ferrugo`. The Rust crate is
kept in the workspace for normal formatting, Clippy, and test coverage, but it
is marked `publish = false`; npm is the distribution boundary for this binding.

The package exposes the stateless Rust-native backend through Node-API:

- `render(input, options?)` renders one thumbnail on the Node worker pool.
- `inspect(input)` returns the native document metadata subset.
- Errors carry stable `code` values and unsupported-feature errors also carry a
  `bucket` string.

Local validation for the Node package is:

```sh
cd crates/ferrugo-node
npm ci
npm run build:debug
npm test
```

Release prebuilds are produced for:

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

When Release Please creates a GitHub release, the publish workflow builds those
`.node` artifacts, downloads them into `crates/ferrugo-node`, runs
`npm run prepack`, and publishes the npm package with `NPM_TOKEN`.

## Consumer Migration Checklist

- Remove PDFium dynamic-library packaging from normal deployment images.
- Build `ferrugo` normally; there is no `pdfium` feature.
- Use `render` / `render-auto` for normal native-only runtime rendering.
- Use `render-native` when scripts must make the native backend explicit.
- Remove `--allow-pdfium-fallback`; runtime PDFium fallback has been removed.
- Treat `unsupported` feature buckets as native renderer backlog, not as a
  packaging signal to bundle PDFium again.
- Follow `docs/guides/native-only-consumer-migration.md` and
  `docs/policies/unsupported-feature-sla.md` for class/bucket routing.

## External PDFium Oracle

`visual-diff` and `benchmark-matrix --backend pdfium` use an external PDFium
renderer through `--pdfium` or `FERRUGO_PDFIUM_RENDERER`. The renderer is a
local maintainer tool and is not packaged by Ferrugo.

## Workspace Defaults

Root-level `cargo build`, `cargo check`, and `cargo test` focus on the
Rust-native stack.

The dependency graph difference is visible with:

```sh
cargo tree -p ferrugo --no-default-features
```

The graph must have no `ferrugo-pdfium` edge.

## Native-Only Maintenance Gate

The 0120 maintenance gate confirmed that:

- `cargo tree -p ferrugo --no-default-features` has no
  `ferrugo-pdfium` dependency edge.
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
2. `ferrugo-simd`
3. `ferrugo-object`
4. `ferrugo-content`
5. `ferrugo-render`
6. `ferrugo-native`
7. `ferrugo`

Run the local publish-readiness gate before starting the release train:

```sh
bash scripts/check_crate_publish_ready.sh
```

The gate writes package-file lists for every publishable crate to
`target/publish-ready/`, then builds package archives for the two leaf crates
that have no internal registry dependency: `ferrugo-syntax`,
`ferrugo-thumbnail`, and `ferrugo-simd`.

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
cargo package -p ferrugo-simd --allow-dirty --no-verify
```

Once the previous crates in the sequence are visible on crates.io, publish the
next crate from the same checkout:

```sh
cargo publish -p ferrugo-syntax --locked
cargo publish -p ferrugo-thumbnail --locked
cargo publish -p ferrugo-simd --locked
cargo publish -p ferrugo-object --locked
cargo publish -p ferrugo-content --locked
cargo publish -p ferrugo-render --locked
cargo publish -p ferrugo-native --locked
cargo publish -p ferrugo --locked
```

## Release Automation

Ferrugo follows the same Release Please and crates.io Trusted Publishing pattern
used by Ferrocat.

The release train is commit-driven. Do not rewrite Cargo manifests,
`.release-please-manifest.json`, changelogs, or release notes to a placeholder
`0.9`, `1.0`, or other planning version. Use clear Conventional Commits and let
Release Please infer the next SemVer version from the merged change history.
The current `0.1.x` metadata is the prepared crate state, not a statement that
the scoped native release work is complete.

- `.github/workflows/publish.yml` runs on pushes to `main` and on manual
  dispatch.
- The first job runs `googleapis/release-please-action` with
  `.release-please-config.json` and `.release-please-manifest.json`.
- The config uses `cargo-workspace` plus `linked-versions`, so the Ferrugo
  release train stays on one version while Release Please updates local Cargo
  dependency versions.
- `crates/ferrugo-node` is part of the same linked-version group with the
  `node` release type, so Release Please updates the npm `package.json`,
  `package-lock.json`, and the binding crate's Cargo metadata together.
- Changelogs and release notes are generated from the same Conventional Commit
  history. Manual release prose should preserve the scoped native
  server/runtime claim and the explicit non-claim that Ferrugo is not yet a
  broad drop-in PDF renderer replacement.
- When Release Please creates GitHub releases, the `publish-rust` job requests a
  temporary crates.io token through `rust-lang/crates-io-auth-action@v1` and
  runs `scripts/publish_crates.sh`.
- `scripts/publish_crates.sh` publishes crates in dependency order, skips
  versions that already exist on crates.io, and retries dependency-chain crates
  while the registry index catches up.

Repository setup required outside Git:

1. Add a `RELEASE_PLEASE_TOKEN` repository secret if Release Please PRs should
   trigger normal CI workflows. The workflow falls back to `GITHUB_TOKEN` when
   the secret is absent.
2. In crates.io, configure Trusted Publishing for each published crate:
   `ferrugo-syntax`, `ferrugo-thumbnail`, `ferrugo-object`, `ferrugo-content`,
   `ferrugo-simd`, `ferrugo-render`, `ferrugo-native`, and `ferrugo`.
3. Point each trusted publisher at repository `sebastian-software/ferrugo` and
   workflow `.github/workflows/publish.yml`.


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

The product release boundary, blockers, non-blockers, local gates, and current
SemVer decision live in
[`docs/reports/scoped-native-1-0-release-train-2026-07-04.md`](reports/scoped-native-1-0-release-train-2026-07-04.md).
