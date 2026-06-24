//! Backend-neutral thumbnail generation facade.

/// Returns the crate version compiled into this library.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_should_match_package_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
