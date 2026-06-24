# 0001: Milestone Tracking Structure

Status: done
Phase: 0
Size: small
Depends on: none

## Goal

Create a stable milestone-tracking structure that allows small pieces of work to
move from `todo` to `done` without moving files or breaking links.

## Scope

- Add a milestone index.
- Define allowed statuses.
- Define update rules.
- Seed the first granular Phase 0 milestones.

## Non-Goals

- Implement project code.
- Create a Cargo workspace.
- Start PDFium build work.

## Deliverables

- `docs/milestones/README.md`.
- Numbered milestone files under `docs/milestones/`.
- Status tracking by document metadata and index table.

## Acceptance Criteria

- The milestone index has `Todo`, `In Progress`, and `Done` sections.
- Each seeded milestone has a stable numbered file.
- Status changes do not require moving files.

## Validation

- Confirm all milestone links in the index resolve.
- Confirm each milestone has a `Status:` field.

## Completion Notes

Created as part of the initial milestone-tracking setup.

