#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo "CARGO_REGISTRY_TOKEN must be set by the crates.io trusted publishing auth step" >&2
  exit 1
fi

packages=(
  ferrugo-syntax
  ferrugo-thumbnail
  ferrugo-object
  ferrugo-content
  ferrugo-simd
  ferrugo-render
  ferrugo-native
  ferrugo-pdfium
  ferrugo
)

crate_version() {
  local package="$1"
  local package_id
  package_id="$(cargo pkgid -p "$package")"
  local suffix="${package_id##*#}"
  echo "${suffix##*@}"
}

crate_version_exists() {
  local package="$1"
  local version="$2"
  local index_path

  index_path="${package:0:2}/${package:2:2}/${package}"
  curl -fsSL "https://index.crates.io/${index_path}" 2>/dev/null \
    | grep -F "\"vers\":\"${version}\"" > /dev/null
}

publish_with_retry() {
  local package="$1"
  local attempts="$2"
  local delay="$3"
  local attempt
  local version

  version="$(crate_version "$package")"
  if crate_version_exists "$package" "$version"; then
    echo "Skipping ${package} ${version}; it already exists on crates.io."
    return 0
  fi

  for attempt in $(seq 1 "$attempts"); do
    if cargo publish -p "$package" --locked; then
      return 0
    fi

    if crate_version_exists "$package" "$version"; then
      echo "Skipping ${package} ${version}; it appeared on crates.io after publish attempt ${attempt}."
      return 0
    fi

    if [[ "$attempt" -eq "$attempts" ]]; then
      return 1
    fi

    echo "Waiting ${delay}s before retrying ${package}..."
    sleep "$delay"
  done
}

publish_with_retry ferrugo-syntax 1 0
publish_with_retry ferrugo-thumbnail 1 0
publish_with_retry ferrugo-object 5 30
publish_with_retry ferrugo-content 5 30
publish_with_retry ferrugo-simd 5 30
publish_with_retry ferrugo-render 5 30
publish_with_retry ferrugo-native 5 30
publish_with_retry ferrugo-pdfium 5 30
publish_with_retry ferrugo 5 30
