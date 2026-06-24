//! Safe PDF object model for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;
use std::io::Read;

use flate2::read::ZlibDecoder;
use pdfrust_syntax::{
    parse_primitive, parse_primitive_prefix, ByteCursor, ByteOffset, PdfBytes, PdfName, PdfNumber,
    PdfPrimitive, SyntaxError,
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
    pub value: ObjectValue<'a>,
}

/// Parsed indirect object value.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue<'a> {
    /// Non-stream PDF primitive.
    Primitive(PdfPrimitive<'a>),
    /// Stream object with dictionary metadata and borrowed raw bytes.
    Stream(StreamObject<'a>),
}

/// Parsed PDF stream object.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamObject<'a> {
    dictionary: Vec<(PdfName<'a>, PdfPrimitive<'a>)>,
    raw: &'a [u8],
    raw_offset: ByteOffset,
}

impl<'a> StreamObject<'a> {
    /// Returns the stream dictionary entries in source order.
    #[must_use]
    pub fn dictionary(&self) -> &[(PdfName<'a>, PdfPrimitive<'a>)] {
        &self.dictionary
    }

    /// Returns the borrowed encoded stream bytes.
    #[must_use]
    pub const fn raw(&self) -> &'a [u8] {
        self.raw
    }

    /// Returns the byte offset where the raw stream data starts.
    #[must_use]
    pub const fn raw_offset(&self) -> ByteOffset {
        self.raw_offset
    }

    /// Decodes this stream with default safety limits.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the filter chain is unsupported,
    /// malformed, or exceeds the configured expansion limit.
    pub fn decode(&self) -> ObjectResult<Vec<u8>> {
        self.decode_with_options(StreamDecodeOptions::default())
    }

    /// Decodes this stream with explicit safety limits.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the filter chain is unsupported,
    /// malformed, or exceeds the configured expansion limit.
    pub fn decode_with_options(&self, options: StreamDecodeOptions) -> ObjectResult<Vec<u8>> {
        let filters = stream_filters(&self.dictionary)?;
        decode_stream_bytes(self.raw, &filters, options)
    }
}

/// Stream decode safety configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamDecodeOptions {
    /// Maximum number of bytes any decode step may produce.
    pub max_decoded_len: usize,
}

impl Default for StreamDecodeOptions {
    fn default() -> Self {
        Self {
            max_decoded_len: DEFAULT_MAX_DECODED_LEN,
        }
    }
}

/// Default decoded stream size limit.
pub const DEFAULT_MAX_DECODED_LEN: usize = 16 * 1024 * 1024;

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
    let (object, consumed) = parse_indirect_object_prefix(input.as_bytes(), ByteOffset::new(0))?;
    let mut parser = RawParser::new(input.as_bytes(), consumed.get());
    parser.skip_whitespace()?;
    if parser.peek().is_some() {
        return Err(ObjectError::malformed(
            parser.offset(),
            "trailing data after indirect object",
        ));
    }
    Ok(object)
}

fn parse_indirect_object_prefix<'a>(
    bytes: &'a [u8],
    base_offset: ByteOffset,
) -> ObjectResult<(IndirectObject<'a>, ByteOffset)> {
    let mut parser = ObjectParser::new(PdfBytes::new(bytes));
    let id = parser.parse_object_id()?;
    parser.skip_whitespace()?;
    parser.consume_keyword(b"obj")?;
    parser.skip_whitespace()?;
    let value = parser.parse_object_value(base_offset)?;
    parser.skip_whitespace()?;
    parser.consume_keyword(b"endobj")?;
    Ok((IndirectObject { id, value }, parser.cursor.offset()))
}

impl<'a> ObjectParser<'a> {
    fn parse_object_value(&mut self, base_offset: ByteOffset) -> ObjectResult<ObjectValue<'a>> {
        let (primitive, consumed) = parse_primitive_prefix(PdfBytes::new(self.cursor.remaining()))?;
        self.cursor.advance(consumed.get())?;
        self.skip_whitespace()?;

        if !self.cursor.remaining().starts_with(b"stream") {
            return Ok(ObjectValue::Primitive(primitive));
        }

        let PdfPrimitive::Dictionary(dictionary) = primitive else {
            return Err(ObjectError::malformed(
                self.cursor.offset(),
                "stream object must start with a dictionary",
            ));
        };
        let raw_len = direct_stream_length(&dictionary)?;
        self.consume_keyword(b"stream")?;
        self.consume_stream_line_break()?;
        let raw_start = self.cursor.offset().get();
        let raw_end = raw_start.checked_add(raw_len).ok_or_else(|| {
            ObjectError::malformed(self.cursor.offset(), "stream length overflow")
        })?;
        if raw_end > self.cursor.input().len() {
            return Err(ObjectError::malformed(
                self.cursor.offset(),
                "stream length exceeds object input",
            ));
        }
        let raw = &self.cursor.input()[raw_start..raw_end];
        self.cursor.advance(raw_len)?;
        self.consume_optional_stream_line_break()?;
        self.consume_keyword(b"endstream")?;
        let raw_offset = ByteOffset::new(base_offset.get().saturating_add(raw_start));

        Ok(ObjectValue::Stream(StreamObject {
            dictionary,
            raw,
            raw_offset,
        }))
    }

    fn consume_stream_line_break(&mut self) -> ObjectResult<()> {
        let offset = self.cursor.offset();
        match self.cursor.read_byte()? {
            b'\r' => {
                if self.cursor.peek() == Some(b'\n') {
                    self.cursor.advance(1)?;
                }
                Ok(())
            }
            b'\n' => Ok(()),
            _ => Err(ObjectError::malformed(
                offset,
                "stream keyword must be followed by a line break",
            )),
        }
    }

    fn consume_optional_stream_line_break(&mut self) -> ObjectResult<()> {
        match self.cursor.peek() {
            Some(b'\r') => {
                self.cursor.advance(1)?;
                if self.cursor.peek() == Some(b'\n') {
                    self.cursor.advance(1)?;
                }
            }
            Some(b'\n') => {
                self.cursor.advance(1)?;
            }
            _ => {}
        }
        Ok(())
    }
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
    let (object, _) = parse_indirect_object_prefix(&bytes[start..], entry.offset)?;
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

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
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

fn direct_stream_length(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> ObjectResult<usize> {
    let value = dictionary_value(dictionary, b"Length")
        .ok_or_else(|| ObjectError::malformed(ByteOffset::new(0), "stream is missing /Length"))?;
    let PdfPrimitive::Number(PdfNumber::Integer(length)) = value else {
        return Err(ObjectError::UnsupportedStreamLength);
    };
    usize::try_from(*length).map_err(|_| {
        ObjectError::malformed(ByteOffset::new(0), "stream length must be non-negative")
    })
}

fn stream_filters(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> ObjectResult<Vec<StreamFilter>> {
    let Some(value) = dictionary_value(dictionary, b"Filter") else {
        return Ok(Vec::new());
    };
    match value {
        PdfPrimitive::Name(name) => Ok(vec![StreamFilter::from_name(name.as_bytes())?]),
        PdfPrimitive::Array(filters) => filters
            .iter()
            .map(|filter| match filter {
                PdfPrimitive::Name(name) => StreamFilter::from_name(name.as_bytes()),
                _ => Err(ObjectError::malformed(
                    ByteOffset::new(0),
                    "stream filter array must contain names",
                )),
            })
            .collect(),
        _ => Err(ObjectError::malformed(
            ByteOffset::new(0),
            "stream /Filter must be a name or array",
        )),
    }
}

fn dictionary_value<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    key: &[u8],
) -> Option<&'a PdfPrimitive<'a>> {
    dictionary
        .iter()
        .find_map(|(name, value)| (name.as_bytes() == key).then_some(value))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamFilter {
    Flate,
    AsciiHex,
    Ascii85,
}

impl StreamFilter {
    fn from_name(name: &[u8]) -> ObjectResult<Self> {
        match name {
            b"FlateDecode" | b"Fl" => Ok(Self::Flate),
            b"ASCIIHexDecode" | b"AHx" => Ok(Self::AsciiHex),
            b"ASCII85Decode" | b"A85" => Ok(Self::Ascii85),
            _ => Err(ObjectError::UnsupportedFilter {
                name: name.to_vec(),
            }),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Flate => "FlateDecode",
            Self::AsciiHex => "ASCIIHexDecode",
            Self::Ascii85 => "ASCII85Decode",
        }
    }
}

fn decode_stream_bytes(
    raw: &[u8],
    filters: &[StreamFilter],
    options: StreamDecodeOptions,
) -> ObjectResult<Vec<u8>> {
    ensure_decode_limit(raw.len(), options.max_decoded_len)?;
    let mut decoded = raw.to_vec();
    for filter in filters {
        decoded = match filter {
            StreamFilter::Flate => decode_flate(&decoded, options.max_decoded_len)?,
            StreamFilter::AsciiHex => decode_ascii_hex(&decoded, options.max_decoded_len)?,
            StreamFilter::Ascii85 => decode_ascii85(&decoded, options.max_decoded_len)?,
        };
    }
    Ok(decoded)
}

fn decode_flate(raw: &[u8], max_decoded_len: usize) -> ObjectResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(raw);
    let mut decoded = Vec::new();
    let read_limit = u64::try_from(max_decoded_len)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    decoder
        .by_ref()
        .take(read_limit)
        .read_to_end(&mut decoded)
        .map_err(|_| ObjectError::Decode {
            filter: StreamFilter::Flate.label(),
            message: "invalid flate stream",
        })?;
    ensure_decode_limit(decoded.len(), max_decoded_len)?;
    Ok(decoded)
}

fn decode_ascii_hex(raw: &[u8], max_decoded_len: usize) -> ObjectResult<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut high_nibble = None;

    for byte in raw.iter().copied() {
        if is_whitespace(byte) {
            continue;
        }
        if byte == b'>' {
            break;
        }
        let nibble = hex_nibble(byte).ok_or(ObjectError::Decode {
            filter: StreamFilter::AsciiHex.label(),
            message: "invalid ASCIIHex digit",
        })?;
        if let Some(high) = high_nibble.take() {
            push_limited(&mut decoded, (high << 4) | nibble, max_decoded_len)?;
        } else {
            high_nibble = Some(nibble);
        }
    }

    if let Some(high) = high_nibble {
        push_limited(&mut decoded, high << 4, max_decoded_len)?;
    }
    Ok(decoded)
}

fn decode_ascii85(raw: &[u8], max_decoded_len: usize) -> ObjectResult<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut group = [0_u8; 5];
    let mut group_len = 0_usize;
    let mut index = 0_usize;

    while index < raw.len() {
        let byte = raw[index];
        index += 1;
        if is_whitespace(byte) {
            continue;
        }
        if byte == b'~' {
            if raw.get(index) != Some(&b'>') {
                return Err(ObjectError::Decode {
                    filter: StreamFilter::Ascii85.label(),
                    message: "ASCII85 terminator must be ~>",
                });
            }
            break;
        }
        if byte == b'z' {
            if group_len != 0 {
                return Err(ObjectError::Decode {
                    filter: StreamFilter::Ascii85.label(),
                    message: "ASCII85 z shortcut must start a group",
                });
            }
            extend_limited(&mut decoded, &[0, 0, 0, 0], max_decoded_len)?;
            continue;
        }
        if !(b'!'..=b'u').contains(&byte) {
            return Err(ObjectError::Decode {
                filter: StreamFilter::Ascii85.label(),
                message: "invalid ASCII85 digit",
            });
        }
        group[group_len] = byte - b'!';
        group_len += 1;
        if group_len == 5 {
            append_ascii85_group(&mut decoded, &group, 4, max_decoded_len)?;
            group_len = 0;
        }
    }

    if group_len == 1 {
        return Err(ObjectError::Decode {
            filter: StreamFilter::Ascii85.label(),
            message: "ASCII85 final group is too short",
        });
    }
    if group_len > 1 {
        group[group_len..].fill(b'u' - b'!');
        append_ascii85_group(&mut decoded, &group, group_len - 1, max_decoded_len)?;
    }
    Ok(decoded)
}

fn append_ascii85_group(
    decoded: &mut Vec<u8>,
    group: &[u8; 5],
    output_len: usize,
    max_decoded_len: usize,
) -> ObjectResult<()> {
    let value = group.iter().try_fold(0_u32, |accumulator, digit| {
        accumulator
            .checked_mul(85)
            .and_then(|value| value.checked_add(u32::from(*digit)))
            .ok_or(ObjectError::Decode {
                filter: StreamFilter::Ascii85.label(),
                message: "ASCII85 group overflow",
            })
    })?;
    let bytes = value.to_be_bytes();
    extend_limited(decoded, &bytes[..output_len], max_decoded_len)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn push_limited(decoded: &mut Vec<u8>, byte: u8, max_decoded_len: usize) -> ObjectResult<()> {
    ensure_decode_limit(decoded.len().saturating_add(1), max_decoded_len)?;
    decoded.push(byte);
    Ok(())
}

fn extend_limited(decoded: &mut Vec<u8>, bytes: &[u8], max_decoded_len: usize) -> ObjectResult<()> {
    ensure_decode_limit(decoded.len().saturating_add(bytes.len()), max_decoded_len)?;
    decoded.extend_from_slice(bytes);
    Ok(())
}

fn ensure_decode_limit(len: usize, max_decoded_len: usize) -> ObjectResult<()> {
    if len > max_decoded_len {
        return Err(ObjectError::StreamLimitExceeded {
            limit: max_decoded_len,
        });
    }
    Ok(())
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
    /// Stream length uses a form this loader cannot resolve yet.
    UnsupportedStreamLength,
    /// Stream filter is valid PDF syntax but unsupported.
    UnsupportedFilter {
        /// Raw filter name bytes.
        name: Vec<u8>,
    },
    /// Stream decoding failed.
    Decode {
        /// Filter that reported the failure.
        filter: &'static str,
        /// Static diagnostic message.
        message: &'static str,
    },
    /// Decoded stream output exceeded the configured limit.
    StreamLimitExceeded {
        /// Configured decoded byte limit.
        limit: usize,
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
            Self::UnsupportedStreamLength
            | Self::UnsupportedFilter { .. }
            | Self::Decode { .. }
            | Self::StreamLimitExceeded { .. } => None,
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
            Self::UnsupportedStreamLength => f.write_str("unsupported stream length"),
            Self::UnsupportedFilter { name } => {
                write!(
                    f,
                    "unsupported stream filter /{}",
                    String::from_utf8_lossy(name)
                )
            }
            Self::Decode { filter, message } => write!(f, "{filter} decode error: {message}"),
            Self::StreamLimitExceeded { limit } => {
                write!(f, "decoded stream exceeds limit of {limit} bytes")
            }
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

    use std::io::Write;

    use flate2::write::ZlibEncoder;
    use flate2::Compression;

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
        assert!(matches!(
            object.value,
            ObjectValue::Primitive(PdfPrimitive::Dictionary(_))
        ));
    }

    #[test]
    fn parse_indirect_object_should_parse_stream_raw_range() {
        let object = parse_indirect_object(PdfBytes::new(
            b"4 0 obj\n<< /Length 5 >>\nstream\nhello\nendstream\nendobj",
        ))
        .expect("stream object");

        let ObjectValue::Stream(stream) = object.value else {
            panic!("expected stream object");
        };
        assert_eq!(stream.raw(), b"hello");
        assert_eq!(stream.raw_offset(), ByteOffset::new(31));
    }

    #[test]
    fn stream_decode_should_copy_unfiltered_bytes_within_limit() {
        let decoded = with_test_stream(b"<< /Length 5 >>\nstream\nhello\nendstream", |stream| {
            stream
                .decode_with_options(StreamDecodeOptions { max_decoded_len: 5 })
                .expect("decoded stream")
        });

        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn stream_decode_should_decode_flate_filter() {
        let compressed = zlib_compress(b"BT /F1 12 Tf ET");
        let object = build_stream_object(
            b"<< /Length ",
            compressed.len(),
            b" /Filter /FlateDecode >>",
            &compressed,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("flate decoded stream")
        });

        assert_eq!(decoded, b"BT /F1 12 Tf ET");
    }

    #[test]
    fn stream_decode_should_decode_ascii_hex_filter() {
        let decoded = with_test_stream(
            b"<< /Length 11 /Filter /ASCIIHexDecode >>\nstream\n48656c6c6f>\nendstream",
            |stream| stream.decode().expect("ASCIIHex decoded stream"),
        );

        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn stream_decode_should_apply_filter_arrays_in_order() {
        let decoded = with_test_stream(
            b"<< /Length 7 /Filter [ /ASCIIHexDecode /ASCII85Decode ] >>\nstream\n7A7E3E>\nendstream",
            |stream| stream.decode().expect("filter array decoded stream"),
        );

        assert_eq!(decoded, &[0, 0, 0, 0]);
    }

    #[test]
    fn stream_decode_should_reject_unsupported_filter() {
        let error = with_test_stream(
            b"<< /Length 5 /Filter /DCTDecode >>\nstream\nhello\nendstream",
            |stream| stream.decode().expect_err("unsupported filter"),
        );

        assert!(matches!(error, ObjectError::UnsupportedFilter { .. }));
    }

    #[test]
    fn stream_decode_should_reject_expansion_past_limit() {
        let error = with_test_stream(b"<< /Length 5 >>\nstream\nhello\nendstream", |stream| {
            stream
                .decode_with_options(StreamDecodeOptions { max_decoded_len: 4 })
                .expect_err("stream limit")
        });

        assert_eq!(error, ObjectError::StreamLimitExceeded { limit: 4 });
    }

    #[test]
    fn stream_decode_should_reject_malformed_ascii_hex() {
        let error = with_test_stream(
            b"<< /Length 3 /Filter /ASCIIHexDecode >>\nstream\nxx>\nendstream",
            |stream| stream.decode().expect_err("malformed ASCIIHex"),
        );

        assert!(matches!(error, ObjectError::Decode { .. }));
    }

    #[test]
    fn parse_indirect_object_should_reject_missing_endobj() {
        let error =
            parse_indirect_object(PdfBytes::new(b"12 0 obj true")).expect_err("missing endobj");

        assert_eq!(error.offset(), Some(ByteOffset::new(13)));
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
    fn load_classic_document_should_decode_compressed_stream_object() {
        let compressed = zlib_compress(b"BT /F1 12 Tf ET");
        let pdf = build_classic_stream_pdf(&compressed);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");
        let object = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(1).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("stream object");
        let ObjectValue::Stream(stream) = &object.value else {
            panic!("expected stream object");
        };

        assert_eq!(stream.decode().expect("decoded stream"), b"BT /F1 12 Tf ET");
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

    fn with_test_stream<T>(input: &[u8], test: impl FnOnce(&StreamObject<'_>) -> T) -> T {
        let mut object = b"4 0 obj\n".to_vec();
        object.extend_from_slice(input);
        object.extend_from_slice(b"\nendobj");
        let object = parse_indirect_object(PdfBytes::new(&object)).expect("stream object");
        let ObjectValue::Stream(stream) = &object.value else {
            panic!("expected stream object");
        };
        test(stream)
    }

    fn build_stream_object(
        prefix: &[u8],
        length: usize,
        suffix: &[u8],
        stream_bytes: &[u8],
    ) -> Vec<u8> {
        let mut object = Vec::new();
        object.extend_from_slice(prefix);
        object.extend_from_slice(length.to_string().as_bytes());
        object.extend_from_slice(suffix);
        object.extend_from_slice(b"\nstream\n");
        object.extend_from_slice(stream_bytes);
        object.extend_from_slice(b"\nendstream");
        object
    }

    fn build_classic_stream_pdf(stream_bytes: &[u8]) -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let stream_object = build_stream_object(
            b"1 0 obj\n<< /Length ",
            stream_bytes.len(),
            b" /Filter /FlateDecode >>",
            stream_bytes,
        );
        let object_1 = append_object(&mut pdf, &stream_object);
        pdf.extend_from_slice(b"\nendobj\n");
        let xref_offset = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n0 2\n0000000000 65535 f \n{object_1:010} 00000 n \ntrailer\n<< /Size 2 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn zlib_compress(input: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(input).expect("write compressed input");
        encoder.finish().expect("finish compressed input")
    }
}
