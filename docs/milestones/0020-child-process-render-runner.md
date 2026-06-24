# 0020: Child Process Render Runner

Status: todo
Phase: 1
Size: medium
Depends on: 0019

## Goal

Enforce hard per-render timeouts by running one PDFium render job in a child
process.

## Scope

- Add a private render-worker entry point for one render job.
- Add a parent runner that spawns the worker and enforces the wall-clock
  timeout.
- Write output through a temporary file and promote it only after success.
- Map timeout, worker failure, and malformed input into the existing thumbnail
  error taxonomy.
- Validate with the generated text fixture and a deliberately tiny timeout.

## Non-Goals

- Build a reusable worker pool.
- Add Node-API timeout behavior.
- Add OS-level sandbox or memory-limit policy.
- Package PDFium binaries.

## Deliverables

- Parent/worker render path.
- Tests or smoke scripts for success and timeout behavior.
- Updated timeout documentation.

## Acceptance Criteria

- A normal generated fixture render succeeds through the isolated path.
- A tiny timeout terminates the child and reports `timeout`.
- Failed renders do not leave a final output artifact behind.
- Existing direct in-process rendering remains available for local probes.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.
- Run one live isolated render against the local PDFium dylib.

## Completion Notes

Empty until done.
