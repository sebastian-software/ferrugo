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
