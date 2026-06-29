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

## Execution Focus

The primary product path is server-side, PDFium-free rendering with a
Rust-native renderer. Work should prioritize features that improve typical
office, browser-print, scan, report, contract, form, and batch-thumbnail
documents in server deployments.

WASM, embedded, mobile, and low-memory profiles remain useful compatibility
signals, but they are secondary unless a milestone explicitly says otherwise.
They should not block server-side PDFium replacement gates on their own. Treat
them as profile checks, packaging constraints, or follow-up optimization work
when the server renderer is already correct and bounded.

## Todo

| ID | Milestone | Phase | Size | Depends On |
| --- | --- | --- | --- | --- |
| 0196 | [WASM Low Memory Mobile Browser Gate](0196-wasm-low-memory-mobile-browser-gate.md) | 37 | medium | 0195 |
| 0213 | [Transparency Stack Memory Optimization](0213-transparency-stack-memory-optimization.md) | 40 | medium | 0212 |
| 0214 | [Incremental Parser And Object Cache Fusion](0214-incremental-parser-and-object-cache-fusion.md) | 40 | medium | 0213 |
| 0215 | [PDFium Comparison Tool Removal Decision Gate](0215-pdfium-comparison-tool-removal-decision-gate.md) | 40 | medium | 0214 |
| 0216 | [Cross-Producer Typical Document Fusion Corpus](0216-cross-producer-typical-document-fusion-corpus.md) | 41 | medium | 0215 |
| 0217 | [Low-End Device Reliability Sweep](0217-low-end-device-reliability-sweep.md) | 41 | medium | 0216 |
| 0218 | [Server And WASM Scheduler Tuning Gate](0218-server-and-wasm-scheduler-tuning-gate.md) | 41 | medium | 0216 |
| 0219 | [Unsupported Feature SLA And Consumer Migration Guide](0219-unsupported-feature-sla-and-consumer-migration-guide.md) | 41 | small | 0218 |
| 0220 | [PDFium-Free 1.4 Readiness Gate](0220-pdfium-free-1-4-readiness-gate.md) | 41 | medium | 0219 |

## In Progress

| ID | Milestone | Phase | Size | Depends On |
| --- | --- | --- | --- | --- |
| 0182 | [Accessible Tagged PDF Reading Order Coverage](0182-accessible-tagged-pdf-reading-order-coverage.md) | 34 | medium | 0181 |

## Done

| ID | Milestone | Phase | Size | Completed |
| --- | --- | --- | --- | --- |
| 0212 | [Rust-Native Font Cache Compaction](0212-rust-native-font-cache-compaction.md) | 40 | medium | 2026-06-29 |
| 0211 | [PDF Operator Semantic Snapshot Suite](0211-pdf-operator-semantic-snapshot-suite.md) | 40 | medium | 2026-06-29 |
| 0210 | [Native Renderer 1.3 Hardening Gate](0210-native-renderer-1-3-hardening-gate.md) | 39 | medium | 2026-06-29 |
| 0209 | [Rust-Native Image Codec Deployment Policy](0209-rust-native-image-codec-deployment-policy.md) | 39 | medium | 2026-06-29 |
| 0208 | [Color Managed Print Preview Extended Gate](0208-color-managed-print-preview-extended-gate.md) | 39 | medium | 2026-06-29 |
| 0207 | [Annotation Popup Stamp And FreeText Fidelity](0207-annotation-popup-stamp-and-freetext-fidelity.md) | 39 | medium | 2026-06-29 |
| 0206 | [Form Filling Appearance Update And Flattening Coverage](0206-form-filling-appearance-update-and-flattening-coverage.md) | 39 | medium | 2026-06-29 |
| 0205 | [PDFium-Free 1.3 Typical Document Gate](0205-pdfium-free-1-3-typical-document-gate.md) | 38 | medium | 2026-06-29 |
| 0204 | [Office Chart SmartArt And Vector Effect Fidelity](0204-office-chart-smartart-and-vector-effect-fidelity.md) | 38 | medium | 2026-06-29 |
| 0203 | [Dense Office Table And Spreadsheet Refinement](0203-dense-office-table-and-spreadsheet-refinement.md) | 38 | medium | 2026-06-29 |
| 0202 | [Text Selection Geometry And Search Highlight Parity](0202-text-selection-geometry-and-search-highlight-parity.md) | 38 | medium | 2026-06-29 |
| 0201 | [Native Renderer 1.3 Coverage Scorecard Baseline](0201-native-renderer-1-3-coverage-scorecard-baseline.md) | 38 | medium | 2026-06-29 |
| 0200 | [PDFium-Free 1.2 Readiness Gate](0200-pdfium-free-1-2-readiness-gate.md) | 37 | medium | 2026-06-29 |
| 0199 | [Unsupported Feature Burn-Down Release Candidate Gate](0199-unsupported-feature-burn-down-release-candidate-gate.md) | 37 | medium | 2026-06-29 |
| 0198 | [Native Renderer Telemetry Privacy And Diagnostics Policy](0198-native-renderer-telemetry-privacy-and-diagnostics-policy.md) | 37 | small | 2026-06-29 |
| 0197 | [Serverless Cold Start And Binary Size Budget](0197-serverless-cold-start-and-binary-size-budget.md) | 37 | medium | 2026-06-29 |
| 0195 | [High Page Count Batch Thumbnail Gate](0195-high-page-count-batch-thumbnail-gate.md) | 36 | medium | 2026-06-29 |
| 0194 | [Forms Appearance State Mutation Boundary](0194-forms-appearance-state-mutation-boundary.md) | 36 | medium | 2026-06-29 |
| 0193 | [Annotation Print Preview Fidelity Gate](0193-annotation-print-preview-fidelity-gate.md) | 36 | medium | 2026-06-29 |
| 0192 | [Optional Content UI State And Layer Flattening Policy](0192-optional-content-ui-state-and-layer-flattening-policy.md) | 36 | medium | 2026-06-29 |
| 0191 | [DeviceN Spot Color Visual Review Samples](0191-devicen-spot-color-visual-review-samples.md) | 36 | medium | 2026-06-29 |
| 0190 | [Cross-Producer Regression Bisect Workflow](0190-cross-producer-regression-bisect-workflow.md) | 35 | medium | 2026-06-29 |
| 0189 | [Layout Stress Corpus For Tables Columns And Footnotes](0189-layout-stress-corpus-for-tables-columns-and-footnotes.md) | 35 | medium | 2026-06-29 |
| 0188 | [Resource Deduplication And Shared Object Cache](0188-resource-deduplication-and-shared-object-cache.md) | 35 | medium | 2026-06-29 |
| 0187 | [Incremental Document Streaming Memory Budget](0187-incremental-document-streaming-memory-budget.md) | 35 | medium | 2026-06-29 |
| 0186 | [Native Text Extraction And Search Parity Gate](0186-native-text-extraction-and-search-parity-gate.md) | 35 | medium | 2026-06-29 |
| 0185 | [PDF/A And Archival Document Conformance Boundary](0185-pdf-a-and-archival-document-conformance-boundary.md) | 34 | medium | 2026-06-29 |
| 0184 | [Print Shop Imposition And Booklet PDF Coverage](0184-print-shop-imposition-and-booklet-pdf-coverage.md) | 34 | medium | 2026-06-29 |
| 0183 | [Mixed Vector Raster Transparency Edge Cases](0183-mixed-vector-raster-transparency-edge-cases.md) | 34 | medium | 2026-06-29 |
| 0181 | [PDF 2.0 Feature Usage Corpus Gate](0181-pdf-2-0-feature-usage-corpus-gate.md) | 34 | medium | 2026-06-28 |
| 0180 | [PDFium-Free 1.1 Coverage Gate](0180-pdfium-free-1-1-coverage-gate.md) | 33 | medium | 2026-06-26 |
| 0179 | [Corpus Governance And Regression Dashboard](0179-corpus-governance-and-regression-dashboard.md) | 33 | medium | 2026-06-26 |
| 0178 | [Security Fuzz Nightly And Crash Triage Loop](0178-security-fuzz-nightly-and-crash-triage-loop.md) | 33 | medium | 2026-06-26 |
| 0177 | [Server-Side Batch Rendering Isolation Gate](0177-server-side-batch-rendering-isolation-gate.md) | 33 | medium | 2026-06-26 |
| 0176 | [WASM Viewer Integration Performance Gate](0176-wasm-viewer-integration-performance-gate.md) | 33 | medium | 2026-06-28 |
| 0175 | [Native Render Trace And Operator Replay Tool](0175-native-render-trace-and-operator-replay-tool.md) | 32 | medium | 2026-06-26 |
| 0174 | [Typed Unsupported Boundary API Freeze](0174-typed-unsupported-boundary-api-freeze.md) | 32 | small | 2026-06-26 |
| 0173 | [Corrupt-But-Common PDF Recovery Corpus](0173-corrupt-but-common-pdf-recovery-corpus.md) | 32 | medium | 2026-06-26 |
| 0172 | [High-DPI Thumbnail And Preview Fidelity](0172-high-dpi-thumbnail-and-preview-fidelity.md) | 32 | medium | 2026-06-26 |
| 0171 | [Long Document Navigation And Page Cache Gate](0171-long-document-navigation-and-page-cache-gate.md) | 32 | medium | 2026-06-26 |
| 0170 | [Raster Image Heavy Document Memory Gate](0170-raster-image-heavy-document-memory-gate.md) | 31 | medium | 2026-06-26 |
| 0169 | [Font Fallback Script Mixing And Emoji Coverage](0169-font-fallback-script-mixing-and-emoji-coverage.md) | 31 | medium | 2026-06-26 |
| 0168 | [Email Client And Web Archive PDF Coverage](0168-email-client-and-web-archive-pdf-coverage.md) | 31 | medium | 2026-06-26 |
| 0167 | [Browser Print CSS Edge Case Coverage](0167-browser-print-css-edge-case-coverage.md) | 31 | medium | 2026-06-26 |
| 0166 | [Office Vector Effects And Clip Mask Fidelity](0166-office-vector-effects-and-clip-mask-fidelity.md) | 31 | medium | 2026-06-26 |
| 0165 | [Native-Only CI And Release Artifact Hardening](0165-native-only-ci-and-release-artifact-hardening.md) | 30 | medium | 2026-06-26 |
| 0164 | [Independent Reference Oracle Strategy](0164-independent-reference-oracle-strategy.md) | 30 | medium | 2026-06-26 |
| 0163 | [Producer Compatibility Matrix Expansion](0163-producer-compatibility-matrix-expansion.md) | 30 | medium | 2026-06-26 |
| 0162 | [PDF 2.0 Compatibility Boundary Gate](0162-pdf-2-0-compatibility-boundary-gate.md) | 30 | medium | 2026-06-26 |
| 0161 | [Post-1.0 Unsupported Feature Triage Loop](0161-post-1-0-unsupported-feature-triage-loop.md) | 30 | medium | 2026-06-26 |
| 0160 | [PDFium-Free 1.0 Readiness Gate](0160-pdfium-free-1-0-readiness-gate.md) | 29 | medium | 2026-06-26 |
| 0159 | [SIMD Raster Hot Path Evaluation](0159-simd-raster-hot-path-evaluation.md) | 29 | medium | 2026-06-26 |
| 0158 | [Memory Arena And Scratch Buffer Audit](0158-memory-arena-and-scratch-buffer-audit.md) | 29 | medium | 2026-06-26 |
| 0157 | [Plugin-Free Distribution And Install Gate](0157-plugin-free-distribution-and-install-gate.md) | 29 | medium | 2026-06-26 |
| 0156 | [Native Renderer API And Semver Policy](0156-native-renderer-api-and-semver-policy.md) | 29 | small | 2026-06-26 |
| 0152 | [Geospatial Map PDF Rendering Coverage](0152-geospatial-map-pdf-rendering-coverage.md) | 28 | medium | 2026-06-26 |
| 0153 | [E-Signature Workflow Document Coverage](0153-e-signature-workflow-document-coverage.md) | 28 | medium | 2026-06-26 |
| 0154 | [Accessibility Tagged PDF Visual Integrity](0154-accessibility-tagged-pdf-visual-integrity.md) | 28 | medium | 2026-06-26 |
| 0155 | [Incremental Loading Interactive Preview](0155-incremental-loading-interactive-preview.md) | 28 | medium | 2026-06-26 |
| 0151 | [Engineering Drawing Precision Gate](0151-engineering-drawing-precision-gate.md) | 28 | medium | 2026-06-26 |
| 0150 | [Academic Publisher Corpus Gate](0150-academic-publisher-corpus-gate.md) | 27 | medium | 2026-06-26 |
| 0149 | [Financial Report And Statement Fidelity](0149-financial-report-and-statement-fidelity.md) | 27 | medium | 2026-06-26 |
| 0148 | [Government Form And Certificate Coverage](0148-government-form-and-certificate-coverage.md) | 27 | medium | 2026-06-26 |
| 0147 | [Scanner And OCR Workflow Corpus](0147-scanner-and-ocr-workflow-corpus.md) | 27 | medium | 2026-06-26 |
| 0146 | [Browser Print Corpus Refresh](0146-browser-print-corpus-refresh.md) | 27 | medium | 2026-06-26 |
| 0145 | [Office Suite Regression Corpus Refresh](0145-office-suite-regression-corpus-refresh.md) | 26 | medium | 2026-06-26 |
| 0144 | [Renderer Operator Coverage Audit](0144-renderer-operator-coverage-audit.md) | 26 | medium | 2026-06-26 |
| 0143 | [Native Renderer Conformance Triage Loop](0143-native-renderer-conformance-triage-loop.md) | 26 | medium | 2026-06-26 |
| 0142 | [PDFium Comparison Tooling Quarantine](0142-pdfium-comparison-tooling-quarantine.md) | 26 | medium | 2026-06-26 |
| 0141 | [PDFium Runtime Deletion Execution](0141-pdfium-runtime-deletion-execution.md) | 26 | medium | 2026-06-26 |
| 0140 | [Typical Document Coverage GA2 Gate](0140-typical-document-coverage-ga2-gate.md) | 25 | medium | 2026-06-26 |
| 0139 | [Native Renderer Security And Fuzz Refresh](0139-native-renderer-security-and-fuzz-refresh.md) | 25 | medium | 2026-06-26 |
| 0138 | [Transparency And Blend Conformance Corpus](0138-transparency-and-blend-conformance-corpus.md) | 25 | medium | 2026-06-26 |
| 0137 | [Image Downsampling And Color Conversion Optimization](0137-image-downsampling-and-color-conversion-optimization.md) | 25 | medium | 2026-06-26 |
| 0136 | [Font Subset Regression Corpus Expansion](0136-font-subset-regression-corpus-expansion.md) | 25 | medium | 2026-06-26 |
| 0135 | [Renderer Diagnostics And Debug Artifact Bundle](0135-renderer-diagnostics-and-debug-artifact-bundle.md) | 24 | medium | 2026-06-25 |
| 0134 | [Persistent Page Cache And Reuse Policy](0134-persistent-page-cache-and-reuse-policy.md) | 24 | medium | 2026-06-25 |
| 0133 | [Server-Side Batch Rendering Throughput Gate](0133-server-side-batch-rendering-throughput-gate.md) | 24 | medium | 2026-06-25 |
| 0132 | [WASM Renderer Packaging And Size Gate](0132-wasm-renderer-packaging-and-size-gate.md) | 24 | medium | 2026-06-25 |
| 0131 | [Low-Memory Embedded Renderer Profile](0131-low-memory-embedded-renderer-profile.md) | 24 | medium | 2026-06-25 |
| 0130 | [Legal Contract And Redaction Document Coverage](0130-legal-contract-and-redaction-document-coverage.md) | 23 | medium | 2026-06-25 |
| 0129 | [Mobile Scan And Camera PDF Robustness](0129-mobile-scan-and-camera-pdf-robustness.md) | 23 | medium | 2026-06-25 |
| 0128 | [Print Production And Prepress Boundary](0128-print-production-and-prepress-boundary.md) | 23 | medium | 2026-06-25 |
| 0127 | [Book Ebook And Longform Text Coverage](0127-book-ebook-and-longform-text-coverage.md) | 23 | medium | 2026-06-25 |
| 0126 | [Scientific Paper And Long Report Layout Coverage](0126-scientific-paper-and-long-report-layout-coverage.md) | 23 | medium | 2026-06-25 |
| 0125 | [Chart Map And Dashboard Export Coverage](0125-chart-map-and-dashboard-export-coverage.md) | 22 | medium | 2026-06-25 |
| 0124 | [Technical Drawing And CAD PDF Fidelity](0124-technical-drawing-and-cad-pdf-fidelity.md) | 22 | medium | 2026-06-25 |
| 0123 | [Spreadsheet Grid And Dense Table Fidelity](0123-spreadsheet-grid-and-dense-table-fidelity.md) | 22 | medium | 2026-06-25 |
| 0122 | [Presentation And Slide Export Fidelity](0122-presentation-and-slide-export-fidelity.md) | 22 | medium | 2026-06-25 |
| 0121 | [Invoice Statement And Business Form Corpus Gate](0121-invoice-statement-and-business-form-corpus-gate.md) | 22 | medium | 2026-06-25 |
| 0120 | [PDFium-Free Maintenance Gate And Deletion Backlog](0120-pdfium-free-maintenance-gate-and-deletion-backlog.md) | 21 | medium | 2026-06-25 |
| 0119 | [Cross-Platform Rendering Determinism Gate](0119-cross-platform-rendering-determinism-gate.md) | 21 | medium | 2026-06-25 |
| 0118 | [Real Corpus Acquisition And Privacy Review Loop](0118-real-corpus-acquisition-and-privacy-review-loop.md) | 21 | medium | 2026-06-25 |
| 0117 | [Tagged PDF Metadata And Accessibility Signals](0117-tagged-pdf-metadata-and-accessibility-signals.md) | 21 | medium | 2026-06-25 |
| 0116 | [OCR Text Layer And Invisible Text Handling](0116-ocr-text-layer-and-invisible-text-handling.md) | 21 | medium | 2026-06-25 |
| 0115 | [Multi-Page Document Scheduler And Cancellation](0115-multi-page-document-scheduler-and-cancellation.md) | 20 | medium | 2026-06-25 |
| 0114 | [Linearized PDF Fast First Page Loading](0114-linearized-pdf-fast-first-page-loading.md) | 20 | medium | 2026-06-25 |
| 0113 | [Embedded Files Portfolio And Attachment Visibility](0113-embedded-files-portfolio-and-attachment-visibility.md) | 20 | medium | 2026-06-25 |
| 0112 | [Digital Signature Appearance And Validation Boundary](0112-digital-signature-appearance-and-validation-boundary.md) | 20 | small | 2026-06-25 |
| 0111 | [XFA And Dynamic Form Fallback Policy](0111-xfa-and-dynamic-form-fallback-policy.md) | 20 | small | 2026-06-25 |
| 0110 | [Overprint Simulation For Print-Oriented PDFs](0110-overprint-simulation-for-print-oriented-pdfs.md) | 19 | medium | 2026-06-25 |
| 0109 | [Transparency Isolation Knockout And Luminosity Masks](0109-transparency-isolation-knockout-and-luminosity-masks.md) | 19 | medium | 2026-06-25 |
| 0108 | [Advanced Shading Mesh Tessellation](0108-advanced-shading-mesh-tessellation.md) | 19 | medium | 2026-06-25 |
| 0107 | [Tiling Patterns And Pattern Color Spaces](0107-tiling-patterns-and-pattern-color-spaces.md) | 19 | medium | 2026-06-25 |
| 0106 | [ICC Profile Cache And Transform Optimization](0106-icc-profile-cache-and-transform-optimization.md) | 19 | medium | 2026-06-25 |
| 0105 | [Separation DeviceN And Spot Color Approximation](0105-separation-devicen-and-spot-color-approximation.md) | 19 | medium | 2026-06-25 |
| 0104 | [Advanced CMap Encodings And Identity Mapping](0104-advanced-cmap-encodings-and-identity-mapping.md) | 18 | medium | 2026-06-25 |
| 0103 | [OpenType Layout Feature Coverage For PDFs](0103-opentype-layout-feature-coverage-for-pdfs.md) | 18 | medium | 2026-06-25 |
| 0102 | [CFF Type1 Charstring Interpreter Hardening](0102-cff-type1-charstring-interpreter-hardening.md) | 18 | medium | 2026-06-25 |
| 0101 | [Common Font Fallback And System Font Policy](0101-common-font-fallback-and-system-font-policy.md) | 18 | medium | 2026-06-25 |
| 0100 | [Native Renderer General Availability Gate](0100-native-renderer-general-availability-gate.md) | 17 | medium | 2026-06-25 |
| 0099 | [PDFium Fallback Removal Drill](0099-pdfium-fallback-removal-drill.md) | 17 | medium | 2026-06-25 |
| 0098 | [Native-Only Packaging And Consumer Migration](0098-native-only-packaging-and-consumer-migration.md) | 17 | medium | 2026-06-25 |
| 0097 | [Fuzzing And Adversarial PDF Hardening](0097-fuzzing-and-adversarial-pdf-hardening.md) | 16 | medium | 2026-06-25 |
| 0096 | [Hot Path Profiling And Raster Optimization](0096-hot-path-profiling-and-raster-optimization.md) | 16 | medium | 2026-06-25 |
| 0095 | [Large Document Memory Eviction And Spooling](0095-large-document-memory-eviction-and-spooling.md) | 16 | medium | 2026-06-25 |
| 0094 | [Structure Outline Metadata And Page Labels](0094-structure-outline-metadata-and-page-labels.md) | 16 | small | 2026-06-25 |
| 0093 | [Incremental Update Edge Case Hardening](0093-incremental-update-edge-case-hardening.md) | 16 | medium | 2026-06-25 |
| 0092 | [AcroForm Field Synthesis For Common Widgets](0092-acroform-field-synthesis-for-common-widgets.md) | 15 | medium | 2026-06-25 |
| 0091 | [Annotation Without Appearance Fallbacks](0091-annotation-without-appearance-fallbacks.md) | 15 | medium | 2026-06-25 |
| 0090 | [Shading Mesh Gradient And Pattern Fidelity](0090-shading-mesh-gradient-and-pattern-fidelity.md) | 15 | medium | 2026-06-25 |
| 0089 | [JPX JBIG2 And Specialized Image Codec Policy](0089-jpx-jbig2-and-specialized-image-codec-policy.md) | 15 | medium | 2026-06-25 |
| 0088 | [Image Mask Stencil And Bitmap Edge Cases](0088-image-mask-stencil-and-bitmap-edge-cases.md) | 15 | medium | 2026-06-25 |
| 0087 | [Font Hinting Glyph Cache And Subpixel Policy](0087-font-hinting-glyph-cache-and-subpixel-policy.md) | 15 | medium | 2026-06-25 |
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
