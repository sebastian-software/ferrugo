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
| 0087 | [Font Hinting Glyph Cache And Subpixel Policy](0087-font-hinting-glyph-cache-and-subpixel-policy.md) | 15 | medium | 0086 |
| 0088 | [Image Mask Stencil And Bitmap Edge Cases](0088-image-mask-stencil-and-bitmap-edge-cases.md) | 15 | medium | 0087 |
| 0089 | [JPX JBIG2 And Specialized Image Codec Policy](0089-jpx-jbig2-and-specialized-image-codec-policy.md) | 15 | medium | 0088 |
| 0090 | [Shading Mesh Gradient And Pattern Fidelity](0090-shading-mesh-gradient-and-pattern-fidelity.md) | 15 | medium | 0089 |
| 0091 | [Annotation Without Appearance Fallbacks](0091-annotation-without-appearance-fallbacks.md) | 15 | medium | 0090 |
| 0092 | [AcroForm Field Synthesis For Common Widgets](0092-acroform-field-synthesis-for-common-widgets.md) | 15 | medium | 0091 |
| 0093 | [Incremental Update Edge Case Hardening](0093-incremental-update-edge-case-hardening.md) | 16 | medium | 0092 |
| 0094 | [Structure Outline Metadata And Page Labels](0094-structure-outline-metadata-and-page-labels.md) | 16 | small | 0093 |
| 0095 | [Large Document Memory Eviction And Spooling](0095-large-document-memory-eviction-and-spooling.md) | 16 | medium | 0094 |
| 0096 | [Hot Path Profiling And Raster Optimization](0096-hot-path-profiling-and-raster-optimization.md) | 16 | medium | 0095 |
| 0097 | [Fuzzing And Adversarial PDF Hardening](0097-fuzzing-and-adversarial-pdf-hardening.md) | 16 | medium | 0096 |
| 0098 | [Native-Only Packaging And Consumer Migration](0098-native-only-packaging-and-consumer-migration.md) | 17 | medium | 0097 |
| 0099 | [PDFium Fallback Removal Drill](0099-pdfium-fallback-removal-drill.md) | 17 | medium | 0098 |
| 0100 | [Native Renderer General Availability Gate](0100-native-renderer-general-availability-gate.md) | 17 | medium | 0099 |
| 0101 | [Common Font Fallback And System Font Policy](0101-common-font-fallback-and-system-font-policy.md) | 18 | medium | 0100 |
| 0102 | [CFF Type1 Charstring Interpreter Hardening](0102-cff-type1-charstring-interpreter-hardening.md) | 18 | medium | 0101 |
| 0103 | [OpenType Layout Feature Coverage For PDFs](0103-opentype-layout-feature-coverage-for-pdfs.md) | 18 | medium | 0102 |
| 0104 | [Advanced CMap Encodings And Identity Mapping](0104-advanced-cmap-encodings-and-identity-mapping.md) | 18 | medium | 0103 |
| 0105 | [Separation DeviceN And Spot Color Approximation](0105-separation-devicen-and-spot-color-approximation.md) | 19 | medium | 0104 |
| 0106 | [ICC Profile Cache And Transform Optimization](0106-icc-profile-cache-and-transform-optimization.md) | 19 | medium | 0105 |
| 0107 | [Tiling Patterns And Pattern Color Spaces](0107-tiling-patterns-and-pattern-color-spaces.md) | 19 | medium | 0106 |
| 0108 | [Advanced Shading Mesh Tessellation](0108-advanced-shading-mesh-tessellation.md) | 19 | medium | 0107 |
| 0109 | [Transparency Isolation Knockout And Luminosity Masks](0109-transparency-isolation-knockout-and-luminosity-masks.md) | 19 | medium | 0108 |
| 0110 | [Overprint Simulation For Print-Oriented PDFs](0110-overprint-simulation-for-print-oriented-pdfs.md) | 19 | medium | 0109 |
| 0111 | [XFA And Dynamic Form Fallback Policy](0111-xfa-and-dynamic-form-fallback-policy.md) | 20 | small | 0110 |
| 0112 | [Digital Signature Appearance And Validation Boundary](0112-digital-signature-appearance-and-validation-boundary.md) | 20 | small | 0111 |
| 0113 | [Embedded Files Portfolio And Attachment Visibility](0113-embedded-files-portfolio-and-attachment-visibility.md) | 20 | medium | 0112 |
| 0114 | [Linearized PDF Fast First Page Loading](0114-linearized-pdf-fast-first-page-loading.md) | 20 | medium | 0113 |
| 0115 | [Multi-Page Document Scheduler And Cancellation](0115-multi-page-document-scheduler-and-cancellation.md) | 20 | medium | 0114 |
| 0116 | [OCR Text Layer And Invisible Text Handling](0116-ocr-text-layer-and-invisible-text-handling.md) | 21 | medium | 0115 |
| 0117 | [Tagged PDF Metadata And Accessibility Signals](0117-tagged-pdf-metadata-and-accessibility-signals.md) | 21 | medium | 0116 |
| 0118 | [Real Corpus Acquisition And Privacy Review Loop](0118-real-corpus-acquisition-and-privacy-review-loop.md) | 21 | medium | 0117 |
| 0119 | [Cross-Platform Rendering Determinism Gate](0119-cross-platform-rendering-determinism-gate.md) | 21 | medium | 0118 |
| 0120 | [PDFium-Free Maintenance Gate And Deletion Backlog](0120-pdfium-free-maintenance-gate-and-deletion-backlog.md) | 21 | medium | 0119 |

## In Progress

| ID | Milestone | Phase | Size | Depends On |
| --- | --- | --- | --- | --- |

## Done

| ID | Milestone | Phase | Size | Completed |
| --- | --- | --- | --- | --- |
| 0086 | [Type3 Font And CharProc Rendering](0086-type3-font-and-charproc-rendering.md) | 15 | medium | 2026-06-25 |
| 0085 | [Page Geometry Boxes Rotation And User Units](0085-page-geometry-boxes-rotation-and-user-units.md) | 15 | medium | 2026-06-24 |
| 0084 | [Visual Diff Dashboard And Review Workflow](0084-visual-diff-dashboard-and-review-workflow.md) | 14 | medium | 2026-06-24 |
| 0083 | [Real-World Corpus Ingestion And Classification](0083-real-world-corpus-ingestion-and-classification.md) | 14 | medium | 2026-06-24 |
| 0082 | [Native Default API And CLI Stabilization](0082-native-default-api-and-cli-stabilization.md) | 14 | medium | 2026-06-24 |
| 0081 | [RC Gap Synthesis And PDFium Retirement Backlog](0081-rc-gap-synthesis-and-pdfium-retirement-backlog.md) | 14 | small | 2026-06-24 |
| 0080 | [Native Renderer Release Candidate Gate](0080-native-renderer-release-candidate-gate.md) | 13 | medium | 2026-06-24 |
| 0079 | [Optional PDFium Build Feature Split](0079-optional-pdfium-build-feature-split.md) | 13 | medium | 2026-06-24 |
| 0078 | [Renderer Benchmark Suite And Budgets](0078-renderer-benchmark-suite-and-budgets.md) | 13 | medium | 2026-06-24 |
| 0077 | [Parallel Page Rendering Scheduler](0077-parallel-page-rendering-scheduler.md) | 12 | medium | 2026-06-24 |
| 0076 | [Streaming Parse And Incremental Rendering](0076-streaming-parse-and-incremental-rendering.md) | 12 | medium | 2026-06-24 |
| 0075 | [Color Management And Output Intent Policy](0075-color-management-and-output-intent-policy.md) | 12 | medium | 2026-06-24 |
| 0074 | [Annotation And Form Interaction Coverage](0074-annotation-and-form-interaction-coverage.md) | 12 | medium | 2026-06-24 |
| 0073 | [Table And Report Layout Fidelity](0073-table-and-report-layout-fidelity.md) | 12 | medium | 2026-06-24 |
| 0072 | [Vector Graphics Stress Coverage](0072-vector-graphics-stress-coverage.md) | 11 | medium | 2026-06-24 |
| 0071 | [Transparency Stack Fidelity Gate](0071-transparency-stack-fidelity-gate.md) | 11 | medium | 2026-06-24 |
| 0070 | [Bidirectional And Shaped Text Policy](0070-bidirectional-and-shaped-text-policy.md) | 11 | medium | 2026-06-24 |
| 0069 | [Vertical And CJK Text Coverage](0069-vertical-and-cjk-text-coverage.md) | 11 | medium | 2026-06-24 |
| 0068 | [Complex Font Subsetting And CID Fonts](0068-complex-font-subsetting-and-cid-fonts.md) | 11 | medium | 2026-06-24 |
| 0067 | [Mixed Text Image Page Fidelity](0067-mixed-text-image-page-fidelity.md) | 10 | medium | 2026-06-24 |
| 0066 | [Scanned Document And Large Image Coverage](0066-scanned-document-and-large-image-coverage.md) | 10 | medium | 2026-06-24 |
| 0065 | [Browser Print Document Coverage](0065-browser-print-document-coverage.md) | 10 | medium | 2026-06-24 |
| 0064 | [Office Export Document Coverage](0064-office-export-document-coverage.md) | 10 | medium | 2026-06-24 |
| 0063 | [Corpus Taxonomy And Sampling Expansion](0063-corpus-taxonomy-and-sampling-expansion.md) | 9 | medium | 2026-06-24 |
| 0062 | [PDFium Fallback Telemetry And Kill Switch](0062-pdfium-fallback-telemetry-and-kill-switch.md) | 9 | small | 2026-06-24 |
| 0061 | [Native Renderer Default Rollout](0061-native-renderer-default-rollout.md) | 9 | medium | 2026-06-24 |
| 0060 | [PDFium Retirement Gate](0060-pdfium-retirement-gate.md) | 8 | medium | 2026-06-24 |
| 0059 | [Native Backend Facade Parity](0059-native-backend-facade-parity.md) | 8 | medium | 2026-06-24 |
| 0058 | [Renderer Cache And Memory Budgets](0058-renderer-cache-and-memory-budgets.md) | 8 | medium | 2026-06-24 |
| 0057 | [Malformed PDF Recovery Budget](0057-malformed-pdf-recovery-budget.md) | 7 | medium | 2026-06-24 |
| 0056 | [Encryption And Permissions Policy](0056-encryption-and-permissions-policy.md) | 7 | small | 2026-06-24 |
| 0055 | [Incremental Updates And Hybrid References](0055-incremental-updates-and-hybrid-references.md) | 7 | medium | 2026-06-24 |
| 0054 | [Optional Content And Layer Policy](0054-optional-content-and-layer-policy.md) | 7 | medium | 2026-06-24 |
| 0053 | [AcroForm Appearance Rendering](0053-acroform-appearance-rendering.md) | 7 | medium | 2026-06-24 |
| 0052 | [Annotation Appearance Rendering](0052-annotation-appearance-rendering.md) | 7 | medium | 2026-06-24 |
| 0051 | [Advanced Stroke And Clipping Fidelity](0051-advanced-stroke-and-clipping-fidelity.md) | 6 | medium | 2026-06-24 |
| 0050 | [Patterns Shadings And Gradients](0050-patterns-shadings-and-gradients.md) | 6 | medium | 2026-06-24 |
| 0049 | [Blend Modes And Overprint Policy](0049-blend-modes-and-overprint-policy.md) | 6 | medium | 2026-06-24 |
| 0048 | [Soft Masks And Transparency Groups](0048-soft-masks-and-transparency-groups.md) | 6 | medium | 2026-06-24 |
| 0047 | [Image Filter Coverage](0047-image-filter-coverage.md) | 6 | medium | 2026-06-24 |
| 0046 | [Color Spaces And Decode Arrays](0046-color-spaces-and-decode-arrays.md) | 6 | medium | 2026-06-24 |
| 0045 | [Complex Text Positioning Baseline](0045-complex-text-positioning-baseline.md) | 5 | medium | 2026-06-24 |
| 0044 | [Glyph Outline Extraction](0044-glyph-outline-extraction.md) | 5 | medium | 2026-06-24 |
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
| 0038 | [Image Rasterization And Alpha](0038-image-rasterization-and-alpha.md) | 3 | medium | 2026-06-24 |
| 0039 | [Basic Text Rasterization](0039-basic-text-rasterization.md) | 3 | medium | 2026-06-24 |
| 0040 | [Typical Document Coverage Gate](0040-typical-document-coverage-gate.md) | 4 | medium | 2026-06-24 |
| 0041 | [Renderer Gap Triage And Support Matrix](0041-renderer-gap-triage-and-support-matrix.md) | 5 | small | 2026-06-24 |
| 0041a | [Inline Image Stream Execution](0041a-inline-image-stream-execution.md) | 5 | small | 2026-06-24 |
| 0041b | [Form XObject Native Composition](0041b-form-xobject-native-composition.md) | 5 | small | 2026-06-24 |
| 0042 | [Font Program Loading](0042-font-program-loading.md) | 5 | medium | 2026-06-24 |
| 0043 | [CMap And ToUnicode Mapping](0043-cmap-and-tounicode-mapping.md) | 5 | medium | 2026-06-24 |

## Update Rules

- When starting work, move the row from `Todo` to `In Progress` and set
  `Status: in-progress`.
- When completing work, move the row to `Done`, set `Status: done`, and fill in
  `Completion Notes` with commits, measurements, artifacts, and follow-ups.
- When blocked, move the row to a `Blocked` section if needed, set
  `Status: blocked`, and document the unblock condition.
- Keep milestones small. If a milestone grows beyond two focused days, split it.
