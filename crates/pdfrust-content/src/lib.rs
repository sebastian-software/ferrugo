//! Page content interpretation for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

use pdfrust_syntax::{
    parse_primitive_prefix, ByteOffset, PdfBytes, PdfPrimitive, SyntaxError, SyntaxErrorKind,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "content";

/// Result alias for content-stream operations.
pub type ContentResult<T> = Result<T, ContentError>;

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

/// Creates a borrowed tokenizer over one decoded PDF content stream.
#[must_use]
pub const fn tokenize_content(input: PdfBytes<'_>) -> ContentTokenizer<'_> {
    ContentTokenizer::new(input)
}

/// Borrowed content-stream token iterator.
#[derive(Debug, Clone)]
pub struct ContentTokenizer<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> ContentTokenizer<'a> {
    /// Creates a tokenizer over borrowed content stream bytes.
    #[must_use]
    pub const fn new(input: PdfBytes<'a>) -> Self {
        Self {
            input: input.as_bytes(),
            offset: 0,
        }
    }

    fn next_token(&mut self) -> ContentResult<Option<ContentToken<'a>>> {
        self.skip_whitespace_and_comments();
        if self.offset == self.input.len() {
            return Ok(None);
        }

        let offset = ByteOffset::new(self.offset);
        if should_parse_primitive(&self.input[self.offset..]) {
            let (value, consumed) =
                parse_primitive_prefix(PdfBytes::new(&self.input[self.offset..]))
                    .map_err(|error| ContentError::from_syntax(self.offset, error))?;
            self.offset += consumed.get();
            return Ok(Some(ContentToken::Operand { offset, value }));
        }

        if is_delimiter(self.input[self.offset]) {
            return Err(ContentError::new(offset, ContentErrorKind::InvalidOperator));
        }

        let start = self.offset;
        while self
            .input
            .get(self.offset)
            .is_some_and(|byte| !is_whitespace(*byte) && !is_delimiter(*byte))
        {
            self.offset += 1;
        }

        Ok(Some(ContentToken::Operator {
            offset,
            name: OperatorName::new(&self.input[start..self.offset]),
        }))
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.input.get(self.offset).copied() {
                Some(byte) if is_whitespace(byte) => {
                    self.offset += 1;
                }
                Some(b'%') => {
                    self.skip_comment();
                }
                _ => return,
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(byte) = self.input.get(self.offset).copied() {
            self.offset += 1;
            if byte == b'\n' || byte == b'\r' {
                break;
            }
        }
    }
}

impl<'a> Iterator for ContentTokenizer<'a> {
    type Item = ContentResult<ContentToken<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(error) => {
                self.offset = self.input.len();
                Some(Err(error))
            }
        }
    }
}

/// A token from a decoded PDF content stream.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentToken<'a> {
    /// Operand parsed with the shared PDF primitive parser.
    Operand {
        /// Byte offset where the operand starts.
        offset: ByteOffset,
        /// Parsed operand value.
        value: PdfPrimitive<'a>,
    },
    /// Graphics/text/content operator.
    Operator {
        /// Byte offset where the operator starts.
        offset: ByteOffset,
        /// Borrowed operator name bytes.
        name: OperatorName<'a>,
    },
}

/// Borrowed content-stream operator name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorName<'a> {
    bytes: &'a [u8],
}

impl<'a> OperatorName<'a> {
    /// Creates a borrowed operator name.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the borrowed operator bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the operator as UTF-8 text when valid.
    #[must_use]
    pub fn as_str(self) -> Option<&'a str> {
        std::str::from_utf8(self.bytes).ok()
    }
}

/// Content-stream tokenizer error with a source byte offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentError {
    offset: ByteOffset,
    kind: ContentErrorKind,
}

impl ContentError {
    /// Creates a content error at a byte offset.
    #[must_use]
    pub const fn new(offset: ByteOffset, kind: ContentErrorKind) -> Self {
        Self { offset, kind }
    }

    /// Returns the source offset for the error.
    #[must_use]
    pub const fn offset(&self) -> ByteOffset {
        self.offset
    }

    /// Returns the content error kind.
    #[must_use]
    pub const fn kind(&self) -> ContentErrorKind {
        self.kind
    }

    fn from_syntax(base_offset: usize, error: SyntaxError) -> Self {
        let offset = ByteOffset::new(base_offset + error.offset().get());
        let kind = match error.kind() {
            SyntaxErrorKind::MalformedInput => ContentErrorKind::MalformedOperand,
            SyntaxErrorKind::UnexpectedEof => ContentErrorKind::UnexpectedEof,
            SyntaxErrorKind::InvalidToken => ContentErrorKind::InvalidOperand,
            SyntaxErrorKind::Unsupported => ContentErrorKind::UnsupportedOperand,
        };
        Self { offset, kind }
    }
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.offset)
    }
}

impl std::error::Error for ContentError {}

/// Content-stream tokenizer error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentErrorKind {
    /// Operand syntax is malformed.
    MalformedOperand,
    /// Operand reached EOF before completion.
    UnexpectedEof,
    /// Operand token is invalid.
    InvalidOperand,
    /// Operand syntax is valid PDF but unsupported for now.
    UnsupportedOperand,
    /// Operator token starts with a delimiter or is otherwise invalid.
    InvalidOperator,
}

impl fmt::Display for ContentErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedOperand => f.write_str("malformed content operand"),
            Self::UnexpectedEof => f.write_str("unexpected end of content stream"),
            Self::InvalidOperand => f.write_str("invalid content operand"),
            Self::UnsupportedOperand => f.write_str("unsupported content operand"),
            Self::InvalidOperator => f.write_str("invalid content operator"),
        }
    }
}

fn should_parse_primitive(input: &[u8]) -> bool {
    match input.first().copied() {
        Some(b'/') | Some(b'(') | Some(b'[') | Some(b'<') | Some(b'+') | Some(b'-')
        | Some(b'.') | Some(b'0'..=b'9') => true,
        Some(b'n') => starts_with_keyword(input, b"null"),
        Some(b't') => starts_with_keyword(input, b"true"),
        Some(b'f') => starts_with_keyword(input, b"false"),
        _ => false,
    }
}

fn starts_with_keyword(input: &[u8], keyword: &[u8]) -> bool {
    if !input.starts_with(keyword) {
        return false;
    }
    match input.get(keyword.len()) {
        Some(byte) => is_whitespace(*byte) || is_delimiter(*byte),
        None => true,
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

#[cfg(test)]
mod tests {
    use super::*;
    use pdfrust_object::{
        load_classic_document, GenerationNumber, ObjectId, ObjectNumber, ObjectValue,
    };
    use pdfrust_syntax::{PdfNumber, PdfPrimitive};

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "content");
    }

    #[test]
    fn content_should_depend_on_object_model() {
        assert_eq!(object_role(), "object");
    }

    #[test]
    fn tokenizer_should_parse_operands_and_operators() {
        let tokens = tokenize_content(PdfBytes::new(b"q 1 0 0 1 10 20 cm /Im0 Do Q"))
            .collect::<ContentResult<Vec<_>>>()
            .expect("valid content stream");

        assert_eq!(
            tokens[0],
            ContentToken::Operator {
                offset: ByteOffset::new(0),
                name: OperatorName::new(b"q"),
            }
        );
        assert_eq!(
            tokens[1],
            ContentToken::Operand {
                offset: ByteOffset::new(2),
                value: PdfPrimitive::Number(PdfNumber::Integer(1)),
            }
        );
        assert_eq!(
            tokens[7],
            ContentToken::Operator {
                offset: ByteOffset::new(16),
                name: OperatorName::new(b"cm"),
            }
        );
        assert_eq!(
            tokens[9],
            ContentToken::Operator {
                offset: ByteOffset::new(24),
                name: OperatorName::new(b"Do"),
            }
        );
    }

    #[test]
    fn tokenizer_should_parse_generated_fixture_content_stream() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let document = load_classic_document(PdfBytes::new(bytes))
            .expect("fixture should load as classic PDF");
        let content_id = ObjectId::new(
            ObjectNumber::new(1).expect("object number"),
            GenerationNumber::new(0),
        );
        let content = document
            .objects
            .get(content_id)
            .expect("content stream object");
        let ObjectValue::Stream(stream) = &content.value else {
            panic!("content object should be a stream");
        };
        let decoded = stream.decode().expect("content stream should decode");
        let tokens = tokenize_content(PdfBytes::new(&decoded))
            .collect::<ContentResult<Vec<_>>>()
            .expect("fixture content stream should tokenize");

        assert_eq!(
            tokens[0],
            ContentToken::Operator {
                offset: ByteOffset::new(0),
                name: OperatorName::new(b"BT"),
            }
        );
        assert!(tokens.iter().any(|token| {
            matches!(
                token,
                ContentToken::Operator { name, .. } if name.as_bytes() == b"Tj"
            )
        }));
        assert_eq!(
            tokens.last(),
            Some(&ContentToken::Operator {
                offset: ByteOffset::new(53),
                name: OperatorName::new(b"ET"),
            })
        );
    }

    #[test]
    fn tokenizer_should_skip_comments() {
        let tokens = tokenize_content(PdfBytes::new(b"q % ignore this\nQ"))
            .collect::<ContentResult<Vec<_>>>()
            .expect("valid content stream");

        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[1],
            ContentToken::Operator {
                offset: ByteOffset::new(16),
                name: OperatorName::new(b"Q"),
            }
        );
    }

    #[test]
    fn tokenizer_should_keep_boolean_operands_distinct_from_operators() {
        let tokens = tokenize_content(PdfBytes::new(b"false f true n"))
            .collect::<ContentResult<Vec<_>>>()
            .expect("valid content stream");

        assert_eq!(
            tokens[0],
            ContentToken::Operand {
                offset: ByteOffset::new(0),
                value: PdfPrimitive::Boolean(false),
            }
        );
        assert_eq!(
            tokens[1],
            ContentToken::Operator {
                offset: ByteOffset::new(6),
                name: OperatorName::new(b"f"),
            }
        );
        assert_eq!(
            tokens[3],
            ContentToken::Operator {
                offset: ByteOffset::new(13),
                name: OperatorName::new(b"n"),
            }
        );
    }

    #[test]
    fn tokenizer_should_report_operand_error_with_absolute_offset() {
        let error = tokenize_content(PdfBytes::new(b"q (unterminated"))
            .collect::<ContentResult<Vec<_>>>()
            .expect_err("unterminated literal string should fail");

        assert_eq!(error.offset(), ByteOffset::new(2));
        assert_eq!(error.kind(), ContentErrorKind::UnexpectedEof);
    }

    #[test]
    fn tokenizer_should_report_invalid_operator_delimiter() {
        let error = tokenize_content(PdfBytes::new(b"q ] Q"))
            .collect::<ContentResult<Vec<_>>>()
            .expect_err("delimiter cannot start operator");

        assert_eq!(error.offset(), ByteOffset::new(2));
        assert_eq!(error.kind(), ContentErrorKind::InvalidOperator);
    }

    #[test]
    fn operator_name_should_borrow_bytes() {
        let name = OperatorName::new(b"BT");

        assert_eq!(name.as_bytes(), b"BT");
        assert_eq!(name.as_str(), Some("BT"));
    }
}
