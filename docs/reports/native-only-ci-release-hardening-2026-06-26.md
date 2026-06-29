# Native-Only CI And Release Artifact Hardening 2026-06-26

Milestone: 0165

## Summary

Native-only validation is now represented by a single local CI-equivalent gate:

```sh
bash scripts/check_native_only_release.sh
```

The gate does not install, configure, or load PDFium. It validates the
Rust-native release posture with native-only check/test, plugin-free and PDFium
quarantine scans, package artifact inspection, leaf package artifact dry-runs,
optional registry-backed workspace verification, and all-features clippy.

## Gate Steps

| Step | Purpose |
| --- | --- |
| `cargo check --workspace --no-default-features` | Compile the native-only workspace without PDFium features. |
| `cargo test --workspace --no-default-features` | Run native-only tests without PDFium libraries or environment variables. |
| `scripts/check_plugin_free_distribution.sh` | Reject network/download/plugin hooks and native binary artifacts under runtime crates. |
| `scripts/check_pdfium_quarantine.sh` | Reject accidental PDFium dependency edges or runtime-crate symbols. |
| `cargo package -p ferrugo-cli --allow-dirty --no-verify --list` | Record consumer CLI package files for artifact inspection. |
| Package file scan | Reject PDFium runtime assets, native binary archives, and `FERRUGO_PDFIUM_LIBRARY` references in the CLI package file list. |
| `cargo package -p ferrugo-syntax --allow-dirty --no-verify` | Validate leaf package artifact creation without registry/network access. |
| `cargo package -p ferrugo-thumbnail --allow-dirty --no-verify` | Validate the second leaf package artifact without registry/network access. |
| Optional `FERRUGO_NATIVE_RELEASE_VERIFY_REGISTRY=1` | Run `cargo package --workspace --allow-dirty` when registry access is available. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Keep maintainer-only feature paths compiling cleanly without making them default runtime behavior. |

## Artifact Boundaries

The gate writes the CLI package file list to:

```text
target/native-only-release-ferrugo-cli-package-files.txt
```

The file is a local validation artifact, not a committed release artifact. It is
intended to make package-content regressions easy to inspect when a future CI
job fails.

Maintainer comparison tooling remains opt-in:

- PDFium commands stay behind `--features pdfium`.
- Native-only package and runtime checks must not require `FERRUGO_PDFIUM_LIBRARY`
  or `DYLD_LIBRARY_PATH`.
- `ferrugo-pdfium` may still be packaged as a maintainer crate, but it must not
  appear in the default `ferrugo-cli --no-default-features` dependency graph.

## Validation Result

Final command:

```sh
bash scripts/check_native_only_release.sh
```

Result: passed.

Observed step results:

| Step | Result |
| --- | --- |
| Native-only `cargo check` | Passed. |
| Native-only `cargo test` | Passed. |
| Plugin-free distribution check | Passed. |
| PDFium quarantine check | Passed. |
| `ferrugo-cli` package file inspection | Passed; no PDFium runtime asset or native binary entry was found. |
| Leaf package artifact dry-runs | Passed for `ferrugo-syntax` and `ferrugo-thumbnail`. |
| Registry-backed workspace package verification | Skipped by default because no registry access is required for the local CI-equivalent gate. |
| All-features clippy | Passed with `-D warnings`. |

## Registry Verification Note

Initial runs exposed that `cargo package --workspace --allow-dirty`, and even a
workspace package artifact pass for non-leaf internal crates, tries to resolve
crates.io while preparing internal dependencies. That is not suitable for the
native-only no-network CI-equivalent gate. The gate therefore performs package
file inspection for `ferrugo-cli`, leaf package artifact dry-runs for
`ferrugo-syntax` and `ferrugo-thumbnail`, and keeps dependency/runtime
guarantees in the explicit tree, quarantine, plugin-free, and clippy steps.
Registry-backed full workspace package verification is opt-in via
`FERRUGO_NATIVE_RELEASE_VERIFY_REGISTRY=1`.
