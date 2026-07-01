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
  local status

  status="$(curl --silent --show-error --output /dev/null --write-out "%{http_code}" \
    "https://crates.io/api/v1/crates/${package}/${version}")"

  case "$status" in
    200)
      return 0
      ;;
    404)
      return 1
      ;;
    *)
      echo "Unexpected crates.io response ${status} while checking ${package} ${version}." >&2
      return 1
      ;;
  esac
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
publish_with_retry ferrugo-render 5 30
publish_with_retry ferrugo-native 5 30
publish_with_retry ferrugo-pdfium 5 30
publish_with_retry ferrugo 5 30
