#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

node --input-type=module <<'NODE'
import fs from "node:fs";

const path = "fixtures/low-end-reliability-profile-matrix.tsv";
const lines = fs.readFileSync(path, "utf8").trimEnd().split("\n");
const header = lines[0].split("\t");
if (header.length !== 11) {
  throw new Error(`profile matrix header has ${header.length} columns, expected 11`);
}

const allowedBlocking = new Set(["server-primary", "server-constrained", "secondary-profile"]);
const rows = lines.slice(1).map((line, index) => {
  const columns = line.split("\t");
  if (columns.length !== 11) {
    throw new Error(`${path}:${index + 2} has ${columns.length} columns, expected 11`);
  }
  return Object.fromEntries(header.map((name, columnIndex) => [name, columns[columnIndex]]));
});

const profiles = new Set();
for (const row of rows) {
  if (profiles.has(row.profile)) {
    throw new Error(`duplicate profile ${row.profile}`);
  }
  profiles.add(row.profile);
  if (row.result !== "passed") {
    throw new Error(`${row.profile} must record a passed validation result`);
  }
  if (!allowedBlocking.has(row.blocking_scope)) {
    throw new Error(`${row.profile} has invalid blocking_scope ${row.blocking_scope}`);
  }
  if (!row.artifact.startsWith("target/")) {
    throw new Error(`${row.profile} artifact must be target-local`);
  }
  if (!row.budget_gate || !row.constraints || !row.notes) {
    throw new Error(`${row.profile} must include constraints, budget gate, and notes`);
  }
}

for (const required of [
  "low-memory-summary",
  "low-memory-repeat",
  "server-constrained-batch",
  "wasm-smoke",
  "deterministic-reduced-canvas",
]) {
  if (!profiles.has(required)) {
    throw new Error(`missing required profile ${required}`);
  }
}

console.log(`${rows.length} low-end reliability profiles validated`);
NODE
