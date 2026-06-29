# Reference Oracle Strategy

Status: accepted.
Date: 2026-06-26.

This policy defines how `ferrugo` validates visual behavior while keeping
normal runtime and release gates independent from PDFium.

## Roles

| Role | PDFium allowed | Purpose |
| --- | --- | --- |
| Runtime rendering | No | User-facing rendering through the Rust-native backend. |
| Release gate | No | Supported-family pass/fail checks for packaging, fallback, and budget regressions. |
| Maintainer oracle | Yes, explicit feature only | Local triage when a visual difference needs an external reference. |
| Historical evidence | Yes, archived only | Previously recorded reports that explain why a gap or threshold exists. |
| Manual review | No runtime dependency | Human decision for ambiguous or multi-oracle disagreements. |

PDFium-enabled commands must stay behind `--features pdfium` and are not release
prerequisites for the supported runtime slice.

## 0215 Removal Decision

Milestone 0215 keeps PDFium comparison tooling as maintainer-only infrastructure
instead of deleting it. The retained tools are `ferrugo-pdfium`,
`render-pdfium`, `render-isolated`, `compare-metadata`, `benchmark-pdfium`, and
`visual-diff`. They remain outside supported runtime and release gates.

Deletion is blocked until native-only golden comparison coverage, retention
policy, CI golden samples, and multi-oracle records cover the same triage value.
Active corpus expectations should describe native behavior, such as
`expected:native` or `expected:native-unsupported`, rather than a PDFium runtime
fallback.

## Validation Modes

| Mode | Primary command or artifact | CI suitability | Release suitability | Use when |
| --- | --- | --- | --- | --- |
| Native supported gate | `summarize-fallbacks --fail-on-fallback` with no default features | Yes | Yes | A family is expected to render without typed fallback or render errors. |
| Native budget gate | `benchmark-native` with memory/output/time budgets | Yes | Yes | Throughput or memory regressions could affect server-side rendering. |
| Package/quarantine gate | `cargo package`, `check_plugin_free_distribution.sh`, `check_pdfium_quarantine.sh` | Yes | Yes | Verifying the default artifact has no PDFium runtime edge. |
| Metadata baseline | `compare-metadata` or backend-neutral baseline JSON | Local or scheduled | No, unless a native-only equivalent exists | Page count and page geometry need oracle confirmation. |
| Pixel visual diff | `visual-diff` with an explicit maintainer oracle | Local or scheduled | No | A renderer subsystem needs triage against external output. |
| Golden image comparison | Future committed baseline compare command | Yes after tooling lands | Yes for bounded fixture sets | A reviewed fixture has stable expected output independent from live PDFium. |
| Multi-oracle review | PDFium, Poppler, MuPDF, PDF.js, or Ghostscript reports | Local or scheduled | No | Engines disagree or PDF semantics are underspecified for the fixture. |
| Manual review record | Human-reviewed report linked from a milestone | Yes as recorded evidence | Yes for bounded exceptions | Pixel thresholds cannot express acceptability safely. |

## Document Family Routing

| Family or pressure | Release gate | Maintainer oracle | Manual review trigger |
| --- | --- | --- | --- |
| Browser print, office export, static forms | Native supported gate plus native budget gate | PDFium visual diff only for regressions or planned fidelity work | User-visible text/layout shift, clipped content, blank page, or form appearance mismatch. |
| Scanner and mobile scan | Native supported gate plus output-size budget | PDFium, Poppler, or Ghostscript for raster placement and decode disputes | Resampling, rotation, crop, or compression drift changes document readability. |
| Financial, government, e-signature | Native supported gate plus typed unsupported classification | PDFium plus Poppler or Ghostscript for high-impact disagreements | Signature appearance, stamp, barcode, totals, or official-form content differs materially. |
| Presentation, chart, dashboard, map | Native supported gate for accepted subset | PDFium plus PDF.js or Poppler for viewer-facing layout disputes | Charts, legends, layers, or map labels become misleading. |
| Optional content, pattern shading, transparency, advanced color | Typed unsupported bucket until implemented | Multi-oracle comparison after native implementation work starts | Any single oracle disagrees with another or semantics depend on viewer policy. |
| Encrypted, malformed, dynamic XFA | Typed policy outcome | Oracle only when investigating parser or security behavior | Error class is ambiguous or could hide renderable user content. |

## Threshold Calibration

Thresholds are a review aid, not a way to waive missing renderer behavior.
Changes must record:

- affected fixture families;
- before and after `exact`, `accepted_drift`, and `blocker` counts;
- subsystem owner;
- reason the new threshold distinguishes antialiasing drift from semantic
  regressions;
- whether the decision came from a golden image, multi-oracle evidence, or
  manual review.

Release gates must prefer exact native outcomes: no fallback, no render error,
and no budget failure for supported families.

## Historical Evidence

Historical PDFium reports remain useful for understanding the project history,
but they are not proof that current release validation requires PDFium. Reports
that cite PDFium should state whether the evidence is:

- runtime evidence: must be native-only;
- comparison evidence: may use explicit maintainer oracle tooling;
- historical evidence: archived context from an earlier milestone.

## Manual Review Fallback

Ambiguous cases need a small review record instead of threshold inflation. The
record must include fixture path, document family, renderer outputs inspected,
decision, reviewer, and follow-up owner. Until a record exists, the result stays
as a blocker or typed unsupported bucket.
