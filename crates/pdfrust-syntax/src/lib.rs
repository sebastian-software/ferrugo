//! PDF byte syntax primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "syntax";

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "syntax");
    }
}
