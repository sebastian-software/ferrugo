# Native Renderer Conformance Triage

Date: 2026-06-26
Milestone: 0143

## Summary

The 0143 triage loop converts the current full-corpus visual oracle result into
small follow-up slices. Native runtime execution remains PDFium-free for the
core supported families, but visual parity still needs focused work in
text/fonts, form appearances, rendering-core details, image/color behavior, and
page geometry.

Artifacts:

- Visual oracle: `target/conformance-0143-visual-diff.json`
- Native supported gate: `target/conformance-0143-supported-gate.json`

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/conformance-0143-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 0 | 0 |
| `form` | 15 | 15 | 0 | 0 |
| `office-export` | 44 | 44 | 0 | 0 |
| **Core total** | **67** | **67** | **0** | **0** |

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/conformance-0143-visual-diff.json
```

Thresholds:

- `max_mean_abs_error`: `2.0`
- `max_p95_channel_delta`: `16`
- `max_changed_ratio`: `0.05`

Summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 155 | 32 | 23 | 91 | 8 | 0 | 1 |

## Family Triage

| Family | Total | Exact | Accepted drift | Blockers | Native errors | Both errors | Primary next action |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `office-export` | 44 | 0 | 3 | 41 | 0 | 0 | Split text/font fidelity from table/grid rendering-core work. |
| `form` | 15 | 0 | 1 | 14 | 0 | 0 | Isolate synthesized versus explicit appearance stream parity. |
| `report` | 34 | 8 | 8 | 15 | 3 | 0 | Triage rendering-core, page geometry, vector, and transparency boundaries separately. |
| `scan` | 22 | 10 | 1 | 8 | 3 | 0 | Separate image/color drift from unsupported image codecs. |
| `mixed-layout` | 22 | 8 | 6 | 6 | 1 | 1 | Keep encryption/XFA policy separate from rendering bugs. |
| `presentation` | 9 | 3 | 0 | 5 | 1 | 0 | Split page geometry, image/color, vector, and optional-content work. |
| `browser-print` | 8 | 2 | 4 | 2 | 0 | 0 | Audit missing-font and simple geometry drift. |
| `adversarial` | 1 | 1 | 0 | 0 | 0 | 0 | No conformance action. |

## Subsystem Triage

| Subsystem | Total | Exact | Accepted drift | Blockers | Native errors | Both errors | Recommended next action |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `rendering-core` | 43 | 4 | 4 | 34 | 1 | 0 | Split dense tables, scientific/report layout, dashboard, and dynamic XFA policy. |
| `text-fonts` | 27 | 0 | 3 | 24 | 0 | 0 | Audit fallback metrics, spacing, subset widths, and missing-font substitution. |
| `annotations-forms` | 23 | 5 | 5 | 13 | 0 | 0 | Focus AcroForm widgets and annotation appearance parity. |
| `page-geometry` | 13 | 2 | 2 | 9 | 0 | 0 | Audit crop, rotation, user-unit, and first-page transform parity. |
| `images-color` | 18 | 9 | 0 | 6 | 3 | 0 | Separate resampling/color drift from unsupported codec boundaries. |
| `vector-graphics` | 15 | 4 | 7 | 3 | 1 | 0 | Keep low-amplitude gradient drift accepted; isolate high-delta vector blockers. |
| `transparency` | 8 | 3 | 2 | 1 | 2 | 0 | Keep unsupported soft-mask/blend boundaries typed; triage one alpha drift blocker. |
| `document-structure` | 4 | 3 | 0 | 1 | 0 | 0 | Investigate hybrid-reference visual parity. |
| `optional-content` | 3 | 2 | 0 | 0 | 1 | 0 | Implement or formalize OCMD membership policy. |
| `document-security` | 1 | 0 | 0 | 0 | 0 | 1 | Encrypted input policy boundary. |

## Blocker Clusters

| Rank | Cluster | Count | Representative fixtures | Next action |
| ---: | --- | ---: | --- | --- |
| 1 | `text-fonts` / `office-export` | 21 | `embedded-font.pdf`, `text-spacing.pdf`, `missing-font-office-export.pdf`, `type1-fontfile-text.pdf` | Split into font metrics, spacing, subset widths, Type1/CFF/CID, and missing-font slices. |
| 2 | `rendering-core` / `office-export` | 16 | `spreadsheet-clipped-cells.pdf`, `spreadsheet-dense-numeric-grid.pdf`, `business-invoice-dense.pdf`, `office-table.pdf` | Build operator/table-grid triage before broad fixes. |
| 3 | `annotations-forms` / `form` | 12 | `business-form-stamp-signature.pdf` and AcroForm widget fixtures | Compare explicit appearance streams with synthesized native appearances. |
| 4 | `rendering-core` / `report` | 12 | `reference-footnote-layout.pdf`, scientific/report fixtures | Route through 0144 operator coverage audit. |
| 5 | `images-color` / `scan` | 5 | `cmyk-image.pdf`, `icc-cmyk-image.pdf` | Separate CMYK/color-conversion and scan resampling parity from codec support. |
| 6 | `page-geometry` mixed families | 9 | `multi-page-report.pdf`, `rotated-office-export.pdf`, `slide-speaker-notes-page.pdf` | Focus page-transform parity tests and visual subset. |

## Expected Drift

Accepted drift remains explicit and should not be hidden by threshold changes.
Representative accepted rows:

| Fixture | Subsystem | Rationale |
| --- | --- | --- |
| `axial-gradient.pdf`, `radial-gradient.pdf`, `type4-mesh-shading.pdf` | `vector-graphics` | Broad changed-pixel ratios but low MAE and p95 deltas, consistent with low-amplitude sampling drift. |
| `transparency-isolated-alpha-group.pdf`, `transparency-knockout-group.pdf` | `transparency` | Low-amplitude alpha drift within thresholds. |
| `markup-annotations-without-appearance.pdf`, `text-note-annotation-without-appearance.pdf` | `annotations-forms` | Appearance synthesis/omission policy stays visible, not silently upgraded to exact. |
| `vector-paths.pdf`, `line-joins.pdf` | `vector-graphics` | Small edge/rasterization differences within current visual thresholds. |
| `page-size-letter.pdf`, `linearized-first-page.pdf` | `page-geometry` | Small scaling/edge differences below blocker thresholds. |

## Typed Unsupported And Policy Boundaries

| Fixture | Subsystem | Error class | Bucket | Decision |
| --- | --- | --- | --- | --- |
| `unsupported-ccitt-image.pdf` | `images-color` | `unsupported` | `image.filter` | Keep typed unsupported until safe CCITT decoder strategy exists. |
| `unsupported-jbig2-image.pdf` | `images-color` | `unsupported` | `image.filter` | Keep typed unsupported until sandboxed or isolated JBIG2 strategy exists. |
| `unsupported-jpx-image.pdf` | `images-color` | `unsupported` | `image.filter` | Keep typed unsupported until JPEG 2000 strategy exists. |
| `extgstate-luminosity-soft-mask.pdf` | `transparency` | `unsupported` | `graphics.transparency` | Needs bounded soft-mask implementation. |
| `unsupported-blend-mode.pdf` | `transparency` | `unsupported` | `graphics.transparency` | Needs explicit blend-mode support decision. |
| `optional-content-ocmd.pdf` | `optional-content` | `unsupported` | `graphics.optional-content` | Needs OCMD membership/flattening policy. |
| `mesh-shading-unsupported.pdf` | `vector-graphics` | `unsupported` | `graphics.pattern-shading` | Keep as vector/shading follow-up. |
| `xfa-dynamic-no-static-appearance.pdf` | `rendering-core` | `unsupported` | `form.xfa-dynamic` | Dynamic XFA remains out of normal thumbnail scope. |
| `encrypted-placeholder.pdf` | `document-security` | `encrypted` | n/a | Both native and PDFium report encrypted; not a render-fidelity blocker. |

## Follow-Up Backlog

The owner-ready backlog is tracked in
`docs/backlogs/native-renderer-conformance-backlog.md`. The first implementation
slices should prioritize:

1. Office text/font fidelity.
2. Dense office table/grid rendering.
3. Form and annotation appearance parity.
4. Report rendering-core fidelity.
5. Scan image/color parity.
6. Page geometry drift.

## Report Format

`docs/policies/native-conformance-triage.md` defines the stable triage report
contract. No standalone schema validator exists yet; validation for this
milestone is the successful CLI JSON generation with `schema_version: 1`, count
preservation in this report, and the existing CLI JSON unit coverage.

## Validation

Commands run:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/conformance-0143-visual-diff.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/conformance-0143-supported-gate.json
```

Both CLI commands passed. A local Node one-liner read
`target/conformance-0143-visual-diff.json` and aggregated the generated JSON by
family, subsystem, blocker cluster, accepted drift, and native error rows for
the tables above.
