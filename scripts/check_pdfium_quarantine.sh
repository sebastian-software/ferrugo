#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -d crates/ferrugo-pdfium ]]; then
  echo "crates/ferrugo-pdfium must not exist in the workspace" >&2
  exit 1
fi

if cargo tree --workspace --all-features | rg -q 'ferrugo-pdfium'; then
  echo "ferrugo-pdfium leaked into the workspace dependency tree" >&2
  exit 1
fi

binding_sources=(
  Cargo.toml
  Cargo.lock
  crates/*/Cargo.toml
  crates/*/src
)

if rg -n 'ferrugo-pdfium|ferrugo_pdfium|PdfiumBackend|FERRUGO_PDFIUM_LIBRARY|libpdfium|pdfium\.dll' "${binding_sources[@]}"; then
  echo "PDFium binding reference found in workspace source or manifests" >&2
  exit 1
fi

package_list="target/pdfium-quarantine-package-files.txt"
cargo package -p ferrugo --allow-dirty --no-verify --list > "${package_list}"
if rg -n '\.(dylib|so|dll|a|framework)(/|$)|libpdfium|pdfium\.dll|FERRUGO_PDFIUM_LIBRARY' "${package_list}"; then
  echo "PDFium runtime asset or native binary found in ferrugo package file list" >&2
  exit 1
fi

echo "PDFium quarantine check passed"
