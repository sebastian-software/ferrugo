#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

artifact_dir="target/native-golden"
report="${artifact_dir}/native-golden-comparison.json"
summary="${artifact_dir}/native-golden-summary.txt"

mkdir -p "${artifact_dir}"

cargo run -p ferrugo --no-default-features -- compare-golden fixtures/generated \
  --manifest fixtures/native-golden-manifest.tsv \
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

if ! rg -q '"matched":5' "${report}"; then
  echo "native golden report did not match all five release samples" >&2
  exit 1
fi

{
  echo "Native golden image gate passed"
  echo "report=${report}"
  echo "manifest=fixtures/native-golden-manifest.tsv"
  echo "samples=5"
  echo "backend=rust-native"
  echo "max_edge=160"
} > "${summary}"

cat "${summary}"
