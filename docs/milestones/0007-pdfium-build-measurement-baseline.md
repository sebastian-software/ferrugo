# 0007: PDFium Build Measurement Baseline

Status: todo
Phase: 0
Size: medium
Depends on: 0006

## Goal

Build the cut-down PDFium configuration and record baseline measurements.

## Scope

- Build PDFium locally from source.
- Record binary size.
- Record cold start behavior for a simple render command or probe.
- Record first-page render time.
- Record thumbnail render time at fixed output sizes.
- Record memory high-water mark.

## Non-Goals

- Produce production-ready binaries.
- Optimize build size.
- Create npm artifacts.

## Deliverables

- Measurement notes in docs.
- Exact build revision and GN args.
- A short conclusion on whether the cut-down build is operationally plausible.

## Acceptance Criteria

- Measurements are reproducible enough for a second local run.
- The report includes hardware and OS context.
- Failures are recorded instead of hidden.

## Validation

- Run at least one successful build or document the blocking failure.
- Record commands and measured outputs.

## Completion Notes

Empty until done.

