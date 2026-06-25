# 0217: Low-End Device Reliability Sweep

Status: todo
Phase: 41
Size: medium
Depends on: 0216

## Goal

Validate low-end Rust-native rendering reliability as a secondary profile sweep
using realistic typical-document workflows. Constrained server behavior remains
important; mobile browser and embedded findings are compatibility signals unless
they reveal shared renderer defects.

## Scope

- Define low-memory, low-thread, high-latency I/O, and reduced canvas-size test
  profiles.
- Run typical-document workflows for thumbnails, first page, page navigation,
  search highlighting, and batch rendering.
- Measure peak memory, scratch allocation reuse, cache eviction, timeout, and
  recovery behavior.
- Document profile-specific unsupported or degraded modes.
- Promote shared renderer correctness, safety, and unbounded resource issues to
  the main server-side backlog.

## Non-Goals

- Optimize for devices below documented minimum requirements.
- Treat low-end profiles as a reason to reduce desktop fidelity.
- Hide profile failures behind PDFium fallback.
- Block server-side release gates solely on mobile or embedded profile limits.

## Deliverables

- Low-end reliability profile matrix.
- Memory, timeout, and degradation report.
- Profile-specific budget updates.

## Acceptance Criteria

- Supported low-end profiles complete typical workflows without panics.
- Memory and timeout budgets are enforced and documented.
- Degraded behavior is typed and visible to consumers.
- Server-constrained failures are classified separately from browser or embedded
  profile limitations.

## Validation

- Run native-only `cargo test`.
- Run low-memory renderer profile.
- Run WASM low-memory browser gate.
- Run server constrained batch gate.
- Run deterministic render checks for constrained profiles.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
