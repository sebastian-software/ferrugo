#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

output_dir="${FERRUGO_PUBLISH_READY_DIR:-target/publish-ready}"
mkdir -p "${output_dir}"

packages=(
  ferrugo-syntax
  ferrugo-thumbnail
  ferrugo-object
  ferrugo-content
  ferrugo-render
  ferrugo-native
  ferrugo-pdfium
  ferrugo
)

echo "==> cargo metadata"
cargo metadata --no-deps --format-version 1 > "${output_dir}/metadata.json"

for package in "${packages[@]}"; do
  echo "==> ${package} package file list"
  cargo package -p "${package}" --allow-dirty --no-verify --list > "${output_dir}/${package}-package-files.txt"
done

echo "==> leaf package archive dry-runs"
cargo package -p ferrugo-syntax --allow-dirty --no-verify
cargo package -p ferrugo-thumbnail --allow-dirty --no-verify

if [[ "${FERRUGO_VERIFY_REGISTRY_PACKAGES:-0}" == "1" ]]; then
  echo "==> registry-backed package dry-runs"
  for package in "${packages[@]}"; do
    cargo package -p "${package}" --allow-dirty --no-verify
  done
else
  echo "==> skipping registry-backed package dry-runs for dependency-chain crates"
  echo "    set FERRUGO_VERIFY_REGISTRY_PACKAGES=1 after prior crates are visible on crates.io"
fi

echo "Crate publish-readiness check passed"
