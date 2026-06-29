# 0165: Native-Only CI And Release Artifact Hardening

Status: done
Phase: 30
Size: medium
Depends on: 0164

## Goal

Make native-only validation and packaging the default release posture after the
PDFium-free renderer reaches the 1.0 gate.

## Scope

- Ensure CI has an explicit native-only job that does not install or configure
  PDFium.
- Add package checks that verify release artifacts do not include PDFium runtime
  assets or default dependencies.
- Keep maintainer comparison tooling opt-in and excluded from consumer packages.
- Document release commands and expected native-only validation artifacts.

## Non-Goals

- Remove all maintainer comparison tooling.
- Change public API behavior outside packaging and CI guarantees.
- Add new renderer features.

## Deliverables

- Native-only CI job or local equivalent script.
- Release artifact inspection checks.
- Updated release documentation.

## Acceptance Criteria

- Release validation passes in an environment without PDFium.
- Package artifacts are free of PDFium runtime dependencies.
- Maintainer-only comparison features remain opt-in.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run package dry-run or artifact inspection.
- Run forbidden-reference checks for runtime crates.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Added `scripts/check_native_only_release.sh` as the local CI-equivalent
  native-only release gate.
- Added package file inspection for the consumer CLI artifact list at
  `target/native-only-release-ferrugo-cli-package-files.txt`.
- Updated `docs/packaging.md` with the release-candidate validation command.
- Added
  `docs/reports/native-only-ci-release-hardening-2026-06-26.md`.
