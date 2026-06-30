# ferrugo-content

`ferrugo-content` tokenizes decoded PDF content streams into operands, operators, and inline image records for the native renderer.

## Use It For

- Iterating through content-stream tokens without owning the input bytes.
- Classifying operators before display-list construction.
- Keeping content parsing separate from object loading and rasterization.

## Release Notes

This crate is part of the Ferrugo release train. Publish lower-level crates in
order before publishing crates that depend on them; see the repository packaging
documentation for the current sequence.
