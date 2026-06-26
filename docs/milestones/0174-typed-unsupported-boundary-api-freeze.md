# 0174: Typed Unsupported Boundary API Freeze

Status: done
Phase: 32
Size: small
Depends on: 0173

## Goal

Stabilize the public unsupported-feature boundary so consumers can build
reliable fallback, reporting, and retry behavior around the native renderer.

## Scope

- Audit public error types for unsupported, malformed, budget, and policy
  outcomes.
- Remove ambiguous error variants where a typed unsupported reason exists.
- Add documentation for consumer handling of unsupported features.
- Mark any unstable diagnostic fields as internal or experimental.

## Non-Goals

- Freeze every internal renderer diagnostic.
- Add runtime PDFium fallback.
- Convert security or malformed input errors into unsupported feature errors.

## Deliverables

- Public error taxonomy update.
- API documentation for unsupported handling.
- Regression tests for stable error classification.

## Acceptance Criteria

- Consumers can distinguish unsupported features from parse failures and budget
  limits.
- Error names are specific enough for telemetry and support decisions.
- Public docs include stable handling guidance.

## Validation

- Run native-only `cargo test`.
- Run API documentation checks where available.
- Run unsupported corpus classification.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed 2026-06-26.

- Promoted unsupported-feature bucket names into
  `pdfrust_thumbnail::unsupported_feature_buckets` plus
  `STABLE_UNSUPPORTED_FEATURE_BUCKETS`.
- Updated the native backend to emit the facade bucket constants instead of
  private duplicate strings.
- Added regression tests for stable bucket strings and representative native
  unsupported boundaries.
- Updated public consumer handling guidance in `docs/errors.md` and
  `docs/policies/native-renderer-api-semver.md`.
- Recorded the API freeze in
  `docs/reports/typed-unsupported-boundary-api-freeze-2026-06-26.md`.

Validation:

- `cargo test -p pdfrust-thumbnail unsupported_feature_buckets -- --nocapture`
- `cargo test -p pdfrust-native typed_unsupported_boundary -- --nocapture`
- Unsupported corpus classification with `summarize-fallbacks`.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
