# ferrugo-native

`ferrugo-native` wires the Ferrugo syntax, object, content, render, and thumbnail crates into a Rust-native PDF preview backend.

## Use It For

- Rendering bounded thumbnails without a PDFium runtime dependency.
- Extracting privacy-safe document metadata and text through backend-neutral traits.
- Surfacing unsupported PDF features as typed error classes and buckets.

## Release Notes

This crate is part of the Ferrugo release train. Publish lower-level crates in
order before publishing crates that depend on them; see the repository packaging
documentation for the current sequence.
