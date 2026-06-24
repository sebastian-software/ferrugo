# 0003: Phase 0 Decision Baseline

Status: done
Phase: 0
Size: small
Depends on: 0002

## Goal

Turn early project questions into explicit Phase 0 planning defaults.

## Scope

- Define Phase 0 as thumbnail-first.
- Set MIT/Apache-2.0 as license intent.
- Choose Rust CLI plus Rust library before Node-API.
- Choose a source-built, cut-down PDFium probe.
- Set serialized PDFium backend, single-page API, RGBA/PNG outputs, bounded
  defaults, and fixture policy.

## Non-Goals

- Create implementation crates.
- Build PDFium.
- Ship npm packages or prebuilt binaries.

## Deliverables

- `docs/plans/phase-0-decisions.md`.
- Updated `README.md`.
- Updated `docs/plans/2026-06-24-thumbnail-generation-plan.md`.
- Updated `docs/roadmap.md`.

## Acceptance Criteria

- Phase 0 has clear defaults and deferred decisions.
- Node-API, npm prebuilds, bundled PDFium, and full renderer parity are
  explicitly deferred.
- The roadmap reflects a measurement-first thumbnail probe.

## Validation

- Confirm README links resolve.
- Search for contradictory Phase 0 wording around Node-API, prebuilds, and full
  renderer scope.

## Completion Notes

Completed in commit `43e95c9` (`docs: establish phase 0 thumbnail plan`).

