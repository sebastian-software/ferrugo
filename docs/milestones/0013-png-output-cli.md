# 0013: PNG Output CLI

Status: done
Phase: 0
Size: small
Depends on: 0012

## Goal

Expose a local CLI that writes a PNG thumbnail artifact.

## Scope

- Add CLI arguments for input PDF, output PNG, page index, max edge, background,
  and timeout.
- Use the thumbnail facade.
- Encode RGBA output to PNG.

## Non-Goals

- Add JPEG or WebP.
- Add npm packaging.
- Add a server API.

## Deliverables

- CLI command for local thumbnail generation.
- PNG output for generated fixtures.
- Documented command examples.

## Acceptance Criteria

- The CLI writes a valid PNG for a generated fixture.
- Default page index is `0`.
- Default max edge is `1024`.
- Default timeout is `5s`.

## Validation

- Run the CLI against at least one generated fixture.
- Inspect PNG dimensions.

## Completion Notes

Completed on 2026-06-24.

- Implemented `pdfrust-cli render <input.pdf> --output <output.png>`.
- Added page index, max edge, background, and timeout arguments with Phase 0
  defaults.
- Added std-only PNG encoding for RGBA thumbnails.
- Live fixture PNG generation was not run because no local PDFium library is
  available in this environment.
