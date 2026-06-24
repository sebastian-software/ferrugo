# Encryption And Permissions Policy

Status: accepted for 0056.
Date: 2026-06-24.

The native backend does not decrypt PDFs. Encrypted documents must fail before
page content, streams, annotations, images, or form appearances are interpreted
as plain data.

## Supported

- Detecting trailer `/Encrypt` entries.
- Detecting unusual catalog `/Encrypt` metadata before returning a loaded
  document.
- Returning the public `encrypted` error class through render and metadata
  inspection paths.

## Unsupported

- User-password or owner-password workflows.
- Decryption algorithms and security handler negotiation.
- Permission-bit interpretation.
- Bypassing or ignoring document permissions.
- Rendering encrypted payload bytes as if they were plaintext objects.

## Future Decision Point

Any password, permission, or decryption support must be designed as a separate
milestone with explicit API ownership for password input, credential lifetime,
and permission policy. Until then, encrypted documents fail closed with the
stable encrypted error class.
