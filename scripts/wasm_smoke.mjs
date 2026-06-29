#!/usr/bin/env node

import { performance } from "node:perf_hooks";
import { readFile, writeFile } from "node:fs/promises";

const [, , wasmPath, outputPath = "target/wasm-0132-smoke.json"] = process.argv;

if (!wasmPath) {
  console.error("usage: node scripts/wasm_smoke.mjs <artifact.wasm> [output.json]");
  process.exit(2);
}

const bytes = await readFile(wasmPath);
const compileStart = performance.now();
const module = await WebAssembly.compile(bytes);
const compileMs = performance.now() - compileStart;

const instantiateStart = performance.now();
const instance = await WebAssembly.instantiate(module, {});
const instantiateMs = performance.now() - instantiateStart;

const smoke = instance.exports.pdfrust_wasm_smoke_status;
if (typeof smoke !== "function") {
  console.error("missing pdfrust_wasm_smoke_status export");
  process.exit(3);
}
const fixtureCount = instance.exports.pdfrust_wasm_smoke_fixture_count;
if (typeof fixtureCount !== "function") {
  console.error("missing pdfrust_wasm_smoke_fixture_count export");
  process.exit(3);
}
const totalOutputBytes = instance.exports.pdfrust_wasm_smoke_total_output_bytes;
if (typeof totalOutputBytes !== "function") {
  console.error("missing pdfrust_wasm_smoke_total_output_bytes export");
  process.exit(3);
}

const smokeStart = performance.now();
const status = smoke();
const smokeMs = performance.now() - smokeStart;

if (status === 0) {
  console.error("WASM smoke render returned failure status");
  process.exit(4);
}
const totalOutputByteCount = totalOutputBytes();
if (totalOutputByteCount === 0) {
  console.error("WASM smoke output byte report returned failure status");
  process.exit(5);
}

const result = {
  schema_version: 1,
  wasm_path: wasmPath,
  size_bytes: bytes.length,
  compile_ms: Number(compileMs.toFixed(3)),
  instantiate_ms: Number(instantiateMs.toFixed(3)),
  smoke_ms: Number(smokeMs.toFixed(3)),
  status,
  width: status >>> 16,
  height: status & 0xffff,
  fixture_count: fixtureCount(),
  total_output_bytes: totalOutputByteCount,
};

await writeFile(outputPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify(result));
