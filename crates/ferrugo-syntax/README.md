# ferrugo-syntax

`ferrugo-syntax` contains the low-level PDF byte syntax primitives used by the Ferrugo Rust-native renderer.

## Use It For

- Tokenizing and parsing PDF primitive values from borrowed input bytes.
- Working with byte offsets, names, strings, numbers, arrays, dictionaries, and references.
- Keeping syntax parsing independent from object graph loading and rendering.

## Release Notes

This crate is part of the Ferrugo release train. Publish lower-level crates in
order before publishing crates that depend on them; see the repository packaging
documentation for the current sequence.
