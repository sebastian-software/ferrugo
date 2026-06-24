//! Safe PDF object model for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

use pdfrust_syntax::{
    parse_primitive, ByteCursor, ByteOffset, PdfBytes, PdfName, PdfPrimitive, SyntaxError,
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

/// Parsed classic cross-reference entry for one in-use indirect object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassicXrefEntry {
    /// Object ID described by the xref entry.
    pub id: ObjectId,
    /// Byte offset to the indirect object.
    pub offset: ByteOffset,
}

/// Parsed classic cross-reference table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassicXrefTable {
    startxref: ByteOffset,
    entries: Vec<ClassicXrefEntry>,
}

impl ClassicXrefTable {
    /// Returns the `startxref` byte offset.
    #[must_use]
    pub const fn startxref(&self) -> ByteOffset {
        self.startxref
    }

    /// Returns in-use xref entries.
    #[must_use]
    pub fn entries(&self) -> &[ClassicXrefEntry] {
        &self.entries
    }
}

/// Parsed trailer dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct Trailer<'a> {
    dictionary: Vec<(PdfName<'a>, PdfPrimitive<'a>)>,
}

impl<'a> Trailer<'a> {
    /// Returns trailer dictionary entries in source order.
    #[must_use]
    pub fn entries(&self) -> &[(PdfName<'a>, PdfPrimitive<'a>)] {
        &self.dictionary
    }
}

/// Loaded classic-xref PDF document.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassicDocument<'a> {
    /// Parsed classic xref table.
    pub xref: ClassicXrefTable,
    /// Parsed trailer dictionary.
    pub trailer: Trailer<'a>,
    /// Indirect objects resolved through in-use xref entries.
    pub objects: ObjectTable<'a>,
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

/// Loads a simple PDF that uses a classic xref table.
///
/// # Errors
///
/// Returns [`ObjectError`] when `startxref`, the xref table, trailer, or any
/// in-use indirect object is malformed.
pub fn load_classic_document(input: PdfBytes<'_>) -> ObjectResult<ClassicDocument<'_>> {
    let bytes = input.as_bytes();
    let startxref = locate_startxref(bytes)?;
    let (xref, trailer) = parse_classic_xref_and_trailer(bytes, startxref)?;
    let mut objects = ObjectTable::new();

    for entry in xref.entries() {
        let object = parse_object_at_offset(bytes, *entry)?;
        objects.insert(object)?;
    }

    Ok(ClassicDocument {
        xref,
        trailer,
        objects,
    })
}

fn parse_object_at_offset<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
) -> ObjectResult<IndirectObject<'a>> {
    let start = entry.offset.get();
    if start >= bytes.len() {
        return Err(ObjectError::malformed(
            entry.offset,
            "xref offset is outside input",
        ));
    }
    let tail = &bytes[start..];
    let endobj_offset = find_keyword(tail, b"endobj")
        .ok_or_else(|| ObjectError::malformed(entry.offset, "xref object is missing endobj"))?;
    let end = start + endobj_offset + b"endobj".len();
    let object = parse_indirect_object(PdfBytes::new(&bytes[start..end]))?;
    if object.id != entry.id {
        return Err(ObjectError::XrefOffsetMismatch {
            expected: entry.id,
            actual: object.id,
            offset: entry.offset,
        });
    }
    Ok(object)
}

fn locate_startxref(bytes: &[u8]) -> ObjectResult<ByteOffset> {
    let startxref_keyword = find_last_keyword(bytes, b"startxref")
        .ok_or_else(|| ObjectError::malformed(ByteOffset::new(0), "missing startxref"))?;
    let mut parser = RawParser::new(bytes, startxref_keyword + b"startxref".len());
    parser.skip_whitespace()?;
    parser.parse_byte_offset()
}

fn parse_classic_xref_and_trailer<'a>(
    bytes: &'a [u8],
    startxref: ByteOffset,
) -> ObjectResult<(ClassicXrefTable, Trailer<'a>)> {
    let mut parser = RawParser::new(bytes, startxref.get());
    parser.consume_keyword(b"xref")?;
    let mut entries = Vec::new();

    loop {
        parser.skip_whitespace()?;
        if parser.starts_with(b"trailer") {
            break;
        }
        let first_object = parser.parse_u32()?;
        parser.skip_horizontal_whitespace()?;
        let count = parser.parse_u32()?;
        parser.skip_line_break()?;

        for index in 0..count {
            parser.skip_blank_lines()?;
            let offset = parser.parse_byte_offset()?;
            parser.skip_horizontal_whitespace()?;
            let generation = parser.parse_u16()?;
            parser.skip_horizontal_whitespace()?;
            let marker = parser.read_byte()?;
            parser.skip_until_next_line()?;
            if marker == b'n' {
                let raw_number = first_object.checked_add(index).ok_or_else(|| {
                    ObjectError::malformed(parser.offset(), "xref object number overflow")
                })?;
                let number = ObjectNumber::new(raw_number).map_err(|_| {
                    ObjectError::malformed(parser.offset(), "xref in-use object number is zero")
                })?;
                entries.push(ClassicXrefEntry {
                    id: ObjectId::new(number, GenerationNumber::new(generation)),
                    offset,
                });
            } else if marker != b'f' {
                return Err(ObjectError::malformed(
                    parser.offset(),
                    "xref entry marker must be n or f",
                ));
            }
        }
    }

    parser.consume_keyword(b"trailer")?;
    parser.skip_whitespace()?;
    let trailer_start = parser.offset().get();
    let trailer_end = find_keyword(&bytes[trailer_start..], b"startxref")
        .map(|relative| trailer_start + relative)
        .ok_or_else(|| ObjectError::malformed(parser.offset(), "trailer is missing startxref"))?;
    let trailer_value = parse_primitive(PdfBytes::new(&bytes[trailer_start..trailer_end]))?;
    let PdfPrimitive::Dictionary(dictionary) = trailer_value else {
        return Err(ObjectError::malformed(
            ByteOffset::new(trailer_start),
            "trailer must be a dictionary",
        ));
    };

    Ok((
        ClassicXrefTable { startxref, entries },
        Trailer { dictionary },
    ))
}

struct RawParser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> RawParser<'a> {
    const fn new(bytes: &'a [u8], offset: usize) -> Self {
        Self { bytes, offset }
    }

    const fn offset(&self) -> ByteOffset {
        ByteOffset::new(self.offset)
    }

    fn starts_with(&self, keyword: &[u8]) -> bool {
        self.bytes[self.offset..].starts_with(keyword)
    }

    fn consume_keyword(&mut self, keyword: &[u8]) -> ObjectResult<()> {
        if !self.starts_with(keyword) {
            return Err(ObjectError::malformed(self.offset(), "expected keyword"));
        }
        self.offset += keyword.len();
        Ok(())
    }

    fn read_byte(&mut self) -> ObjectResult<u8> {
        let byte = self
            .bytes
            .get(self.offset)
            .copied()
            .ok_or_else(|| ObjectError::malformed(self.offset(), "unexpected end of input"))?;
        self.offset += 1;
        Ok(byte)
    }

    fn skip_whitespace(&mut self) -> ObjectResult<()> {
        while matches!(self.bytes.get(self.offset), Some(byte) if is_whitespace(*byte)) {
            self.offset += 1;
        }
        Ok(())
    }

    fn skip_horizontal_whitespace(&mut self) -> ObjectResult<()> {
        while matches!(self.bytes.get(self.offset), Some(b' ' | b'\t')) {
            self.offset += 1;
        }
        Ok(())
    }

    fn skip_line_break(&mut self) -> ObjectResult<()> {
        match self.bytes.get(self.offset) {
            Some(b'\r') => {
                self.offset += 1;
                if self.bytes.get(self.offset) == Some(&b'\n') {
                    self.offset += 1;
                }
                Ok(())
            }
            Some(b'\n') => {
                self.offset += 1;
                Ok(())
            }
            _ => Err(ObjectError::malformed(self.offset(), "expected line break")),
        }
    }

    fn skip_blank_lines(&mut self) -> ObjectResult<()> {
        loop {
            let checkpoint = self.offset;
            self.skip_horizontal_whitespace()?;
            match self.bytes.get(self.offset) {
                Some(b'\r' | b'\n') => self.skip_line_break()?,
                _ => {
                    self.offset = checkpoint;
                    return Ok(());
                }
            }
        }
    }

    fn skip_until_next_line(&mut self) -> ObjectResult<()> {
        while let Some(byte) = self.bytes.get(self.offset) {
            if *byte == b'\r' || *byte == b'\n' {
                return self.skip_line_break();
            }
            self.offset += 1;
        }
        Ok(())
    }

    fn parse_byte_offset(&mut self) -> ObjectResult<ByteOffset> {
        let raw = self.parse_unsigned_decimal()?;
        let text = std::str::from_utf8(raw)
            .map_err(|_| ObjectError::malformed(self.offset(), "offset is not valid UTF-8"))?;
        let value = text
            .parse::<usize>()
            .map_err(|_| ObjectError::malformed(self.offset(), "offset is out of range"))?;
        Ok(ByteOffset::new(value))
    }

    fn parse_u32(&mut self) -> ObjectResult<u32> {
        parse_u32(self.parse_unsigned_decimal()?, self.offset())
    }

    fn parse_u16(&mut self) -> ObjectResult<u16> {
        parse_u16(self.parse_unsigned_decimal()?, self.offset())
    }

    fn parse_unsigned_decimal(&mut self) -> ObjectResult<&'a [u8]> {
        let start = self.offset;
        while matches!(self.bytes.get(self.offset), Some(byte) if byte.is_ascii_digit()) {
            self.offset += 1;
        }
        if start == self.offset {
            return Err(ObjectError::malformed(
                ByteOffset::new(start),
                "expected unsigned decimal number",
            ));
        }
        Ok(&self.bytes[start..self.offset])
    }
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

fn find_last_keyword(haystack: &[u8], keyword: &[u8]) -> Option<usize> {
    haystack
        .windows(keyword.len())
        .rposition(|window| window == keyword)
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
    /// Xref entry points at a different indirect object.
    XrefOffsetMismatch {
        /// Object ID expected from the xref table.
        expected: ObjectId,
        /// Object ID parsed at the xref offset.
        actual: ObjectId,
        /// Xref byte offset.
        offset: ByteOffset,
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
            Self::XrefOffsetMismatch { offset, .. } => Some(*offset),
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
            Self::XrefOffsetMismatch {
                expected,
                actual,
                offset,
            } => write!(
                f,
                "xref offset mismatch at {offset}: expected {} {}, got {} {}",
                expected.number.get(),
                expected.generation.get(),
                actual.number.get(),
                actual.generation.get()
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

    #[test]
    fn load_classic_document_should_load_xref_trailer_and_objects() {
        let pdf = build_classic_pdf(false);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        assert_eq!(document.objects.len(), 3);
        assert_eq!(document.xref.entries().len(), 3);
        assert_eq!(document.trailer.entries().len(), 2);
        assert!(document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(1).expect("object number"),
                GenerationNumber::new(0)
            ))
            .is_some());
        assert_eq!(
            document.trailer.entries()[1].1,
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(1, 0))
        );
    }

    #[test]
    fn load_classic_document_should_report_offset_mismatch() {
        let pdf = build_classic_pdf(true);

        let error = load_classic_document(PdfBytes::new(&pdf)).expect_err("bad xref offset");

        assert!(matches!(error, ObjectError::XrefOffsetMismatch { .. }));
    }

    #[test]
    fn load_classic_document_should_require_startxref() {
        let error =
            load_classic_document(PdfBytes::new(b"%PDF-1.7\n")).expect_err("missing startxref");

        assert_eq!(error.offset(), Some(ByteOffset::new(0)));
    }

    fn build_classic_pdf(use_wrong_first_offset: bool) -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let object_1 = append_object(
            &mut pdf,
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
        );
        let object_2 = append_object(
            &mut pdf,
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
        );
        let object_3 = append_object(
            &mut pdf,
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 160] >>\nendobj\n",
        );
        let xref_offset = pdf.len();
        let object_1_xref = if use_wrong_first_offset {
            object_2
        } else {
            object_1
        };
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1_xref:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn append_object(pdf: &mut Vec<u8>, object: &[u8]) -> usize {
        let offset = pdf.len();
        pdf.extend_from_slice(object);
        offset
    }
}
