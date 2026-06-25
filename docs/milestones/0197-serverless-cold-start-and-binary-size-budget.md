# 0197: Serverless Cold Start And Binary Size Budget

Status: todo
Phase: 37
Size: medium
Depends on: 0196

## Goal

Keep serverless and short-lived batch rendering practical by bounding binary
size, startup time, and first-render latency without PDFium runtime baggage.

## Scope

- Measure cold start and first-render latency for native-only builds.
- Audit feature flags, dependencies, and embedded assets for size impact.
- Add package profiles for serverless thumbnail use.
- Document tradeoffs between size, codec coverage, and performance.

## Non-Goals

- Optimize every deployment platform.
- Remove features required for typical documents.
- Reintroduce dynamic PDFium distribution.

## Deliverables

- Serverless cold-start report.
- Binary size budget and package profile.
- Dependency and feature-flag follow-up list.

## Acceptance Criteria

- Native-only artifacts meet documented size and startup budgets.
- Optional heavyweight features are controlled by explicit feature flags.
- First-render latency is measured and reproducible.

## Validation

- Run release builds for target package profiles.
- Run cold-start benchmark script.
- Run package dry-runs.
- Run native-only `cargo test`.

## Completion Notes

Empty until done.
