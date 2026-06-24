# 0031: Graphics State And Transforms

Status: todo
Phase: 2
Size: medium
Depends on: 0030

## Goal

Interpret graphics-state stack operations and current transformation matrices.

## Scope

- Implement `q`, `Q`, and `cm`.
- Track line width, fill color, stroke color, and clipping placeholder state.
- Define matrix math types for page and device transforms.
- Enforce stack-depth limits.

## Non-Goals

- Rasterize paths.
- Interpret text operators.
- Implement transparency groups.

## Deliverables

- Graphics state model.
- Matrix and transform utilities.
- Tests for stack behavior, matrix composition, and depth limits.

## Acceptance Criteria

- Generated transform fixtures produce expected graphics-state snapshots.
- Stack underflow and depth overflow return typed errors.
- Matrix operations are deterministic and covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
