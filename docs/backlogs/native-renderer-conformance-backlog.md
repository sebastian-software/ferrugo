# Native Renderer Conformance Backlog

Status: accepted for 0143.
Date: 2026-06-26.

This backlog turns the 0143 visual oracle result into small native-renderer
follow-up slices. Counts come from
`target/conformance-0143-visual-diff.json`.

## Priority Slices

| Rank | Slice | Evidence | Recommended next action | Validation gate |
| ---: | --- | --- | --- | --- |
| 1 | Office text/font fidelity | `text-fonts` + `office-export`: 21 blockers. | Audit fallback text metrics, text spacing, subset font widths, Type1/CFF/CID positioning, and missing-font substitution as separate fixture groups. | Focused visual diff over office text fixtures plus `cargo test -p pdfrust-render text_display_list`. |
| 2 | Dense office table/grid rendering | `rendering-core` + `office-export`: 16 blockers. | Split spreadsheet/table fixtures into operator semantics, clipping, hairline/grid stroke, and cell-overflow cases. | Focused visual diff over spreadsheet and business office fixtures. |
| 3 | Form and annotation appearance parity | `annotations-forms` + `form`: 12 blockers. | Compare native synthesized appearances against explicit appearance streams; isolate checkbox/radio/text-field/signature/stamp differences. | Focused visual diff over form fixtures and native form appearance tests. |
| 4 | Report rendering-core fidelity | `rendering-core` + `report`: 12 blockers. | Triage scientific, long-report, technical, and dashboard fixtures by operator surface before broad fixes. | Focused report-family visual diff and operator snapshot coverage from 0144. |
| 5 | Scan image/color parity | `images-color` + `scan`: 5 blockers plus 3 typed codec errors. | Separate resampling/color-conversion drift from unsupported CCITT/JBIG2/JPX codec policy. | Image visual diff subset plus typed unsupported checks for deferred codecs. |
| 6 | Page geometry drift | `page-geometry`: 9 blockers across office, scan, presentation, report, and browser-print. | Audit rotation, user-unit, crop-box, and linearized first-page transform parity by fixture. | Page geometry visual subset and `page_transform` unit tests. |
| 7 | Remaining vector/transparency boundaries | 3 vector blockers, 1 transparency blocker, 3 native unsupported vector/transparency errors. | Keep gradients/shadings with accepted low-amplitude drift separate from high-delta vector and soft-mask/blend work. | Vector/transparency visual subset plus typed unsupported feature tests. |
| 8 | Document structure and policy boundaries | 1 hybrid-reference blocker, 1 encrypted both-error, 1 dynamic XFA native error. | Keep encryption and dynamic XFA as policy boundaries; investigate hybrid visual parity separately. | Metadata/render policy tests plus focused hybrid-reference visual diff. |

## Operator-Audit Routing

Milestone 0144 found no fully unsupported content-stream operators in the
scanned generated corpus. The next fidelity work should therefore focus on
partial operator semantics instead of broad operator discovery:

| Operator group | Count | Bucket | Backlog tie-in |
| --- | ---: | --- | --- |
| `gs` | 33 | `graphics.transparency` | Transparency and overprint visual parity. |
| `W` / `W*` | 29 | `graphics.stroke-clip` | Dense tables, drawings, and page-geometry clipping drift. |
| `cs`, `CS`, `scn`, `SCN` | 18 | `image.color-space` | Color-space, spot-color, and pattern-color parity. |
| `sh` | 5 | `graphics.pattern-shading` | Vector/shading follow-up work. |

## Office Corpus Refresh Delta

Milestone 0145 expanded `office-export` from 44 to 47 fixtures with mixed
Word/LibreOffice, spreadsheet, and presentation-handout coverage. The native
supported gate remains green at 47/47 rendered, 0 fallbacks, and 0 errors.

The focused office visual oracle now reports:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `text-fonts` | 23 | 2 | 21 | 0 |
| `rendering-core` | 20 | 1 | 19 | 0 |
| `page-geometry` | 3 | 0 | 3 | 0 |
| `vector-graphics` | 1 | 0 | 1 | 0 |

This reinforces the first two backlog slices. The three new fixtures should be
used as representative reductions for header/footer/link composition,
spreadsheet chart grids, and presentation handout layout.

## Browser Print Corpus Refresh Delta

Milestone 0146 expanded `browser-print` from 8 to 11 fixtures with Chromium,
Firefox, and WebKit-style synthetic print reductions. The native supported gate
remains green at 11/11 rendered, 0 fallbacks, and 0 errors.

The focused browser visual oracle now reports:

| Subsystem | Total | Exact | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `images-color` | 1 | 1 | 0 | 0 | 0 |
| `page-geometry` | 3 | 0 | 2 | 1 | 0 |
| `rendering-core` | 4 | 0 | 1 | 3 | 0 |
| `text-fonts` | 1 | 0 | 0 | 1 | 0 |
| `vector-graphics` | 2 | 1 | 1 | 0 | 0 |

Use the new browser fixtures as reductions for CSS backgrounds, table/grid
rules, clipped overflow, chart geometry, link appearances, and form-like print
controls.

## Scanner OCR Corpus Refresh Delta

Milestone 0147 added a focused scanner/OCR workflow manifest with seven
supported families and one unsupported codec-backlog family. The supported gate
is green at 10/10 rendered, 0 fallbacks, 0 errors, and 0 benchmark budget
failures.

The unsupported codec backlog remains explicit:

| Family | Total | Fallback required | Bucket |
| --- | ---: | ---: | --- |
| `unsupported-filter` | 3 | 3 | `image.filter` |

The focused scanner visual oracle reports 6 blockers across scan resampling,
page geometry/skew parity, and overlay composition. Use
`scanner-skewed-mailroom-page.pdf`, `scanner-large-image-budget.pdf`, and
`scanner-ocr-form-overlay.pdf` as reductions for those follow-up slices.

## Government Form And Certificate Delta

Milestone 0148 added a focused government/form manifest with three new
synthetic fixtures for permit forms, certificate pages, and tax notices. The
supported gate is green at 8/8 rendered, 0 fallbacks, and 0 errors.

The unsupported dynamic-form boundary remains explicit:

| Family | Total | Fallback required | Bucket |
| --- | ---: | ---: | --- |
| `dynamic-xfa-unsupported` | 1 | 1 | `form.xfa-dynamic` |

The focused government visual oracle reports 6 blockers across
`annotations-forms` and `rendering-core`. Use
`government-permit-checkbox-form.pdf`,
`government-certificate-seal-signature.pdf`, and
`government-tax-notice-barcode.pdf` as reductions for widget appearance,
signature/seal composition, line/table geometry, and barcode/stamp parity.

## Financial Report And Statement Delta

Milestone 0149 added a focused financial-document manifest with three new
synthetic fixtures for annual-report, cashflow-statement, and KPI chart-summary
pages. The supported gate is green at 8/8 rendered, 0 fallbacks, and 0 errors;
the dense-page benchmark reports 0 budget failures.

The focused financial visual oracle reports 8 blockers:

| Subsystem | Total | Blockers | Native errors |
| --- | ---: | ---: | ---: |
| `page-geometry` | 2 | 2 | 0 |
| `rendering-core` | 5 | 5 | 0 |
| `text-fonts` | 1 | 1 | 0 |

Use `financial-annual-report-page.pdf`,
`financial-cashflow-statement.pdf`, and `financial-chart-summary.pdf` as
reductions for decimal text alignment, dense table-rule fidelity, chart vector
geometry, and report page-geometry parity.

## Academic Publisher Corpus Delta

Milestone 0150 added a focused academic-publisher manifest with three new
synthetic fixtures for publisher article first pages, equation/symbol pages,
and references/appendix pages. The supported gate is green at 9/9 rendered,
0 fallbacks, and 0 errors; the benchmark reports 0 budget failures.

The focused academic visual oracle reports 1 accepted drift row and 8 blockers:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `page-geometry` | 2 | 0 | 2 | 0 |
| `rendering-core` | 5 | 0 | 5 | 0 |
| `text-fonts` | 2 | 1 | 1 | 0 |

Use `academic-publisher-first-page.pdf`,
`academic-equation-symbols-page.pdf`, and
`academic-references-appendix.pdf` as reductions for multi-column layout,
small-text metrics, equation/symbol placement, and figure/vector fidelity.

## Engineering Drawing Precision Delta

Milestone 0151 expanded the technical-drawing manifest with three new
synthetic engineering fixtures for floorplans, schematic symbols, and
large-coordinate transform details. The supported gate is green at 11/11
rendered, 0 fallbacks, and 0 errors; the vector benchmark reports 0 budget
failures.

The focused engineering visual oracle reports 2 exact rows and 9 blockers:

| Subsystem | Total | Exact | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `page-geometry` | 1 | 0 | 1 | 0 |
| `rendering-core` | 7 | 0 | 7 | 0 |
| `vector-graphics` | 3 | 2 | 1 | 0 |

Use `engineering-floorplan-precision.pdf`,
`engineering-schematic-symbols.pdf`, and
`engineering-large-transform-detail.pdf` as reductions for thin-stroke
placement, dashed grids, repeated symbols, and large-coordinate transform
parity.

## Geospatial Map Rendering Delta

Milestone 0152 added a focused map-rendering manifest with three new synthetic
fixtures for raster-tile routes, transparent zoning overlays, and deterministic
simple OCG layer-off policy. The supported gate is green at 7/7 rendered,
0 fallbacks, and 0 errors; the benchmark reports 0 budget failures.

The OCMD optional-content membership policy remains explicit:

| Family | Total | Fallback required | Bucket |
| --- | ---: | ---: | --- |
| `unsupported-optional-policy` | 1 | 1 | `graphics.optional-content` |

The focused map visual oracle reports 2 exact rows and 5 blockers:

| Subsystem | Total | Exact | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `rendering-core` | 5 | 0 | 5 | 0 |
| `vector-graphics` | 2 | 2 | 0 | 0 |

Use `map-raster-tile-routes.pdf`, `map-transparent-zoning-overlay.pdf`, and
`map-optional-layer-policy.pdf` as reductions for raster tile placement,
transparent overlays, label/route parity, and deterministic OCG layer handling.

## E-Signature Workflow Delta

Milestone 0153 added a focused e-signature workflow manifest with three new
synthetic fixtures for contract signing, audit-trail certificates, and
incrementally updated signed revisions. The supported gate is green at 5/5
rendered, 0 fallbacks, and 0 errors; the benchmark reports 0 budget failures.

The signature validation boundary remains explicit: signature fields and
`/ByteRange` are reported as presence-only metadata, and the native renderer
does not validate cryptographic trust, digest contents, timestamps, or legal
signature status.

The focused e-signature visual oracle reports 1 accepted drift row and
4 blockers:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `annotations-forms` | 5 | 1 | 4 | 0 |

Use `e-signature-contract-workflow.pdf`,
`e-signature-audit-certificate.pdf`, and
`e-signature-incremental-revision.pdf` as reductions for static signature
appearance, stamp appearance, audit-table text/grid parity, and incremental
catalog/page revision rendering.

## Tagged PDF Visual Integrity Delta

Milestone 0154 added a focused tagged-PDF visual manifest with four new
synthetic fixtures for report, form, office figure/alt-text, and
structure-heavy tagged documents. The supported gate is green at 5/5 rendered,
0 fallbacks, and 0 errors; the default benchmark and the structure-heavy
low-memory benchmark both report 0 budget failures.

The accessibility boundary remains explicit: tagged metadata, RoleMap, alt
text, and reading-order structures are diagnostics inputs, not visual drawing
commands. Full reading-order extraction remains deferred to milestone 0182.

The focused tagged visual oracle reports 1 accepted drift row and 4 blockers:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `rendering-core` | 4 | 1 | 3 | 0 |
| `text-fonts` | 1 | 0 | 1 | 0 |

Use `tagged-report-visual-integrity.pdf`,
`tagged-form-visual-integrity.pdf`, `tagged-office-alt-text.pdf`, and
`tagged-structure-heavy-report.pdf` as reductions for marked-content visual
parity, tagged form/widget rendering, figure/table text metrics, and bounded
structure traversal.

## Typed Unsupported Boundaries

| Feature bucket | Fixtures | Current decision |
| --- | --- | --- |
| `image.filter` | `unsupported-ccitt-image.pdf`, `unsupported-jbig2-image.pdf`, `unsupported-jpx-image.pdf` | Defer until safe codec strategy and scan corpus need justify implementation. |
| `graphics.transparency` | `extgstate-luminosity-soft-mask.pdf`, `unsupported-blend-mode.pdf` | Keep typed unsupported until blend/soft-mask support has bounded raster tests. |
| `graphics.optional-content` | `optional-content-ocmd.pdf` | Needs explicit layer membership and flattening policy before rendering. |
| `graphics.pattern-shading` | `mesh-shading-unsupported.pdf` | Keep as vector/shading follow-up rather than fallback. |
| `form.xfa-dynamic` | `xfa-dynamic-no-static-appearance.pdf` | Dynamic XFA stays unsupported unless a separate policy milestone changes scope. |

## Not Release Blockers By Themselves

| Area | Rationale |
| --- | --- |
| Low-amplitude gradients and mesh drift | Current accepted-drift rows have low mean absolute error and low p95 deltas despite broad changed-pixel ratios. |
| OCR invisible text layer drift | Visual output remains native-renderable; text extraction/search parity is a later API milestone. |
| Encrypted placeholder fixture | Both native and PDFium return encrypted; this is a document-security policy boundary. |
| Dynamic XFA without static appearance | Not part of the normal server-side thumbnail rendering target unless explicitly reprioritized. |

## Backlog Use

Each implementation milestone should pick one row, add or reuse a focused
manifest where useful, and keep the native-only supported-family gate green.
Broad corpus visual improvement is measured by reducing blocker count without
loosening thresholds.
