# Native-Only Consumer Migration Guide

Status: accepted for 0219.
Date: 2026-06-29.

This guide is for applications moving from PDFium-backed rendering to the
Rust-native renderer.

## Build And Packaging

- Build normal production paths without `--features pdfium`.
- Remove `FERRUGO_PDFIUM_LIBRARY`, bundled PDFium dynamic libraries, and plugin
  download steps from runtime images.
- Keep PDFium-enabled commands only for maintainer oracle comparison or local
  migration audits.
- Use `ferrugo-thumbnail` as the stable facade for application error handling.

CLI production examples:

```sh
cargo install --path crates/ferrugo-cli --no-default-features --locked
ferrugo render input.pdf --max-edge 256 --output thumbnail.png
```

Library dependency example:

```toml
[dependencies]
ferrugo-native = "0.1.0"
ferrugo-thumbnail = "0.1.0"
```

## Error Handling

Route by stable class first and bucket second:

```rust
use ferrugo_thumbnail::{
    unsupported_feature_buckets, ThumbnailError, ThumbnailErrorClass,
};

fn route_render_error(error: &ThumbnailError) -> &'static str {
    match error.class() {
        ThumbnailErrorClass::Unsupported => match error.unsupported_feature_bucket() {
            Some(unsupported_feature_buckets::IMAGE_FILTER) => "scan-codec-review",
            Some(unsupported_feature_buckets::FORM_XFA_DYNAMIC) => "producer-migration",
            Some(_) => "native-feature-backlog",
            None => "generic-native-unsupported",
        },
        ThumbnailErrorClass::Encrypted => "request-password-policy",
        ThumbnailErrorClass::Malformed => "reject-or-repair-input",
        ThumbnailErrorClass::Timeout => "retry-with-explicit-timeout-policy",
        ThumbnailErrorClass::Internal => "renderer-defect",
    }
}
```

Do not inspect backend-internal renderer state or parse display strings for
control flow. Bucket constants are the stable feature-specific boundary.

## Deployment Profiles

| Profile | Recommended behavior |
| --- | --- |
| Server thumbnail or preview workers | Treat supported family regressions, budget failures, and internal errors as blockers. Route typed unsupported buckets through the SLA. |
| Serverless workers | Use the `serverless` Cargo profile and the native-only release gate before deployment. |
| Batch rendering | Use explicit worker and in-flight-pixel budgets; treat `fallback_required` as typed unsupported, not hidden fallback. |
| Low-memory constrained hosts | Use the low-memory profile as a reliability guard; promote only shared correctness or unbounded-resource failures to server blockers. |
| WASM/browser viewers | Treat as secondary compatibility unless a shared renderer correctness or bounded-resource defect appears. |

## Migration Checklist

- Replace implicit PDFium fallback with explicit routing for
  `ThumbnailErrorClass::Unsupported`.
- Record unsupported buckets in telemetry using privacy-safe diagnostics.
- Update CI to run native-only tests and package dry-runs.
- Gate broad release claims on supported families, not on headline corpus pass
  rates alone.
- Keep maintainer comparison tools behind `--features pdfium`.
- Document any application-owned alternate renderer path as a product policy,
  not as hidden ferrugo runtime fallback.

## Validation Commands

```sh
bash scripts/check_unsupported_feature_sla.sh
cargo test -p ferrugo-thumbnail consumer_migration -- --nocapture
cargo test --workspace --no-default-features
cargo package -p ferrugo-thumbnail --allow-dirty --no-verify --list
cargo package -p ferrugo --allow-dirty --no-verify --list
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
