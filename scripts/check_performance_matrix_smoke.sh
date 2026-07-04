#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${OUTPUT:-target/performance-matrix-smoke.json}"
REPORT="${REPORT:-target/performance-matrix-smoke.md}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/performance-matrix-smoke-artifacts}"
INPUT="${INPUT:-fixtures/generated}"
FAMILY="${FAMILY:-small-text}"
MAX_EDGE="${MAX_EDGE:-120}"
ITERATIONS="${ITERATIONS:-20}"
WARMUP="${WARMUP:-3}"
MAX_COV="${MAX_COV:-0.50}"
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
  --backend native \
  --mode hot-render \
  --include-family "$FAMILY" \
  --max-edge "$MAX_EDGE" \
  --iterations "$ITERATIONS" \
  --warmup "$WARMUP" \
  --max-cov "$MAX_COV" \
  --timeout "$TIMEOUT" \
  --output "$OUTPUT" \
  --report "$REPORT" \
  --artifact-dir "$ARTIFACT_DIR"

node --input-type=module - "$OUTPUT" "$FAMILY" <<'NODE'
import fs from "node:fs";

const [reportPath, family] = process.argv.slice(2);
const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
const records = report.records ?? [];

if (records.length === 0) {
  throw new Error("performance smoke produced no records");
}
if (!report.timing_reliability) {
  throw new Error("performance smoke must include timing_reliability");
}
if (report.timing_reliability.cov_exceeded_records !== 0) {
  throw new Error(
    `performance smoke has ${report.timing_reliability.cov_exceeded_records} CoV breach(es)`,
  );
}
if (report.summary?.errors !== 0) {
  throw new Error(`performance smoke reported ${report.summary.errors} errors`);
}
if (report.summary?.fallback_required !== 0) {
  throw new Error(`performance smoke reported ${report.summary.fallback_required} fallbacks`);
}
if (report.summary?.missing_tool !== 0) {
  throw new Error(
    `native-only performance smoke reported ${report.summary.missing_tool} missing tools`,
  );
}

for (const record of records) {
  if (record.family !== family) {
    throw new Error(`unexpected family ${record.family}; expected ${family}`);
  }
  if (record.backend !== "native" || record.mode !== "hot-render") {
    throw new Error(`unexpected backend/mode ${record.backend}/${record.mode}`);
  }
  if (record.status !== "rendered") {
    throw new Error(`${record.fixture} status is ${record.status}`);
  }
  if (typeof record.timing?.p95_ms !== "number") {
    throw new Error(`${record.fixture} is missing p95 timing`);
  }
  if (record.timing.sample_count < 20) {
    throw new Error(`${record.fixture} has too few timing samples: ${record.timing.sample_count}`);
  }
  if (typeof record.timing.stddev_ms !== "number") {
    throw new Error(`${record.fixture} is missing timing stddev`);
  }
  if (typeof record.timing.cov !== "number") {
    throw new Error(`${record.fixture} is missing timing CoV`);
  }
  if (record.timing.cov_exceeded) {
    throw new Error(`${record.fixture} exceeded timing CoV threshold`);
  }
}

console.log(`performance matrix smoke passed for ${records.length} ${family} record(s)`);
NODE
