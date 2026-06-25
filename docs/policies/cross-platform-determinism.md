# Cross-Platform Rendering Determinism

Status: accepted for Phase 21 gates.
Date: 2026-06-25.

Rust-native rendering should be deterministic within each supported target and
must make platform-specific drift visible before a native-only release gate.

## Report Metadata

Renderer benchmark and visual-diff reports include a `platform` object:

- `os`
- `arch`
- `family`
- `endian`
- `pointer_width_bits`

This metadata is intentionally target-level rather than host-name-level. Do not
record machine names, user names, file paths, or private environment variables
in public reports.

## Required Gate Artifacts

A native-only release candidate needs one report set per supported platform:

- `benchmark-native` over `fixtures/generated` with
  `fixtures/corpus-manifest.tsv`
- native supported-family fallback gate for `browser-print`, `office-export`,
  and `form`
- PDFium visual-diff report when a local PDFium oracle is available for that
  target

Every artifact must include the platform metadata block. Missing platform
artifacts are release-candidate blockers, not silent passes.

## Tolerances

The default visual thresholds remain:

- `max_mean_abs_error = 2.0`
- `max_p95_channel_delta = 16`
- `max_changed_ratio = 0.05`

Accepted drift may differ by platform only when the fixture status remains
`exact` or `accepted_drift` under those thresholds. Thresholds must not be
loosened to hide a blocker on one platform.

## Hard Blockers

The following block native-only release candidates:

- Supported-family native fallback or render errors.
- A supported-family visual-diff status of `native_error`, `pdfium_error`, or
  `both_error`, unless the fixture is explicitly outside the release surface.
- A supported-family blocker that appears on only one platform and has no
  documented owner subsystem.
- Missing benchmark or visual-diff platform metadata.
- Missing report artifacts for a declared supported platform.

## Current Local Baseline

The 0119 local gate was run on `macos` / `aarch64` / `unix`, little-endian,
64-bit. It is a valid local baseline and policy check, not full Linux coverage.
Linux artifacts must be added by CI or a maintainer-run Linux machine before a
native-only release candidate can claim cross-platform coverage.
