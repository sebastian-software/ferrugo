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
  render
)

# The fuzz crate is its own workspace with its own lockfile. Release version
# bumps of the path dependencies (release-please) do not touch fuzz/Cargo.lock,
# so re-pin the workspace members before the --locked runs below. This only
# rewrites workspace-member versions; third-party pins stay locked. Offline
# mode is not used because fresh CI runners have no registry index cache.
cargo update --workspace --manifest-path fuzz/Cargo.toml --quiet

{
  echo "Ferrugo fuzz smoke gate"
  echo "Targets: ${targets[*]}"
  echo
} > "${report}"

for target in "${targets[@]}"; do
  echo "==> fuzz smoke: ${target}"
  cargo run --quiet --locked --manifest-path fuzz/Cargo.toml --bin "${target}" -- --smoke \
    | tee -a "${report}"
done

echo "Fuzz smoke gate passed"
echo "Fuzz smoke gate passed" >> "${report}"
