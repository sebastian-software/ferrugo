# Native Renderer API Cleanup Backlog

Status: accepted for 0156.
Date: 2026-06-26.

This backlog records public API cleanup that should happen before the
PDFium-free 1.0 release. Items are intentionally small so they can be reviewed
as explicit SemVer decisions instead of accidental renderer churn.

## Immediate Policy State

| Surface | Status | Decision |
| --- | --- | --- |
| `ferrugo-thumbnail` facade types | Stable consumer boundary | Keep backend-neutral and PDFium-free. |
| `ferrugo-native::NativeBackend` | Stable native entry point | Keep construction, render limits, diagnostics, preview APIs, and trait impls public. |
| `ThumbnailErrorClass::as_str()` | Stable strings | Preserve class strings for logs and baseline metadata. |
| `ThumbnailError::UnsupportedFeature` buckets | Diagnostic boundary | Keep available, but do not make bucket names a 1.0 SLA until the typed unsupported milestone. |
| `ferrugo-pdfium` | Maintainer-only oracle | Keep optional and outside normal runtime SemVer expectations. |
| Low-level renderer crates | Internal implementation surface | Avoid recommending direct application use before a separate API design milestone. |

## Cleanup Candidates

| Candidate | Earliest milestone | Risk | Validation |
| --- | --- | --- | --- |
| Decide which public enums should become `#[non_exhaustive]` before 1.0. | 0174 | Medium: affects exhaustive matches. | Public API docs build plus example migration notes. |
| Decide whether public structs with fields need builders or `#[non_exhaustive]` replacements. | 0174 | Medium: literal construction compatibility. | Compile migration examples for `ThumbnailOptions`, metadata structs, and native limits. |
| Promote selected unsupported-feature buckets into typed public diagnostics. | 0174 | Medium: buckets become long-lived support promises. | Unsupported boundary API tests and corpus report. |
| Split maintainer operator-coverage APIs from consumer native APIs if they become noisy. | 0174 | Low: current APIs are useful but diagnostic-heavy. | Downstream docs still show the smaller consumer path clearly. |
| Add API examples for native rendering, metadata inspection, first-page preview, and partial preview. | 0157 | Low: docs/examples only. | `cargo test --doc` or example compile check. |
| Audit package contents for generated fixture or maintainer-report leakage. | 0157 | Low: packaging only. | `cargo package --allow-dirty` dry-runs. |

## Keep Out Of Public Consumer API

| Area | Reason |
| --- | --- |
| PDFium library handles and load paths | They are deployment-specific maintainer tooling. |
| Visual oracle thresholds | They change as fidelity improves and should not drive app control flow. |
| Raw object graph internals | They expose parser implementation details before a document model API exists. |
| Exact benchmark JSON shape | Reports are release evidence, not a stable wire protocol. |
| Internal error message text | Messages need freedom to improve without breaking consumers. |

## Release Checklist

Before a crate release that claims native API stability:

- `cargo doc --workspace --no-deps` succeeds.
- Native-only tests pass with `cargo test --workspace --no-default-features`.
- Full-feature clippy passes with `cargo clippy --workspace --all-targets
  --all-features -- -D warnings`.
- Package dry-runs succeed for the public crates.
- Migration notes mention any planned public signature or behavior changes.
