#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${OUTPUT:-target/multi-oracle-smoke.json}"
REPORT="${REPORT:-target/multi-oracle-smoke.md}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/multi-oracle-smoke-artifacts}"
FAMILY="${FAMILY:-small-text}"
MAX_EDGE="${MAX_EDGE:-120}"
TIMEOUT="${TIMEOUT:-30}"

cargo run -p ferrugo --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --backend ghostscript \
  --mode cold-process \
  --include-family "$FAMILY" \
  --max-edge "$MAX_EDGE" \
  --iterations 1 \
  --warmup 0 \
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
  throw new Error("multi-oracle smoke produced no records");
}
if (!report.timing_reliability) {
  throw new Error("multi-oracle smoke must include timing_reliability");
}
if (report.timing_reliability.ghostscript_requested !== true) {
  throw new Error("ghostscript must be marked requested");
}

for (const record of records) {
  if (record.backend !== "ghostscript" || record.mode !== "cold-process") {
    throw new Error(`unexpected backend/mode ${record.backend}/${record.mode}`);
  }
  if (record.family !== family) {
    throw new Error(`unexpected family ${record.family}; expected ${family}`);
  }
  if (record.status !== "rendered" && record.status !== "missing-tool") {
    throw new Error(`${record.fixture} status is ${record.status}`);
  }
  if (record.status === "rendered") {
    if (typeof record.output?.width !== "number" || typeof record.output?.height !== "number") {
      throw new Error(`${record.fixture} rendered without output dimensions`);
    }
    if (typeof record.output?.artifact_hash !== "string") {
      throw new Error(`${record.fixture} rendered without artifact hash`);
    }
  }
}

console.log(`multi-oracle smoke passed for ${records.length} ${family} record(s)`);
NODE
