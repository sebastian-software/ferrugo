# Encryption And Permissions Policy

Status: updated for #101.
Date: 2026-07-04.

The native backend opens standard-security PDFs only when the empty user
password is valid. This covers the common permissions-only class used to set
print/copy flags while still opening transparently in mainstream viewers.
Documents that require a non-empty user or owner password still fail before page
content, streams, annotations, images, or form appearances are interpreted.

## Supported

- Detecting trailer `/Encrypt` entries.
- Standard security handler decryption for the empty-user-password case:
  - RC4 V1/V2 stream and string decryption.
  - AES-128 V4 `/AESV2` stream and string decryption.
  - AES-256 V5 `/AESV3` stream and string decryption.
  - `/Identity` stream/string crypt filters.
- Detecting unusual catalog `/Encrypt` metadata before returning a loaded
  document.
- Returning the public `encrypted` error class for genuinely
  password-protected or unsupported security-handler documents.

## Unsupported

- User-password or owner-password workflows.
- Permission-bit interpretation.
- Bypassing or ignoring document permissions.
- Rendering encrypted payload bytes as if they were plaintext objects.
- Public password input APIs and credential lifetime management.
- Non-standard security handlers and revision 6 password hashing.

## Future Decision Point

Any non-empty password workflow must be designed as a separate decision point
with explicit API ownership for password input, credential lifetime, and
permission policy. Until then, password-protected documents fail closed with the
stable encrypted error class.
