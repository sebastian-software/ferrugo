# Plugin-Free Distribution And Install Gate 2026-06-26

Milestone: 0157.

## Decision

The Rust-native renderer is installable and usable without PDFium binaries,
platform plugins, hidden runtime downloads, or a configured PDFium environment.
PDFium remains optional maintainer tooling behind `--features pdfium`.

## Evidence

| Check | Result |
| --- | --- |
| Native-only dependency graph | No `pdfrust-pdfium` edge for `pdfrust-cli --no-default-features`. |
| Network/download dependency graph | No network or TLS download crates in the native-only CLI graph. |
| Hidden source hooks | Runtime sources contain no `curl`, `wget`, fetch, download, or plugin hooks. |
| Checked-in native binaries | No `.dylib`, `.so`, `.dll`, `.a`, or `.framework` files under `crates/`. |
| Workspace package dry-run | All workspace packages verify through Cargo's temporary registry. |
| Clean install smoke | `cargo install --path crates/pdfrust-cli --no-default-features --locked` renders `fixtures/generated/text-page.pdf` without PDFium env vars. |

## Install Contract

Native-only consumers need Cargo, the Rust toolchain, and the Rust standard
library for their target. They do not need PDFium, a system PDF renderer,
platform viewer plugins, dynamic-library search paths, or runtime network
access.

The default CLI install path is:

```text
cargo install --path crates/pdfrust-cli --no-default-features --locked
```

The default library dependency path is:

```text
pdfrust-thumbnail = "0.1.0"
pdfrust-native = "0.1.0"
```

## Validation Commands

```text
bash scripts/check_plugin_free_distribution.sh
bash scripts/check_pdfium_quarantine.sh
cargo package --workspace --allow-dirty
cargo install --path crates/pdfrust-cli --no-default-features --locked --root target/install-0157 --force
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH target/install-0157/bin/pdfrust-cli render fixtures/generated/text-page.pdf --max-edge 96 --output target/install-0157/text-page.png
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.
