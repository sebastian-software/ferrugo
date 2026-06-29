#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

node --input-type=module <<'NODE'
import fs from "node:fs";
import path from "node:path";

const source = fs.readFileSync("crates/pdfrust-thumbnail/src/lib.rs", "utf8");
const constants = [...source.matchAll(/pub const [A-Z0-9_]+: &str = "([^"]+)";/g)]
  .map((match) => match[1])
  .filter((bucket) => bucket.includes("."));

const stableArrayMatch = source.match(/pub const STABLE_UNSUPPORTED_FEATURE_BUCKETS:[\s\S]*?= &\[([\s\S]*?)\];/);
if (!stableArrayMatch) {
  throw new Error("missing STABLE_UNSUPPORTED_FEATURE_BUCKETS array");
}
const stableNames = [...stableArrayMatch[1].matchAll(/unsupported_feature_buckets::([A-Z0-9_]+)/g)]
  .map((match) => match[1]);
if (stableNames.length !== constants.length) {
  throw new Error(`stable bucket count mismatch: ${stableNames.length} array entries, ${constants.length} constants`);
}

const completeBucketDocs = [
  "docs/policies/unsupported-feature-sla.md",
  "docs/errors.md",
];
for (const docPath of completeBucketDocs) {
  const content = fs.readFileSync(docPath, "utf8");
  for (const bucket of constants) {
    if (!content.includes(bucket)) {
      throw new Error(`${docPath} does not mention stable bucket ${bucket}`);
    }
  }
}

const guide = fs.readFileSync("docs/guides/native-only-consumer-migration.md", "utf8");
for (const required of [
  "ThumbnailErrorClass::Unsupported",
  "unsupported_feature_bucket()",
  "unsupported_feature_buckets::IMAGE_FILTER",
  "unsupported_feature_buckets::FORM_XFA_DYNAMIC",
]) {
  if (!guide.includes(required)) {
    throw new Error(`migration guide is missing public routing example ${required}`);
  }
}

const sla = fs.readFileSync("docs/policies/unsupported-feature-sla.md", "utf8");
for (const link of [...sla.matchAll(/`(docs\/[^`]+\.md)`/g)].map((match) => match[1])) {
  if (!fs.existsSync(link)) {
    throw new Error(`broken SLA link ${link}`);
  }
}

for (const link of [...guide.matchAll(/`(docs\/[^`]+\.md)`/g)].map((match) => match[1])) {
  if (!fs.existsSync(path.normalize(link))) {
    throw new Error(`broken migration-guide link ${link}`);
  }
}

console.log(`${constants.length} unsupported feature SLA buckets validated`);
NODE
