#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p target

package_list="target/native-only-release-pdfrust-cli-package-files.txt"

echo "==> native-only cargo check"
cargo check --workspace --no-default-features

echo "==> native-only cargo test"
cargo test --workspace --no-default-features

echo "==> plugin-free distribution check"
bash scripts/check_plugin_free_distribution.sh

echo "==> PDFium quarantine check"
bash scripts/check_pdfium_quarantine.sh

echo "==> pdfrust-cli package file inspection"
cargo package -p pdfrust-cli --allow-dirty --no-verify --list > "${package_list}"

if rg -n '\.(dylib|so|dll|a|framework)(/|$)|libpdfium|pdfium\.dll|PDFRUST_PDFIUM_LIBRARY' "${package_list}"; then
  echo "PDFium runtime asset or native binary found in pdfrust-cli package file list" >&2
  exit 1
fi

echo "==> leaf package artifact dry-runs"
cargo package -p pdfrust-syntax --allow-dirty --no-verify
cargo package -p pdfrust-thumbnail --allow-dirty --no-verify

if [[ "${PDFRUST_NATIVE_RELEASE_VERIFY_REGISTRY:-0}" == "1" ]]; then
  echo "==> registry-backed workspace package verification"
  cargo package --workspace --allow-dirty
else
  echo "==> skipping registry-backed workspace package verification"
  echo "    set PDFRUST_NATIVE_RELEASE_VERIFY_REGISTRY=1 when crates.io access is available"
fi

echo "==> all-features clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "Native-only release gate passed"
