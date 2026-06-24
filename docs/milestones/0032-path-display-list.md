# 0032: Path Display List

Status: todo
Phase: 2
Size: medium
Depends on: 0031

## Goal

Convert basic path construction and painting operators into a display list.

## Scope

- Interpret `m`, `l`, `c`, `h`, `re`, `S`, `s`, `f`, `F`, `f*`, `B`, and
  `B*` where needed by fixtures.
- Store path segments in a Rust-native display list.
- Capture fill and stroke state at paint time.
- Add path complexity limits.

## Non-Goals

- Rasterize paths.
- Implement gradients or patterns.
- Implement clipping beyond storing a placeholder command.

## Deliverables

- Path command representation.
- Display-list path items.
- Tests for simple vector fixtures.

## Acceptance Criteria

- Generated vector PDFs produce inspectable display-list path items.
- Unsupported path operators fail predictably.
- Path memory use is bounded by explicit limits.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare display-list dimensions against PDFium-rendered page metadata.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
