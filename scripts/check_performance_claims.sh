#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

node --input-type=module <<'NODE'
import fs from "node:fs";

const policyPath = "docs/policies/performance-claims.md";
const readmePath = "README.md";
const benchmarksPath = "docs/benchmarks.md";

const policy = fs.readFileSync(policyPath, "utf8");
const readme = fs.readFileSync(readmePath, "utf8");
const benchmarks = fs.readFileSync(benchmarksPath, "utf8");
const normalizedPolicy = policy.replace(/\s+/g, " ");

const requiredChecklistItems = [
  "Two stable matrix runs.",
  "Same host or documented host differences.",
  "Reference renderer versions recorded.",
  "Timing reliability caveats reviewed.",
  "Workload family named.",
  "Metric named.",
  "Local artifacts named.",
  "Claim wording avoids broad renderer parity.",
];

for (const item of requiredChecklistItems) {
  if (!policy.includes(`- [ ] ${item}`)) {
    throw new Error(`${policyPath} is missing checklist item: ${item}`);
  }
}

const requiredPolicyText = [
  "MuPDF remains v2 comparison backlog.",
  "The full benchmark matrix remains a local maintainer tool",
  "Focused fixture subsets may become CI gates only after their variance is measured",
  "Run `bash scripts/check_performance_claims.sh`.",
];

for (const text of requiredPolicyText) {
  if (!normalizedPolicy.includes(text)) {
    throw new Error(`${policyPath} is missing policy text: ${text}`);
  }
}

if (!readme.includes("docs/policies/performance-claims.md")) {
  throw new Error(`${readmePath} must link the performance claims policy`);
}

if (!benchmarks.includes("policies/performance-claims.md")) {
  throw new Error(`${benchmarksPath} must link the performance claims policy`);
}

console.log("performance claims policy check passed");
NODE
