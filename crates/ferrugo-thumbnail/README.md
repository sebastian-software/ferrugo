# ferrugo-thumbnail

`ferrugo-thumbnail` defines Ferrugo's backend-neutral thumbnail API, shared output types, options, metadata types, and typed render errors.

## Use It For

- Calling PDF thumbnail backends through a small stable facade.
- Sharing thumbnail options, page metadata, text extraction, and error classes across backends.
- Routing unsupported PDF features through typed buckets instead of backend-specific strings.

## Release Notes

This crate is part of the Ferrugo release train. Publish lower-level crates in
order before publishing crates that depend on them; see the repository packaging
documentation for the current sequence.
