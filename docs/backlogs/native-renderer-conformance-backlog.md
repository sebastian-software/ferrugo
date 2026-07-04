# Native Renderer Conformance Backlog

Status: accepted for 0143.
Date: 2026-06-26.

This backlog turns the 0143 visual oracle result into small native-renderer
follow-up slices. Counts come from
`target/conformance-0143-visual-diff.json`.

## Priority Slices

| Rank | Slice | Evidence | Recommended next action | Validation gate |
| ---: | --- | --- | --- | --- |
| 1 | Office text/font fidelity | `text-fonts` + `office-export`: missing-font substitution split out with office family classification coverage. | Continue auditing fallback text metrics, text spacing, subset font widths, and Type1/CFF/CID positioning as separate fixture groups. | Focused visual diff over office text fixtures plus `cargo test -p ferrugo-render text_display_list fallback_font_classification`. |
| 2 | Dense office table/grid rendering | `rendering-core` + `office-export`: rectangular text clipping reduced for spreadsheet cell overflow. | Continue splitting spreadsheet/table fixtures into remaining operator semantics, hairline/grid stroke, dense layout, and broad visual parity cases. | Focused visual diff and native benchmark over spreadsheet-grid fixtures plus clipped text regression tests. |
| 3 | Form and annotation appearance parity | `annotations-forms` + `form`: 11 remaining blockers after FreeText missing-appearance synthesis. | Compare native synthesized appearances against explicit appearance streams; isolate checkbox/radio/text-field/signature/stamp differences. | Focused visual diff over form fixtures and native form appearance tests. |
| 4 | Report rendering-core fidelity | `rendering-core` + `report`: 12 blockers. | Triage scientific, long-report, technical, and dashboard fixtures by operator surface before broad fixes. | Focused report-family visual diff and operator snapshot coverage from 0144. |
| 5 | Scan image/color parity | `images-color` + `scan`: 5 blockers; CCITT/JBIG2/JPX are now explicit typed codec boundaries. | Continue resampling/color-conversion drift work separately from the deferred codec policy. | Image visual diff subset plus `scripts/check_codec_transparency_boundaries.sh`. |
| 6 | Page geometry drift | `page-geometry`: 9 blockers across office, scan, presentation, report, and browser-print. | Audit rotation, user-unit, crop-box, and linearized first-page transform parity by fixture. | Page geometry visual subset and `page_transform` unit tests. |
| 7 | Remaining vector/transparency boundaries | 3 vector blockers, 1 transparency blocker; luminosity soft masks and Overlay/advanced blends are now explicit typed boundaries. | Keep gradients/shadings with accepted low-amplitude drift separate from high-delta vector work and future one-semantic transparency reductions. | Vector/transparency visual subset plus `scripts/check_codec_transparency_boundaries.sh`. |
| 8 | Document structure and policy boundaries | 1 hybrid-reference blocker, 1 encrypted both-error, 1 dynamic XFA native error. | Keep encryption and dynamic XFA as policy boundaries; investigate hybrid visual parity separately. | Metadata/render policy tests plus focused hybrid-reference visual diff. |

## Operator-Audit Routing

## Office Text And Font Delta

Issue 71 isolates one missing-font substitution reduction: subset-prefixed
office family names are classified before deterministic fallback rasterization.
Cambria, Constantia, Garamond, and Minion route to the serif fallback; Wingdings
and Webdings route to the symbol fallback; Aptos and unknown families continue
to use the sans fallback. This reduces family-level substitution drift without
adding host font discovery or full OpenType shaping to the runtime graph.

## Dense Table And Report Delta

Issue 72 reduces one rendering-core blocker group for dense tables: active
rectangular clips now apply to fallback text glyph rectangles and Type3 CharProc
glyph paths during ordered display-list rasterization. This targets spreadsheet
cell-overflow reductions such as `spreadsheet-clipped-cells.pdf`, where `W n`
clips precede clipped cell text. Remaining dense table/report work still covers
hairline/grid stroke parity, table operator semantics, chart/report layout
drift, and broad visual threshold tightening.

## Static Form And Annotation Delta

Issue 69 adds bounded native synthesis for appearance-free FreeText annotations.
Existing explicit appearance streams remain authoritative, and dynamic XFA plus
viewer-side annotation mutation stay out of scope. Current focused annotation
manifests now classify `freetext-annotation-without-appearance.pdf` as
`expected:native`; remaining parity work is visual fidelity for synthesized
static forms, stamps, signatures, and explicit appearance stream matching.

## Codec And Transparency Boundary Delta

Issues 67 and 65 close the release-train ambiguity around specialized scan
codecs and advanced transparency. CCITT Fax is the first future codec candidate
when a safe decoder slice is accepted; JPX and JBIG2 remain deferred behind
isolation and safety evidence. For the current scoped runtime, CCITT/JBIG2/JPX
must continue to produce typed `image.filter` fallbacks.

Luminosity soft masks and Overlay/advanced blend modes remain typed
`graphics.transparency` boundaries. Supported transparency work remains focused
on alpha, isolated groups, knockout metadata, Multiply/Screen, blend arrays that
fall through to a supported mode, and image soft masks. Future transparency
reductions should add one semantic at a time with visual oracle evidence and
explicit intermediate-surface budgets.

The shared executable gate is
`scripts/check_codec_transparency_boundaries.sh`; it combines supported
fallback-summary coverage, typed unsupported assertions, low-memory/profile
coverage, focused transparency visual comparison, and fuzz smoke coverage.

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

Simple OCMD optional-content membership is now part of the native supported
slice:

| Family | Total | Fallback required | Bucket |
| --- | ---: | ---: | --- |
| `optional-membership` | 1 | 0 | `none` |

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
commands. Full reading-order extraction remains deferred to the tagged-PDF
extraction backlog.

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
| `graphics.optional-content` | `optional-content-usage-application.pdf`, OCMD `/VE` or unknown `/P` policies | Usage application and viewer-state policies stay typed unsupported until thumbnail flattening has an explicit viewer-state contract. |
| `graphics.pattern-shading` | `mesh-shading-unsupported.pdf` | Keep as vector/shading follow-up rather than fallback. |
| `form.xfa-dynamic` | `xfa-dynamic-no-static-appearance.pdf` | Dynamic XFA stays unsupported unless a separate policy decision changes scope. |

## Not Release Blockers By Themselves

| Area | Rationale |
| --- | --- |
| Low-amplitude gradients and mesh drift | Current accepted-drift rows have low mean absolute error and low p95 deltas despite broad changed-pixel ratios. |
| OCR invisible text layer drift | Visual output remains native-renderable; text extraction/search parity is later API work. |
| Encrypted placeholder fixture | Both native and PDFium return encrypted; this is a document-security policy boundary. |
| Dynamic XFA without static appearance | Not part of the normal server-side thumbnail rendering target unless explicitly reprioritized. |

## Backlog Use

Each implementation slice should pick one row, add or reuse a focused manifest
where useful, and keep the native-only supported-family gate green.
Broad corpus visual improvement is measured by reducing blocker count without
loosening thresholds.
