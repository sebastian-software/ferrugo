#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

node --input-type=module <<'NODE'
import fs from "node:fs";

const path = "fixtures/scheduler-tuning-profile-matrix.tsv";
const lines = fs.readFileSync(path, "utf8").trimEnd().split("\n");
const header = lines[0].split("\t");
if (header.length !== 9) {
  throw new Error(`scheduler matrix header has ${header.length} columns, expected 9`);
}

const allowedBlocking = new Set([
  "server-primary",
  "server-constrained",
  "secondary-profile",
]);
const rows = lines.slice(1).map((line, index) => {
  const columns = line.split("\t");
  if (columns.length !== 9) {
    throw new Error(`${path}:${index + 2} has ${columns.length} columns, expected 9`);
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
  if (!row.constraints || !row.budget_gate || !row.notes) {
    throw new Error(`${row.profile} must include constraints, budget gate, and notes`);
  }
}

for (const required of [
  "server-batch",
  "cancellation",
  "low-memory-batch",
  "repeat-cache",
  "wasm-smoke",
]) {
  if (!profiles.has(required)) {
    throw new Error(`missing required profile ${required}`);
  }
}

console.log(`${rows.length} scheduler tuning profiles validated`);
NODE
