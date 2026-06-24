# 0008: Fixture Policy And Seed Fixtures

Status: done
Phase: 0
Size: small
Depends on: 0003

## Goal

Create the first fixture policy and seed a small, license-safe fixture set.

## Scope

- Define which generated PDFs can be committed.
- Define how local real-world corpus files are documented without committing
  them.
- Seed simple generated fixtures for page size, text, vector paths, and image
  placement.

## Non-Goals

- Add large public corpora.
- Commit private or user-supplied PDFs.
- Cover all PDF features.

## Deliverables

- Fixture policy documentation.
- Generated seed fixtures or scripts to create them.
- Local corpus metadata template.

## Acceptance Criteria

- Fixtures are license-safe.
- Real-world corpus guidance prevents accidental repository commits.
- The seed set is enough for a first thumbnail render smoke test.

## Validation

- Confirm fixtures can be regenerated or are simple enough to inspect.
- Confirm `.gitignore` protects local corpus paths if needed.

## Completion Notes

Completed on 2026-06-24.

- Added `docs/fixtures.md`.
- Added local-only corpus ignore rules and metadata template.
- Added `scripts/generate_fixtures.py`.
- Generated the initial seed PDFs under `fixtures/generated/`.
