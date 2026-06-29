#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

output_dir="${1:-target/coverage-scorecard-0201}"
dashboard_dir="${output_dir}/dashboard"
mkdir -p "${output_dir}"

bash scripts/generate_corpus_dashboard.sh "${dashboard_dir}"

node --input-type=module - "${output_dir}" "${dashboard_dir}" <<'NODE'
import fs from "node:fs";
import path from "node:path";

const outputDir = process.argv[2];
const dashboardDir = process.argv[3];
const read = (name) => JSON.parse(fs.readFileSync(path.join(dashboardDir, `${name}.json`), "utf8"));

const support = read("support");
const operators = read("operators");
const performance = read("performance");
const batch = read("batch");

const familyWeights = {
  "report": 0.24,
  "office-export": 0.22,
  "scan": 0.18,
  "form": 0.14,
  "mixed-layout": 0.12,
  "presentation": 0.10
};

const categoryRoutes = {
  "annotation.appearance": {
    milestone: "0207",
    title: "Annotation Popup Stamp And FreeText Fidelity",
    severity: "medium"
  },
  "form.xfa-dynamic": {
    milestone: "0206",
    title: "Form Filling Appearance Update And Flattening Coverage",
    severity: "documented-boundary"
  },
  "graphics.color-management": {
    milestone: "0208",
    title: "Color Managed Print Preview Extended Gate",
    severity: "medium"
  },
  "graphics.optional-content": {
    milestone: "0211",
    title: "PDF Operator Semantic Snapshot Suite",
    severity: "medium"
  },
  "graphics.pattern-shading": {
    milestone: "0204",
    title: "Office Chart SmartArt And Vector Effect Fidelity",
    severity: "medium"
  },
  "graphics.transparency": {
    milestone: "0213",
    title: "Transparency Stack Memory Optimization",
    severity: "high"
  },
  "image.filter": {
    milestone: "0209",
    title: "Rust-Native Image Codec Deployment Policy",
    severity: "high"
  },
  "text.font-program": {
    milestone: "0202",
    title: "Text Selection Geometry And Search Highlight Parity",
    severity: "high"
  }
};

const thresholds = {
  "1.3_pdfium_free_typical_document_gate": {
    weighted_score_minimum: 94,
    family_score_minimum: 88,
    supported_family_native_pass_rate: 1,
    supported_family_error_budget: 0,
    runtime_pdfium_allowed: false,
    server_batch_budget_failures_max: 0,
    unsupported_categories_must_be_typed: true,
    visual_drift_channel: "tracked separately; not folded into runtime score"
  },
  "score_formula": {
    family_score: "100 * ((0.8 * native_pass_rate) + (0.2 * operator_maturity_rate))",
    operator_maturity_rate: "(implemented + ignored + 0.5 * partial) / total_operators",
    weighted_score: "sum(family_score * typical_document_weight)"
  }
};

const round = (value, digits = 3) => Number(value.toFixed(digits));
const pct = (value) => `${round(value * 100, 1).toFixed(1)}%`;
const countMap = (value) => value ?? {};
const countMapTotal = (value) => Object.values(countMap(value)).reduce((sum, count) => sum + count, 0);
const ratio = (num, den, fallback = 1) => den > 0 ? num / den : fallback;

function operatorMaturity(family) {
  const counts = operators.families?.[family]?.status_counts ?? {};
  const implemented = counts.implemented ?? 0;
  const partial = counts.partial ?? 0;
  const unsupported = counts.unsupported ?? 0;
  const ignored = counts.ignored ?? 0;
  const total = implemented + partial + unsupported + ignored;
  return {
    total,
    implemented,
    partial,
    unsupported,
    ignored,
    maturity_rate: ratio(implemented + ignored + (0.5 * partial), total)
  };
}

function performanceSummary(family) {
  const summary = performance.families?.[family];
  if (!summary) {
    return {
      sampled: false,
      pass: 0,
      unsupported: 0,
      timeout: 0,
      memory_budget: 0,
      other_budget: 0,
      max_ms: null,
      mean_ms: null
    };
  }

  let timeout = 0;
  let memoryBudget = 0;
  let otherBudget = 0;
  for (const fixture of performance.fixtures ?? []) {
    if (fixture.family !== family) {
      continue;
    }
    for (const violation of fixture.budget_violations ?? []) {
      if (violation.includes("timeout") || violation.includes("time")) {
        timeout += 1;
      } else if (violation.includes("memory") || violation.includes("bytes")) {
        memoryBudget += 1;
      } else if (violation !== "native_fallback") {
        otherBudget += 1;
      }
    }
  }

  return {
    sampled: true,
    pass: summary.native_rendered ?? 0,
    unsupported: summary.fallback_required ?? 0,
    timeout,
    memory_budget: memoryBudget,
    other_budget: otherBudget,
    max_ms: summary.max_ms ?? null,
    mean_ms: summary.mean_ms ?? null
  };
}

const families = Object.entries(familyWeights).map(([family, weight]) => {
  const supportFamily = support.families?.[family] ?? {};
  const total = supportFamily.total ?? 0;
  const nativeRendered = supportFamily.native_rendered ?? 0;
  const unsupported = supportFamily.fallback_required ?? 0;
  const errors = countMapTotal(supportFamily.errors);
  const operator = operatorMaturity(family);
  const perf = performanceSummary(family);
  const nativePassRate = ratio(nativeRendered, total, 0);
  const familyScore = 100 * ((0.8 * nativePassRate) + (0.2 * operator.maturity_rate));
  const fallbackCategories = countMap(supportFamily.fallback_categories);

  return {
    family,
    typical_document_weight: weight,
    score: round(familyScore, 2),
    weighted_points: round(familyScore * weight, 2),
    outcomes: {
      pass: nativeRendered,
      partial: operator.partial,
      unsupported,
      timeout: perf.timeout,
      memory_budget: perf.memory_budget,
      visual_drift: "not_measured_in_runtime_score",
      errors
    },
    support: {
      total,
      native_pass_rate: round(nativePassRate, 4),
      fallback_categories: fallbackCategories,
      errors: countMap(supportFamily.errors)
    },
    operator_coverage: {
      total_operators: operator.total,
      implemented: operator.implemented,
      partial: operator.partial,
      unsupported: operator.unsupported,
      ignored: operator.ignored,
      maturity_rate: round(operator.maturity_rate, 4)
    },
    performance: perf
  };
}).sort((left, right) => right.typical_document_weight - left.typical_document_weight);

const weightedScore = round(families.reduce((sum, family) => sum + family.weighted_points, 0), 2);

const gapMap = new Map();
for (const family of families) {
  for (const [category, count] of Object.entries(family.support.fallback_categories)) {
    const route = categoryRoutes[category] ?? {
      milestone: "unrouted",
      title: "Needs triage",
      severity: "medium"
    };
    const impact = 100 * family.typical_document_weight * ratio(count, family.support.total, 0);
    const existing = gapMap.get(category) ?? {
      category,
      count: 0,
      weighted_gap_points: 0,
      families: [],
      ...route
    };
    existing.count += count;
    existing.weighted_gap_points += impact;
    existing.families.push({ family: family.family, count, impact_points: round(impact, 3) });
    gapMap.set(category, existing);
  }
}

const gaps = [...gapMap.values()]
  .map((gap) => ({ ...gap, weighted_gap_points: round(gap.weighted_gap_points, 3) }))
  .sort((left, right) => right.weighted_gap_points - left.weighted_gap_points || right.count - left.count);

const scorecard = {
  schema_version: 1,
  generated_at: new Date().toISOString(),
  dashboard_dir: path.relative(outputDir, dashboardDir),
  runtime_pdfium: {
    supported_path: false,
    policy: "PDFium may remain maintainer-only oracle tooling, but not runtime coverage."
  },
  thresholds,
  summary: {
    weighted_score: weightedScore,
    family_count: families.length,
    native_rendered: support.native_rendered,
    fallback_required: support.fallback_required,
    errors: support.errors,
    server_batch_budget_failures: batch.summary?.budget_failures ?? null
  },
  families,
  gaps
};

fs.writeFileSync(path.join(outputDir, "scorecard.json"), `${JSON.stringify(scorecard, null, 2)}\n`);

const rows = families
  .map((family) => [
    `\`${family.family}\``,
    family.typical_document_weight.toFixed(2),
    family.score.toFixed(2),
    pct(family.support.native_pass_rate),
    family.outcomes.unsupported,
    family.outcomes.errors,
    family.operator_coverage.partial,
    family.performance.sampled ? family.performance.memory_budget : "n/a",
    family.performance.sampled ? family.performance.timeout : "n/a"
  ].join(" | "))
  .map((row) => `| ${row} |`)
  .join("\n");

const gapRows = gaps
  .map((gap) => [
    `\`${gap.category}\``,
    gap.count,
    gap.weighted_gap_points.toFixed(3),
    gap.severity,
    `${gap.milestone} ${gap.title}`
  ].join(" | "))
  .map((row) => `| ${row} |`)
  .join("\n");

const markdown = `# Native Renderer 1.3 Coverage Scorecard

Generated from the native-only corpus dashboard in \`${path.relative(outputDir, dashboardDir)}\`.

## Summary

| Weighted score | Native rendered | Typed unsupported | Errors | Server batch budget failures |
| ---: | ---: | ---: | ---: | ---: |
| ${weightedScore.toFixed(2)} | ${support.native_rendered} | ${support.fallback_required} | ${countMapTotal(support.errors)} | ${batch.summary?.budget_failures ?? "n/a"} |

Runtime PDFium remains outside the supported path. Visual drift is tracked as a
separate validation channel and is not hidden inside the runtime score.

## Family Scorecard

| Family | Weight | Score | Native pass | Unsupported | Errors | Partial operators | Memory budget | Timeout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
${rows}

## Weighted Gap Queue

| Category | Count | Weighted gap points | Severity | Routed milestone |
| --- | ---: | ---: | --- | --- |
${gapRows || "| none | 0 | 0.000 | n/a | n/a |"}

## 1.3 Thresholds

- Weighted score must be at least ${thresholds["1.3_pdfium_free_typical_document_gate"].weighted_score_minimum}.
- Every family score must be at least ${thresholds["1.3_pdfium_free_typical_document_gate"].family_score_minimum}.
- Supported-family native pass rate must be ${pct(thresholds["1.3_pdfium_free_typical_document_gate"].supported_family_native_pass_rate)} with zero supported-family errors.
- Server batch budget failures must stay at or below ${thresholds["1.3_pdfium_free_typical_document_gate"].server_batch_budget_failures_max}.
- Unsupported categories must remain typed and routed to follow-up milestones.
`;

fs.writeFileSync(path.join(outputDir, "scorecard.md"), markdown);
NODE

echo "Coverage scorecard written to ${output_dir}/scorecard.json"
echo "Coverage scorecard markdown written to ${output_dir}/scorecard.md"
