# 0041: Renderer Gap Triage And Support Matrix

Status: todo
Phase: 5
Size: small
Depends on: 0040

## Goal

Turn the typical-document coverage gate into a prioritized native renderer
support matrix.

## Scope

- Classify failures from the typical corpus by missing PDF feature.
- Separate product-visible rendering gaps from parser, IO, and harness gaps.
- Define support levels: rendered, degraded, unsupported, malformed, and
  encrypted.
- Rank gaps by document frequency, thumbnail impact, implementation risk, and
  memory risk.

## Non-Goals

- Implement new renderer features.
- Claim full PDF compatibility.
- Remove PDFium fallback paths.

## Deliverables

- Native renderer support matrix.
- Ranked implementation backlog for the next renderer gaps.
- Updated unsupported-feature taxonomy if new classes are needed.

## Acceptance Criteria

- Each corpus failure has a stable category and owner milestone.
- The next rendering work is ordered by measured product value.
- The matrix explicitly states where PDFium remains required.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Re-run the typical-document corpus comparison command.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
