#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

artifact_dir="target/native-golden"
report="${artifact_dir}/native-golden-comparison.json"
summary="${artifact_dir}/native-golden-summary.txt"
manifest="fixtures/native-golden-manifest.tsv"

mkdir -p "${artifact_dir}"

expected_samples="$(awk 'NR > 1 && NF > 0 { count++ } END { print count + 0 }' "${manifest}")"
if [[ "${expected_samples}" -le 0 ]]; then
  echo "native golden manifest contains no samples" >&2
  exit 1
fi

cargo run -p ferrugo --no-default-features -- compare-golden fixtures \
  --manifest "${manifest}" \
  --max-edge 160 \
  --output "${report}"

if ! rg -q '"report_kind": "native-golden-comparison"' "${report}"; then
  echo "native golden report_kind missing" >&2
  exit 1
fi

if ! rg -q '"failures":0' "${report}"; then
  echo "native golden report contains failures" >&2
  exit 1
fi

if ! rg -q "\"matched\":${expected_samples}" "${report}"; then
  echo "native golden report did not match all ${expected_samples} manifest samples" >&2
  exit 1
fi

{
  echo "Native golden image gate passed"
  echo "report=${report}"
  echo "manifest=${manifest}"
  echo "samples=${expected_samples}"
  echo "backend=rust-native"
  echo "max_edge=160"
} > "${summary}"

cat "${summary}"
