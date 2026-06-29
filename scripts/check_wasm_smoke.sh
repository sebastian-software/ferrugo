#!/usr/bin/env sh
set -eu

target="wasm32-unknown-unknown"
profile="release"
max_bytes="${PDFRUST_WASM_MAX_BYTES:-4194304}"
max_compile_ms="${PDFRUST_WASM_MAX_COMPILE_MS:-250}"
max_instantiate_ms="${PDFRUST_WASM_MAX_INSTANTIATE_MS:-100}"
max_smoke_ms="${PDFRUST_WASM_MAX_SMOKE_MS:-250}"
max_total_output_bytes="${PDFRUST_WASM_MAX_TOTAL_OUTPUT_BYTES:-524288}"
min_fixtures="${PDFRUST_WASM_MIN_FIXTURES:-5}"
artifact="target/${target}/${profile}/pdfrust_wasm_smoke.wasm"
report="${PDFRUST_WASM_REPORT:-target/wasm-0132-smoke.json}"

cargo check -p pdfrust-wasm-smoke --target "${target}" --no-default-features
cargo build -p pdfrust-wasm-smoke --target "${target}" --release

if [ ! -f "${artifact}" ]; then
  echo "missing WASM artifact: ${artifact}" >&2
  exit 1
fi

node scripts/wasm_smoke.mjs "${artifact}" "${report}"

node --input-type=module - "${report}" "${max_bytes}" "${max_compile_ms}" "${max_instantiate_ms}" "${max_smoke_ms}" "${max_total_output_bytes}" "${min_fixtures}" <<'EOF'
import { readFile } from "node:fs/promises";

const [
  ,
  ,
  reportPath,
  maxBytesRaw,
  maxCompileRaw,
  maxInstantiateRaw,
  maxSmokeRaw,
  maxTotalOutputBytesRaw,
  minFixturesRaw,
] = process.argv;
const report = JSON.parse(await readFile(reportPath, "utf8"));
const limits = {
  max_bytes: Number(maxBytesRaw),
  max_compile_ms: Number(maxCompileRaw),
  max_instantiate_ms: Number(maxInstantiateRaw),
  max_smoke_ms: Number(maxSmokeRaw),
  max_total_output_bytes: Number(maxTotalOutputBytesRaw),
  min_fixtures: Number(minFixturesRaw),
};

const failures = [];
if (report.size_bytes > limits.max_bytes) {
  failures.push(`size_bytes ${report.size_bytes} > ${limits.max_bytes}`);
}
if (report.compile_ms > limits.max_compile_ms) {
  failures.push(`compile_ms ${report.compile_ms} > ${limits.max_compile_ms}`);
}
if (report.instantiate_ms > limits.max_instantiate_ms) {
  failures.push(`instantiate_ms ${report.instantiate_ms} > ${limits.max_instantiate_ms}`);
}
if (report.smoke_ms > limits.max_smoke_ms) {
  failures.push(`smoke_ms ${report.smoke_ms} > ${limits.max_smoke_ms}`);
}
if (report.total_output_bytes > limits.max_total_output_bytes) {
  failures.push(
    `total_output_bytes ${report.total_output_bytes} > ${limits.max_total_output_bytes}`,
  );
}
if (report.fixture_count < limits.min_fixtures) {
  failures.push(`fixture_count ${report.fixture_count} < ${limits.min_fixtures}`);
}

if (failures.length > 0) {
  console.error(`WASM smoke budget failure: ${failures.join("; ")}`);
  process.exit(1);
}

console.log(JSON.stringify({ schema_version: 1, status: "passed", limits, report }));
EOF
