#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if cargo tree -p ferrugo-cli --no-default-features | rg -q 'ferrugo-pdfium'; then
  echo "ferrugo-pdfium leaked into the native-only ferrugo-cli dependency tree" >&2
  exit 1
fi

runtime_crates=(
  crates/ferrugo-content
  crates/ferrugo-native
  crates/ferrugo-object
  crates/ferrugo-render
  crates/ferrugo-syntax
  crates/ferrugo-thumbnail
  crates/ferrugo-wasm-smoke
)

if rg -n 'ferrugo_pdfium|PdfiumBackend|FERRUGO_PDFIUM' "${runtime_crates[@]}"; then
  echo "PDFium reference found in runtime crates outside quarantined maintainer tooling" >&2
  exit 1
fi

echo "PDFium quarantine check passed"
