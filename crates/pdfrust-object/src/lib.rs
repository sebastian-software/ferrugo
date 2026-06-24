//! Safe PDF object model for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

use pdfrust_syntax::{
    parse_primitive, ByteCursor, ByteOffset, PdfBytes, PdfPrimitive, SyntaxError,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "object";

/// Result alias for object-model operations.
pub type ObjectResult<T> = Result<T, ObjectError>;

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the lower-level syntax dependency.
#[must_use]
pub fn syntax_role() -> &'static str {
    pdfrust_syntax::crate_role()
}

/// Non-zero PDF object number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectNumber(u32);

impl ObjectNumber {
    /// Creates an object number.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when `value` is zero.
    pub fn new(value: u32) -> ObjectResult<Self> {
        if value == 0 {
            return Err(ObjectError::malformed(
                ByteOffset::new(0),
                "object number must be greater than zero",
            ));
        }
        Ok(Self(value))
    }

    /// Returns the raw object number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// PDF generation number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenerationNumber(u16);

impl GenerationNumber {
    /// Creates a generation number.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the raw generation number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// PDF indirect object identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    /// Object number.
    pub number: ObjectNumber,
    /// Generation number.
    pub generation: GenerationNumber,
}

impl ObjectId {
    /// Creates an object identifier.
    #[must_use]
    pub const fn new(number: ObjectNumber, generation: GenerationNumber) -> Self {
        Self { number, generation }
    }
}

/// PDF indirect reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reference {
    /// Referenced object identifier.
    pub id: ObjectId,
}

impl Reference {
    /// Creates a reference to an object identifier.
    #[must_use]
    pub const fn new(id: ObjectId) -> Self {
        Self { id }
    }
}

/// Parsed indirect PDF object.
#[derive(Debug, Clone, PartialEq)]
pub struct IndirectObject<'a> {
    /// Object identifier from the indirect object header.
    pub id: ObjectId,
    /// Parsed object value between `obj` and `endobj`.
    pub value: PdfPrimitive<'a>,
}

/// Object table with duplicate detection.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ObjectTable<'a> {
    objects: Vec<IndirectObject<'a>>,
}

impl<'a> ObjectTable<'a> {
    /// Creates an empty object table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Inserts an indirect object.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError::DuplicateObject`] when the object ID already
    /// exists in the table.
    pub fn insert(&mut self, object: IndirectObject<'a>) -> ObjectResult<()> {
        if self.objects.iter().any(|existing| existing.id == object.id) {
            return Err(ObjectError::DuplicateObject { id: object.id });
        }
        self.objects.push(object);
        Ok(())
    }

    /// Returns an object by ID.
    #[must_use]
    pub fn get(&self, id: ObjectId) -> Option<&IndirectObject<'a>> {
        self.objects.iter().find(|object| object.id == id)
    }

    /// Returns all objects in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &IndirectObject<'a>> {
        self.objects.iter()
    }

    /// Returns the number of objects in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Returns true when the table contains no objects.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

/// Parses one indirect reference such as `12 0 R`.
///
/// # Errors
///
/// Returns [`ObjectError`] when the reference is malformed or has trailing
/// tokens.
pub fn parse_reference(input: PdfBytes<'_>) -> ObjectResult<Reference> {
    let mut parser = ObjectParser::new(input);
    let id = parser.parse_object_id()?;
    parser.skip_whitespace()?;
    parser.consume_keyword(b"R")?;
    parser.skip_whitespace()?;
    if parser.cursor.peek().is_some() {
        return Err(ObjectError::malformed(
            parser.cursor.offset(),
            "trailing data after indirect reference",
        ));
    }
    Ok(Reference::new(id))
}

/// Parses one indirect object such as `12 0 obj ... endobj`.
///
/// # Errors
///
/// Returns [`ObjectError`] when the object header, body, or terminator is
/// malformed.
pub fn parse_indirect_object(input: PdfBytes<'_>) -> ObjectResult<IndirectObject<'_>> {
    let mut parser = ObjectParser::new(input);
    let id = parser.parse_object_id()?;
    parser.skip_whitespace()?;
    parser.consume_keyword(b"obj")?;
    parser.skip_whitespace()?;
    let body_start = parser.cursor.offset().get();
    let remaining = parser.cursor.remaining();
    let endobj_start = find_keyword(remaining, b"endobj").ok_or_else(|| {
        ObjectError::malformed(parser.cursor.offset(), "indirect object is missing endobj")
    })?;
    let body_end = body_start + endobj_start;
    let body = &parser.cursor.input()[body_start..body_end];
    let value = parse_primitive(PdfBytes::new(body))?;
    parser.cursor.advance(endobj_start + b"endobj".len())?;
    parser.skip_whitespace()?;
    if parser.cursor.peek().is_some() {
        return Err(ObjectError::malformed(
            parser.cursor.offset(),
            "trailing data after indirect object",
        ));
    }
    Ok(IndirectObject { id, value })
}

struct ObjectParser<'a> {
    cursor: ByteCursor<'a>,
}

impl<'a> ObjectParser<'a> {
    const fn new(input: PdfBytes<'a>) -> Self {
        Self {
            cursor: input.cursor(),
        }
    }

    fn parse_object_id(&mut self) -> ObjectResult<ObjectId> {
        self.skip_whitespace()?;
        let number_offset = self.cursor.offset();
        let number = parse_u32(self.parse_unsigned_decimal()?, number_offset)?;
        let number = ObjectNumber::new(number).map_err(|_| {
            ObjectError::malformed(number_offset, "object number must be greater than zero")
        })?;
        self.skip_whitespace()?;
        let generation_offset = self.cursor.offset();
        let generation = parse_u16(self.parse_unsigned_decimal()?, generation_offset)?;
        Ok(ObjectId::new(number, GenerationNumber::new(generation)))
    }

    fn parse_unsigned_decimal(&mut self) -> ObjectResult<&'a [u8]> {
        let start = self.cursor.offset().get();
        while let Some(byte) = self.cursor.peek() {
            if !byte.is_ascii_digit() {
                break;
            }
            self.cursor.advance(1)?;
        }
        let end = self.cursor.offset().get();
        if start == end {
            return Err(ObjectError::malformed(
                ByteOffset::new(start),
                "expected unsigned decimal number",
            ));
        }
        Ok(&self.cursor.input()[start..end])
    }

    fn consume_keyword(&mut self, keyword: &[u8]) -> ObjectResult<()> {
        let offset = self.cursor.offset();
        if !self.cursor.remaining().starts_with(keyword) {
            return Err(ObjectError::malformed(offset, "expected object keyword"));
        }
        self.cursor.advance(keyword.len())?;
        Ok(())
    }

    fn skip_whitespace(&mut self) -> ObjectResult<()> {
        while matches!(self.cursor.peek(), Some(byte) if is_whitespace(byte)) {
            self.cursor.advance(1)?;
        }
        Ok(())
    }
}

fn parse_u32(raw: &[u8], offset: ByteOffset) -> ObjectResult<u32> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| ObjectError::malformed(offset, "number is not valid UTF-8"))?;
    text.parse::<u32>()
        .map_err(|_| ObjectError::malformed(offset, "number is out of range"))
}

fn parse_u16(raw: &[u8], offset: ByteOffset) -> ObjectResult<u16> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| ObjectError::malformed(offset, "generation is not valid UTF-8"))?;
    text.parse::<u16>()
        .map_err(|_| ObjectError::malformed(offset, "generation is out of range"))
}

fn find_keyword(haystack: &[u8], keyword: &[u8]) -> Option<usize> {
    haystack
        .windows(keyword.len())
        .position(|window| window == keyword)
}

const fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\0' | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

/// Object model error.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectError {
    /// Lower-level syntax parser error.
    Syntax(SyntaxError),
    /// Malformed object-model syntax.
    Malformed {
        /// Source offset.
        offset: ByteOffset,
        /// Static diagnostic message.
        message: &'static str,
    },
    /// Duplicate indirect object ID.
    DuplicateObject {
        /// Duplicated object ID.
        id: ObjectId,
    },
}

impl ObjectError {
    fn malformed(offset: ByteOffset, message: &'static str) -> Self {
        Self::Malformed { offset, message }
    }

    /// Returns the source offset when one is available.
    #[must_use]
    pub const fn offset(&self) -> Option<ByteOffset> {
        match self {
            Self::Syntax(error) => Some(error.offset()),
            Self::Malformed { offset, .. } => Some(*offset),
            Self::DuplicateObject { .. } => None,
        }
    }
}

impl From<SyntaxError> for ObjectError {
    fn from(error: SyntaxError) -> Self {
        Self::Syntax(error)
    }
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(error) => write!(f, "{error}"),
            Self::Malformed { offset, message } => write!(f, "{message} at {offset}"),
            Self::DuplicateObject { id } => write!(
                f,
                "duplicate object {} {}",
                id.number.get(),
                id.generation.get()
            ),
        }
    }
}

impl std::error::Error for ObjectError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "object");
    }

    #[test]
    fn object_should_depend_on_syntax() {
        assert_eq!(syntax_role(), "syntax");
    }

    #[test]
    fn parse_reference_should_return_typed_id() {
        let reference = parse_reference(PdfBytes::new(b"12 0 R")).expect("reference");

        assert_eq!(reference.id.number.get(), 12);
        assert_eq!(reference.id.generation.get(), 0);
    }

    #[test]
    fn parse_reference_should_reject_zero_object_number() {
        let error = parse_reference(PdfBytes::new(b"0 0 R")).expect_err("zero object number");

        assert_eq!(error.offset(), Some(ByteOffset::new(0)));
    }

    #[test]
    fn parse_indirect_object_should_parse_header_and_value() {
        let object = parse_indirect_object(PdfBytes::new(
            b"12 0 obj\n<< /Type /Page /MediaBox [0 0 300 160] >>\nendobj",
        ))
        .expect("indirect object");

        assert_eq!(object.id.number.get(), 12);
        assert_eq!(object.id.generation.get(), 0);
        assert!(matches!(object.value, PdfPrimitive::Dictionary(_)));
    }

    #[test]
    fn parse_indirect_object_should_reject_missing_endobj() {
        let error =
            parse_indirect_object(PdfBytes::new(b"12 0 obj true")).expect_err("missing endobj");

        assert_eq!(error.offset(), Some(ByteOffset::new(9)));
    }

    #[test]
    fn object_table_should_lookup_without_exposing_indexes() {
        let object =
            parse_indirect_object(PdfBytes::new(b"7 0 obj true endobj")).expect("indirect object");
        let id = object.id;
        let mut table = ObjectTable::new();

        table.insert(object).expect("insert");

        assert_eq!(table.len(), 1);
        assert_eq!(table.get(id).expect("lookup").id, id);
    }

    #[test]
    fn object_table_should_reject_duplicates() {
        let first =
            parse_indirect_object(PdfBytes::new(b"7 0 obj true endobj")).expect("first object");
        let second =
            parse_indirect_object(PdfBytes::new(b"7 0 obj false endobj")).expect("second object");
        let mut table = ObjectTable::new();

        table.insert(first).expect("first insert");
        let error = table.insert(second).expect_err("duplicate");

        assert!(matches!(error, ObjectError::DuplicateObject { .. }));
    }
}
