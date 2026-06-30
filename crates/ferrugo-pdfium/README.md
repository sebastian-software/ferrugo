# ferrugo-pdfium

`ferrugo-pdfium` is an optional backend used by Ferrugo maintainers for reference rendering, metadata comparison, and local PDFium probes.

## Use It For

- Loading a locally provided PDFium dynamic library through `FERRUGO_PDFIUM_LIBRARY`.
- Comparing native renderer behavior against a mature reference renderer.
- Keeping PDFium out of the default Ferrugo runtime and package graph.

## Release Notes

This crate is part of the Ferrugo release train. Publish lower-level crates in
order before publishing crates that depend on them; see the repository packaging
documentation for the current sequence.
