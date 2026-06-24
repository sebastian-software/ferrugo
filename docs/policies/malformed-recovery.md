# Malformed PDF Recovery Policy

Status: accepted.
Date: 2026-06-24.

The Rust-native parser may recover from malformed PDF structure only when the
repair is local, bounded, and preserves strict failure behavior for inputs that
cannot be repaired safely.

## Supported Recovery

- Classic and xref-stream in-use entries may recover from small indirect-object
  offset drift.
- Offset recovery scans at most
  `DEFAULT_XREF_OFFSET_RECOVERY_SCAN_BYTES` bytes on either side of the declared
  xref offset.
- A recovered candidate must parse as the exact object number and generation
  declared by the xref entry.

## Non-Recoverable Cases

- Missing or corrupt encryption metadata.
- Cross-reference chains that cycle or exceed the incremental update budget.
- Object offsets that point at a different valid indirect object.
- Whole-file rescans for arbitrary object headers.
- Security-sensitive repair that would reinterpret encrypted or permissioned
  content as plain PDF.

## Diagnostics

Strict parsing remains the first path. If no bounded recovery candidate parses,
the original parser error is returned. This keeps existing malformed diagnostics
stable while allowing common producer offset drift to render natively.
