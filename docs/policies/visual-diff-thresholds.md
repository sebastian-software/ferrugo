# Visual Diff Thresholds And Review Workflow

This policy covers local native-versus-PDFium visual review runs produced by
`pdfrust-cli visual-diff`.

## Default Thresholds

The default classification thresholds are intentionally conservative:

| Field | Default | Meaning |
| --- | ---: | --- |
| `max_mean_abs_error` | `2.000` | Average RGB channel delta across the page. |
| `max_p95_channel_delta` | `16` | 95th percentile RGB channel delta. |
| `max_changed_ratio` | `0.050000` | Share of pixels with any RGB channel delta. |

`alpha` is ignored for delta classification because the thumbnail facade uses
RGBA output over an explicit background. RGB output is the user-visible review
surface.

## Statuses

| Status | Meaning | Review action |
| --- | --- | --- |
| `exact` | No RGB pixel changed. | No manual review needed. |
| `accepted_drift` | All thresholds passed. | Keep as expected antialiasing or rounding drift. |
| `blocker` | At least one threshold failed. | Review before treating the category as covered. |
| `native_error` | Native renderer failed and PDFium rendered. | Track as native gap or unsupported bucket. |
| `pdfium_error` | PDFium failed and native rendered. | Validate fixture and comparison setup. |
| `both_error` | Both renderers failed. | Classify by shared error class, not visual drift. |

Blockers must not be hidden by loosening thresholds. If a threshold change is
needed, record the family, subsystem, before/after counts, and reason in the
milestone report.

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

The buckets are coarse on purpose. They are for triage and milestone planning,
not a replacement for typed renderer errors.

## Review Workflow

1. Run `visual-diff` with the current corpus manifest and local PDFium build.
2. Review the top-level summary first.
3. Review `subsystems` next to identify the renderer area that should own the
   work.
4. Review `families` to understand document-category impact.
5. Open blocker fixtures only after the aggregate report identifies the owning
   subsystem.
6. Keep encrypted or unsupported-category outcomes separate from pixel drift.

The JSON report is the committed evidence source for milestone notes. Generated
PNG or diff-image artifacts should stay local unless a later milestone adds a
bounded artifact retention policy.
