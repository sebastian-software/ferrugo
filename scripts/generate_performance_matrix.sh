#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${OUTPUT:-target/performance-matrix.json}"
REPORT="${REPORT:-target/performance-matrix.md}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/performance-matrix-artifacts}"
MAX_EDGE="${MAX_EDGE:-160}"
ITERATIONS="${ITERATIONS:-3}"
WARMUP="${WARMUP:-1}"
TIMEOUT="${TIMEOUT:-30}"

features=(--no-default-features)
if [[ -n "${FERRUGO_PDFIUM_LIBRARY:-}" ]]; then
  features=(--features pdfium)
fi

cargo run -p ferrugo-cli "${features[@]}" -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge "$MAX_EDGE" \
  --iterations "$ITERATIONS" \
  --warmup "$WARMUP" \
  --timeout "$TIMEOUT" \
  --output "$OUTPUT" \
  --report "$REPORT" \
  --artifact-dir "$ARTIFACT_DIR"
