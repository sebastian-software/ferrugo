# 0219: Unsupported Feature SLA And Consumer Migration Guide

Status: todo
Phase: 41
Size: small
Depends on: 0218

## Goal

Define the consumer-facing service level for unsupported PDF features and
document migration guidance for applications that need predictable native-only
renderer behavior.

## Scope

- Consolidate unsupported categories, diagnostics, severity, retry behavior, and
  fallback recommendations.
- Define which unsupported features are release blockers, documented limits, or
  backlog candidates.
- Write migration guidance for applications previously depending on PDFium.
- Add examples for handling typed unsupported outcomes without inspecting
  internal renderer state.

## Non-Goals

- Promise support for every PDF feature.
- Encourage applications to ship private PDFium fallback paths.
- Expose unstable internal diagnostics as public API.

## Deliverables

- Unsupported feature SLA.
- Consumer migration guide.
- Public diagnostic example updates.

## Acceptance Criteria

- Consumers can distinguish unsupported, degraded, failed, and successful
  outcomes through stable APIs.
- Migration guidance covers common native-only deployment profiles.
- Release-blocking unsupported categories are explicit.

## Validation

- Run documentation link checks.
- Run public API examples.
- Run native-only `cargo test`.
- Run unsupported diagnostic snapshot tests.
- Run package dry-runs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
