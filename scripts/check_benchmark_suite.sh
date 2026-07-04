#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT_DIR="${OUTPUT_DIR:-target/benchmark-suite}"
MATRIX_JSON="${MATRIX_JSON:-${OUTPUT_DIR}/performance-matrix-smoke.json}"
MATRIX_REPORT="${MATRIX_REPORT:-${OUTPUT_DIR}/performance-matrix-smoke.md}"
MATRIX_ARTIFACT_DIR="${MATRIX_ARTIFACT_DIR:-${OUTPUT_DIR}/artifacts}"
SUMMARY="${SUMMARY:-${OUTPUT_DIR}/benchmark-suite-summary.txt}"

mkdir -p "$OUTPUT_DIR" "$MATRIX_ARTIFACT_DIR"

echo "==> benchmark suite policy check"
bash scripts/check_performance_claims.sh

echo "==> benchmark suite native matrix smoke"
OUTPUT="$MATRIX_JSON" \
REPORT="$MATRIX_REPORT" \
ARTIFACT_DIR="$MATRIX_ARTIFACT_DIR" \
bash scripts/check_performance_matrix_smoke.sh

node --input-type=module - "$MATRIX_JSON" "$MATRIX_REPORT" "$SUMMARY" <<'NODE'
import fs from "node:fs";

const [jsonPath, markdownPath, summaryPath] = process.argv.slice(2);
const report = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
const markdown = fs.readFileSync(markdownPath, "utf8");
const records = report.records ?? [];

if (report.schema_version !== 1) {
  throw new Error(`unexpected benchmark schema version: ${report.schema_version}`);
}
if (report.report_kind !== "renderer-performance-matrix") {
  throw new Error(`unexpected report kind: ${report.report_kind}`);
}
if (!report.platform?.os || !report.platform?.arch) {
  throw new Error("benchmark matrix must include platform os and arch");
}
if (!report.config || report.config.max_edge !== 120) {
  throw new Error("benchmark matrix must include the release smoke config");
}
if (!report.timing_reliability) {
  throw new Error("benchmark matrix must include timing_reliability");
}
if (report.summary?.errors !== 0 || report.summary?.fallback_required !== 0) {
  throw new Error("benchmark matrix release smoke must have zero errors and fallbacks");
}
if (report.summary?.missing_tool !== 0) {
  throw new Error("native-only benchmark matrix release smoke must not require tools");
}
if (records.length === 0) {
  throw new Error("benchmark matrix release smoke produced no records");
}
for (const record of records) {
  if (record.backend !== "native" || record.mode !== "hot-render") {
    throw new Error(`unexpected record backend/mode ${record.backend}/${record.mode}`);
  }
  if (record.family !== "small-text") {
    throw new Error(`unexpected benchmark family ${record.family}`);
  }
  if (record.status !== "rendered") {
    throw new Error(`${record.fixture} status is ${record.status}`);
  }
  if (typeof record.timing?.p95_ms !== "number") {
    throw new Error(`${record.fixture} is missing p95 timing`);
  }
}
if (!markdown.includes("# Ferrugo Renderer Performance Matrix")) {
  throw new Error("benchmark matrix markdown report has unexpected title");
}

const lines = [
  "Ferrugo benchmark suite gate",
  `JSON: ${jsonPath}`,
  `Markdown: ${markdownPath}`,
  `Records: ${records.length}`,
  `Families: ${[...new Set(records.map((record) => record.family))].join(", ")}`,
  `Backends: ${[...new Set(records.map((record) => record.backend))].join(", ")}`,
  `Modes: ${[...new Set(records.map((record) => record.mode))].join(", ")}`,
  "Result: passed",
  "",
];
fs.writeFileSync(summaryPath, lines.join("\n"));
console.log(`benchmark suite gate passed for ${records.length} record(s)`);
NODE

echo "Benchmark suite gate passed"
