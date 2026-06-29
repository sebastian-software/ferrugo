#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

output_dir="${1:-target/corpus-dashboard}"
mkdir -p "${output_dir}"

primary_families=(
  scan
  mixed-layout
  office-export
  form
  report
  presentation
)

family_args=()
for family in "${primary_families[@]}"; do
  family_args+=(--include-family "${family}")
done

echo "==> corpus metadata"
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --output "${output_dir}/metadata.json"

echo "==> local corpus metadata validation"
cargo run -p ferrugo-cli --no-default-features -- validate-local-corpus \
  fixtures/local-corpus.example.toml \
  --allow-missing > "${output_dir}/local-corpus-validation.json"

echo "==> native support classification"
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  "${family_args[@]}" \
  --max-edge 160 \
  --output "${output_dir}/support.json"

echo "==> operator coverage"
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  "${family_args[@]}" \
  --output "${output_dir}/operators.json"

echo "==> native performance sample"
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family report \
  --include-family presentation \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output "${output_dir}/performance.json"

echo "==> server batch sample"
cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/server-batch-manifest.tsv \
  --include-family small \
  --include-family mixed-size \
  --include-family image-heavy \
  --include-family repeated-resources \
  --include-family vector-stress \
  --repetitions 2 \
  --max-workers 4 \
  --max-in-flight-pixels 102400 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --fail-on-budget \
  --output "${output_dir}/batch.json"

node --input-type=module - "${output_dir}" <<'NODE'
import fs from "node:fs";
import path from "node:path";

const dir = process.argv[2];
const read = (name) => JSON.parse(fs.readFileSync(path.join(dir, `${name}.json`), "utf8"));

const support = read("support");
const operators = read("operators");
const performance = read("performance");
const batch = read("batch");
const local = read("local-corpus-validation");

const dashboard = {
  schema_version: 1,
  generated_at: new Date().toISOString(),
  artifacts: {
    metadata: "metadata.json",
    local_corpus_validation: "local-corpus-validation.json",
    support: "support.json",
    operators: "operators.json",
    performance: "performance.json",
    batch: "batch.json"
  },
  support_summary: {
    total: support.total,
    native_rendered: support.native_rendered,
    fallback_required: support.fallback_required,
    fallback_categories: support.fallback_categories,
    errors: support.errors
  },
  operator_summary: operators.summary,
  performance_summary: performance.summary,
  batch_summary: batch.summary,
  local_corpus_summary: {
    status: local.status,
    sample_count: local.sample_count,
    document_count: local.document_count,
    categories: local.categories,
    privacy: local.privacy,
    synthetic_replacements: local.synthetic_replacements
  },
  regression_visibility: {
    unsupported_categories: support.fallback_categories ?? {},
    budget_failures: {
      performance: performance.summary?.budget_failures ?? 0,
      batch: batch.summary?.budget_failures ?? 0
    },
    errors: {
      support: support.errors ?? {},
      operators: operators.summary?.errors ?? 0,
      performance: performance.summary?.errors ?? 0,
      batch: batch.summary?.errors ?? 0
    }
  }
};

fs.writeFileSync(path.join(dir, "dashboard.json"), `${JSON.stringify(dashboard, null, 2)}\n`);
NODE

echo "Corpus dashboard written to ${output_dir}/dashboard.json"
