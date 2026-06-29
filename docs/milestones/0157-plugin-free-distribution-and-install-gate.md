# 0157: Plugin-Free Distribution And Install Gate

Status: done
Phase: 29
Size: medium
Depends on: 0156

## Goal

Verify that consumers can install and use the Rust-native renderer without
external PDFium binaries, platform plugins, or hidden runtime downloads.

## Scope

- Audit crate, CLI, and package installation paths.
- Test clean-machine native-only build and usage flows.
- Document required system libraries, if any.
- Add checks that prevent hidden network or binary fetch behavior.

## Non-Goals

- Build every downstream package manager integration.
- Bundle platform-specific viewer applications.
- Remove optional maintainer comparison dependencies.

## Deliverables

- Plugin-free install report.
- Clean-machine validation scripts or checklist.
- Packaging docs update.

## Acceptance Criteria

- Native rendering works from documented install steps without PDFium binaries.
- Package metadata does not imply hidden runtime downloads.
- Optional comparison tooling is clearly separate.

## Validation

- Run package dry-runs.
- Run clean target native-only build.
- Run CLI smoke tests without PDFium configured.
- Run forbidden network/download checks where available.

## Completion Notes

Completed on 2026-06-26.

- Added `scripts/check_plugin_free_distribution.sh` to guard the native-only
  CLI dependency graph against PDFium, network/TLS download crates, hidden
  fetch/plugin hooks, and checked-in native binary artifacts.
- Updated `docs/packaging.md` with plugin-free install commands, system
  requirements, and the full workspace package dry-run.
- Added `docs/reports/plugin-free-distribution-install-gate-2026-06-26.md`
  with dependency, package, install, and CLI smoke evidence.
- Verified a clean native-only `cargo install` and render smoke without
  `FERRUGO_PDFIUM_LIBRARY` or dynamic-library path configuration.
