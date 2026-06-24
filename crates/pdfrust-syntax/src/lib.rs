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

    /// Returns the original input bytes behind the cursor.
    #[must_use]
    pub const fn input(self) -> &'a [u8] {
        self.bytes
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

/// Parsed core PDF primitive value.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfPrimitive<'a> {
    /// `null`.
    Null,
    /// Boolean value.
    Boolean(bool),
    /// Integer or real number.
    Number(PdfNumber),
    /// Name object without the leading slash.
    Name(PdfName<'a>),
    /// Literal or hexadecimal string.
    String(PdfString<'a>),
    /// Array object.
    Array(Vec<PdfPrimitive<'a>>),
    /// Dictionary object.
    Dictionary(Vec<(PdfName<'a>, PdfPrimitive<'a>)>),
    /// Indirect reference, such as `12 0 R`.
    Reference(PdfReference),
}

/// Parsed PDF number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfNumber {
    /// Integer number.
    Integer(i64),
    /// Real number.
    Real(f64),
}

/// Parsed PDF indirect reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfReference {
    /// Object number.
    pub object: u32,
    /// Generation number.
    pub generation: u16,
}

impl PdfReference {
    /// Creates a parsed indirect reference.
    #[must_use]
    pub const fn new(object: u32, generation: u16) -> Self {
        Self { object, generation }
    }
}

/// Parsed PDF name bytes without the leading slash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfName<'a> {
    bytes: &'a [u8],
}

impl<'a> PdfName<'a> {
    /// Creates a borrowed PDF name.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the borrowed raw name bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// Parsed PDF string bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfString<'a> {
    /// Literal string content between `(` and `)`, with escapes still raw.
    Literal(&'a [u8]),
    /// Hexadecimal string content between `<` and `>`, still undecoded.
    Hex(&'a [u8]),
}

/// Parses one complete core PDF primitive.
///
/// # Errors
///
/// Returns [`SyntaxError`] when the input is empty, malformed, or has trailing
/// non-whitespace bytes after the primitive.
pub fn parse_primitive(input: PdfBytes<'_>) -> SyntaxResult<PdfPrimitive<'_>> {
    let mut parser = PrimitiveParser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace_and_comments()?;
    if parser.cursor.peek().is_some() {
        return Err(SyntaxError::new(
            parser.cursor.offset(),
            SyntaxErrorKind::InvalidToken,
        ));
    }
    Ok(value)
}

struct PrimitiveParser<'a> {
    cursor: ByteCursor<'a>,
}

impl<'a> PrimitiveParser<'a> {
    const fn new(input: PdfBytes<'a>) -> Self {
        Self {
            cursor: input.cursor(),
        }
    }

    fn parse_value(&mut self) -> SyntaxResult<PdfPrimitive<'a>> {
        self.skip_whitespace_and_comments()?;
        let offset = self.cursor.offset();
        let byte = self
            .cursor
            .peek()
            .ok_or_else(|| SyntaxError::new(offset, SyntaxErrorKind::UnexpectedEof))?;

        match byte {
            b'n' => {
                self.consume_keyword(b"null")?;
                Ok(PdfPrimitive::Null)
            }
            b't' => {
                self.consume_keyword(b"true")?;
                Ok(PdfPrimitive::Boolean(true))
            }
            b'f' => {
                self.consume_keyword(b"false")?;
                Ok(PdfPrimitive::Boolean(false))
            }
            b'/' => self.parse_name().map(PdfPrimitive::Name),
            b'(' => self.parse_literal_string().map(PdfPrimitive::String),
            b'[' => self.parse_array(),
            b'<' if self.starts_with(b"<<") => self.parse_dictionary(),
            b'<' => self.parse_hex_string().map(PdfPrimitive::String),
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.parse_number_or_reference(),
            _ => Err(SyntaxError::new(offset, SyntaxErrorKind::InvalidToken)),
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> SyntaxResult<()> {
        loop {
            match self.cursor.peek() {
                Some(byte) if is_whitespace(byte) => {
                    self.cursor.advance(1)?;
                }
                Some(b'%') => {
                    self.skip_comment()?;
                }
                _ => return Ok(()),
            }
        }
    }

    fn skip_comment(&mut self) -> SyntaxResult<()> {
        while let Some(byte) = self.cursor.peek() {
            self.cursor.advance(1)?;
            if byte == b'\n' || byte == b'\r' {
                break;
            }
        }
        Ok(())
    }

    fn consume_keyword(&mut self, keyword: &[u8]) -> SyntaxResult<()> {
        let offset = self.cursor.offset();
        if !self.starts_with(keyword) {
            return Err(SyntaxError::new(offset, SyntaxErrorKind::InvalidToken));
        }
        self.cursor.advance(keyword.len())?;
        match self.cursor.peek() {
            Some(byte) if !is_whitespace(byte) && !is_delimiter(byte) => Err(SyntaxError::new(
                self.cursor.offset(),
                SyntaxErrorKind::InvalidToken,
            )),
            _ => Ok(()),
        }
    }

    fn parse_name(&mut self) -> SyntaxResult<PdfName<'a>> {
        let offset = self.cursor.offset();
        self.expect_byte(b'/')?;
        let start = self.cursor.offset().get();
        while let Some(byte) = self.cursor.peek() {
            if is_whitespace(byte) || is_delimiter(byte) {
                break;
            }
            self.cursor.advance(1)?;
        }
        let end = self.cursor.offset().get();
        if start == end {
            return Err(SyntaxError::new(offset, SyntaxErrorKind::InvalidToken));
        }
        Ok(PdfName::new(&self.cursor.input()[start..end]))
    }

    fn parse_literal_string(&mut self) -> SyntaxResult<PdfString<'a>> {
        let offset = self.cursor.offset();
        self.expect_byte(b'(')?;
        let start = self.cursor.offset().get();
        let mut depth = 1_u32;
        let mut escaped = false;

        while self.cursor.peek().is_some() {
            let byte_offset = self.cursor.offset().get();
            let byte = self.cursor.read_byte()?;
            if escaped {
                escaped = false;
                continue;
            }
            match byte {
                b'\\' => escaped = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(PdfString::Literal(&self.cursor.input()[start..byte_offset]));
                    }
                }
                _ => {}
            }
        }

        Err(SyntaxError::new(offset, SyntaxErrorKind::UnexpectedEof))
    }

    fn parse_hex_string(&mut self) -> SyntaxResult<PdfString<'a>> {
        let offset = self.cursor.offset();
        self.expect_byte(b'<')?;
        let start = self.cursor.offset().get();
        while let Some(byte) = self.cursor.peek() {
            let byte_offset = self.cursor.offset().get();
            self.cursor.advance(1)?;
            if byte == b'>' {
                return Ok(PdfString::Hex(&self.cursor.input()[start..byte_offset]));
            }
        }
        Err(SyntaxError::new(offset, SyntaxErrorKind::UnexpectedEof))
    }

    fn parse_number_or_reference(&mut self) -> SyntaxResult<PdfPrimitive<'a>> {
        let first_offset = self.cursor.offset();
        let first_raw = self.parse_number_raw()?;
        let after_first = self.cursor;

        if is_unsigned_integer_raw(first_raw) {
            self.skip_whitespace_and_comments()?;
            let second_offset = self.cursor.offset();
            if let Ok(second_raw) = self.parse_number_raw() {
                if is_unsigned_integer_raw(second_raw) {
                    self.skip_whitespace_and_comments()?;
                    if self.cursor.peek() == Some(b'R') {
                        self.cursor.advance(1)?;
                        match self.cursor.peek() {
                            Some(byte) if !is_whitespace(byte) && !is_delimiter(byte) => {}
                            _ => {
                                let object = parse_u32(first_raw, first_offset)?;
                                let generation = parse_u16(second_raw, second_offset)?;
                                return Ok(PdfPrimitive::Reference(PdfReference::new(
                                    object, generation,
                                )));
                            }
                        }
                    }
                }
            }
        }

        self.cursor = after_first;
        parse_number_raw_value(first_raw, first_offset).map(PdfPrimitive::Number)
    }

    fn parse_number_raw(&mut self) -> SyntaxResult<&'a [u8]> {
        let start = self.cursor.offset().get();
        while let Some(byte) = self.cursor.peek() {
            if is_whitespace(byte) || is_delimiter(byte) {
                break;
            }
            self.cursor.advance(1)?;
        }
        if start == self.cursor.offset().get() {
            return Err(SyntaxError::new(
                ByteOffset::new(start),
                SyntaxErrorKind::InvalidToken,
            ));
        }
        Ok(&self.cursor.input()[start..self.cursor.offset().get()])
    }

    fn parse_array(&mut self) -> SyntaxResult<PdfPrimitive<'a>> {
        self.expect_byte(b'[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments()?;
            match self.cursor.peek() {
                Some(b']') => {
                    self.cursor.advance(1)?;
                    return Ok(PdfPrimitive::Array(values));
                }
                Some(_) => values.push(self.parse_value()?),
                None => {
                    return Err(SyntaxError::new(
                        self.cursor.offset(),
                        SyntaxErrorKind::UnexpectedEof,
                    ));
                }
            }
        }
    }

    fn parse_dictionary(&mut self) -> SyntaxResult<PdfPrimitive<'a>> {
        self.expect_bytes(b"<<")?;
        let mut entries = Vec::new();
        loop {
            self.skip_whitespace_and_comments()?;
            if self.starts_with(b">>") {
                self.cursor.advance(2)?;
                return Ok(PdfPrimitive::Dictionary(entries));
            }
            if self.cursor.peek().is_none() {
                return Err(SyntaxError::new(
                    self.cursor.offset(),
                    SyntaxErrorKind::UnexpectedEof,
                ));
            }
            let key = self.parse_name()?;
            let value = self.parse_value()?;
            entries.push((key, value));
        }
    }

    fn expect_byte(&mut self, expected: u8) -> SyntaxResult<()> {
        let offset = self.cursor.offset();
        match self.cursor.read_byte() {
            Ok(byte) if byte == expected => Ok(()),
            Ok(_) => Err(SyntaxError::new(offset, SyntaxErrorKind::InvalidToken)),
            Err(error) => Err(error),
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> SyntaxResult<()> {
        let offset = self.cursor.offset();
        if !self.starts_with(expected) {
            return Err(SyntaxError::new(offset, SyntaxErrorKind::InvalidToken));
        }
        self.cursor.advance(expected.len())
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.cursor.remaining().starts_with(prefix)
    }
}

const fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\0' | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

const fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn parse_number_raw_value(raw: &[u8], offset: ByteOffset) -> SyntaxResult<PdfNumber> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))?;
    if text.contains('.') {
        text.parse::<f64>()
            .map(PdfNumber::Real)
            .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))
    } else {
        text.parse::<i64>()
            .map(PdfNumber::Integer)
            .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))
    }
}

fn parse_u32(raw: &[u8], offset: ByteOffset) -> SyntaxResult<u32> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))?;
    text.parse::<u32>()
        .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))
}

fn parse_u16(raw: &[u8], offset: ByteOffset) -> SyntaxResult<u16> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))?;
    text.parse::<u16>()
        .map_err(|_| SyntaxError::new(offset, SyntaxErrorKind::InvalidToken))
}

const fn is_unsigned_integer_raw(raw: &[u8]) -> bool {
    if raw.is_empty() {
        return false;
    }
    let mut index = 0;
    while index < raw.len() {
        if !raw[index].is_ascii_digit() {
            return false;
        }
        index += 1;
    }
    true
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

        assert_eq!(cursor.input(), b"abc");
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

    #[test]
    fn parse_primitive_should_parse_scalar_values() {
        assert_eq!(
            parse_primitive(PdfBytes::new(b" null ")).expect("null"),
            PdfPrimitive::Null
        );
        assert_eq!(
            parse_primitive(PdfBytes::new(b"true")).expect("boolean"),
            PdfPrimitive::Boolean(true)
        );
        assert_eq!(
            parse_primitive(PdfBytes::new(b"-42")).expect("integer"),
            PdfPrimitive::Number(PdfNumber::Integer(-42))
        );
        assert_eq!(
            parse_primitive(PdfBytes::new(b"3.5")).expect("real"),
            PdfPrimitive::Number(PdfNumber::Real(3.5))
        );
        assert_eq!(
            parse_primitive(PdfBytes::new(b"12 0 R")).expect("reference"),
            PdfPrimitive::Reference(PdfReference::new(12, 0))
        );
    }

    #[test]
    fn parse_primitive_should_parse_names_and_strings_as_borrowed_slices() {
        let name = parse_primitive(PdfBytes::new(b"/Type")).expect("name");
        let literal = parse_primitive(PdfBytes::new(b"(hello\\) world)")).expect("literal");
        let hex = parse_primitive(PdfBytes::new(b"<48656c6c6f>")).expect("hex");

        assert_eq!(name, PdfPrimitive::Name(PdfName::new(b"Type")));
        assert_eq!(
            literal,
            PdfPrimitive::String(PdfString::Literal(b"hello\\) world"))
        );
        assert_eq!(hex, PdfPrimitive::String(PdfString::Hex(b"48656c6c6f")));
    }

    #[test]
    fn parse_primitive_should_parse_arrays_and_dictionaries() {
        let value = parse_primitive(PdfBytes::new(
            b"<< /Type /Page /Parent 1 0 R /MediaBox [0 0 300 160] /Visible true >>",
        ))
        .expect("dictionary");

        assert_eq!(
            value,
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"Type"),
                    PdfPrimitive::Name(PdfName::new(b"Page")),
                ),
                (
                    PdfName::new(b"Parent"),
                    PdfPrimitive::Reference(PdfReference::new(1, 0)),
                ),
                (
                    PdfName::new(b"MediaBox"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(300)),
                        PdfPrimitive::Number(PdfNumber::Integer(160)),
                    ]),
                ),
                (PdfName::new(b"Visible"), PdfPrimitive::Boolean(true)),
            ])
        );
    }

    #[test]
    fn parse_primitive_should_skip_comments_and_whitespace() {
        let value = parse_primitive(PdfBytes::new(b"% comment\n[1 % inline\r\n2]"))
            .expect("array with comments");

        assert_eq!(
            value,
            PdfPrimitive::Array(vec![
                PdfPrimitive::Number(PdfNumber::Integer(1)),
                PdfPrimitive::Number(PdfNumber::Integer(2)),
            ])
        );
    }

    #[test]
    fn parse_primitive_should_reject_trailing_tokens() {
        let error = parse_primitive(PdfBytes::new(b"true false")).expect_err("trailing token");

        assert_eq!(error.offset(), ByteOffset::new(5));
        assert_eq!(error.kind(), SyntaxErrorKind::InvalidToken);
    }

    #[test]
    fn parse_primitive_should_report_malformed_dictionary_offset() {
        let error =
            parse_primitive(PdfBytes::new(b"<< /Type >>")).expect_err("missing value should fail");

        assert_eq!(error.offset(), ByteOffset::new(9));
        assert_eq!(error.kind(), SyntaxErrorKind::InvalidToken);
    }
}
