# Empty-Password Encryption Support - 2026-07-04

## Scope

Issue #101 narrows the encrypted-document boundary. Ferrugo now opens standard
security handler PDFs when the empty user password validates, which covers the
common permissions-only class used to set print/copy flags. Documents requiring
a non-empty password still return the public `encrypted` error class.

Supported in this slice:

- RC4 V1/V2 object stream/string decryption.
- AES-128 V4 `/AESV2` object stream/string decryption.
- AES-256 V5 `/AESV3` object stream/string decryption.
- `/Identity` stream/string crypt filters.
- Fail-closed behavior for missing, malformed, non-standard, or non-empty
  password security dictionaries.

Permission bits are parsed only as security-handler input. Ferrugo does not
interpret, enforce, bypass, or expose permissions as product policy.

## Dependency Decision

The object crate now depends on audited pure-Rust RustCrypto primitives instead
of hand-rolled cryptography:

- `md-5` for the PDF standard security handler revisions 2-4 key derivation.
- `sha2` for revision 5 validation/key derivation.
- `rc4` for V1/V2 object data.
- `aes` and `cbc` for AES-CBC stream/string payloads and revision 5 key
  wrapping fields.

This is intentionally kept inside `ferrugo-object`, where encrypted object bytes
enter the trusted renderer pipeline. Downstream render/content consumers keep
using `StreamObject::raw()` and `StreamObject::decode()`; the object layer
decrypts stream payloads before filter decoding.

## Validation

Unit coverage builds deterministic in-memory PDFs for:

- empty-user-password RC4 revision 2 stream decryption;
- empty-user-password AES-128 revision 4 `/AESV2` stream decryption;
- empty-user-password AES-256 revision 5 `/AESV3` stream decryption;
- encrypted trailer/catalog failure paths that must still return
  `ObjectError::Encrypted`.

Local validation:

```sh
cargo test -p ferrugo-object
cargo test -p ferrugo-render
cargo test -p ferrugo-content
cargo check -p ferrugo-native
cargo test -p ferrugo-native
cargo test -p ferrugo
cargo test -p ferrugo-native native_backend_should_report_encrypted_generated_fixture_for_render -- --nocapture
cargo run -p ferrugo -- render-native /private/tmp/ferrugo-empty-password-aesv2.pdf --output /private/tmp/ferrugo-empty-password-aesv2.png --max-edge 128
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/check_fuzz_smoke.sh
git diff --check
```

The native CLI smoke used a temporary AES-128 V4 `/AESV2` permissions-only PDF
generated independently from the Rust unit-test fixture builder.
