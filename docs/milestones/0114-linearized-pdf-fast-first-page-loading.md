# 0114: Linearized PDF Fast First Page Loading

Status: in-progress
Phase: 20
Size: medium
Depends on: 0113

## Goal

Use linearized PDF structure to load and render the first page quickly while
preserving safe fallback behavior for malformed files.

## Scope

- Detect linearization dictionaries and first-page hint tables.
- Load the minimum object graph needed for the requested thumbnail.
- Fall back to the normal loader when hints are missing or invalid.
- Add fixtures for valid, malformed, and non-linearized PDFs.

## Non-Goals

- Implement network range fetching in this slice.
- Trust hint tables without validation.
- Rework all parser storage around streaming.

## Deliverables

- Linearization-aware first-page load path.
- Parser metrics for bytes read and objects loaded.
- Benchmark report for first-page rendering.

## Acceptance Criteria

- Linearized first-page thumbnails load fewer bytes or objects when possible.
- Malformed hints do not compromise correctness.
- Non-linearized documents keep existing behavior.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run linearized fixture comparisons.
- Run first-page load benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
