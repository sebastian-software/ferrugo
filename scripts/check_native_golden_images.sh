#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

artifact_dir="target/native-golden"
summary="${artifact_dir}/native-golden-summary.txt"

mkdir -p "${artifact_dir}"
: > "${summary}"

run_gate() {
  local manifest="$1"
  local max_edge="$2"
  local report="$3"

  local expected_samples
  expected_samples="$(awk 'NR > 1 && NF > 0 { count++ } END { print count + 0 }' "${manifest}")"
  if [[ "${expected_samples}" -le 0 ]]; then
    echo "native golden manifest ${manifest} contains no samples" >&2
    exit 1
  fi

  cargo run -p ferrugo --no-default-features -- compare-golden fixtures \
    --manifest "${manifest}" \
    --max-edge "${max_edge}" \
    --output "${report}"

  if ! rg -q '"report_kind": "native-golden-comparison"' "${report}"; then
    echo "native golden report_kind missing in ${report}" >&2
    exit 1
  fi

  if ! rg -q '"failures":0' "${report}"; then
    echo "native golden report ${report} contains failures" >&2
    exit 1
  fi

  if ! rg -q "\"matched\":${expected_samples}" "${report}"; then
    echo "native golden report ${report} did not match all ${expected_samples} manifest samples" >&2
    exit 1
  fi

  {
    echo "report=${report}"
    echo "manifest=${manifest}"
    echo "samples=${expected_samples}"
    echo "max_edge=${max_edge}"
  } >> "${summary}"
}

run_gate fixtures/native-golden-manifest.tsv 160 \
  "${artifact_dir}/native-golden-comparison.json"

# High-resolution tier: long partial-coverage spans route through the SIMD
# integer kernels only at this scale, so thumbnail-size goldens alone would
# never exercise them.
run_gate fixtures/native-golden-highdpi-manifest.tsv 1024 \
  "${artifact_dir}/native-golden-highdpi-comparison.json"

{
  echo "backend=rust-native"
  echo "Native golden image gate passed"
} >> "${summary}"

cat "${summary}"
