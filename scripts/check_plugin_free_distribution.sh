#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if cargo tree -p pdfrust-cli --no-default-features | rg -q 'pdfrust-pdfium'; then
  echo "pdfrust-pdfium leaked into the native-only CLI dependency tree" >&2
  exit 1
fi

dependency_pattern='\b(curl|fetch|hyper|isahc|native-tls|openssl|plugin|reqwest|rustls|ureq|wget)\b'

if cargo tree -p pdfrust-cli --no-default-features | rg -q "${dependency_pattern}"; then
  echo "network or TLS dependency found in the native-only CLI dependency tree" >&2
  exit 1
fi

runtime_sources=(
  Cargo.toml
  crates/pdfrust-cli/Cargo.toml
  crates/pdfrust-cli/src
  crates/pdfrust-content/Cargo.toml
  crates/pdfrust-content/src
  crates/pdfrust-native/Cargo.toml
  crates/pdfrust-native/src
  crates/pdfrust-object/Cargo.toml
  crates/pdfrust-object/src
  crates/pdfrust-render/Cargo.toml
  crates/pdfrust-render/src
  crates/pdfrust-syntax/Cargo.toml
  crates/pdfrust-syntax/src
  crates/pdfrust-thumbnail/Cargo.toml
  crates/pdfrust-thumbnail/src
  crates/pdfrust-wasm-smoke/Cargo.toml
  crates/pdfrust-wasm-smoke/src
)

if rg -n "${dependency_pattern}" "${runtime_sources[@]}"; then
  echo "hidden network, download, or plugin hook found in native runtime sources" >&2
  exit 1
fi

if find crates -type f \( -name '*.dylib' -o -name '*.so' -o -name '*.dll' -o -name '*.a' -o -name '*.framework' \) | rg -q .; then
  echo "native binary artifact found under crates/" >&2
  exit 1
fi

echo "Plugin-free distribution check passed"
