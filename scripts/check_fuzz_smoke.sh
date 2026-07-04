#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p target

report="target/fuzz-smoke-summary.txt"

targets=(
  primitive_parse
  xref_load
  stream_decode
  content_tokenize
  render_setup
)

{
  echo "Ferrugo fuzz smoke gate"
  echo "Targets: ${targets[*]}"
  echo
} > "${report}"

for target in "${targets[@]}"; do
  echo "==> fuzz smoke: ${target}"
  cargo run --quiet --manifest-path fuzz/Cargo.toml --bin "${target}" -- --smoke \
    | tee -a "${report}"
done

echo "Fuzz smoke gate passed"
echo "Fuzz smoke gate passed" >> "${report}"
