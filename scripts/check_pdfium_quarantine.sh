#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if cargo tree -p pdfrust-cli --no-default-features | rg -q 'pdfrust-pdfium'; then
  echo "pdfrust-pdfium leaked into the native-only pdfrust-cli dependency tree" >&2
  exit 1
fi

runtime_crates=(
  crates/pdfrust-content
  crates/pdfrust-native
  crates/pdfrust-object
  crates/pdfrust-render
  crates/pdfrust-syntax
  crates/pdfrust-thumbnail
  crates/pdfrust-wasm-smoke
)

if rg -n 'pdfrust_pdfium|PdfiumBackend|PDFRUST_PDFIUM' "${runtime_crates[@]}"; then
  echo "PDFium reference found in runtime crates outside quarantined maintainer tooling" >&2
  exit 1
fi

echo "PDFium quarantine check passed"
