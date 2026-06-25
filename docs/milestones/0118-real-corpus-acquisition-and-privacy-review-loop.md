# 0118: Real Corpus Acquisition And Privacy Review Loop

Status: in-progress
Phase: 21
Size: medium
Depends on: 0117

## Goal

Create a repeatable, privacy-safe loop for adding real-world PDFs to the local
coverage program without committing sensitive files.

## Scope

- Define intake metadata for source, category, permissions, and redaction state.
- Add review steps for privacy, licensing, and reproducibility.
- Track coverage deltas without storing private PDFs in git.
- Add synthetic replacements for private cases that expose renderer gaps.

## Non-Goals

- Commit customer or personal PDFs.
- Treat private corpus access as required for open-source contributors.
- Skip generated fixture minimization when a bug can be reduced.

## Deliverables

- Corpus acquisition policy.
- Intake checklist and manifest fields.
- Coverage dashboard update for real-world categories.

## Acceptance Criteria

- Every real-corpus item has source and privacy classification.
- Private files are excluded from git by policy and tooling.
- Renderer gaps from private files are reduced into shareable fixtures when
  practical.

## Validation

- Run manifest validation.
- Run corpus metadata extraction on reviewed local corpus entries.
- Run synthetic replacement fixture checks.
- Run `cargo fmt --check`.
- Run `cargo check`.

## Completion Notes

Empty until done.
