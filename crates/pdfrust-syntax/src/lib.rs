//! PDF byte syntax primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "syntax";

/// Result alias for syntax parsing operations.
pub type SyntaxResult<T> = Result<T, SyntaxError>;

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Byte offset into the original PDF input.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(usize);

impl ByteOffset {
    /// Creates a byte offset from a zero-based index.
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based byte index.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "byte {}", self.0)
    }
}

/// Borrowed PDF bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfBytes<'a> {
    bytes: &'a [u8],
}

impl<'a> PdfBytes<'a> {
    /// Creates a borrowed PDF byte input.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the borrowed bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the input length in bytes.
    #[must_use]
    pub const fn len(self) -> usize {
        self.bytes.len()
    }

    /// Returns true when the input contains no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Creates a cursor at the beginning of the input.
    #[must_use]
    pub const fn cursor(self) -> ByteCursor<'a> {
        ByteCursor {
            bytes: self.bytes,
            offset: ByteOffset::new(0),
        }
    }
}

/// Cursor over borrowed PDF bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: ByteOffset,
}

impl<'a> ByteCursor<'a> {
    /// Returns the current byte offset.
    #[must_use]
    pub const fn offset(self) -> ByteOffset {
        self.offset
    }

    /// Returns the unread bytes from the current offset.
    #[must_use]
    pub fn remaining(self) -> &'a [u8] {
        &self.bytes[self.offset.get()..]
    }

    /// Returns the next byte without advancing the cursor.
    #[must_use]
    pub fn peek(self) -> Option<u8> {
        self.bytes.get(self.offset.get()).copied()
    }

    /// Reads one byte and advances the cursor.
    pub fn read_byte(&mut self) -> SyntaxResult<u8> {
        let byte = self
            .peek()
            .ok_or_else(|| SyntaxError::new(self.offset, SyntaxErrorKind::UnexpectedEof))?;
        self.offset = ByteOffset::new(self.offset.get() + 1);
        Ok(byte)
    }

    /// Advances the cursor by `count` bytes.
    pub fn advance(&mut self, count: usize) -> SyntaxResult<()> {
        let next = self
            .offset
            .get()
            .checked_add(count)
            .ok_or_else(|| SyntaxError::new(self.offset, SyntaxErrorKind::MalformedInput))?;
        if next > self.bytes.len() {
            return Err(SyntaxError::new(
                ByteOffset::new(self.bytes.len()),
                SyntaxErrorKind::UnexpectedEof,
            ));
        }
        self.offset = ByteOffset::new(next);
        Ok(())
    }
}

/// Syntax parser error with the source byte offset that triggered the failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxError {
    offset: ByteOffset,
    kind: SyntaxErrorKind,
}

impl SyntaxError {
    /// Creates a syntax error at a byte offset.
    #[must_use]
    pub const fn new(offset: ByteOffset, kind: SyntaxErrorKind) -> Self {
        Self { offset, kind }
    }

    /// Returns the source offset for the error.
    #[must_use]
    pub const fn offset(self) -> ByteOffset {
        self.offset
    }

    /// Returns the error kind.
    #[must_use]
    pub const fn kind(self) -> SyntaxErrorKind {
        self.kind
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.offset)
    }
}

impl std::error::Error for SyntaxError {}

/// Syntax parser error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxErrorKind {
    /// Input is structurally malformed.
    MalformedInput,
    /// Parser reached the end of input before a complete token was available.
    UnexpectedEof,
    /// Parser found a byte sequence that cannot form the requested token.
    InvalidToken,
    /// Parser encountered syntax that is valid PDF but unsupported for now.
    Unsupported,
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedInput => f.write_str("malformed PDF syntax"),
            Self::UnexpectedEof => f.write_str("unexpected end of input"),
            Self::InvalidToken => f.write_str("invalid PDF token"),
            Self::Unsupported => f.write_str("unsupported PDF syntax"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "syntax");
    }

    #[test]
    fn pdf_bytes_should_borrow_input() {
        let raw = b"%PDF-1.7";
        let input = PdfBytes::new(raw);

        assert_eq!(input.as_bytes().as_ptr(), raw.as_ptr());
        assert_eq!(input.len(), raw.len());
        assert!(!input.is_empty());
    }

    #[test]
    fn cursor_should_track_offsets_while_reading() {
        let mut cursor = PdfBytes::new(b"abc").cursor();

        assert_eq!(cursor.offset(), ByteOffset::new(0));
        assert_eq!(cursor.read_byte().expect("byte"), b'a');
        assert_eq!(cursor.offset(), ByteOffset::new(1));
        assert_eq!(cursor.peek(), Some(b'b'));
    }

    #[test]
    fn cursor_should_report_unexpected_eof_offset() {
        let mut cursor = PdfBytes::new(b"a").cursor();
        cursor.advance(1).expect("advance to eof");

        let error = cursor.read_byte().expect_err("eof should fail");

        assert_eq!(error.offset(), ByteOffset::new(1));
        assert_eq!(error.kind(), SyntaxErrorKind::UnexpectedEof);
    }

    #[test]
    fn cursor_should_reject_advance_past_end() {
        let mut cursor = PdfBytes::new(b"abc").cursor();

        let error = cursor.advance(4).expect_err("past end should fail");

        assert_eq!(error.offset(), ByteOffset::new(3));
        assert_eq!(error.kind(), SyntaxErrorKind::UnexpectedEof);
    }

    #[test]
    fn syntax_error_display_should_include_kind_and_offset() {
        let error = SyntaxError::new(ByteOffset::new(42), SyntaxErrorKind::InvalidToken);

        assert_eq!(error.to_string(), "invalid PDF token at byte 42");
    }
}
