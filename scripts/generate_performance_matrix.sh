#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${OUTPUT:-target/performance-matrix.json}"
REPORT="${REPORT:-target/performance-matrix.md}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/performance-matrix-artifacts}"
INPUT="${INPUT:-fixtures/generated}"
MAX_EDGE="${MAX_EDGE:-160}"
ITERATIONS="${ITERATIONS:-20}"
WARMUP="${WARMUP:-3}"
MAX_COV="${MAX_COV:-0.15}"
TIMEOUT="${TIMEOUT:-30}"
PROFILE="${PROFILE:-release}"

cargo_args=(run -p ferrugo)
case "$PROFILE" in
  release)
    cargo_args+=(--release)
    ;;
  dev | debug)
    ;;
  *)
    echo "PROFILE must be one of: release, dev, debug" >&2
    exit 2
    ;;
esac

cargo "${cargo_args[@]}" --no-default-features -- benchmark-matrix "$INPUT" \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge "$MAX_EDGE" \
  --iterations "$ITERATIONS" \
  --warmup "$WARMUP" \
  --max-cov "$MAX_COV" \
  --timeout "$TIMEOUT" \
  --output "$OUTPUT" \
  --report "$REPORT" \
  --artifact-dir "$ARTIFACT_DIR"
