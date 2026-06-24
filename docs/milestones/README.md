# Milestones

This directory tracks small, stable project milestones. Milestones are intended
to feel completable: most should fit into half a day to two days of focused
work.

Do not move milestone files between `todo` and `done` directories. Keep paths
stable and update the status in both places:

1. The `Status:` field in the milestone document.
2. The table in this index.

Allowed statuses:

- `todo`
- `in-progress`
- `done`
- `blocked`

## Todo

| ID | Milestone | Phase | Size | Depends On |
| --- | --- | --- | --- | --- |
| 0038 | [Image Rasterization And Alpha](0038-image-rasterization-and-alpha.md) | 3 | medium | 0037 |
| 0039 | [Basic Text Rasterization](0039-basic-text-rasterization.md) | 3 | medium | 0038 |
| 0040 | [Typical Document Coverage Gate](0040-typical-document-coverage-gate.md) | 4 | medium | 0039 |
| 0041 | [Renderer Gap Triage And Support Matrix](0041-renderer-gap-triage-and-support-matrix.md) | 5 | small | 0040 |
| 0042 | [Font Program Loading](0042-font-program-loading.md) | 5 | medium | 0041 |
| 0043 | [CMap And ToUnicode Mapping](0043-cmap-and-tounicode-mapping.md) | 5 | medium | 0042 |
| 0044 | [Glyph Outline Extraction](0044-glyph-outline-extraction.md) | 5 | medium | 0043 |
| 0045 | [Complex Text Positioning Baseline](0045-complex-text-positioning-baseline.md) | 5 | medium | 0044 |
| 0046 | [Color Spaces And Decode Arrays](0046-color-spaces-and-decode-arrays.md) | 6 | medium | 0045 |
| 0047 | [Image Filter Coverage](0047-image-filter-coverage.md) | 6 | medium | 0046 |
| 0048 | [Soft Masks And Transparency Groups](0048-soft-masks-and-transparency-groups.md) | 6 | medium | 0047 |
| 0049 | [Blend Modes And Overprint Policy](0049-blend-modes-and-overprint-policy.md) | 6 | medium | 0048 |
| 0050 | [Patterns Shadings And Gradients](0050-patterns-shadings-and-gradients.md) | 6 | medium | 0049 |
| 0051 | [Advanced Stroke And Clipping Fidelity](0051-advanced-stroke-and-clipping-fidelity.md) | 6 | medium | 0050 |
| 0052 | [Annotation Appearance Rendering](0052-annotation-appearance-rendering.md) | 7 | medium | 0051 |
| 0053 | [AcroForm Appearance Rendering](0053-acroform-appearance-rendering.md) | 7 | medium | 0052 |
| 0054 | [Optional Content And Layer Policy](0054-optional-content-and-layer-policy.md) | 7 | medium | 0053 |
| 0055 | [Incremental Updates And Hybrid References](0055-incremental-updates-and-hybrid-references.md) | 7 | medium | 0054 |
| 0056 | [Encryption And Permissions Policy](0056-encryption-and-permissions-policy.md) | 7 | small | 0055 |
| 0057 | [Malformed PDF Recovery Budget](0057-malformed-pdf-recovery-budget.md) | 7 | medium | 0056 |
| 0058 | [Renderer Cache And Memory Budgets](0058-renderer-cache-and-memory-budgets.md) | 8 | medium | 0057 |
| 0059 | [Native Backend Facade Parity](0059-native-backend-facade-parity.md) | 8 | medium | 0058 |
| 0060 | [PDFium Retirement Gate](0060-pdfium-retirement-gate.md) | 8 | medium | 0059 |

## In Progress

No milestones are currently in progress.

## Done

| ID | Milestone | Phase | Size | Completed |
| --- | --- | --- | --- | --- |
| 0001 | [Milestone Tracking Structure](0001-milestone-tracking-structure.md) | 0 | small | 2026-06-24 |
| 0002 | [Research And Porting Baseline](0002-research-and-porting-baseline.md) | 0 | small | 2026-06-24 |
| 0003 | [Phase 0 Decision Baseline](0003-phase-0-decision-baseline.md) | 0 | small | 2026-06-24 |
| 0004 | [License Files And Attribution Policy](0004-license-files-and-attribution-policy.md) | 0 | small | 2026-06-24 |
| 0005 | [PDFium Source Checkout Recipe](0005-pdfium-source-checkout-recipe.md) | 0 | small | 2026-06-24 |
| 0006 | [Minimal PDFium GN Configuration](0006-minimal-pdfium-gn-configuration.md) | 0 | small | 2026-06-24 |
| 0007 | [PDFium Build Measurement Baseline](0007-pdfium-build-measurement-baseline.md) | 0 | medium | 2026-06-24 |
| 0008 | [Fixture Policy And Seed Fixtures](0008-fixture-policy-and-seed-fixtures.md) | 0 | small | 2026-06-24 |
| 0009 | [Rust Workspace Skeleton](0009-rust-workspace-skeleton.md) | 0 | small | 2026-06-24 |
| 0010 | [Thumbnail API Facade](0010-thumbnail-api-facade.md) | 0 | small | 2026-06-24 |
| 0011 | [PDFium Backend Linkage](0011-pdfium-backend-linkage.md) | 0 | medium | 2026-06-24 |
| 0012 | [Render Page Zero To RGBA](0012-render-page-zero-to-rgba.md) | 0 | small | 2026-06-24 |
| 0013 | [PNG Output CLI](0013-png-output-cli.md) | 0 | small | 2026-06-24 |
| 0014 | [Error Taxonomy Mapping](0014-error-taxonomy-mapping.md) | 0 | small | 2026-06-24 |
| 0015 | [Differential Baseline Format](0015-differential-baseline-format.md) | 0 | small | 2026-06-24 |
| 0016 | [Phase 0 Report And Pivot Decision](0016-phase-0-report-and-pivot-decision.md) | 0 | small | 2026-06-24 |
| 0017 | [Run Local PDFium Build](0017-run-local-pdfium-build.md) | 1 | medium | 2026-06-24 |
| 0018 | [Live Thumbnail Fixture Render](0018-live-thumbnail-fixture-render.md) | 1 | small | 2026-06-24 |
| 0019 | [Timeout And Isolation Decision](0019-timeout-and-isolation-decision.md) | 1 | small | 2026-06-24 |
| 0020 | [Child Process Render Runner](0020-child-process-render-runner.md) | 1 | medium | 2026-06-24 |
| 0021 | [Rust Native Crate Layout](0021-rust-native-crate-layout.md) | 1 | small | 2026-06-24 |
| 0022 | [Byte Input And Offset Errors](0022-byte-input-and-offset-errors.md) | 1 | small | 2026-06-24 |
| 0023 | [PDF Primitive Parser](0023-pdf-primitive-parser.md) | 1 | medium | 2026-06-24 |
| 0024 | [Indirect Objects And References](0024-indirect-objects-and-references.md) | 1 | medium | 2026-06-24 |
| 0025 | [Classic Xref And Trailer Loader](0025-classic-xref-and-trailer-loader.md) | 1 | medium | 2026-06-24 |
| 0026 | [Streams And Basic Filters](0026-streams-and-basic-filters.md) | 1 | medium | 2026-06-24 |
| 0027 | [Xref Streams And Object Streams](0027-xref-streams-and-object-streams.md) | 1 | medium | 2026-06-24 |
| 0028 | [Catalog And Page Tree](0028-catalog-and-page-tree.md) | 1 | medium | 2026-06-24 |
| 0029 | [Rust Backend Differential Harness](0029-rust-backend-differential-harness.md) | 1 | medium | 2026-06-24 |
| 0030 | [Content Stream Tokenizer](0030-content-stream-tokenizer.md) | 2 | medium | 2026-06-24 |
| 0031 | [Graphics State And Transforms](0031-graphics-state-and-transforms.md) | 2 | medium | 2026-06-24 |
| 0032 | [Path Display List](0032-path-display-list.md) | 2 | medium | 2026-06-24 |
| 0033 | [Text State And Font Stubs](0033-text-state-and-font-stubs.md) | 2 | medium | 2026-06-24 |
| 0034 | [Image XObject Decoding And Placement](0034-image-xobject-decoding-and-placement.md) | 2 | medium | 2026-06-24 |
| 0035 | [Form XObject Recursion And Budgets](0035-form-xobject-recursion-and-budgets.md) | 2 | medium | 2026-06-24 |
| 0036 | [Raster Device And Page Transform](0036-raster-device-and-page-transform.md) | 3 | medium | 2026-06-24 |
| 0037 | [Path Rasterization](0037-path-rasterization.md) | 3 | medium | 2026-06-24 |

## Update Rules

- When starting work, move the row from `Todo` to `In Progress` and set
  `Status: in-progress`.
- When completing work, move the row to `Done`, set `Status: done`, and fill in
  `Completion Notes` with commits, measurements, artifacts, and follow-ups.
- When blocked, move the row to a `Blocked` section if needed, set
  `Status: blocked`, and document the unblock condition.
- Keep milestones small. If a milestone grows beyond two focused days, split it.
