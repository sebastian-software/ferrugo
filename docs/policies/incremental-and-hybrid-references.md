# Incremental And Hybrid Reference Policy

Status: accepted for 0055.
Date: 2026-06-24.

The native object loader supports common producer output that stores newer
object revisions after the original file body. It does not validate signatures
or repair arbitrary damaged update chains.

## Supported

- Classic xref tables followed from the latest `startxref`.
- Trailer `/Prev` chains with a fixed `16`-revision depth limit.
- Newest reachable xref entries winning over older entries for the same object
  identifier.
- Older xref entries filling objects that are absent from newer revisions.
- Newer free xref entries tombstoning older in-use entries for the same object
  number, including common generation bumps on deleted objects.
- Hybrid-reference files whose current classic trailer contains `/XRefStm`.
- Direct in-use type-1 xref-stream entries from `/XRefStm` when they do not
  conflict with already selected classic entries.

## Unsupported

- Signature validation.
- Writing incremental updates.
- Repairing corrupt update chains beyond explicit cycle and depth checks.
- Allowing `/XRefStm` entries to override newer classic/incremental entries.
- Using compressed type-2 xref-stream entries from a hybrid trailer in the
  classic loader path.

## Error Behavior

Repeated `/Prev` offsets return an incremental-update cycle error. Revision
chains beyond the configured depth return an incremental-update depth error.
Hybrid xref streams that are malformed, missing, or unsupported fail the load
instead of silently rendering a partial document.
