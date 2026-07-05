#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

targets=(
  primitive_parse
  xref_load
  stream_decode
  content_tokenize
  render_setup
  render
)

runs="${FUZZ_RUNS:-256}"
max_total_time="${FUZZ_MAX_TOTAL_TIME:-60}"
artifact_dir="${FUZZ_ARTIFACT_DIR:-target/fuzz-artifacts}"

mkdir -p "${artifact_dir}"

for target in "${targets[@]}"; do
  echo "==> cargo fuzz: ${target}"
  mkdir -p "${artifact_dir}/${target}"
  cargo fuzz run "${target}" "fuzz/corpus/${target}" -- \
    "-runs=${runs}" \
    "-max_total_time=${max_total_time}" \
    "-artifact_prefix=${artifact_dir}/${target}/"
done

echo "Coverage-guided fuzz campaign passed"
