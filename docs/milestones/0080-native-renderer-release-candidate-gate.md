# 0080: Native Renderer Release Candidate Gate

Status: todo
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

Empty until done.
