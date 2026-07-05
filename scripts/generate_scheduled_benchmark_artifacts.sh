#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT_DIR="${OUTPUT_DIR:-target/scheduled-benchmarks}"
PROFILE="${PROFILE:-release}"
MATRIX_ITERATIONS="${MATRIX_ITERATIONS:-10}"
MATRIX_WARMUP="${MATRIX_WARMUP:-2}"
MATRIX_MAX_EDGE="${MATRIX_MAX_EDGE:-160}"
REAL_WORLD_ITERATIONS="${REAL_WORLD_ITERATIONS:-5}"
REAL_WORLD_WARMUP="${REAL_WORLD_WARMUP:-1}"
REAL_WORLD_MAX_EDGE="${REAL_WORLD_MAX_EDGE:-1024}"
TIMEOUT="${TIMEOUT:-30}"
TREND_HISTORY="${TREND_HISTORY:-}"

mkdir -p "$OUTPUT_DIR" "$OUTPUT_DIR/generated-artifacts" "$OUTPUT_DIR/real-world-artifacts"

echo "==> scheduled generated performance matrix"
PROFILE="$PROFILE" \
INPUT=fixtures/generated \
OUTPUT="$OUTPUT_DIR/performance-matrix.json" \
REPORT="$OUTPUT_DIR/performance-matrix.md" \
ARTIFACT_DIR="$OUTPUT_DIR/generated-artifacts" \
MAX_EDGE="$MATRIX_MAX_EDGE" \
ITERATIONS="$MATRIX_ITERATIONS" \
WARMUP="$MATRIX_WARMUP" \
TIMEOUT="$TIMEOUT" \
bash scripts/generate_performance_matrix.sh

echo "==> scheduled real-world performance matrix"
PROFILE="$PROFILE" \
INPUT=fixtures/real-world \
OUTPUT="$OUTPUT_DIR/real-world-performance-matrix.json" \
REPORT="$OUTPUT_DIR/real-world-performance-matrix.md" \
ARTIFACT_DIR="$OUTPUT_DIR/real-world-artifacts" \
MAX_EDGE="$REAL_WORLD_MAX_EDGE" \
ITERATIONS="$REAL_WORLD_ITERATIONS" \
WARMUP="$REAL_WORLD_WARMUP" \
TIMEOUT="$TIMEOUT" \
bash scripts/generate_performance_matrix.sh

echo "==> scheduled native golden image comparison"
cargo run -p ferrugo --no-default-features -- compare-golden fixtures \
  --manifest fixtures/native-golden-manifest.tsv \
  --max-edge 160 \
  --output "$OUTPUT_DIR/native-golden-comparison.json"

echo "==> scheduled Poppler visual diff"
cargo run -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/cross-producer-fusion-manifest.tsv \
  --include-family fused-report \
  --include-family fused-table-statement \
  --include-family fused-form \
  --include-family fused-scan \
  --include-family fused-dashboard-map \
  --max-edge 120 \
  --max-mae 12 \
  --max-p95 96 \
  --max-changed-ratio 0.30 \
  --timeout "$TIMEOUT" \
  --output "$OUTPUT_DIR/poppler-visual-diff.json"

echo "==> scheduled benchmark trend summary"
node --input-type=module - "$OUTPUT_DIR" "$TREND_HISTORY" <<'NODE'
import fs from "node:fs";
import path from "node:path";

const [outputDir, historyPath] = process.argv.slice(2);
const readJson = (name) => JSON.parse(fs.readFileSync(path.join(outputDir, name), "utf8"));
const matrix = readJson("performance-matrix.json");
const realWorld = readJson("real-world-performance-matrix.json");
const golden = readJson("native-golden-comparison.json");
const visual = readJson("poppler-visual-diff.json");

const smallText = (matrix.records ?? []).find(
  (record) =>
    record.backend === "native" &&
    record.mode === "hot-render" &&
    record.family === "small-text" &&
    record.status === "rendered",
);

const entry = {
  schema_version: 1,
  recorded_at: new Date().toISOString(),
  github: {
    run_id: process.env.GITHUB_RUN_ID ?? null,
    run_attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    sha: process.env.GITHUB_SHA ?? null,
    ref: process.env.GITHUB_REF_NAME ?? null,
    runner_os: process.env.RUNNER_OS ?? null,
    runner_arch: process.env.RUNNER_ARCH ?? null,
  },
  generated_matrix: {
    summary: matrix.summary,
    timing_reliability: matrix.timing_reliability,
    oracle_versions: matrix.config?.oracle_versions ?? [],
    small_text_native_hot_p95_ms: smallText?.timing?.p95_ms ?? null,
    process_startup_baseline_ms: matrix.config?.process_startup_baseline_ms ?? null,
  },
  real_world_matrix: {
    summary: realWorld.summary,
    timing_reliability: realWorld.timing_reliability,
  },
  native_golden: {
    summary: golden.summary,
  },
  poppler_visual_diff: {
    summary: visual.summary,
    families: visual.families ?? {},
    subsystems: visual.subsystems ?? {},
  },
};

const previous = historyPath && fs.existsSync(historyPath)
  ? fs.readFileSync(historyPath, "utf8").split(/\n+/).filter(Boolean).map((line) => JSON.parse(line))
  : [];
const series = [...previous, entry];
const trendPath = path.join(outputDir, "benchmark-trend.jsonl");
fs.writeFileSync(trendPath, `${series.map((item) => JSON.stringify(item)).join("\n")}\n`);

const values = series
  .map((item) => item.generated_matrix?.small_text_native_hot_p95_ms)
  .filter((value) => typeof value === "number")
  .slice(-5);
const mean = values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : null;
const stddev = values.length > 1
  ? Math.sqrt(values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / (values.length - 1))
  : null;
const cov = mean && stddev !== null ? stddev / mean : null;
const threshold = 0.20;
const enough = values.length >= 5;
const regression = enough && cov !== null && cov > threshold;

const md = [
  "# Scheduled Benchmark Trend Summary",
  "",
  `Entries: ${series.length}`,
  `Last-5 small-text native hot p95 samples: ${values.map((value) => value.toFixed(3)).join(", ") || "none"}`,
  `Last-5 mean ms: ${mean === null ? "n/a" : mean.toFixed(3)}`,
  `Last-5 stddev ms: ${stddev === null ? "n/a" : stddev.toFixed(3)}`,
  `Last-5 CoV: ${cov === null ? "n/a" : cov.toFixed(4)}`,
  `CoV threshold: ${threshold.toFixed(2)}`,
  `Enough samples for variance decision: ${enough ? "yes" : "no"}`,
  `Regression threshold exceeded: ${regression ? "yes" : "no"}`,
  "",
  "Shared-runner data is for variance and trend detection. Claim-bearing numbers still need review under docs/policies/performance-claims.md.",
  "",
].join("\n");
fs.writeFileSync(path.join(outputDir, "benchmark-trend-summary.md"), md);

if (regression) {
  throw new Error(`scheduled benchmark CoV ${cov.toFixed(4)} exceeded ${threshold}`);
}
NODE

echo "Scheduled benchmark artifacts written to ${OUTPUT_DIR}"
