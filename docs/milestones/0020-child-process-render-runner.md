# 0020: Child Process Render Runner

Status: done
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

Completed on 2026-06-24.

- Added `pdfrust-cli render-isolated` as the parent command for hard timeout
  enforcement.
- Added private `pdfrust-cli render-worker` execution for one direct PDFium
  render job.
- The parent writes through a temporary output path, renames only after worker
  success, and removes temporary output on failure.
- Worker timeout maps to `render error [timeout]: thumbnail rendering timed out`.
- Live validation against the local PDFium dylib:
  - success: `target/pdfrust-thumbnails/text-page-isolated-256.png`, 256x137,
    SHA-256 `1711931704d73467a89f35f4ff523dabecd3b1bf4f4716924e350c4dfc957593`.
  - timeout: `--timeout 0` exits with `timeout` and leaves no final output
    artifact.
