# 0174: Typed Unsupported Boundary API Freeze

Status: todo
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

Empty until done.
