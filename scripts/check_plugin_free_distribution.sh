#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if cargo tree -p ferrugo --no-default-features | rg -q 'ferrugo-pdfium'; then
  echo "ferrugo-pdfium leaked into the native-only CLI dependency tree" >&2
  exit 1
fi

dependency_pattern='\b(curl|fetch|hyper|isahc|native-tls|openssl|plugin|reqwest|rustls|ureq|wget)\b'

if cargo tree -p ferrugo --no-default-features | rg -q "${dependency_pattern}"; then
  echo "network or TLS dependency found in the native-only CLI dependency tree" >&2
  exit 1
fi

runtime_sources=(
  Cargo.toml
  crates/ferrugo-cli/Cargo.toml
  crates/ferrugo-cli/src
  crates/ferrugo-content/Cargo.toml
  crates/ferrugo-content/src
  crates/ferrugo-native/Cargo.toml
  crates/ferrugo-native/src
  crates/ferrugo-object/Cargo.toml
  crates/ferrugo-object/src
  crates/ferrugo-render/Cargo.toml
  crates/ferrugo-render/src
  crates/ferrugo-syntax/Cargo.toml
  crates/ferrugo-syntax/src
  crates/ferrugo-thumbnail/Cargo.toml
  crates/ferrugo-thumbnail/src
  crates/ferrugo-wasm-smoke/Cargo.toml
  crates/ferrugo-wasm-smoke/src
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
