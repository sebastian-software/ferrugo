# 0002: Timeout And Process Isolation

Date: 2026-06-24.
Status: accepted for Phase 1 implementation.

## Context

The Phase 0 PDFium backend now renders the generated text fixture through a
local dynamic PDFium build. The measured release CLI path is fast for the seed
fixture, roughly 0.03-0.04s wall time with about 24 MiB max RSS, but that does
not make hostile or malformed real-world PDFs safe to render in-process.

The public thumbnail facade already exposes `ThumbnailOptions::timeout` and a
stable `timeout` error class. Phase 1 must define whether that timeout is a
best-effort hint or a hard caller-visible deadline.

## Decision

Use process isolation for product-facing timeout enforcement.

The in-process PDFium backend remains useful for local probes, trusted inputs,
and differential harness work. It must not be the surface that promises hard
cancellation for untrusted PDFs.

Phase 1 should introduce a small isolated render runner:

- the parent process owns the caller timeout,
- the child process performs exactly one render job,
- the parent kills the child when the wall-clock deadline expires,
- timeout exits map to the existing `timeout` error class,
- backend failures still map through the existing thumbnail taxonomy,
- temporary output is written outside the final destination and promoted only
  after success.

## Evaluated Options

### Serialized In-Process Rendering

This is the current PDFium backend shape. It uses a process-local mutex around
PDFium calls and keeps the public Rust facade simple.

Strengths:

- minimal overhead,
- already implemented and measured,
- simple memory ownership inside Rust,
- acceptable for trusted local measurement.

Weaknesses:

- a stuck native call blocks the process-local PDFium lock,
- Rust cannot safely stop a running foreign function call,
- memory owned by PDFium is only reclaimed if control returns,
- a timeout can only be checked before or after rendering, not enforced during
  rendering.

Conclusion: keep for trusted probes, but do not use it for hard timeout
semantics.

### Worker Thread Boundary

A worker thread can keep the caller thread responsive and makes an async or
queued API easier later.

Strengths:

- lower startup overhead than a process,
- integrates naturally with future worker pools,
- can report timeout to the caller while work continues.

Weaknesses:

- Rust has no safe way to kill the blocked worker thread,
- the PDFium global lock may remain held forever,
- PDFium or allocator state can remain corrupted or inflated after timeout,
- the process may still keep burning CPU or memory after the caller receives a
  timeout.

Conclusion: useful later for scheduling trusted workloads, but insufficient as
the hostile-PDF containment boundary.

### Process Isolation

A child process gives the parent an operating-system boundary for cancellation
and cleanup.

Strengths:

- hard wall-clock timeout can be enforced by killing the child,
- PDFium state and native allocations are discarded on process exit,
- crashes become child exit statuses rather than parent process crashes,
- memory and CPU limits can be added later at the same boundary.

Weaknesses:

- more implementation code,
- process startup adds overhead,
- input and output need explicit serialization or file handoff,
- platform-specific resource limits still need separate design.

Conclusion: choose this for product-facing timeout behavior. The measured seed
fixture render time leaves room to measure and optimize process overhead in the
next slice.

## Caller Semantics

For isolated rendering, `ThumbnailOptions::timeout` means a hard wall-clock
deadline for one render job. If the child process does not complete before the
deadline, the parent terminates it and returns `ThumbnailError::Timeout`.

For direct in-process `PdfiumBackend::render`, timeout remains a configuration
field but is not a hard cancellation guarantee. Callers that need robust
timeout behavior must use the isolated runner once it exists.

## Security And Memory Tradeoffs

Process isolation is not a full sandbox. It does not by itself restrict file
system access, network access, CPU usage, or memory usage. It is still the right
next boundary because it provides deterministic cancellation and native-memory
cleanup without committing to a platform-specific sandbox API too early.

Later hardening can add:

- memory ceilings,
- CPU ceilings,
- file-system sandboxing,
- restricted temporary directories,
- crash telemetry for child exits,
- reusable warm worker processes if cold startup is too expensive.

## Next Implementation Slice

Implement one small child-process render path before Node-API work:

1. Add a private render-worker command that accepts one input PDF, one output
   path, page index, max edge, background, and timeout value.
2. Add a parent command or library runner that spawns the worker and enforces
   the wall-clock deadline.
3. Write output through a temporary file and rename it after successful worker
   completion.
4. Map timeout, non-zero exit, signal termination, and malformed-input failures
   into the existing thumbnail error taxonomy.
5. Validate with the generated text fixture and a deliberately tiny timeout.

This keeps the next milestone independently testable without adding batch
scheduling, Node-API behavior, or platform sandbox policy.
