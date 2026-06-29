# Optional Content UI State Report

Date: 2026-06-29
Milestone: 0192

## Summary

The native renderer now exposes bounded optional-content metadata and keeps
thumbnail flattening deterministic. Default-on, default-off, and nested OCG
fixtures render natively and match Poppler exactly. `/D /AS` usage application
arrays and `/OCMD` membership policies remain typed fallback boundaries under
`graphics.optional-content`, while metadata inspection reports those unsupported
signals for consumers.

## Coverage

`fixtures/optional-content-ui-state-manifest.tsv` contains:

- `default-on`: default-visible OCG content.
- `default-off`: default-hidden OCG content.
- `nested`: visible outer OCG with hidden inner OCG.
- `map-layer-off`: existing map zoning OCG hidden by default.
- `unsupported-usage-application`: `/OCProperties /D /AS` boundary.
- `unsupported-membership`: `/Type /OCMD` boundary.

## Metadata Contract

`DocumentMetadata.optional_content` reports:

- whether `/OCProperties` and `/D` are present;
- OCG count, base state, default `/ON` count, and default `/OFF` count;
- whether usage applications, OCMD policies, or direct OCG dictionaries were
  found;
- a single `has_unsupported_behavior` flag for consumer routing.

The metadata path is intentionally more permissive than rendering. It can
classify unsupported layer behavior without requiring callers to attempt native
rasterization first.

## Validation

Focused native tests:

```sh
cargo test -p ferrugo-native optional_content -- --nocapture
```

Result: 6 passed.

Supported native fallback gate:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/optional-content-ui-state-manifest.tsv \
  --include-family default-on \
  --include-family default-off \
  --include-family nested \
  --include-family map-layer-off \
  --fail-on-fallback
```

Result: 4 total, 4 native rendered, 0 fallback required.

Unsupported boundary gate:

```sh
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/optional-content-ui-state-manifest.tsv \
  --include-family unsupported-usage-application \
  --include-family unsupported-membership
```

Result: 2 total, 2 fallback required, both categorized as
`graphics.optional-content`.

Visual comparison for the 0192 OCG fixtures:

```sh
cargo run -p ferrugo-cli -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/optional-content-ui-state-manifest.tsv \
  --include-family default-on \
  --include-family default-off \
  --include-family nested \
  --max-mae 2 \
  --max-p95 12 \
  --max-changed-ratio 0.05
```

Result: 3 total, 3 exact, 0 blockers, 0 native errors, 0 reference errors.

Single-file metadata checks confirmed:

- nested layers: `group_count=2`, `base_state="on"`, `default_off_count=1`,
  `has_unsupported_behavior=false`;
- usage application: `has_usage_application=true`,
  `has_unsupported_behavior=true`;
- OCMD: `has_unsupported_membership_policy=true`,
  `has_unsupported_behavior=true`.

The existing `map-layer-off` fixture is covered by the native fallback summary.
It was not included in the final Poppler visual gate because Poppler timed out
on that larger fixture during this run while the native renderer completed it.
