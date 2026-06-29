# 0118: Real Corpus Acquisition And Privacy Review Loop

Status: done
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

- Added a Rust-native `ferrugo-cli validate-local-corpus` command for
  aggregate local corpus TOML validation without new dependencies.
- Updated `fixtures/local-corpus.example.toml` to use privacy-safe `[[sample]]`
  entries instead of per-document private paths.
- Expanded `docs/policies/corpus-intake.md`, `docs/corpus-taxonomy.md`, and
  `docs/fixtures.md` with the local metadata field vocabulary and validation
  flow.
- Confirmed `fixtures/local-corpus/` is excluded from Git by committed ignore
  rules.
- Validated the committed example, the optional missing local manifest path,
  metadata extraction, and fallback summary for the synthetic-realistic
  `fixtures/real-world-style-manifest.tsv`.
- Report: `docs/reports/real-corpus-privacy-review-loop-2026-06-25.md`.
- Implementation commit: `fcb2f87 feat: validate local corpus intake metadata`.
