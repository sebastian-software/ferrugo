#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

targets=(
  primitive_parse
  xref_load
  stream_decode
  content_tokenize
  render_setup
)

for target in "${targets[@]}"; do
  echo "==> fuzz smoke: ${target}"
  cargo run --manifest-path fuzz/Cargo.toml --bin "${target}" -- --smoke
done

echo "Fuzz smoke gate passed"
