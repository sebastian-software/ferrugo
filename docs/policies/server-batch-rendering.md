# Server Batch Rendering Policy

Status: accepted for 0177.
Date: 2026-06-26.

Server-side batch rendering is a primary PDFium-replacement path. The default
policy favors isolation and bounded resource use over maximum raw throughput.

## Isolation Boundary

Batch rendering must keep untrusted document state local to one job:

- create a fresh native backend for each independent document job;
- do not share parsed document objects across jobs;
- do not use process-global mutable document caches;
- keep the default cache policy at `isolated-render`;
- do not persist document-derived cache entries to disk.

Shared renderer code and immutable configuration are allowed. Shared decoded
document resources are not allowed until a tenant-safe cache policy explicitly
defines keys, ownership, eviction, and privacy boundaries.

## Concurrency And Memory

Server batch tools must bound both worker count and in-flight pixels. Effective
workers are the lower of the requested worker count and the pixel budget divided
by the maximum page raster size.

If the pixel budget cannot schedule one job, the batch must fail with a typed
budget error instead of silently overcommitting memory.

## Cancellation

Batch cancellation is cooperative. Already-started jobs may finish, but no new
jobs should be scheduled after the cancellation boundary. Reports must expose:

- scheduled job count;
- skipped job count;
- whether cancellation occurred;
- the backend scope and cache policy used for scheduled jobs.

Cancellation must not leave partially shared document state because scheduled
jobs own their native backend and source path independently.

## Timeout And Failure Handling

Timeouts remain per-render options. A timed-out or malformed document must be
reported as a per-job outcome and must not contaminate subsequent jobs.

Operational consumers should treat:

- `fallback_required` as a typed native feature boundary;
- `error` as a document or runtime failure for that job;
- `budget_failures` as a failed server gate when configured with
  `--fail-on-budget`.

## Recommended Gate Profile

For CI-sized server gates, start with:

- `--max-edge 160`;
- `--max-workers 4`;
- `--max-in-flight-pixels 102400`;
- `--repetitions 3`;
- `--max-p95-ms 1000`;
- `--max-errors 0`;
- `--fail-on-budget`.

Production deployments should tune worker and pixel budgets from host memory,
expected thumbnail size, and queue concurrency rather than CPU count alone.
