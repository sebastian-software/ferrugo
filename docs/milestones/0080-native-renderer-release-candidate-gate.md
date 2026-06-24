# 0080: Native Renderer Release Candidate Gate

Status: done
Phase: 13
Size: medium
Depends on: 0079

## Goal

Decide whether the Rust renderer is ready to ship as the primary production
renderer for the targeted typical-document surface.

## Scope

- Run the full corpus, benchmark suite, and native-only validation path.
- Compare pass rate, visual fidelity, error quality, render time, and memory
  behavior against the PDFium-enabled baseline.
- Define release candidate criteria and blockers.
- Produce the next milestone wave based on measured remaining gaps.

## Non-Goals

- Declare complete PDF specification coverage.
- Remove all fallback code without a separate removal milestone.
- Ship without documented unsupported categories.

## Deliverables

- Native renderer release candidate report.
- Go/no-go decision for production primary renderer status.
- Follow-up milestone recommendations for post-RC gaps.

## Acceptance Criteria

- Native renderer readiness is based on current evidence, not aspiration.
- Release blockers are concrete and mapped to document categories.
- The next plan wave is derived from corpus and benchmark data.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run full corpus comparisons.
- Run renderer benchmark suite.
- Run native-only validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Ran native-only build, test, fallback, and benchmark validation.
- Ran PDFium-enabled CLI tests and benchmark comparison.
- Ran corpus-wide metadata comparison against PDFium for all 52 generated
  fixtures.
- Published the RC decision report in
  `docs/reports/native-renderer-rc-gate-2026-06-24.md`.
- Decision: no-go for broad primary production renderer status; continue
  native-first only behind explicit fallback and category gates.
- Main blockers:
  `optional-content-ocmd.pdf` fallback,
  `vector-stress.pdf` smoke render-time budget,
  missing full-corpus visual-diff automation,
  remaining text-fidelity risk.

Validation passed:

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo test -p pdfrust-cli --features pdfium`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Strict native-only fallback gate failed as expected with
`1 native fallback(s) required`, which is the release-blocker evidence.
