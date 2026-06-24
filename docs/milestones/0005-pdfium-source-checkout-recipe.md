# 0005: PDFium Source Checkout Recipe

Status: done
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

Completed on 2026-06-24.

- Added `docs/build/pdfium-checkout.md`.
- Pinned PDFium revision
  `573758fe2dd928279cd52b5a4bc955a6938aab39`.
- Verified the remote revision with `git ls-remote`.
- Full checkout remains a machine-local heavy network step because
  `depot_tools` is not installed in this environment.
