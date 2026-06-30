#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

profile="${FERRUGO_SERVERLESS_PROFILE:-serverless}"
binary="target/${profile}/ferrugo"
fixture="${FERRUGO_SERVERLESS_FIXTURE:-fixtures/generated/text-page.pdf}"
output="${FERRUGO_SERVERLESS_OUTPUT:-target/serverless-profile-0197.json}"
package_list="${FERRUGO_SERVERLESS_PACKAGE_LIST:-target/serverless-profile-ferrugo-package-files.txt}"
iterations="${FERRUGO_SERVERLESS_ITERATIONS:-7}"
max_binary_bytes="${FERRUGO_SERVERLESS_MAX_BINARY_BYTES:-8388608}"
max_startup_p95_ms="${FERRUGO_SERVERLESS_MAX_STARTUP_P95_MS:-500}"
max_first_render_p95_ms="${FERRUGO_SERVERLESS_MAX_FIRST_RENDER_P95_MS:-250}"
max_render_output_bytes="${FERRUGO_SERVERLESS_MAX_RENDER_OUTPUT_BYTES:-1048576}"
max_edge="${FERRUGO_SERVERLESS_MAX_EDGE:-160}"

mkdir -p target

echo "==> serverless native-only build (${profile})"
cargo build --profile "${profile}" -p ferrugo --no-default-features

echo "==> ferrugo package file inspection"
cargo package -p ferrugo --allow-dirty --no-verify --list > "${package_list}"
if rg -n '\.(dylib|so|dll|a|framework)(/|$)|libpdfium|pdfium\.dll|FERRUGO_PDFIUM_LIBRARY' "${package_list}"; then
  echo "PDFium runtime asset or native binary found in ferrugo package file list" >&2
  exit 1
fi

echo "==> startup and first-render measurement"
FERRUGO_SERVERLESS_BINARY="${binary}" \
FERRUGO_SERVERLESS_PROFILE="${profile}" \
FERRUGO_SERVERLESS_FIXTURE="${fixture}" \
FERRUGO_SERVERLESS_OUTPUT="${output}" \
FERRUGO_SERVERLESS_ITERATIONS="${iterations}" \
FERRUGO_SERVERLESS_MAX_BINARY_BYTES="${max_binary_bytes}" \
FERRUGO_SERVERLESS_MAX_STARTUP_P95_MS="${max_startup_p95_ms}" \
FERRUGO_SERVERLESS_MAX_FIRST_RENDER_P95_MS="${max_first_render_p95_ms}" \
FERRUGO_SERVERLESS_MAX_RENDER_OUTPUT_BYTES="${max_render_output_bytes}" \
FERRUGO_SERVERLESS_MAX_EDGE="${max_edge}" \
node --input-type=module <<'NODE'
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const binary = process.env.FERRUGO_SERVERLESS_BINARY;
const fixture = process.env.FERRUGO_SERVERLESS_FIXTURE;
const output = process.env.FERRUGO_SERVERLESS_OUTPUT;
const iterations = Number.parseInt(process.env.FERRUGO_SERVERLESS_ITERATIONS, 10);
const maxBinaryBytes = Number.parseInt(process.env.FERRUGO_SERVERLESS_MAX_BINARY_BYTES, 10);
const maxStartupP95Ms = Number.parseFloat(process.env.FERRUGO_SERVERLESS_MAX_STARTUP_P95_MS);
const maxFirstRenderP95Ms = Number.parseFloat(process.env.FERRUGO_SERVERLESS_MAX_FIRST_RENDER_P95_MS);
const maxRenderOutputBytes = Number.parseInt(process.env.FERRUGO_SERVERLESS_MAX_RENDER_OUTPUT_BYTES, 10);
const maxEdge = process.env.FERRUGO_SERVERLESS_MAX_EDGE;

if (!Number.isInteger(iterations) || iterations <= 0) {
  throw new Error("FERRUGO_SERVERLESS_ITERATIONS must be greater than zero");
}

const binarySizeBytes = fs.statSync(binary).size;
const renderDir = path.join("target", "serverless-profile-renders");
fs.mkdirSync(renderDir, { recursive: true });

function timedSpawn(args, stdout = "ignore") {
  const started = process.hrtime.bigint();
  const result = spawnSync(binary, args, { stdio: ["ignore", stdout, "ignore"] });
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1_000_000;
  if (result.status !== 0) {
    throw new Error(`${binary} ${args.join(" ")} exited with ${result.status}`);
  }
  return elapsedMs;
}

function percentile(values, percentileValue) {
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(
    sorted.length - 1,
    Math.ceil((percentileValue / 100) * sorted.length) - 1,
  );
  return sorted[Math.max(0, index)];
}

function stats(values) {
  const sum = values.reduce((total, value) => total + value, 0);
  return {
    min_ms: Math.min(...values),
    mean_ms: sum / values.length,
    p50_ms: percentile(values, 50),
    p95_ms: percentile(values, 95),
    max_ms: Math.max(...values),
  };
}

const startupMs = [];
const firstRenderMs = [];
let renderOutputBytes = 0;

for (let index = 0; index < iterations; index += 1) {
  startupMs.push(timedSpawn(["--help"]));
  const renderOutput = path.join(renderDir, `render-${index}.png`);
  firstRenderMs.push(
    timedSpawn([
      "render-native",
      fixture,
      "--max-edge",
      maxEdge,
      "--output",
      renderOutput,
    ]),
  );
  renderOutputBytes = Math.max(renderOutputBytes, fs.statSync(renderOutput).size);
}

const startup = stats(startupMs);
const firstRender = stats(firstRenderMs);
const budgetFailures = [];

if (binarySizeBytes > maxBinaryBytes) {
  budgetFailures.push({
    budget: "binary_size",
    actual: binarySizeBytes,
    limit: maxBinaryBytes,
  });
}
if (startup.p95_ms > maxStartupP95Ms) {
  budgetFailures.push({
    budget: "startup_p95_ms",
    actual: startup.p95_ms,
    limit: maxStartupP95Ms,
  });
}
if (firstRender.p95_ms > maxFirstRenderP95Ms) {
  budgetFailures.push({
    budget: "first_render_p95_ms",
    actual: firstRender.p95_ms,
    limit: maxFirstRenderP95Ms,
  });
}
if (renderOutputBytes > maxRenderOutputBytes) {
  budgetFailures.push({
    budget: "render_output_bytes",
    actual: renderOutputBytes,
    limit: maxRenderOutputBytes,
  });
}

const report = {
  schema_version: 1,
  profile: process.env.FERRUGO_SERVERLESS_PROFILE ?? "serverless",
  binary,
  fixture,
  platform: {
    os: process.platform,
    arch: process.arch,
    node: process.version,
    cpu_count: os.cpus().length,
  },
  config: {
    iterations,
    max_edge: Number.parseInt(maxEdge, 10),
    max_binary_bytes: maxBinaryBytes,
    max_startup_p95_ms: maxStartupP95Ms,
    max_first_render_p95_ms: maxFirstRenderP95Ms,
    max_render_output_bytes: maxRenderOutputBytes,
  },
  summary: {
    binary_size_bytes: binarySizeBytes,
    render_output_bytes: renderOutputBytes,
    budget_failures: budgetFailures.length,
  },
  startup,
  first_render: firstRender,
  budget_failures: budgetFailures,
};

fs.writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report.summary));

if (budgetFailures.length > 0) {
  process.exitCode = 1;
}
NODE

echo "Serverless profile report written to ${output}"
