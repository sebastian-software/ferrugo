# 0005: PDFium Source Checkout Recipe

Status: todo
Phase: 0
Size: small
Depends on: 0003

## Goal

Document the exact local steps required to fetch PDFium source for the Phase 0
probe.

## Scope

- Identify required tools.
- Document checkout commands.
- Pin the tested PDFium revision.
- Document expected source directory layout.

## Non-Goals

- Build PDFium.
- Vendor PDFium into this repository.
- Automate depot_tools installation.

## Deliverables

- A checkout recipe under project docs.
- A recorded PDFium revision or commit hash.
- Notes for macOS arm64 as the first local environment.

## Acceptance Criteria

- A developer can fetch the same PDFium source revision.
- The recipe avoids writing PDFium source into this repository.
- Network-heavy steps are clearly separated from local build steps.

## Validation

- Run the documented checkout once locally.
- Record the resulting revision.

## Completion Notes

Empty until done.

