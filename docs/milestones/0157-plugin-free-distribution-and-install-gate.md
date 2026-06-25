# 0157: Plugin-Free Distribution And Install Gate

Status: todo
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

Empty until done.
