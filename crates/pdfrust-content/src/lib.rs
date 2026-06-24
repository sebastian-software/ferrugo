//! Page content interpretation for the Rust-native renderer.

#![forbid(unsafe_code)]

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "content";

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the lower-level object-model dependency.
#[must_use]
pub fn object_role() -> &'static str {
    pdfrust_object::crate_role()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "content");
    }

    #[test]
    fn content_should_depend_on_object_model() {
        assert_eq!(object_role(), "object");
    }
}
