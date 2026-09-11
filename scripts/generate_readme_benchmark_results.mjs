#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const START = "<!-- ferrugo:performance-results:start -->";
const END = "<!-- ferrugo:performance-results:end -->";

const defaults = {
  input: "docs/benchmarks/promoted/performance-matrix-2026-07-04.json",
  readme: "README.md.src",
  report: "docs/reports/performance-matrix-promoted-2026-07-04.md",
};

function usage() {
  console.error(
    "usage: node scripts/generate_readme_benchmark_results.mjs [--check|--write] [--input PATH] [--readme PATH] [--report PATH]",
  );
}

function parseArgs(argv) {
  const options = { ...defaults, mode: "write" };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--check") {
      options.mode = "check";
    } else if (arg === "--write") {
      options.mode = "write";
    } else if (arg === "--input" || arg === "--readme" || arg === "--report") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error(`${arg} requires a value`);
      }
      options[arg.slice(2)] = value;
      index += 1;
    } else if (arg === "--help" || arg === "-h") {
      usage();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return options;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeIfChanged(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  if (fs.existsSync(filePath) && fs.readFileSync(filePath, "utf8") === content) {
    return false;
  }
  fs.writeFileSync(filePath, content);
  return true;
}

function replaceBetweenMarkers(content, replacement) {
  const start = content.indexOf(START);
  const end = content.indexOf(END);
  if (start === -1 || end === -1 || end < start) {
    throw new Error(`README is missing ${START} / ${END} markers`);
  }
  return `${content.slice(0, start)}${replacement}${content.slice(end + END.length)}`;
}

function dateFromPath(filePath) {
  const match = path.basename(filePath).match(/(\d{4}-\d{2}-\d{2})/);
  return match ? match[1] : "undated";
}

function fmtMs(value) {
  return Number.isFinite(value) ? `${value.toFixed(3)} ms` : "n/a";
}

function fmtRatio(value) {
  return Number.isFinite(value) ? `${value.toFixed(2)}x` : "n/a";
}

function fmtBytes(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  if (value >= 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MiB`;
  }
  if (value >= 1024) {
    return `${(value / 1024).toFixed(1)} KiB`;
  }
  return `${value} B`;
}

function markdownCell(value) {
  return String(value ?? "n/a").replaceAll("|", "\\|").replaceAll("\n", " ");
}

function statusLabel(record) {
  if (!record) {
    return "n/a";
  }
  if (record.status === "rendered") {
    const ms = record.timing?.adjusted_wall_ms ?? record.timing?.wall_ms;
    const rss = record.memory?.rss_peak_bytes;
    return `${fmtMs(ms)} / ${fmtBytes(rss)}`;
  }
  return record.status;
}

function pickRecord(report, family, backend, mode) {
  return report.records.find(
    (record) =>
      record.family === family && record.backend === backend && record.mode === mode,
  );
}

function nativePeakRss(report, family) {
  const peaks = report.records
    .filter((record) => record.family === family && record.backend === "native")
    .map((record) => record.memory?.rss_peak_bytes)
    .filter(Number.isFinite);
  return peaks.length > 0 ? Math.max(...peaks) : null;
}

function platformCaption(report, sourcePath) {
  const platform = report.platform ?? {};
  const config = report.config ?? {};
  const bits = [
    `artifact ${sourcePath}`,
    `promoted ${dateFromPath(sourcePath)}`,
    `${platform.os ?? "unknown-os"}/${platform.arch ?? "unknown-arch"}`,
    platform.cpu_brand,
    platform.rustc_version,
    `max_edge=${config.max_edge}`,
    `iterations=${config.iterations}`,
    `warmup=${config.warmup}`,
  ].filter(Boolean);
  return bits.join("; ");
}

function oracleVersionSummary(report) {
  const versions = report.config?.oracle_versions ?? [];
  if (versions.length === 0) {
    return "No oracle version manifest was recorded.";
  }
  return versions
    .map((entry) => {
      const detected = entry.detected ?? "missing";
      const expected = entry.expected ? ` expected ${entry.expected}` : " unpinned";
      return `${entry.backend}: ${detected} (${entry.status};${expected})`;
    })
    .join("; ");
}

function familyRows(report) {
  return Object.entries(report.families ?? {}).map(([family, summary]) => {
    const nativeCold = pickRecord(report, family, "native", "cold-process");
    const pdfiumCold = pickRecord(report, family, "pdfium", "cold-process");
    const popplerCold = pickRecord(report, family, "poppler", "cold-process");
    const ghostscriptCold = pickRecord(report, family, "ghostscript", "cold-process");
    const mutoolCold = pickRecord(report, family, "mutool", "cold-process");
    return {
      family,
      nativeHotP95: summary.native_hot_p95_ms,
      nativeCold: statusLabel(nativeCold),
      nativePeakRss: nativePeakRss(report, family),
      pdfiumCold: statusLabel(pdfiumCold),
      popplerCold: statusLabel(popplerCold),
      ghostscriptCold: statusLabel(ghostscriptCold),
      mutoolCold: statusLabel(mutoolCold),
      pdfiumHotRatio: summary.ferrugo_to_pdfium_hot_ratio,
      errors: summary.errors ?? 0,
    };
  });
}

function renderFamilyTable(report, compact = false) {
  const rows = familyRows(report);
  const header = compact
    ? "| Family | Ferrugo hot p95 | Ferrugo cold / RSS | PDFium cold / RSS | Poppler cold / RSS | Ghostscript cold / RSS | MuPDF cold / RSS |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n"
    : "| Family | Ferrugo hot p95 | Ferrugo cold / RSS | Ferrugo peak RSS | PDFium cold / RSS | Poppler cold / RSS | Ghostscript cold / RSS | MuPDF cold / RSS | Ferrugo/PDFium hot | Errors |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n";
  const body = rows
    .map((row) => {
      if (compact) {
        return `| \`${markdownCell(row.family)}\` | ${fmtMs(row.nativeHotP95)} | ${markdownCell(row.nativeCold)} | ${markdownCell(row.pdfiumCold)} | ${markdownCell(row.popplerCold)} | ${markdownCell(row.ghostscriptCold)} | ${markdownCell(row.mutoolCold)} |`;
      }
      return `| \`${markdownCell(row.family)}\` | ${fmtMs(row.nativeHotP95)} | ${markdownCell(row.nativeCold)} | ${fmtBytes(row.nativePeakRss)} | ${markdownCell(row.pdfiumCold)} | ${markdownCell(row.popplerCold)} | ${markdownCell(row.ghostscriptCold)} | ${markdownCell(row.mutoolCold)} | ${fmtRatio(row.pdfiumHotRatio)} | ${row.errors} |`;
    })
    .join("\n");
  return `${header}${body}\n`;
}

function renderReadmeBlock(report, sourcePath, reportPath) {
  const reliability = report.timing_reliability ?? {};
  const caveats = reliability.caveats?.length
    ? reliability.caveats.map((caveat) => `\`${caveat}\``).join(", ")
    : "none";
  return `${START}
Generated from [\`${sourcePath}\`](${sourcePath}) with the full report in [\`${reportPath}\`](${reportPath}).

${renderFamilyTable(report, true)}
Caption: ${platformCaption(report, sourcePath)}.

Reference renderer versions: ${oracleVersionSummary(report)}.

Caveats: ${caveats}. External PDFium, Poppler, Ghostscript, and MuPDF rows are cold-process oracle runs; hot-render p95 is reported only for Ferrugo native. Treat these as scoped benchmark-matrix results, not a broad renderer-parity claim.
${END}`;
}

function renderReport(report, sourcePath) {
  const reliability = report.timing_reliability ?? {};
  const config = report.config ?? {};
  const summary = report.summary ?? {};
  const caveats = reliability.caveats?.length
    ? reliability.caveats.map((caveat) => `\`${caveat}\``).join(", ")
    : "none";
  const oracleRows = (config.oracle_versions ?? [])
    .map(
      (entry) =>
        `| \`${markdownCell(entry.backend)}\` | \`${markdownCell(entry.command)}\` | ${markdownCell(entry.detected ?? "missing")} | ${markdownCell(entry.expected ?? "unpinned")} | \`${markdownCell(entry.status)}\` |`,
    )
    .join("\n");
  return `# Promoted Renderer Performance Matrix

Status: generated.
Date: ${dateFromPath(sourcePath)}.

Source artifact: [\`${sourcePath}\`](../../${sourcePath}).

This report is generated by \`scripts/generate_readme_benchmark_results.mjs\`
from the promoted benchmark-matrix JSON. The README performance block is
generated from the same source and \`scripts/check_readme_benchmark_results.sh\`
fails when either file drifts.

## Provenance

${platformCaption(report, sourcePath)}.

Command:

\`\`\`text
${report.command ?? "n/a"}
\`\`\`

| Field | Value |
| --- | --- |
| Input | \`${markdownCell(config.input)}\` |
| Manifest | \`${markdownCell(config.manifest)}\` |
| Families | ${markdownCell(config.include_families?.length ? config.include_families.join(", ") : "manifest default")} |
| Modes | ${markdownCell(config.modes?.join(", "))} |
| Backends | ${markdownCell(config.backends?.join(", "))} |
| Native profile | \`${markdownCell(config.native_profile)}\` |
| Max edge | ${config.max_edge ?? "n/a"} |
| Iterations | ${config.iterations ?? "n/a"} |
| Warmup | ${config.warmup ?? "n/a"} |
| CoV threshold | ${config.max_cov ?? "n/a"} |

## Reliability

| Signal | Value |
| --- | --- |
| Records | ${summary.total_records ?? "n/a"} |
| Rendered | ${summary.rendered ?? "n/a"} |
| Missing tool rows | ${summary.missing_tool ?? "n/a"} |
| Not applicable rows | ${summary.not_applicable ?? "n/a"} |
| Errors | ${summary.errors ?? "n/a"} |
| RSS samples available | ${reliability.rss_available ? "yes" : "no"} |
| Cold reference available | ${reliability.cold_reference_available ? "yes" : "no"} |
| Oracle version drift | ${reliability.oracle_version_drift ? "yes" : "no"} |
| Records over CoV threshold | ${reliability.cov_exceeded_records ?? "n/a"} |
| Caveats | ${caveats} |

## Oracle Versions

| Backend | Command | Detected | Expected | Status |
| --- | --- | --- | --- | --- |
${oracleRows || "| n/a | n/a | n/a | n/a | n/a |"}

## Family Results

${renderFamilyTable(report)}

External reference renderers are measured as cold-process oracles in this
pipeline. Ferrugo native is the only hot-render row, so this report supports
family-scoped observations and regression tracking rather than blanket
cross-renderer speed claims.
`;
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const report = readJson(options.input);
  const readmeBlock = renderReadmeBlock(report, options.input, options.report);
  const generatedReadme = replaceBetweenMarkers(
    fs.readFileSync(options.readme, "utf8"),
    readmeBlock,
  );
  const generatedReport = renderReport(report, options.input);

  if (options.mode === "check") {
    const existingReadme = fs.readFileSync(options.readme, "utf8");
    const existingReport = fs.existsSync(options.report)
      ? fs.readFileSync(options.report, "utf8")
      : "";
    const failures = [];
    if (existingReadme !== generatedReadme) {
      failures.push(options.readme);
    }
    if (existingReport !== generatedReport) {
      failures.push(options.report);
    }
    if (failures.length > 0) {
      throw new Error(`generated benchmark results are stale: ${failures.join(", ")}`);
    }
    console.log("generated benchmark results are current");
    return;
  }

  writeIfChanged(options.readme, generatedReadme);
  writeIfChanged(options.report, generatedReport);
  console.log(`updated ${options.readme} and ${options.report} from ${options.input}`);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  usage();
  process.exit(1);
}
