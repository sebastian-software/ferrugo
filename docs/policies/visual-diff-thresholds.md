# Visual Diff Thresholds And Review Workflow

This policy covers local native-versus-reference visual review runs produced by
`ferrugo visual-diff` and `ferrugo visual-diff-poppler`. It is a
maintainer comparison policy, not a runtime or release-gate requirement. The
release/oracle split is defined in `docs/policies/reference-oracle-strategy.md`.

`visual-diff` uses an external PDFium renderer configured by `--pdfium PATH` or
`FERRUGO_PDFIUM_RENDERER`; it does not require the `pdfium` Cargo feature.
`visual-diff-poppler` uses an external `pdftoppm` binary and writes no PDF bytes
or rendered rasters to the repository. It uses a per-process writable
Fontconfig cache so server-style sandbox runs do not depend on host
home-directory caches.

Visual-diff JSON includes target platform metadata (`os`, `arch`, `family`,
`endian`, and `pointer_width_bits`). Use that block when comparing drift across
machines, and do not infer cross-platform coverage from an artifact that lacks
the target platform needed for a release gate. Fixture metrics include `ssim`
and `dssim`; family and subsystem summaries include `comparable`, `mean_ssim`,
`min_ssim`, and `max_dssim` so drift can be trended over repeated reports.

## Default Thresholds

The default classification thresholds are intentionally conservative:

| Field | Default | Meaning |
| --- | ---: | --- |
| `max_mean_abs_error` | `2.000` | Average RGB channel delta across the page. |
| `max_p95_channel_delta` | `16` | 95th percentile RGB channel delta. |
| `max_changed_ratio` | `0.050000` | Share of pixels with any RGB channel delta. |
| `min_ssim` | `0.000000` | Optional global SSIM floor; pass `--min-ssim` when running a family-specific oracle gate. |

`alpha` is ignored for delta classification because the thumbnail facade uses
RGBA output over an explicit background. RGB output is the user-visible review
surface.

SSIM is calculated over RGB samples and reported on a 0.0-1.0 scale where 1.0 is
exact structural similarity. DSSIM is reported as `(1 - ssim) / 2` for trend
readability. The default `min_ssim` is intentionally non-blocking to preserve
the historical MAE/p95/changed-ratio review workflow; release or family-specific
oracle runs should set the family threshold explicitly.

## Per-Family SSIM Thresholds

Use these minimum SSIM floors when a visual-diff run is meant to block on
perceptual fidelity rather than only record trend evidence:

| Family tier | Families | Minimum SSIM | Notes |
| --- | --- | ---: | --- |
| Text and forms | `office-export`, `form`, `email-web-archive`, `real-office-export` | `0.995000` | Text placement and table/form geometry should remain structurally stable. |
| Browser and mixed layout | `browser-print`, `mixed-layout`, `real-browser-print` | `0.990000` | Allows small antialiasing differences around clipped backgrounds and annotations. |
| Reports and presentations | `report`, `presentation`, `real-report` | `0.985000` | Charts, spot colors, and transparency often drift perceptually before they fail p95. |
| Scan/image heavy | `scan`, `real-scan` | `0.980000` | Raster scaling and patent/scan inputs can have wider reference-renderer differences. |

When a family-specific run uses one of these floors, pass the matching
`--min-ssim` value and keep the MAE, p95, and changed-ratio thresholds in the
report command. If a family needs a different floor, record the before/after
`mean_ssim`, `min_ssim`, `max_dssim`, affected fixtures, and rationale in the
review report.

Low-amplitude field drift is accepted when the page MAE is below the configured
threshold and either the maximum RGB delta is at most 8, or the 95th percentile
RGB delta is at most 4. The p95 rule allows broad 1-4 channel rounding drift
with a small text or edge antialiasing tail, while keeping high-amplitude
renderer mismatches as blockers.

Low-p95 edge drift is accepted when MAE is at most 3.5, the 95th percentile RGB
delta is at most 5, and the changed-pixel ratio is at most 0.5. This keeps
image/text edge antialiasing drift out of the blocker bucket while preserving
blockers for high-p95 page-geometry, text placement, transparency, and vector
linework mismatches.

## Statuses

| Status | Meaning | Review action |
| --- | --- | --- |
| `exact` | No RGB pixel changed. | No manual review needed. |
| `accepted_drift` | All thresholds passed. | Keep as expected antialiasing or rounding drift. |
| `blocker` | At least one threshold failed. | Review before treating the category as covered. |
| `native_error` | Native renderer failed and PDFium rendered. | Track as native gap or unsupported bucket. |
| `pdfium_error` | PDFium failed and native rendered. | Validate fixture and comparison setup. |
| `reference_error` | Poppler failed and native rendered. | Validate fixture, `pdftoppm`, Fontconfig, and timeout setup. |
| `both_error` | Both renderers failed. | Classify by shared error class, not visual drift. |

Blockers must not be hidden by loosening thresholds. If a threshold change is
needed, record the family, subsystem, before/after counts, and reason in the
review report.

## Subsystem Buckets

Each fixture is assigned one review bucket:

- `annotations-forms`
- `document-security`
- `document-structure`
- `images-color`
- `optional-content`
- `page-geometry`
- `rendering-core`
- `text-fonts`
- `transparency`
- `vector-graphics`

The buckets are coarse on purpose. They are for triage and backlog planning,
not a replacement for typed renderer errors.

## Review Workflow

1. Run `visual-diff` with the current corpus manifest and local external PDFium
   renderer.
2. Review the top-level summary first.
3. Review `subsystems` next to identify the renderer area that should own the
   work.
4. Review `families` to understand document-category impact.
5. Compare `mean_ssim`, `min_ssim`, and `max_dssim` against prior reports before
   treating an unchanged pass/fail status as unchanged fidelity.
6. Open blocker fixtures only after the aggregate report identifies the owning
   subsystem.
7. Keep encrypted or unsupported-category outcomes separate from pixel drift.

The JSON report is comparison evidence for review notes. Generated PNG or
diff-image artifacts should stay local unless a later policy adds a bounded
artifact retention policy.

For PDFium-free release evidence, use
`docs/policies/native-golden-images.md` and
`bash scripts/check_native_golden_images.sh`. That gate stores committed
baselines as hashes only and keeps rendered artifacts under `target/`.
