//! Safe PDF object model for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fmt;
use std::io::Read;

use aes::cipher::{block_padding::NoPadding, block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use ferrugo_syntax::{
    parse_primitive, parse_primitive_prefix, ByteCursor, ByteOffset, PdfBytes, PdfName, PdfNumber,
    PdfPrimitive, PdfString, SyntaxError,
};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};
use rc4::{
    consts::{U10, U11, U12, U13, U14, U15, U16, U5, U6, U7, U8, U9},
    KeyInit as Rc4KeyInit, Rc4, StreamCipher,
};
use sha2::Sha256;

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "object";

/// Maximum bytes scanned around a declared xref object offset for local repair.
pub const DEFAULT_XREF_OFFSET_RECOVERY_SCAN_BYTES: usize = 64;

/// Result alias for object-model operations.
pub type ObjectResult<T> = Result<T, ObjectError>;

type DictionaryEntries<'a> = [(PdfName<'a>, PdfPrimitive<'a>)];

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the lower-level syntax dependency.
#[must_use]
pub fn syntax_role() -> &'static str {
    ferrugo_syntax::crate_role()
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
    raw: StreamBytes<'a>,
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
    pub fn raw(&self) -> &[u8] {
        self.raw.as_bytes()
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
        decode_stream_bytes(self.raw(), &filters, options)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum StreamBytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl StreamBytes<'_> {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes,
        }
    }
}

/// Stream decode safety configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamDecodeOptions {
    /// Maximum number of bytes any decode step may produce.
    pub max_decoded_len: usize,
    /// Initial output buffer capacity for decoders with a known expected size.
    pub initial_capacity: Option<usize>,
}

impl Default for StreamDecodeOptions {
    fn default() -> Self {
        Self {
            max_decoded_len: DEFAULT_MAX_DECODED_LEN,
            initial_capacity: None,
        }
    }
}

/// Default decoded stream size limit.
pub const DEFAULT_MAX_DECODED_LEN: usize = 16 * 1024 * 1024;

/// Default maximum number of classic trailer revisions followed through `/Prev`.
pub const DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT: usize = 16;

const PASSWORD_PADDING: [u8; 32] = [
    0x28, 0xbf, 0x4e, 0x5e, 0x4e, 0x75, 0x8a, 0x41, 0x64, 0x00, 0x4e, 0x56, 0xff, 0xfa, 0x01, 0x08,
    0x2e, 0x2e, 0x00, 0xb6, 0xd0, 0x68, 0x3e, 0x80, 0x2f, 0x0c, 0xa9, 0xfe, 0x64, 0x53, 0x69, 0x7a,
];

const AES_OBJECT_KEY_SALT: &[u8; 4] = b"sAlT";
const AES_BLOCK_BYTES: usize = 16;
const PDF_AES256_KEY_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentSecurity {
    file_key: Vec<u8>,
    stream_filter: CryptFilter,
    string_filter: CryptFilter,
    encrypt_metadata: bool,
    encryption_dictionary_id: Option<ObjectId>,
}

impl DocumentSecurity {
    fn decrypt_stream(
        &self,
        id: ObjectId,
        stream: &StreamObject<'_>,
    ) -> ObjectResult<Option<Vec<u8>>> {
        if self.encryption_dictionary_id == Some(id)
            || (!self.encrypt_metadata
                && dictionary_name_is(stream.dictionary(), b"Type", b"Metadata"))
        {
            return Ok(None);
        }
        self.stream_filter.decrypt(self, id, stream.raw()).map(Some)
    }

    fn decrypt_string(&self, id: ObjectId, raw: &[u8]) -> ObjectResult<Vec<u8>> {
        if self.encryption_dictionary_id == Some(id) {
            return Ok(raw.to_vec());
        }
        self.string_filter.decrypt(self, id, raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CryptFilter {
    Identity,
    Standard(CryptAlgorithm),
}

impl CryptFilter {
    fn decrypt(
        self,
        security: &DocumentSecurity,
        id: ObjectId,
        encrypted: &[u8],
    ) -> ObjectResult<Vec<u8>> {
        match self {
            Self::Identity => Ok(encrypted.to_vec()),
            Self::Standard(CryptAlgorithm::Rc4) => {
                let key = object_crypt_key(&security.file_key, id, CryptAlgorithm::Rc4);
                rc4_crypt(&key, encrypted)
            }
            Self::Standard(CryptAlgorithm::AesV2) => {
                let key = object_crypt_key(&security.file_key, id, CryptAlgorithm::AesV2);
                aes_cbc_decrypt_pkcs7(&key, encrypted)
            }
            Self::Standard(CryptAlgorithm::AesV3) => {
                aes_cbc_decrypt_pkcs7(&security.file_key, encrypted)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CryptAlgorithm {
    Rc4,
    AesV2,
    AesV3,
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

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut IndirectObject<'a>> {
        self.objects.iter_mut()
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
    deleted: Vec<ObjectNumber>,
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

/// Parsed xref-stream entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XrefStreamEntry {
    /// In-use object stored at a byte offset.
    InUse {
        /// Object ID described by the xref stream entry.
        id: ObjectId,
        /// Byte offset to the indirect object.
        offset: ByteOffset,
    },
    /// Object stored inside an object stream.
    Compressed {
        /// Object ID described by the xref stream entry.
        id: ObjectId,
        /// Object stream containing the object.
        object_stream: ObjectId,
        /// Zero-based object index inside the object stream.
        index: usize,
    },
}

impl XrefStreamEntry {
    /// Returns the object ID described by this entry.
    #[must_use]
    pub const fn id(self) -> ObjectId {
        match self {
            Self::InUse { id, .. } | Self::Compressed { id, .. } => id,
        }
    }
}

/// Parsed xref stream table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrefStreamTable {
    startxref: ByteOffset,
    entries: Vec<XrefStreamEntry>,
}

impl XrefStreamTable {
    /// Returns the `startxref` byte offset.
    #[must_use]
    pub const fn startxref(&self) -> ByteOffset {
        self.startxref
    }

    /// Returns in-use and compressed entries from the xref stream.
    #[must_use]
    pub fn entries(&self) -> &[XrefStreamEntry] {
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

/// Parsed PDF linearization dictionary metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearizationDictionary {
    /// Declared file length from `/L`.
    pub file_length: usize,
    /// Declared end offset of the first-page section from `/E`.
    pub first_page_end: usize,
    /// First page object number from `/O`.
    pub first_page_object: ObjectNumber,
    /// Declared page count from `/N`.
    pub page_count: usize,
    /// Declared primary hint table offset and length from `/H`, when present.
    pub primary_hint_table: Option<(usize, usize)>,
    /// Declared main xref offset from `/T`, when present.
    pub main_xref_offset: Option<usize>,
}

/// Loader metrics exposed for parser and first-page loading diagnostics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DocumentLoadMetrics {
    /// Total input byte length.
    pub input_bytes: usize,
    /// Number of indirect objects parsed into the document table.
    pub loaded_objects: usize,
    /// Sum of parsed indirect-object byte spans.
    pub loaded_object_bytes: usize,
    /// True when the input declares a linearization dictionary.
    pub is_linearized: bool,
    /// True when this document was loaded through the bounded first-page path.
    pub first_page_only: bool,
    /// Declared first-page section end from `/E`, when present.
    pub first_page_end: Option<usize>,
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
    /// Parsed linearization dictionary, when the document declares one.
    pub linearization: Option<LinearizationDictionary>,
    /// Object loader metrics for diagnostics and benchmarks.
    pub load_metrics: DocumentLoadMetrics,
    security: Option<DocumentSecurity>,
}

impl ClassicDocument<'_> {
    /// Resolves the document catalog and page tree.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the trailer root, catalog, page tree, or
    /// inherited page metadata is malformed.
    pub fn page_tree(&self) -> ObjectResult<PageTree> {
        resolve_page_tree(self)
    }

    /// Decodes and decrypts a string that belongs to `id`.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the string syntax is malformed or the
    /// document encryption policy cannot decrypt it.
    pub fn decode_string(&self, id: ObjectId, string: PdfString<'_>) -> ObjectResult<Vec<u8>> {
        let bytes = decode_pdf_string_bytes(string, DEFAULT_MAX_DECODED_LEN)?;
        match &self.security {
            Some(security) => security.decrypt_string(id, &bytes),
            None => Ok(bytes),
        }
    }

    /// Resolves the linearized first page without traversing every page node.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the linearization dictionary points at a
    /// malformed or unavailable page object.
    pub fn linearized_first_page_tree(&self) -> ObjectResult<Option<PageTree>> {
        let Some(linearization) = self.linearization else {
            return Ok(None);
        };
        let id = ObjectId::new(linearization.first_page_object, GenerationNumber::new(0));
        let object = required_object(self, id)?;
        let dictionary = object_dictionary(&object)?;
        if !dictionary_name_is(dictionary, b"Type", b"Page") {
            return Err(ObjectError::MissingPageTreeField { field: "Type" });
        }
        let inherited = inherit_page_state(dictionary, InheritedPageState::default())?;
        let media_box = inherited
            .media_box
            .ok_or(ObjectError::MissingPageTreeField { field: "MediaBox" })?;
        Ok(Some(PageTree {
            pages: vec![PageMetadata {
                id,
                media_box,
                crop_box: inherited.crop_box,
                rotation_degrees: inherited.rotation_degrees,
                user_unit: inherited.user_unit,
                resources: inherited.resources,
            }],
        }))
    }
}

/// Loaded xref-stream PDF document.
#[derive(Debug, Clone, PartialEq)]
pub struct ModernDocument<'a> {
    /// Parsed xref stream table.
    pub xref: XrefStreamTable,
    /// Trailer data carried by the xref stream dictionary.
    pub trailer: Trailer<'a>,
    /// Direct indirect objects resolved through in-use xref entries.
    pub objects: ObjectTable<'a>,
    object_streams: Vec<LoadedObjectStream>,
    compressed_object_index: Vec<CompressedObjectIndexEntry>,
    security: Option<DocumentSecurity>,
}

impl<'a> ModernDocument<'a> {
    /// Resolves an object from direct xref entries or decoded object streams.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when a compressed object body is malformed.
    pub fn get_object(&self, id: ObjectId) -> ObjectResult<Option<IndirectObject<'_>>> {
        if let Some(object) = self.objects.get(id) {
            return Ok(Some(object.clone()));
        }
        if let Some(entry) = self
            .compressed_object_index
            .iter()
            .find(|entry| entry.id == id)
        {
            let object = self.object_streams[entry.stream_index]
                .parse_object_at_position(entry.object_position, id)?;
            return Ok(Some(object));
        }
        Ok(None)
    }

    /// Decodes and decrypts a string that belongs to `id`.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the string syntax is malformed or the
    /// document encryption policy cannot decrypt it.
    pub fn decode_string(&self, id: ObjectId, string: PdfString<'_>) -> ObjectResult<Vec<u8>> {
        let bytes = decode_pdf_string_bytes(string, DEFAULT_MAX_DECODED_LEN)?;
        match &self.security {
            Some(security) => security.decrypt_string(id, &bytes),
            None => Ok(bytes),
        }
    }

    /// Resolves the document catalog and page tree.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the trailer root, catalog, page tree, or
    /// inherited page metadata is malformed.
    pub fn page_tree(&self) -> ObjectResult<PageTree> {
        resolve_page_tree(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompressedObjectIndexEntry {
    id: ObjectId,
    stream_index: usize,
    object_position: usize,
}

/// Resolved document page tree.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTree {
    pages: Vec<PageMetadata>,
}

impl PageTree {
    /// Returns all pages in document order.
    #[must_use]
    pub fn pages(&self) -> &[PageMetadata] {
        &self.pages
    }

    /// Returns the number of resolved pages.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns the first page metadata when the document has pages.
    #[must_use]
    pub fn first_page(&self) -> Option<&PageMetadata> {
        self.pages.first()
    }

    /// Returns the first page size when the document has pages.
    #[must_use]
    pub fn first_page_size(&self) -> Option<PageSize> {
        self.first_page().map(PageMetadata::size)
    }
}

/// Resolved page metadata needed before content interpretation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMetadata {
    /// Page object ID.
    pub id: ObjectId,
    /// Media box inherited or declared on the page.
    pub media_box: PageBox,
    /// Crop box inherited or declared on the page.
    pub crop_box: Option<PageBox>,
    /// Page rotation in clockwise degrees, normalized to 0, 90, 180, or 270.
    pub rotation_degrees: u16,
    /// PDF user unit multiplier. Defaults to 1.0.
    pub user_unit: f64,
    /// Resource dictionary reference inherited or declared on the page.
    pub resources: Option<Reference>,
}

impl PageMetadata {
    /// Returns the visible page size using `CropBox` when present, otherwise
    /// `MediaBox`.
    #[must_use]
    pub fn size(&self) -> PageSize {
        let size = self.crop_box.unwrap_or(self.media_box).size();
        if page_rotation_swaps_axes(self.rotation_degrees) {
            PageSize {
                width: size.height,
                height: size.width,
            }
        } else {
            size
        }
    }
}

/// Four-number PDF page box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageBox {
    /// Lower-left x coordinate.
    pub left: f64,
    /// Lower-left y coordinate.
    pub bottom: f64,
    /// Upper-right x coordinate.
    pub right: f64,
    /// Upper-right y coordinate.
    pub top: f64,
}

impl PageBox {
    /// Returns the box width and height.
    #[must_use]
    pub fn size(self) -> PageSize {
        PageSize {
            width: self.right - self.left,
            height: self.top - self.bottom,
        }
    }
}

/// Page size in PDF user-space units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    /// Page width.
    pub width: f64,
    /// Page height.
    pub height: f64,
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
            raw: StreamBytes::Borrowed(raw),
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
    let linearization = parse_linearization_dictionary(bytes).ok().flatten();
    let startxref = locate_startxref(bytes)?;
    let (mut xref, trailer) = parse_classic_xref_chain(bytes, startxref)?;
    extend_classic_xref_with_hybrid_stream(bytes, &mut xref, &trailer)?;
    let mut objects = ObjectTable::new();
    let mut loaded_object_bytes = 0usize;

    for entry in xref.entries() {
        let (object, span) = parse_object_with_xref_recovery_with_span(bytes, *entry)?;
        loaded_object_bytes = loaded_object_bytes.saturating_add(span);
        objects.insert(object)?;
    }
    let loaded_objects = objects.len();
    let security = document_security_from_trailer(&trailer, &objects)?;
    decrypt_object_table(&mut objects, security.as_ref())?;

    let document = ClassicDocument {
        xref,
        trailer,
        objects,
        linearization,
        load_metrics: DocumentLoadMetrics {
            input_bytes: bytes.len(),
            loaded_objects,
            loaded_object_bytes,
            is_linearized: linearization.is_some(),
            first_page_only: false,
            first_page_end: linearization.map(|dictionary| dictionary.first_page_end),
        },
        security,
    };
    reject_encrypted_catalog(&document)?;
    Ok(document)
}

/// Loads the first-page object section of a linearized classic-xref PDF.
///
/// # Errors
///
/// Returns [`ObjectError`] when the input is not linearized, has invalid
/// first-page section metadata, or required first-page objects are malformed.
pub fn load_linearized_first_page_document(
    input: PdfBytes<'_>,
) -> ObjectResult<ClassicDocument<'_>> {
    let bytes = input.as_bytes();
    let linearization = parse_linearization_dictionary(bytes)?.ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "missing linearization dictionary")
    })?;
    validate_linearization_dictionary(bytes, linearization)?;
    let startxref = locate_startxref(bytes)?;
    let (xref, trailer) = parse_classic_xref_chain(bytes, startxref)?;
    let mut objects = ObjectTable::new();
    let mut loaded_object_bytes = 0usize;

    for entry in xref
        .entries()
        .iter()
        .filter(|entry| entry.offset.get() < linearization.first_page_end)
    {
        let (object, span) = parse_object_with_xref_recovery_with_span(bytes, *entry)?;
        let object_end = entry.offset.get().saturating_add(span);
        if object_end > linearization.first_page_end {
            return Err(ObjectError::malformed(
                entry.offset,
                "linearized first-page object exceeds first-page section",
            ));
        }
        loaded_object_bytes = loaded_object_bytes.saturating_add(span);
        objects.insert(object)?;
    }
    let loaded_objects = objects.len();
    let security = document_security_from_trailer(&trailer, &objects)?;
    decrypt_object_table(&mut objects, security.as_ref())?;

    if objects
        .get(ObjectId::new(
            linearization.first_page_object,
            GenerationNumber::new(0),
        ))
        .is_none()
    {
        return Err(ObjectError::MissingObject {
            id: ObjectId::new(linearization.first_page_object, GenerationNumber::new(0)),
        });
    }

    let document = ClassicDocument {
        xref,
        trailer,
        objects,
        linearization: Some(linearization),
        load_metrics: DocumentLoadMetrics {
            input_bytes: bytes.len(),
            loaded_objects,
            loaded_object_bytes,
            is_linearized: true,
            first_page_only: true,
            first_page_end: Some(linearization.first_page_end),
        },
        security,
    };
    reject_encrypted_catalog(&document)?;
    Ok(document)
}

/// Loads a PDF whose `startxref` points at an xref stream.
///
/// # Errors
///
/// Returns [`ObjectError`] when the xref stream, direct objects, or referenced
/// object streams are malformed.
pub fn load_modern_document(input: PdfBytes<'_>) -> ObjectResult<ModernDocument<'_>> {
    let bytes = input.as_bytes();
    let startxref = locate_startxref(bytes)?;
    let (xref, trailer) = parse_xref_stream_and_trailer(bytes, startxref)?;
    let mut objects = ObjectTable::new();

    for entry in xref.entries() {
        if let XrefStreamEntry::InUse { id, offset } = *entry {
            let object = parse_object_with_xref_recovery(bytes, ClassicXrefEntry { id, offset })?;
            objects.insert(object)?;
        }
    }

    let security = document_security_from_trailer(&trailer, &objects)?;
    decrypt_object_table(&mut objects, security.as_ref())?;
    let object_streams = load_referenced_object_streams(&objects, xref.entries())?;
    validate_compressed_entries(&object_streams, xref.entries())?;
    let compressed_object_index = compressed_object_index(&object_streams, xref.entries())?;

    let document = ModernDocument {
        xref,
        trailer,
        objects,
        object_streams,
        compressed_object_index,
        security,
    };
    reject_encrypted_catalog(&document)?;
    Ok(document)
}

trait DocumentObjects {
    fn trailer(&self) -> &Trailer<'_>;
    fn get_object_owned(&self, id: ObjectId) -> ObjectResult<Option<IndirectObject<'_>>>;
}

impl DocumentObjects for ClassicDocument<'_> {
    fn trailer(&self) -> &Trailer<'_> {
        &self.trailer
    }

    fn get_object_owned(&self, id: ObjectId) -> ObjectResult<Option<IndirectObject<'_>>> {
        Ok(self.objects.get(id).cloned())
    }
}

impl DocumentObjects for ModernDocument<'_> {
    fn trailer(&self) -> &Trailer<'_> {
        &self.trailer
    }

    fn get_object_owned(&self, id: ObjectId) -> ObjectResult<Option<IndirectObject<'_>>> {
        self.get_object(id)
    }
}

#[derive(Debug, Clone, Copy)]
struct InheritedPageState {
    media_box: Option<PageBox>,
    crop_box: Option<PageBox>,
    rotation_degrees: u16,
    user_unit: f64,
    resources: Option<Reference>,
}

impl Default for InheritedPageState {
    fn default() -> Self {
        Self {
            media_box: None,
            crop_box: None,
            rotation_degrees: 0,
            user_unit: 1.0,
            resources: None,
        }
    }
}

fn resolve_page_tree(document: &impl DocumentObjects) -> ObjectResult<PageTree> {
    let catalog_id = required_reference(document.trailer().entries(), b"Root", "Root")?.id;
    let catalog = required_object(document, catalog_id)?;
    let catalog_dictionary = object_dictionary(&catalog)?;
    if !dictionary_name_is(catalog_dictionary, b"Type", b"Catalog") {
        return Err(ObjectError::MissingPageTreeField { field: "Type" });
    }
    let pages_id = required_reference(catalog_dictionary, b"Pages", "Pages")?.id;
    let mut pages = Vec::new();
    let mut visited = HashSet::new();
    traverse_page_tree(
        document,
        pages_id,
        InheritedPageState::default(),
        &mut visited,
        &mut pages,
    )?;
    Ok(PageTree { pages })
}

fn reject_encrypted_catalog(document: &impl DocumentObjects) -> ObjectResult<()> {
    let Some(PdfPrimitive::Reference(reference)) =
        dictionary_value(document.trailer().entries(), b"Root")
    else {
        return Ok(());
    };
    let root = Reference::new(ObjectId::new(
        ObjectNumber::new(reference.object)?,
        GenerationNumber::new(reference.generation),
    ));
    let Some(catalog) = document.get_object_owned(root.id)? else {
        return Ok(());
    };
    let Ok(dictionary) = object_dictionary(&catalog) else {
        return Ok(());
    };
    if dictionary_value(dictionary, b"Encrypt").is_some() {
        return Err(ObjectError::Encrypted);
    }
    Ok(())
}

fn document_security_from_trailer(
    trailer: &Trailer<'_>,
    objects: &ObjectTable<'_>,
) -> ObjectResult<Option<DocumentSecurity>> {
    let Some(encrypt) = dictionary_value(trailer.entries(), b"Encrypt") else {
        return Ok(None);
    };
    let (dictionary, encryption_dictionary_id) = encryption_dictionary(encrypt, objects)?;
    if !dictionary_name_is(dictionary, b"Filter", b"Standard") {
        return Err(ObjectError::Encrypted);
    }
    let file_id = trailer_file_id(trailer)?;
    let security =
        standard_security_from_dictionary(dictionary, file_id, encryption_dictionary_id)?;
    Ok(Some(security))
}

fn encryption_dictionary<'a>(
    encrypt: &'a PdfPrimitive<'a>,
    objects: &'a ObjectTable<'a>,
) -> ObjectResult<(&'a DictionaryEntries<'a>, Option<ObjectId>)> {
    match encrypt {
        PdfPrimitive::Dictionary(dictionary) => Ok((dictionary, None)),
        PdfPrimitive::Reference(reference) => {
            let id = ObjectId::new(
                ObjectNumber::new(reference.object)?,
                GenerationNumber::new(reference.generation),
            );
            let object = objects.get(id).ok_or(ObjectError::Encrypted)?;
            let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = &object.value else {
                return Err(ObjectError::Encrypted);
            };
            Ok((dictionary, Some(id)))
        }
        _ => Err(ObjectError::Encrypted),
    }
}

fn trailer_file_id(trailer: &Trailer<'_>) -> ObjectResult<Vec<u8>> {
    let Some(PdfPrimitive::Array(ids)) = dictionary_value(trailer.entries(), b"ID") else {
        return Err(ObjectError::Encrypted);
    };
    let Some(PdfPrimitive::String(first_id)) = ids.first() else {
        return Err(ObjectError::Encrypted);
    };
    decode_pdf_string_bytes(*first_id, DEFAULT_MAX_DECODED_LEN).map_err(|_| ObjectError::Encrypted)
}

fn standard_security_from_dictionary(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    file_id: Vec<u8>,
    encryption_dictionary_id: Option<ObjectId>,
) -> ObjectResult<DocumentSecurity> {
    let version = required_i64(dictionary, b"V").map_err(|_| ObjectError::Encrypted)?;
    let revision = required_i64(dictionary, b"R").map_err(|_| ObjectError::Encrypted)?;
    let permissions = required_i64(dictionary, b"P").map_err(|_| ObjectError::Encrypted)?;
    let encrypt_metadata = optional_bool(dictionary, b"EncryptMetadata").unwrap_or(true);
    let file_key = match revision {
        2 => {
            let owner = required_string_bytes(dictionary, b"O")?;
            let user = required_string_bytes(dictionary, b"U")?;
            let file_key = standard_v2_file_key(&owner, permissions, &file_id);
            let expected_user = rc4_crypt(&file_key, &PASSWORD_PADDING)?;
            if user.get(..expected_user.len()) != Some(expected_user.as_slice()) {
                return Err(ObjectError::Encrypted);
            }
            file_key
        }
        3 | 4 => {
            let owner = required_string_bytes(dictionary, b"O")?;
            let user = required_string_bytes(dictionary, b"U")?;
            let key_bits = optional_i64(dictionary, b"Length")
                .unwrap_or(40)
                .try_into()
                .map_err(|_| ObjectError::Encrypted)?;
            let file_key =
                standard_v4_file_key(&owner, permissions, &file_id, key_bits, encrypt_metadata)?;
            let expected_user = standard_v4_user_key(&file_key, &file_id)?;
            if user.get(..16) != expected_user.get(..16) {
                return Err(ObjectError::Encrypted);
            }
            file_key
        }
        5 => {
            let user = required_string_bytes(dictionary, b"U")?;
            let user_encryption_key = required_string_bytes(dictionary, b"UE")?;
            let file_key = standard_v5_file_key(&user, &user_encryption_key)?;
            validate_standard_v5_permissions(dictionary, &file_key)?;
            file_key
        }
        _ => return Err(ObjectError::Encrypted),
    };
    let fallback_algorithm = match (version, revision) {
        (1 | 2, _) | (_, 2 | 3) => CryptAlgorithm::Rc4,
        (_, 4) => CryptAlgorithm::Rc4,
        (_, 5) => CryptAlgorithm::AesV3,
        _ => return Err(ObjectError::Encrypted),
    };
    let stream_filter = crypt_filter(dictionary, b"StmF", fallback_algorithm)?;
    let string_filter = crypt_filter(dictionary, b"StrF", fallback_algorithm)?;
    Ok(DocumentSecurity {
        file_key,
        stream_filter,
        string_filter,
        encrypt_metadata,
        encryption_dictionary_id,
    })
}

fn decrypt_object_table(
    objects: &mut ObjectTable<'_>,
    security: Option<&DocumentSecurity>,
) -> ObjectResult<()> {
    let Some(security) = security else {
        return Ok(());
    };
    for object in objects.iter_mut() {
        if let ObjectValue::Stream(stream) = &mut object.value {
            if let Some(decrypted) = security.decrypt_stream(object.id, stream)? {
                stream.raw = StreamBytes::Owned(decrypted);
            }
        }
    }
    Ok(())
}

fn traverse_page_tree(
    document: &impl DocumentObjects,
    id: ObjectId,
    inherited: InheritedPageState,
    visited: &mut HashSet<ObjectId>,
    pages: &mut Vec<PageMetadata>,
) -> ObjectResult<()> {
    if !visited.insert(id) {
        return Err(ObjectError::PageTreeCycle { id });
    }

    let object = required_object(document, id)?;
    let dictionary = object_dictionary(&object)?;
    let node_type = dictionary_name(dictionary, b"Type")
        .ok_or(ObjectError::MissingPageTreeField { field: "Type" })?;
    let inherited = inherit_page_state(dictionary, inherited)?;

    match node_type {
        b"Pages" => {
            required_usize(dictionary, b"Count")?;
            let kids = required_reference_array(dictionary, b"Kids", "Kids")?;
            for kid in kids {
                traverse_page_tree(document, kid.id, inherited, visited, pages)?;
            }
        }
        b"Page" => {
            let media_box = inherited
                .media_box
                .ok_or(ObjectError::MissingPageTreeField { field: "MediaBox" })?;
            pages.push(PageMetadata {
                id,
                media_box,
                crop_box: inherited.crop_box,
                rotation_degrees: inherited.rotation_degrees,
                user_unit: inherited.user_unit,
                resources: inherited.resources,
            });
        }
        _ => return Err(ObjectError::MissingPageTreeField { field: "Type" }),
    }
    Ok(())
}

fn required_object(
    document: &impl DocumentObjects,
    id: ObjectId,
) -> ObjectResult<IndirectObject<'_>> {
    document
        .get_object_owned(id)?
        .ok_or(ObjectError::MissingObject { id })
}

fn object_dictionary<'a>(
    object: &'a IndirectObject<'a>,
) -> ObjectResult<&'a [(PdfName<'a>, PdfPrimitive<'a>)]> {
    let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = &object.value else {
        return Err(ObjectError::MissingPageTreeField {
            field: "Dictionary",
        });
    };
    Ok(dictionary)
}

fn inherit_page_state(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    mut inherited: InheritedPageState,
) -> ObjectResult<InheritedPageState> {
    if let Some(value) = dictionary_value(dictionary, b"MediaBox") {
        inherited.media_box = Some(page_box(value)?);
    }
    if let Some(value) = dictionary_value(dictionary, b"CropBox") {
        inherited.crop_box = Some(page_box(value)?);
    }
    if let Some(value) = dictionary_value(dictionary, b"Rotate") {
        inherited.rotation_degrees = page_rotation(value)?;
    }
    if let Some(value) = dictionary_value(dictionary, b"UserUnit") {
        inherited.user_unit = page_user_unit(value)?;
    }
    if let Some(value) = dictionary_value(dictionary, b"Resources") {
        inherited.resources = resource_reference(value)?;
    }
    Ok(inherited)
}

fn required_reference(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    field: &'static str,
) -> ObjectResult<Reference> {
    dictionary_value(dictionary, key)
        .ok_or(ObjectError::MissingPageTreeField { field })
        .and_then(reference_from_primitive)
}

fn required_reference_array(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    field: &'static str,
) -> ObjectResult<Vec<Reference>> {
    let value =
        dictionary_value(dictionary, key).ok_or(ObjectError::MissingPageTreeField { field })?;
    let PdfPrimitive::Array(values) = value else {
        return Err(ObjectError::MissingPageTreeField { field });
    };
    values.iter().map(reference_from_primitive).collect()
}

fn resource_reference(value: &PdfPrimitive<'_>) -> ObjectResult<Option<Reference>> {
    match value {
        PdfPrimitive::Reference(_) => reference_from_primitive(value).map(Some),
        PdfPrimitive::Dictionary(_) => Ok(None),
        _ => Err(ObjectError::MissingPageTreeField { field: "Resources" }),
    }
}

fn reference_from_primitive(value: &PdfPrimitive<'_>) -> ObjectResult<Reference> {
    let PdfPrimitive::Reference(reference) = value else {
        return Err(ObjectError::MissingPageTreeField { field: "Reference" });
    };
    let number = ObjectNumber::new(reference.object)?;
    Ok(Reference::new(ObjectId::new(
        number,
        GenerationNumber::new(reference.generation),
    )))
}

fn dictionary_name<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    key: &[u8],
) -> Option<&'a [u8]> {
    let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, key) else {
        return None;
    };
    Some(name.as_bytes())
}

fn page_box(value: &PdfPrimitive<'_>) -> ObjectResult<PageBox> {
    let PdfPrimitive::Array(values) = value else {
        return Err(ObjectError::InvalidPageBox);
    };
    if values.len() != 4 {
        return Err(ObjectError::InvalidPageBox);
    }
    let box_value = PageBox {
        left: page_box_number(&values[0])?,
        bottom: page_box_number(&values[1])?,
        right: page_box_number(&values[2])?,
        top: page_box_number(&values[3])?,
    };
    let size = box_value.size();
    if !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return Err(ObjectError::InvalidPageBox);
    }
    Ok(box_value)
}

fn page_box_number(value: &PdfPrimitive<'_>) -> ObjectResult<f64> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Ok(*value as f64),
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => Ok(*value),
        _ => Err(ObjectError::InvalidPageBox),
    }
}

fn page_rotation(value: &PdfPrimitive<'_>) -> ObjectResult<u16> {
    let PdfPrimitive::Number(PdfNumber::Integer(value)) = value else {
        return Err(ObjectError::InvalidPageRotation);
    };
    if value % 90 != 0 {
        return Err(ObjectError::InvalidPageRotation);
    }
    let normalized = value.rem_euclid(360);
    u16::try_from(normalized).map_err(|_| ObjectError::InvalidPageRotation)
}

fn page_rotation_swaps_axes(rotation_degrees: u16) -> bool {
    matches!(rotation_degrees, 90 | 270)
}

fn page_user_unit(value: &PdfPrimitive<'_>) -> ObjectResult<f64> {
    let unit = match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => *value as f64,
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => *value,
        _ => return Err(ObjectError::InvalidUserUnit),
    };
    if unit <= 0.0 || unit > 75_000.0 {
        return Err(ObjectError::InvalidUserUnit);
    }
    Ok(unit)
}

fn parse_object_at_offset<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
) -> ObjectResult<(IndirectObject<'a>, usize)> {
    let start = entry.offset.get();
    if start >= bytes.len() {
        return Err(ObjectError::malformed(
            entry.offset,
            "xref offset is outside input",
        ));
    }
    let (object, consumed) = parse_indirect_object_prefix(&bytes[start..], entry.offset)?;
    if object.id != entry.id {
        return Err(ObjectError::XrefOffsetMismatch {
            expected: entry.id,
            actual: object.id,
            offset: entry.offset,
        });
    }
    Ok((object, consumed.get()))
}

fn parse_object_with_xref_recovery<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
) -> ObjectResult<IndirectObject<'a>> {
    parse_object_with_xref_recovery_with_span(bytes, entry).map(|(object, _)| object)
}

fn parse_object_with_xref_recovery_with_span<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
) -> ObjectResult<(IndirectObject<'a>, usize)> {
    match parse_object_at_offset(bytes, entry) {
        Ok(object) => Ok(object),
        Err(strict_error) => {
            if matches!(
                strict_error,
                ObjectError::Malformed { .. } | ObjectError::Syntax(_)
            ) {
                recover_xref_offset(bytes, entry, DEFAULT_XREF_OFFSET_RECOVERY_SCAN_BYTES)
                    .unwrap_or(Err(strict_error))
            } else {
                Err(strict_error)
            }
        }
    }
}

fn recover_xref_offset<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
    scan_bytes: usize,
) -> Option<ObjectResult<(IndirectObject<'a>, usize)>> {
    let expected_header = format!(
        "{} {} obj",
        entry.id.number.get(),
        entry.id.generation.get()
    );
    let header = expected_header.as_bytes();
    if header.is_empty() || bytes.len() < header.len() {
        return None;
    }

    let declared = entry.offset.get();
    let start = declared.saturating_sub(scan_bytes);
    let end = declared
        .saturating_add(scan_bytes)
        .saturating_add(header.len())
        .min(bytes.len());
    if end.saturating_sub(start) < header.len() {
        return None;
    }

    for (relative, candidate) in bytes[start..end].windows(header.len()).enumerate() {
        if candidate != header {
            continue;
        }
        let recovered = parse_object_at_offset(
            bytes,
            ClassicXrefEntry {
                id: entry.id,
                offset: ByteOffset::new(start + relative),
            },
        );
        if recovered.is_ok() {
            return Some(recovered);
        }
    }

    None
}

fn parse_linearization_dictionary(bytes: &[u8]) -> ObjectResult<Option<LinearizationDictionary>> {
    let Some(offset) = first_indirect_object_offset(bytes)? else {
        return Ok(None);
    };
    let (object, _) = parse_indirect_object_prefix(&bytes[offset..], ByteOffset::new(offset))?;
    let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = object.value else {
        return Ok(None);
    };
    if !dictionary_has_linearized_marker(&dictionary) {
        return Ok(None);
    }
    let first_page_object = ObjectNumber::new(required_u32(&dictionary, b"O")?)?;
    let primary_hint_table = optional_number_array(&dictionary, b"H")?
        .and_then(|values| (values.len() >= 2).then_some((values[0], values[1])));

    Ok(Some(LinearizationDictionary {
        file_length: required_usize(&dictionary, b"L")?,
        first_page_end: required_usize(&dictionary, b"E")?,
        first_page_object,
        page_count: required_usize(&dictionary, b"N")?,
        primary_hint_table,
        main_xref_offset: optional_usize(&dictionary, b"T")?,
    }))
}

fn validate_linearization_dictionary(
    bytes: &[u8],
    dictionary: LinearizationDictionary,
) -> ObjectResult<()> {
    if dictionary.file_length != bytes.len() {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "linearized file length does not match input",
        ));
    }
    if dictionary.first_page_end == 0 || dictionary.first_page_end > bytes.len() {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "linearized first-page end is outside input",
        ));
    }
    Ok(())
}

fn first_indirect_object_offset(bytes: &[u8]) -> ObjectResult<Option<usize>> {
    if bytes.is_empty() {
        return Ok(None);
    }
    let mut parser = RawParser::new(bytes, 0);
    if parser.starts_with(b"%PDF-") {
        parser.skip_until_next_line()?;
    }

    loop {
        parser.skip_whitespace()?;
        match parser.peek() {
            Some(b'%') => parser.skip_until_next_line()?,
            Some(_) => return Ok(Some(parser.offset().get())),
            None => return Ok(None),
        }
    }
}

fn dictionary_has_linearized_marker(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> bool {
    let Some(value) = dictionary_value(dictionary, b"Linearized") else {
        return false;
    };
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => *value == 1,
        PdfPrimitive::Number(PdfNumber::Real(value)) => (*value - 1.0).abs() < f64::EPSILON,
        _ => false,
    }
}

fn parse_xref_stream_and_trailer<'a>(
    bytes: &'a [u8],
    startxref: ByteOffset,
) -> ObjectResult<(XrefStreamTable, Trailer<'a>)> {
    let (object, _) = parse_indirect_object_prefix(&bytes[startxref.get()..], startxref)?;
    let ObjectValue::Stream(stream) = object.value else {
        return Err(ObjectError::malformed(
            startxref,
            "startxref object must be a stream",
        ));
    };
    if !dictionary_name_is(stream.dictionary(), b"Type", b"XRef") {
        return Err(ObjectError::malformed(
            startxref,
            "startxref stream must have /Type /XRef",
        ));
    }
    let entries = parse_xref_stream_entries(&stream)?;
    Ok((
        XrefStreamTable { startxref, entries },
        Trailer {
            dictionary: stream.dictionary,
        },
    ))
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
    let mut deleted = Vec::new();

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
            } else if marker == b'f' {
                let raw_number = first_object.checked_add(index).ok_or_else(|| {
                    ObjectError::malformed(parser.offset(), "xref object number overflow")
                })?;
                if raw_number != 0 {
                    let number = ObjectNumber::new(raw_number).map_err(|_| {
                        ObjectError::malformed(parser.offset(), "xref object number overflow")
                    })?;
                    deleted.push(number);
                }
            } else {
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
        ClassicXrefTable {
            startxref,
            entries,
            deleted,
        },
        Trailer { dictionary },
    ))
}

fn parse_classic_xref_chain<'a>(
    bytes: &'a [u8],
    startxref: ByteOffset,
) -> ObjectResult<(ClassicXrefTable, Trailer<'a>)> {
    let mut current = startxref;
    let mut seen = Vec::new();
    let mut tables = Vec::new();
    let mut trailers = Vec::new();

    loop {
        if seen.contains(&current) {
            return Err(ObjectError::IncrementalUpdateCycle { offset: current });
        }
        if seen.len() >= DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT {
            return Err(ObjectError::IncrementalUpdateDepthExceeded {
                limit: DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT,
            });
        }
        seen.push(current);
        let (xref, trailer) = parse_classic_xref_and_trailer(bytes, current)?;
        let previous = trailer_prev_offset(&trailer)?;
        tables.push(xref);
        trailers.push(trailer);
        let Some(previous) = previous else {
            break;
        };
        current = previous;
    }

    let mut entries = Vec::new();
    let mut deleted = Vec::new();
    for table in &tables {
        for id in &table.deleted {
            if !entries
                .iter()
                .any(|existing: &ClassicXrefEntry| existing.id.number == *id)
                && !deleted.contains(id)
            {
                deleted.push(*id);
            }
        }
        for entry in table.entries() {
            if !entries
                .iter()
                .any(|existing: &ClassicXrefEntry| existing.id == entry.id)
                && !deleted.contains(&entry.id.number)
            {
                entries.push(*entry);
            }
        }
    }
    let trailer = trailers.remove(0);
    Ok((
        ClassicXrefTable {
            startxref,
            entries,
            deleted,
        },
        trailer,
    ))
}

fn trailer_prev_offset(trailer: &Trailer<'_>) -> ObjectResult<Option<ByteOffset>> {
    dictionary_value(trailer.entries(), b"Prev")
        .map(primitive_usize)
        .transpose()
        .map(|offset| offset.map(ByteOffset::new))
}

fn extend_classic_xref_with_hybrid_stream(
    bytes: &[u8],
    xref: &mut ClassicXrefTable,
    trailer: &Trailer<'_>,
) -> ObjectResult<()> {
    let Some(offset) = trailer_xref_stream_offset(trailer)? else {
        return Ok(());
    };
    let (hybrid_xref, _) = parse_xref_stream_and_trailer(bytes, offset)?;
    for entry in hybrid_xref.entries() {
        let XrefStreamEntry::InUse { id, offset } = *entry else {
            continue;
        };
        if !xref.entries.iter().any(|existing| existing.id == id) {
            xref.entries.push(ClassicXrefEntry { id, offset });
        }
    }
    Ok(())
}

fn trailer_xref_stream_offset(trailer: &Trailer<'_>) -> ObjectResult<Option<ByteOffset>> {
    dictionary_value(trailer.entries(), b"XRefStm")
        .map(primitive_usize)
        .transpose()
        .map(|offset| offset.map(ByteOffset::new))
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

    fn parse_usize(&mut self) -> ObjectResult<usize> {
        parse_usize(self.parse_unsigned_decimal()?, self.offset())
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedObjectStream {
    id: ObjectId,
    decoded: Vec<u8>,
    objects: Vec<ObjectStreamObject>,
}

impl LoadedObjectStream {
    fn parse_object_at_position(
        &self,
        position: usize,
        expected_id: ObjectId,
    ) -> ObjectResult<IndirectObject<'_>> {
        let Some(object) = self.objects.get(position) else {
            return Err(ObjectError::ObjectStreamMismatch {
                expected: expected_id,
                actual: None,
                object_stream: self.id,
                index: position,
            });
        };
        if object.id != expected_id {
            return Err(ObjectError::ObjectStreamMismatch {
                expected: expected_id,
                actual: Some(object.id),
                object_stream: self.id,
                index: object.index,
            });
        }
        let value = parse_primitive(PdfBytes::new(
            &self.decoded[object.value_start..object.value_end],
        ))?;
        Ok(IndirectObject {
            id: expected_id,
            value: ObjectValue::Primitive(value),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectStreamObject {
    id: ObjectId,
    index: usize,
    value_start: usize,
    value_end: usize,
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

fn parse_usize(raw: &[u8], offset: ByteOffset) -> ObjectResult<usize> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| ObjectError::malformed(offset, "number is not valid UTF-8"))?;
    text.parse::<usize>()
        .map_err(|_| ObjectError::malformed(offset, "number is out of range"))
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

fn required_usize(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<usize> {
    let value = dictionary_value(dictionary, key).ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "required dictionary number is missing")
    })?;
    primitive_usize(value)
}

fn required_i64(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<i64> {
    let value = dictionary_value(dictionary, key).ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "required dictionary number is missing")
    })?;
    let PdfPrimitive::Number(PdfNumber::Integer(value)) = value else {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "expected integer number",
        ));
    };
    Ok(*value)
}

fn optional_i64(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)], key: &'static [u8]) -> Option<i64> {
    let PdfPrimitive::Number(PdfNumber::Integer(value)) = dictionary_value(dictionary, key)? else {
        return None;
    };
    Some(*value)
}

fn optional_bool(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> Option<bool> {
    let PdfPrimitive::Boolean(value) = dictionary_value(dictionary, key)? else {
        return None;
    };
    Some(*value)
}

fn required_string_bytes(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<Vec<u8>> {
    let Some(PdfPrimitive::String(string)) = dictionary_value(dictionary, key) else {
        return Err(ObjectError::Encrypted);
    };
    decode_pdf_string_bytes(*string, DEFAULT_MAX_DECODED_LEN).map_err(|_| ObjectError::Encrypted)
}

fn decode_pdf_string_bytes(string: PdfString<'_>, limit: usize) -> ObjectResult<Vec<u8>> {
    match string {
        PdfString::Literal(bytes) => decode_literal_string_bytes(bytes, limit),
        PdfString::Hex(bytes) => decode_hex_string_bytes(bytes, limit),
    }
}

fn decode_literal_string_bytes(raw: &[u8], limit: usize) -> ObjectResult<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut index = 0_usize;
    while index < raw.len() {
        let byte = raw[index];
        index += 1;
        if byte != b'\\' {
            push_limited(&mut decoded, byte, limit)?;
            continue;
        }
        let Some(escaped) = raw.get(index).copied() else {
            break;
        };
        index += 1;
        match escaped {
            b'n' => push_limited(&mut decoded, b'\n', limit)?,
            b'r' => push_limited(&mut decoded, b'\r', limit)?,
            b't' => push_limited(&mut decoded, b'\t', limit)?,
            b'b' => push_limited(&mut decoded, 0x08, limit)?,
            b'f' => push_limited(&mut decoded, 0x0c, limit)?,
            b'(' | b')' | b'\\' => push_limited(&mut decoded, escaped, limit)?,
            b'\r' => {
                if raw.get(index) == Some(&b'\n') {
                    index += 1;
                }
            }
            b'\n' => {}
            b'0'..=b'7' => {
                let mut value = escaped - b'0';
                for _ in 0..2 {
                    let Some(next @ b'0'..=b'7') = raw.get(index).copied() else {
                        break;
                    };
                    index += 1;
                    value = value.saturating_mul(8).saturating_add(next - b'0');
                }
                push_limited(&mut decoded, value, limit)?;
            }
            _ => push_limited(&mut decoded, escaped, limit)?,
        }
    }
    Ok(decoded)
}

fn decode_hex_string_bytes(raw: &[u8], limit: usize) -> ObjectResult<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut high_nibble = None;
    for byte in raw.iter().copied() {
        if is_whitespace(byte) {
            continue;
        }
        let nibble = hex_nibble(byte).ok_or(ObjectError::Encrypted)?;
        if let Some(high) = high_nibble.take() {
            push_limited(&mut decoded, (high << 4) | nibble, limit)?;
        } else {
            high_nibble = Some(nibble);
        }
    }
    if let Some(high) = high_nibble {
        push_limited(&mut decoded, high << 4, limit)?;
    }
    Ok(decoded)
}

fn standard_v2_file_key(owner: &[u8], permissions: i64, file_id: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(padded_empty_password());
    hasher.update(owner);
    hasher.update(permission_bytes(permissions));
    hasher.update(file_id);
    hasher.finalize()[..5].to_vec()
}

fn standard_v4_file_key(
    owner: &[u8],
    permissions: i64,
    file_id: &[u8],
    key_bits: usize,
    encrypt_metadata: bool,
) -> ObjectResult<Vec<u8>> {
    if key_bits == 0 || key_bits % 8 != 0 || key_bits > 128 {
        return Err(ObjectError::Encrypted);
    }
    let key_bytes = key_bits / 8;
    let mut hasher = Md5::new();
    hasher.update(padded_empty_password());
    hasher.update(owner);
    hasher.update(permission_bytes(permissions));
    hasher.update(file_id);
    if !encrypt_metadata {
        hasher.update([0xff, 0xff, 0xff, 0xff]);
    }
    let mut digest = hasher.finalize().to_vec();
    for _ in 0..50 {
        let mut hasher = Md5::new();
        hasher.update(&digest[..key_bytes]);
        digest = hasher.finalize().to_vec();
    }
    Ok(digest[..key_bytes].to_vec())
}

fn standard_v4_user_key(file_key: &[u8], file_id: &[u8]) -> ObjectResult<Vec<u8>> {
    let mut hasher = Md5::new();
    hasher.update(PASSWORD_PADDING);
    hasher.update(file_id);
    let mut value = hasher.finalize().to_vec();
    value = rc4_crypt(file_key, &value)?;
    for round in 1_u8..=19 {
        let round_key: Vec<u8> = file_key.iter().map(|byte| byte ^ round).collect();
        value = rc4_crypt(&round_key, &value)?;
    }
    value.extend_from_slice(&[0; 16]);
    Ok(value)
}

fn standard_v5_file_key(user: &[u8], user_encryption_key: &[u8]) -> ObjectResult<Vec<u8>> {
    if user.len() < 48 || user_encryption_key.len() != PDF_AES256_KEY_BYTES {
        return Err(ObjectError::Encrypted);
    }
    let validation_salt = &user[32..40];
    let key_salt = &user[40..48];
    let mut hasher = Sha256::new();
    hasher.update([]);
    hasher.update(validation_salt);
    let validation_hash = hasher.finalize();
    if user.get(..32) != Some(&validation_hash[..]) {
        return Err(ObjectError::Encrypted);
    }
    let mut hasher = Sha256::new();
    hasher.update([]);
    hasher.update(key_salt);
    let intermediate_key = hasher.finalize();
    aes_cbc_decrypt_no_padding(&intermediate_key, user_encryption_key)
}

fn validate_standard_v5_permissions(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    file_key: &[u8],
) -> ObjectResult<()> {
    let Some(PdfPrimitive::String(perms)) = dictionary_value(dictionary, b"Perms") else {
        return Ok(());
    };
    let encrypted = decode_pdf_string_bytes(*perms, DEFAULT_MAX_DECODED_LEN)
        .map_err(|_| ObjectError::Encrypted)?;
    if encrypted.len() != AES_BLOCK_BYTES {
        return Err(ObjectError::Encrypted);
    }
    let decrypted = aes_cbc_decrypt_no_padding(file_key, &encrypted)?;
    if decrypted.get(9..12) != Some(b"adb") {
        return Err(ObjectError::Encrypted);
    }
    Ok(())
}

fn padded_empty_password() -> &'static [u8] {
    &PASSWORD_PADDING
}

fn permission_bytes(permissions: i64) -> [u8; 4] {
    let raw = permissions as i32;
    raw.to_le_bytes()
}

fn crypt_filter(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    fallback: CryptAlgorithm,
) -> ObjectResult<CryptFilter> {
    let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, key) else {
        return Ok(CryptFilter::Standard(fallback));
    };
    if name.as_bytes() == b"Identity" {
        return Ok(CryptFilter::Identity);
    }
    let Some(PdfPrimitive::Dictionary(filters)) = dictionary_value(dictionary, b"CF") else {
        return Err(ObjectError::Encrypted);
    };
    let Some(PdfPrimitive::Dictionary(filter_dictionary)) =
        dictionary_value(filters, name.as_bytes())
    else {
        return Err(ObjectError::Encrypted);
    };
    let Some(PdfPrimitive::Name(method)) = dictionary_value(filter_dictionary, b"CFM") else {
        return Ok(CryptFilter::Standard(fallback));
    };
    match method.as_bytes() {
        b"None" => Ok(CryptFilter::Identity),
        b"V2" => Ok(CryptFilter::Standard(CryptAlgorithm::Rc4)),
        b"AESV2" => Ok(CryptFilter::Standard(CryptAlgorithm::AesV2)),
        b"AESV3" => Ok(CryptFilter::Standard(CryptAlgorithm::AesV3)),
        _ => Err(ObjectError::Encrypted),
    }
}

fn object_crypt_key(file_key: &[u8], id: ObjectId, algorithm: CryptAlgorithm) -> Vec<u8> {
    if algorithm == CryptAlgorithm::AesV3 {
        return file_key.to_vec();
    }
    let mut material = Vec::with_capacity(file_key.len() + 9);
    material.extend_from_slice(file_key);
    let object_number = id.number.get();
    material.push((object_number & 0xff) as u8);
    material.push(((object_number >> 8) & 0xff) as u8);
    material.push(((object_number >> 16) & 0xff) as u8);
    let generation = id.generation.get();
    material.push((generation & 0xff) as u8);
    material.push(((generation >> 8) & 0xff) as u8);
    if algorithm == CryptAlgorithm::AesV2 {
        material.extend_from_slice(AES_OBJECT_KEY_SALT);
    }
    let digest = Md5::digest(material);
    let key_len = file_key.len().saturating_add(5).min(16);
    digest[..key_len].to_vec()
}

fn rc4_crypt(key: &[u8], data: &[u8]) -> ObjectResult<Vec<u8>> {
    macro_rules! rc4_with_key_len {
        ($key_len:ty) => {{
            let mut output = data.to_vec();
            let key = rc4::Key::<$key_len>::from_slice(key);
            let mut cipher = Rc4::<$key_len>::new(key);
            cipher.apply_keystream(&mut output);
            Ok(output)
        }};
    }
    match key.len() {
        5 => rc4_with_key_len!(U5),
        6 => rc4_with_key_len!(U6),
        7 => rc4_with_key_len!(U7),
        8 => rc4_with_key_len!(U8),
        9 => rc4_with_key_len!(U9),
        10 => rc4_with_key_len!(U10),
        11 => rc4_with_key_len!(U11),
        12 => rc4_with_key_len!(U12),
        13 => rc4_with_key_len!(U13),
        14 => rc4_with_key_len!(U14),
        15 => rc4_with_key_len!(U15),
        16 => rc4_with_key_len!(U16),
        _ => Err(ObjectError::Encrypted),
    }
}

fn aes_cbc_decrypt_pkcs7(key: &[u8], encrypted: &[u8]) -> ObjectResult<Vec<u8>> {
    if encrypted.len() < AES_BLOCK_BYTES {
        return Err(ObjectError::Encrypted);
    }
    let (iv, ciphertext) = encrypted.split_at(AES_BLOCK_BYTES);
    match key.len() {
        16 => cbc::Decryptor::<aes::Aes128>::new(key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| ObjectError::Encrypted),
        32 => cbc::Decryptor::<aes::Aes256>::new(key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| ObjectError::Encrypted),
        _ => Err(ObjectError::Encrypted),
    }
}

fn aes_cbc_decrypt_no_padding(key: &[u8], encrypted: &[u8]) -> ObjectResult<Vec<u8>> {
    if encrypted.len() % AES_BLOCK_BYTES != 0 {
        return Err(ObjectError::Encrypted);
    }
    let iv = [0_u8; AES_BLOCK_BYTES];
    match key.len() {
        32 => cbc::Decryptor::<aes::Aes256>::new(key.into(), (&iv).into())
            .decrypt_padded_vec_mut::<NoPadding>(encrypted)
            .map_err(|_| ObjectError::Encrypted),
        _ => Err(ObjectError::Encrypted),
    }
}

fn optional_usize(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<Option<usize>> {
    dictionary_value(dictionary, key)
        .map(primitive_usize)
        .transpose()
}

fn required_u32(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<u32> {
    let value = dictionary_value(dictionary, key).ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "required dictionary number is missing")
    })?;
    let value = primitive_usize(value)?;
    u32::try_from(value)
        .map_err(|_| ObjectError::malformed(ByteOffset::new(0), "integer number is out of range"))
}

fn primitive_usize(value: &PdfPrimitive<'_>) -> ObjectResult<usize> {
    let PdfPrimitive::Number(PdfNumber::Integer(raw)) = value else {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "expected integer number",
        ));
    };
    usize::try_from(*raw)
        .map_err(|_| ObjectError::malformed(ByteOffset::new(0), "integer number is out of range"))
}

fn required_number_array(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<Vec<usize>> {
    let value = dictionary_value(dictionary, key).ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "required dictionary array is missing")
    })?;
    let PdfPrimitive::Array(values) = value else {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "expected integer array",
        ));
    };
    values.iter().map(primitive_usize).collect()
}

fn optional_number_array(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> ObjectResult<Option<Vec<usize>>> {
    dictionary_value(dictionary, key)
        .map(|value| {
            let PdfPrimitive::Array(values) = value else {
                return Err(ObjectError::malformed(
                    ByteOffset::new(0),
                    "expected integer array",
                ));
            };
            values.iter().map(primitive_usize).collect()
        })
        .transpose()
}

fn dictionary_name_is(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
    expected: &[u8],
) -> bool {
    matches!(
        dictionary_value(dictionary, key),
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == expected
    )
}

fn parse_xref_stream_entries(stream: &StreamObject<'_>) -> ObjectResult<Vec<XrefStreamEntry>> {
    let widths = required_number_array(stream.dictionary(), b"W")?;
    if widths.len() != 3 {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "xref stream /W must have three numbers",
        ));
    }
    let size = required_usize(stream.dictionary(), b"Size")?;
    let index =
        optional_number_array(stream.dictionary(), b"Index")?.unwrap_or_else(|| vec![0, size]);
    if index.len() % 2 != 0 {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "xref stream /Index must contain pairs",
        ));
    }
    let entry_width = widths.iter().try_fold(0_usize, |sum, width| {
        sum.checked_add(*width)
            .ok_or_else(|| ObjectError::malformed(ByteOffset::new(0), "xref entry width overflow"))
    })?;
    if entry_width == 0 {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "xref stream entry width must be non-zero",
        ));
    }

    let decoded = stream.decode()?;
    let expected_entries = index.chunks_exact(2).try_fold(0_usize, |sum, pair| {
        sum.checked_add(pair[1]).ok_or_else(|| {
            ObjectError::malformed(ByteOffset::new(0), "xref stream entry count overflow")
        })
    })?;
    let expected_len = expected_entries.checked_mul(entry_width).ok_or_else(|| {
        ObjectError::malformed(ByteOffset::new(0), "xref stream decoded length overflow")
    })?;
    if decoded.len() < expected_len {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "xref stream decoded data is shorter than declared entries",
        ));
    }

    let mut entries = Vec::new();
    let mut cursor = 0_usize;
    for pair in index.chunks_exact(2) {
        let first_object = pair[0];
        let count = pair[1];
        for relative_index in 0..count {
            let fields = read_xref_fields(&decoded[cursor..cursor + entry_width], &widths)?;
            cursor += entry_width;
            let object_number = first_object.checked_add(relative_index).ok_or_else(|| {
                ObjectError::malformed(ByteOffset::new(0), "xref stream object number overflow")
            })?;
            match fields[0] {
                0 => {}
                1 => {
                    if object_number == 0 {
                        continue;
                    }
                    let id = ObjectId::new(
                        ObjectNumber::new(u32::try_from(object_number).map_err(|_| {
                            ObjectError::malformed(
                                ByteOffset::new(0),
                                "xref stream object number is out of range",
                            )
                        })?)?,
                        GenerationNumber::new(u16::try_from(fields[2]).map_err(|_| {
                            ObjectError::malformed(
                                ByteOffset::new(0),
                                "xref stream generation is out of range",
                            )
                        })?),
                    );
                    entries.push(XrefStreamEntry::InUse {
                        id,
                        offset: ByteOffset::new(fields[1]),
                    });
                }
                2 => {
                    let id = ObjectId::new(
                        ObjectNumber::new(u32::try_from(object_number).map_err(|_| {
                            ObjectError::malformed(
                                ByteOffset::new(0),
                                "xref stream object number is out of range",
                            )
                        })?)?,
                        GenerationNumber::new(0),
                    );
                    let object_stream = ObjectId::new(
                        ObjectNumber::new(u32::try_from(fields[1]).map_err(|_| {
                            ObjectError::malformed(
                                ByteOffset::new(0),
                                "object stream number is out of range",
                            )
                        })?)?,
                        GenerationNumber::new(0),
                    );
                    entries.push(XrefStreamEntry::Compressed {
                        id,
                        object_stream,
                        index: fields[2],
                    });
                }
                _ => {
                    return Err(ObjectError::malformed(
                        ByteOffset::new(0),
                        "unsupported xref stream entry type",
                    ));
                }
            }
        }
    }
    Ok(entries)
}

fn read_xref_fields(raw: &[u8], widths: &[usize]) -> ObjectResult<[usize; 3]> {
    let mut offset = 0_usize;
    let mut fields = [0_usize; 3];
    for (index, width) in widths.iter().copied().enumerate() {
        let end = offset.checked_add(width).ok_or_else(|| {
            ObjectError::malformed(ByteOffset::new(0), "xref field width overflow")
        })?;
        let value = if width == 0 && index == 0 {
            1
        } else {
            read_big_endian_usize(&raw[offset..end])?
        };
        fields[index] = value;
        offset = end;
    }
    Ok(fields)
}

fn read_big_endian_usize(raw: &[u8]) -> ObjectResult<usize> {
    raw.iter().try_fold(0_usize, |value, byte| {
        value
            .checked_mul(256)
            .and_then(|value| value.checked_add(usize::from(*byte)))
            .ok_or_else(|| ObjectError::malformed(ByteOffset::new(0), "xref field overflow"))
    })
}

fn load_referenced_object_streams(
    objects: &ObjectTable<'_>,
    entries: &[XrefStreamEntry],
) -> ObjectResult<Vec<LoadedObjectStream>> {
    let mut loaded = Vec::new();
    for entry in entries {
        let XrefStreamEntry::Compressed { object_stream, .. } = *entry else {
            continue;
        };
        if loaded
            .iter()
            .any(|loaded_stream: &LoadedObjectStream| loaded_stream.id == object_stream)
        {
            continue;
        }
        let object = objects
            .get(object_stream)
            .ok_or(ObjectError::MissingObjectStream { id: object_stream })?;
        loaded.push(load_object_stream(object)?);
    }
    Ok(loaded)
}

fn load_object_stream(object: &IndirectObject<'_>) -> ObjectResult<LoadedObjectStream> {
    let ObjectValue::Stream(stream) = &object.value else {
        return Err(ObjectError::MissingObjectStream { id: object.id });
    };
    if !dictionary_name_is(stream.dictionary(), b"Type", b"ObjStm") {
        return Err(ObjectError::MissingObjectStream { id: object.id });
    }
    let count = required_usize(stream.dictionary(), b"N")?;
    let first = required_usize(stream.dictionary(), b"First")?;
    let decoded = stream.decode()?;
    if first > decoded.len() {
        return Err(ObjectError::malformed(
            ByteOffset::new(0),
            "object stream /First exceeds decoded length",
        ));
    }

    let header = &decoded[..first];
    let mut parser = RawParser::new(header, 0);
    let mut pairs = Vec::with_capacity(count);
    for index in 0..count {
        parser.skip_whitespace()?;
        let object_number = parser.parse_u32()?;
        parser.skip_whitespace()?;
        let relative_offset = parser.parse_usize()?;
        let id = ObjectId::new(ObjectNumber::new(object_number)?, GenerationNumber::new(0));
        pairs.push((index, id, relative_offset));
    }

    let mut stream_objects = Vec::with_capacity(pairs.len());
    for (position, (index, id, relative_offset)) in pairs.iter().copied().enumerate() {
        let value_start = first.checked_add(relative_offset).ok_or_else(|| {
            ObjectError::malformed(ByteOffset::new(0), "object stream object offset overflow")
        })?;
        let value_end = pairs
            .get(position + 1)
            .map_or(decoded.len(), |(_, _, next_offset)| {
                first.saturating_add(*next_offset)
            });
        if value_start > value_end || value_end > decoded.len() {
            return Err(ObjectError::malformed(
                ByteOffset::new(0),
                "object stream object range is invalid",
            ));
        }
        parse_primitive(PdfBytes::new(&decoded[value_start..value_end]))?;
        stream_objects.push(ObjectStreamObject {
            id,
            index,
            value_start,
            value_end,
        });
    }

    Ok(LoadedObjectStream {
        id: object.id,
        decoded,
        objects: stream_objects,
    })
}

fn validate_compressed_entries(
    object_streams: &[LoadedObjectStream],
    entries: &[XrefStreamEntry],
) -> ObjectResult<()> {
    for entry in entries {
        let XrefStreamEntry::Compressed {
            id,
            object_stream,
            index,
        } = *entry
        else {
            continue;
        };
        let loaded = object_streams
            .iter()
            .find(|loaded| loaded.id == object_stream)
            .ok_or(ObjectError::MissingObjectStream { id: object_stream })?;
        let Some(object) = loaded.objects.iter().find(|object| object.index == index) else {
            return Err(ObjectError::ObjectStreamMismatch {
                expected: id,
                actual: None,
                object_stream,
                index,
            });
        };
        if object.id != id {
            return Err(ObjectError::ObjectStreamMismatch {
                expected: id,
                actual: Some(object.id),
                object_stream,
                index,
            });
        }
    }
    Ok(())
}

fn compressed_object_index(
    object_streams: &[LoadedObjectStream],
    entries: &[XrefStreamEntry],
) -> ObjectResult<Vec<CompressedObjectIndexEntry>> {
    let compressed_count = entries
        .iter()
        .filter(|entry| matches!(entry, XrefStreamEntry::Compressed { .. }))
        .count();
    let mut index = Vec::with_capacity(compressed_count);
    for entry in entries {
        let XrefStreamEntry::Compressed {
            id,
            object_stream,
            index: compressed_index,
        } = *entry
        else {
            continue;
        };
        let stream_index = object_streams
            .iter()
            .position(|loaded| loaded.id == object_stream)
            .ok_or(ObjectError::MissingObjectStream { id: object_stream })?;
        let object_position = object_streams[stream_index]
            .objects
            .iter()
            .position(|object| object.index == compressed_index)
            .ok_or(ObjectError::ObjectStreamMismatch {
                expected: id,
                actual: None,
                object_stream,
                index: compressed_index,
            })?;
        index.push(CompressedObjectIndexEntry {
            id,
            stream_index,
            object_position,
        });
    }
    Ok(index)
}

fn stream_filters(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> ObjectResult<Vec<StreamFilter>> {
    let Some(value) = dictionary_value(dictionary, b"Filter") else {
        return Ok(Vec::new());
    };
    match value {
        PdfPrimitive::Name(name) => Ok(vec![StreamFilter::from_name(
            name.as_bytes(),
            stream_decode_params(dictionary, 0, 1)?,
        )?]),
        PdfPrimitive::Array(filters) => filters
            .iter()
            .enumerate()
            .map(|filter| match filter {
                (index, PdfPrimitive::Name(name)) => StreamFilter::from_name(
                    name.as_bytes(),
                    stream_decode_params(dictionary, index, filters.len())?,
                ),
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

fn stream_decode_params<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    index: usize,
    filter_count: usize,
) -> ObjectResult<Option<&'a [(PdfName<'a>, PdfPrimitive<'a>)]>> {
    let Some(value) = dictionary_value(dictionary, b"DecodeParms")
        .or_else(|| dictionary_value(dictionary, b"DP"))
    else {
        return Ok(None);
    };
    match value {
        PdfPrimitive::Null => Ok(None),
        PdfPrimitive::Dictionary(params) if filter_count == 1 => Ok(Some(params.as_slice())),
        PdfPrimitive::Dictionary(_) => Err(ObjectError::malformed(
            ByteOffset::new(0),
            "stream filter arrays require DecodeParms arrays",
        )),
        PdfPrimitive::Array(values) if values.len() == filter_count => match &values[index] {
            PdfPrimitive::Null => Ok(None),
            PdfPrimitive::Dictionary(params) => Ok(Some(params.as_slice())),
            _ => Err(ObjectError::malformed(
                ByteOffset::new(0),
                "DecodeParms array entries must be dictionaries or null",
            )),
        },
        PdfPrimitive::Array(_) => Err(ObjectError::malformed(
            ByteOffset::new(0),
            "DecodeParms array length must match Filter array length",
        )),
        _ => Err(ObjectError::malformed(
            ByteOffset::new(0),
            "stream /DecodeParms must be a dictionary, array, or null",
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
    Lzw { early_change: usize },
    RunLength,
}

impl StreamFilter {
    fn from_name(
        name: &[u8],
        params: Option<&[(PdfName<'_>, PdfPrimitive<'_>)]>,
    ) -> ObjectResult<Self> {
        match name {
            b"FlateDecode" | b"Fl" => Ok(Self::Flate),
            b"ASCIIHexDecode" | b"AHx" => Ok(Self::AsciiHex),
            b"ASCII85Decode" | b"A85" => Ok(Self::Ascii85),
            b"LZWDecode" | b"LZW" => Ok(Self::Lzw {
                early_change: lzw_early_change(params)?,
            }),
            b"RunLengthDecode" | b"RL" => Ok(Self::RunLength),
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
            Self::Lzw { .. } => "LZWDecode",
            Self::RunLength => "RunLengthDecode",
        }
    }
}

fn lzw_early_change(params: Option<&[(PdfName<'_>, PdfPrimitive<'_>)]>) -> ObjectResult<usize> {
    let Some(params) = params else {
        return Ok(1);
    };
    match optional_usize(params, b"EarlyChange")?.unwrap_or(1) {
        value @ (0 | 1) => Ok(value),
        _ => Err(ObjectError::Decode {
            filter: "LZWDecode",
            message: "EarlyChange must be 0 or 1",
        }),
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
            StreamFilter::Flate => {
                decode_flate(&decoded, options.max_decoded_len, options.initial_capacity)?
            }
            StreamFilter::AsciiHex => decode_ascii_hex(&decoded, options.max_decoded_len)?,
            StreamFilter::Ascii85 => decode_ascii85(&decoded, options.max_decoded_len)?,
            StreamFilter::Lzw { early_change } => {
                decode_lzw(&decoded, *early_change, options.max_decoded_len)?
            }
            StreamFilter::RunLength => decode_run_length(&decoded, options.max_decoded_len)?,
        };
    }
    Ok(decoded)
}

fn decode_flate(
    raw: &[u8],
    max_decoded_len: usize,
    initial_capacity: Option<usize>,
) -> ObjectResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(raw);
    let mut decoded = Vec::with_capacity(initial_capacity.unwrap_or(0).min(max_decoded_len));
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

fn decode_run_length(raw: &[u8], max_decoded_len: usize) -> ObjectResult<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut index = 0_usize;
    while index < raw.len() {
        let length = raw[index];
        index += 1;
        match length {
            0..=127 => {
                let count = usize::from(length) + 1;
                let Some(end) = index.checked_add(count) else {
                    return Err(ObjectError::Decode {
                        filter: StreamFilter::RunLength.label(),
                        message: "RunLength literal run overflow",
                    });
                };
                if end > raw.len() {
                    return Err(ObjectError::Decode {
                        filter: StreamFilter::RunLength.label(),
                        message: "truncated RunLength literal run",
                    });
                }
                extend_limited(&mut decoded, &raw[index..end], max_decoded_len)?;
                index = end;
            }
            128 => break,
            129..=255 => {
                let Some(byte) = raw.get(index).copied() else {
                    return Err(ObjectError::Decode {
                        filter: StreamFilter::RunLength.label(),
                        message: "truncated RunLength repeat run",
                    });
                };
                index += 1;
                let count = usize::from(257_u16 - u16::from(length));
                ensure_decode_limit(decoded.len().saturating_add(count), max_decoded_len)?;
                decoded.extend(std::iter::repeat(byte).take(count));
            }
        }
    }
    Ok(decoded)
}

fn decode_lzw(raw: &[u8], early_change: usize, max_decoded_len: usize) -> ObjectResult<Vec<u8>> {
    let mut reader = LzwBitReader::new(raw);
    let mut table = lzw_initial_table();
    let mut code_width = 9_u8;
    let mut next_code = 258_usize;
    let mut previous: Option<Vec<u8>> = None;
    let mut decoded = Vec::new();

    while let Some(code) = reader.read_code(code_width)? {
        match code {
            256 => {
                table = lzw_initial_table();
                code_width = 9;
                next_code = 258;
                previous = None;
            }
            257 => return Ok(decoded),
            _ => {
                let entry = lzw_entry(&table, code, next_code, previous.as_deref())?;
                extend_limited(&mut decoded, &entry, max_decoded_len)?;
                if let Some(previous_entry) = previous.as_deref() {
                    if next_code < table.len() {
                        let mut new_entry = previous_entry.to_vec();
                        new_entry.push(entry[0]);
                        table[next_code] = Some(new_entry);
                        next_code += 1;
                        if next_code.saturating_add(early_change) >= (1_usize << code_width)
                            && code_width < 12
                        {
                            code_width += 1;
                        }
                    }
                }
                previous = Some(entry);
            }
        }
    }

    Ok(decoded)
}

fn lzw_initial_table() -> Vec<Option<Vec<u8>>> {
    let mut table = Vec::with_capacity(4096);
    table.extend((0_u16..=255).map(|byte| Some(vec![byte as u8])));
    table.resize_with(4096, || None);
    table
}

fn lzw_entry(
    table: &[Option<Vec<u8>>],
    code: u16,
    next_code: usize,
    previous: Option<&[u8]>,
) -> ObjectResult<Vec<u8>> {
    let code = usize::from(code);
    if code < next_code {
        if let Some(entry) = table.get(code).and_then(|entry| entry.as_ref()) {
            return Ok(entry.clone());
        }
    }
    if code == next_code {
        if let Some(previous) = previous {
            let mut entry = previous.to_vec();
            entry.push(previous[0]);
            return Ok(entry);
        }
    }
    Err(ObjectError::Decode {
        filter: StreamFilter::Lzw { early_change: 1 }.label(),
        message: "invalid LZW code",
    })
}

struct LzwBitReader<'a> {
    raw: &'a [u8],
    index: usize,
    buffer: u32,
    bit_count: u8,
}

impl<'a> LzwBitReader<'a> {
    const fn new(raw: &'a [u8]) -> Self {
        Self {
            raw,
            index: 0,
            buffer: 0,
            bit_count: 0,
        }
    }

    fn read_code(&mut self, width: u8) -> ObjectResult<Option<u16>> {
        while self.bit_count < width {
            let Some(byte) = self.raw.get(self.index).copied() else {
                if self.bit_count == 0 {
                    return Ok(None);
                }
                return Err(ObjectError::Decode {
                    filter: StreamFilter::Lzw { early_change: 1 }.label(),
                    message: "truncated LZW code",
                });
            };
            self.index += 1;
            self.buffer = (self.buffer << 8) | u32::from(byte);
            self.bit_count += 8;
        }
        let shift = self.bit_count - width;
        let mask = (1_u32 << width) - 1;
        let code = ((self.buffer >> shift) & mask) as u16;
        self.bit_count = shift;
        self.buffer &= if self.bit_count == 0 {
            0
        } else {
            (1_u32 << self.bit_count) - 1
        };
        Ok(Some(code))
    }
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
    /// Document declares encryption metadata and cannot be interpreted as plain PDF.
    Encrypted,
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
    /// Xref stream references an object stream that is missing or invalid.
    MissingObjectStream {
        /// Missing object stream ID.
        id: ObjectId,
    },
    /// Object stream header does not match the xref stream compressed entry.
    ObjectStreamMismatch {
        /// Object ID expected by the xref stream.
        expected: ObjectId,
        /// Object ID found at the object-stream index, if present.
        actual: Option<ObjectId>,
        /// Object stream ID.
        object_stream: ObjectId,
        /// Zero-based index inside the object stream.
        index: usize,
    },
    /// Required indirect object is missing.
    MissingObject {
        /// Missing object ID.
        id: ObjectId,
    },
    /// Required catalog or page tree field is missing or malformed.
    MissingPageTreeField {
        /// Field name.
        field: &'static str,
    },
    /// Page tree traversal encountered a cycle.
    PageTreeCycle {
        /// Repeated object ID.
        id: ObjectId,
    },
    /// Page box is missing, malformed, or not positive-sized.
    InvalidPageBox,
    /// Page rotation is malformed or not a multiple of 90 degrees.
    InvalidPageRotation,
    /// Page user-unit value is malformed or outside the supported range.
    InvalidUserUnit,
    /// Incremental update `/Prev` chain points back to an already parsed xref.
    IncrementalUpdateCycle {
        /// Repeated xref byte offset.
        offset: ByteOffset,
    },
    /// Incremental update `/Prev` chain exceeds the configured depth limit.
    IncrementalUpdateDepthExceeded {
        /// Configured maximum revision count.
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
            Self::Encrypted => None,
            Self::Malformed { offset, .. } => Some(*offset),
            Self::UnsupportedStreamLength
            | Self::UnsupportedFilter { .. }
            | Self::Decode { .. }
            | Self::StreamLimitExceeded { .. }
            | Self::MissingObjectStream { .. }
            | Self::ObjectStreamMismatch { .. }
            | Self::MissingObject { .. }
            | Self::MissingPageTreeField { .. }
            | Self::PageTreeCycle { .. }
            | Self::InvalidPageBox
            | Self::InvalidPageRotation
            | Self::InvalidUserUnit
            | Self::IncrementalUpdateDepthExceeded { .. } => None,
            Self::IncrementalUpdateCycle { offset } => Some(*offset),
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
            Self::Encrypted => f.write_str("PDF is encrypted"),
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
            Self::MissingObjectStream { id } => write!(
                f,
                "missing object stream {} {}",
                id.number.get(),
                id.generation.get()
            ),
            Self::ObjectStreamMismatch {
                expected,
                actual,
                object_stream,
                index,
            } => {
                write!(
                    f,
                    "object stream {} {} index {index} mismatch: expected {} {}",
                    object_stream.number.get(),
                    object_stream.generation.get(),
                    expected.number.get(),
                    expected.generation.get()
                )?;
                if let Some(actual) = actual {
                    write!(
                        f,
                        ", got {} {}",
                        actual.number.get(),
                        actual.generation.get()
                    )?;
                }
                Ok(())
            }
            Self::MissingObject { id } => write!(
                f,
                "missing object {} {}",
                id.number.get(),
                id.generation.get()
            ),
            Self::MissingPageTreeField { field } => {
                write!(f, "missing or malformed page tree field {field}")
            }
            Self::PageTreeCycle { id } => write!(
                f,
                "page tree cycle at object {} {}",
                id.number.get(),
                id.generation.get()
            ),
            Self::InvalidPageBox => f.write_str("invalid page box"),
            Self::InvalidPageRotation => f.write_str("invalid page rotation"),
            Self::InvalidUserUnit => f.write_str("invalid page user unit"),
            Self::IncrementalUpdateCycle { offset } => {
                write!(f, "incremental update cycle at {offset}")
            }
            Self::IncrementalUpdateDepthExceeded { limit } => {
                write!(
                    f,
                    "incremental update chain exceeds limit of {limit} revisions"
                )
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

    use aes::cipher::{BlockEncryptMut, KeyIvInit};
    use std::io::Write;

    use aes::cipher::block_padding::{NoPadding, Pkcs7};
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
                .decode_with_options(StreamDecodeOptions {
                    max_decoded_len: 5,
                    initial_capacity: None,
                })
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
    fn stream_decode_should_decode_flate_with_initial_capacity() {
        let compressed = zlib_compress(b"BT /F1 12 Tf ET");
        let object = build_stream_object(
            b"<< /Length ",
            compressed.len(),
            b" /Filter /FlateDecode >>",
            &compressed,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream
                .decode_with_options(StreamDecodeOptions {
                    max_decoded_len: 64,
                    initial_capacity: Some(16),
                })
                .expect("flate decoded stream")
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
    fn stream_decode_should_decode_run_length_filter() {
        let encoded = [2, b'a', b'b', b'c', 255, b'x', 128];
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /RunLengthDecode >>",
            &encoded,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("RunLength decoded stream")
        });

        assert_eq!(decoded, b"abcxx");
    }

    #[test]
    fn stream_decode_should_decode_lzw_filter() {
        let encoded = pack_lzw_codes(&[256, 72, 101, 108, 108, 111, 257], 9);
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /LZWDecode >>",
            &encoded,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("LZW decoded stream")
        });

        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn stream_decode_should_decode_lzw_dictionary_codes() {
        let encoded = pack_lzw_codes(&[256, 65, 66, 258, 65, 257], 9);
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /LZWDecode >>",
            &encoded,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("LZW decoded stream")
        });

        assert_eq!(decoded, b"ABABA");
    }

    #[test]
    fn stream_decode_should_decode_lzw_after_width_growth() {
        let expected: Vec<u8> = (0_u16..260).map(|value| (value % 251) as u8).collect();
        let encoded = pack_lzw_literal_bytes(&expected, 1);
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /LZWDecode >>",
            &encoded,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("LZW decoded stream")
        });

        assert_eq!(decoded, expected);
    }

    #[test]
    fn stream_decode_should_parse_lzw_early_change_zero() {
        let encoded = pack_lzw_codes(&[256, 72, 105, 257], 9);
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /LZWDecode /DecodeParms << /EarlyChange 0 >> >>",
            &encoded,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("LZW decoded stream")
        });

        assert_eq!(decoded, b"Hi");
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
    fn stream_decode_should_parse_decode_parms_arrays() {
        let encoded = pack_lzw_codes(&[256, 72, 105, 257], 9);
        let mut hex = Vec::with_capacity(encoded.len() * 2);
        for byte in &encoded {
            hex.extend_from_slice(format!("{byte:02X}").as_bytes());
        }
        hex.push(b'>');
        let object = build_stream_object(
            b"<< /Length ",
            hex.len(),
            b" /Filter [ /ASCIIHexDecode /LZWDecode ] /DecodeParms [ null << /EarlyChange 0 >> ] >>",
            &hex,
        );
        let decoded = with_test_stream(&object, |stream| {
            stream.decode().expect("filter array decoded stream")
        });

        assert_eq!(decoded, b"Hi");
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
                .decode_with_options(StreamDecodeOptions {
                    max_decoded_len: 4,
                    initial_capacity: None,
                })
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
    fn stream_decode_should_reject_malformed_run_length() {
        let error = with_test_stream(
            b"<< /Length 2 /Filter /RunLengthDecode >>\nstream\n\x02x\nendstream",
            |stream| stream.decode().expect_err("malformed RunLength"),
        );

        assert!(matches!(error, ObjectError::Decode { .. }));
    }

    #[test]
    fn stream_decode_should_reject_malformed_lzw_code() {
        let encoded = pack_lzw_codes(&[256, 300, 257], 9);
        let object = build_stream_object(
            b"<< /Length ",
            encoded.len(),
            b" /Filter /LZWDecode >>",
            &encoded,
        );
        let error = with_test_stream(&object, |stream| {
            stream.decode().expect_err("malformed LZW")
        });

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
            PdfPrimitive::Reference(ferrugo_syntax::PdfReference::new(1, 0))
        );
    }

    #[test]
    fn load_linearized_first_page_document_should_load_bounded_first_page_objects() {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-first-page.pdf");
        let full = load_classic_document(PdfBytes::new(bytes)).expect("full document");
        let first_page = load_linearized_first_page_document(PdfBytes::new(bytes))
            .expect("linearized first page document");
        let page_tree = first_page
            .linearized_first_page_tree()
            .expect("first page tree")
            .expect("linearization metadata");

        assert!(full.load_metrics.is_linearized);
        assert!(!full.load_metrics.first_page_only);
        assert!(first_page.load_metrics.is_linearized);
        assert!(first_page.load_metrics.first_page_only);
        assert!(first_page.objects.len() < full.objects.len());
        assert!(
            first_page.load_metrics.loaded_object_bytes < full.load_metrics.loaded_object_bytes
        );
        assert_eq!(page_tree.page_count(), 1);
        assert_eq!(
            page_tree.first_page_size(),
            Some(PageSize {
                width: 160.0,
                height: 90.0
            })
        );
    }

    #[test]
    fn load_linearized_first_page_document_should_reject_invalid_hints_without_breaking_full_load()
    {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-malformed-hints.pdf");

        load_linearized_first_page_document(PdfBytes::new(bytes)).expect_err("invalid hint");
        let full = load_classic_document(PdfBytes::new(bytes)).expect("full fallback document");

        assert!(full.load_metrics.is_linearized);
        assert_eq!(full.page_tree().expect("page tree").page_count(), 2);
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
    fn load_classic_document_should_recover_small_xref_offset_drift() {
        let pdf = build_classic_pdf_with_first_offset_delta(1);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("recovered document");

        assert_eq!(document.objects.len(), 3);
    }

    #[test]
    fn load_classic_document_should_use_latest_incremental_object_revision() {
        let pdf = build_incremental_classic_pdf();

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("incremental document");
        let page_tree = document.page_tree().expect("page tree");

        assert_eq!(document.objects.len(), 3);
        assert_eq!(document.xref.entries().len(), 3);
        assert_eq!(
            page_tree.first_page_size(),
            Some(PageSize {
                width: 612.0,
                height: 792.0
            })
        );
    }

    #[test]
    fn load_classic_document_should_not_resurrect_deleted_incremental_object() {
        let pdf = build_incremental_deleted_object_pdf();
        let deleted_id = ObjectId::new(
            ObjectNumber::new(4).expect("valid object number"),
            GenerationNumber::new(0),
        );

        let document =
            load_classic_document(PdfBytes::new(&pdf)).expect("incremental deletion document");

        assert!(document.objects.get(deleted_id).is_none());
        assert!(document
            .xref
            .entries()
            .iter()
            .all(|entry| entry.id.number != deleted_id.number));
    }

    #[test]
    fn load_classic_document_should_reject_incremental_update_cycle() {
        let pdf = build_cyclic_incremental_xref_pdf();

        let error = load_classic_document(PdfBytes::new(&pdf)).expect_err("cycle");

        assert!(matches!(error, ObjectError::IncrementalUpdateCycle { .. }));
    }

    #[test]
    fn load_classic_document_should_reject_incremental_update_depth_overflow() {
        let pdf = build_incremental_depth_overflow_pdf();

        let error = load_classic_document(PdfBytes::new(&pdf)).expect_err("depth overflow");

        assert_eq!(
            error,
            ObjectError::IncrementalUpdateDepthExceeded {
                limit: DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT
            }
        );
    }

    #[test]
    fn load_classic_document_should_include_hybrid_xref_stream_entries() {
        let pdf = build_hybrid_reference_pdf();

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("hybrid document");

        assert_eq!(document.objects.len(), 4);
        assert_eq!(document.xref.entries().len(), 4);
        assert!(document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(4).expect("object number"),
                GenerationNumber::new(0),
            ))
            .is_some());
    }

    #[test]
    fn load_classic_document_should_reject_encrypted_trailer() {
        let pdf = build_encrypted_trailer_pdf();

        let error = load_classic_document(PdfBytes::new(&pdf)).expect_err("encrypted trailer");

        assert_eq!(error, ObjectError::Encrypted);
    }

    #[test]
    fn load_classic_document_should_open_empty_password_rc4_permissions_pdf() {
        let expected = b"0.9 0 0 rg 10 10 40 40 re f";
        let pdf = build_permissions_encrypted_pdf(TestEncryption::Rc4R2, expected);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("encrypted document");
        let content = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(4).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("content object");
        let ObjectValue::Stream(stream) = &content.value else {
            panic!("content should be a stream");
        };

        assert_eq!(stream.decode().expect("decrypted stream"), expected);
        assert_eq!(document.page_tree().expect("page tree").page_count(), 1);
    }

    #[test]
    fn load_classic_document_should_open_empty_password_aes128_permissions_pdf() {
        let expected = b"BT /F1 12 Tf 20 40 Td (AES permissions) Tj ET";
        let pdf = build_permissions_encrypted_pdf(TestEncryption::AesV2R4, expected);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("encrypted document");
        let content = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(4).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("content object");
        let ObjectValue::Stream(stream) = &content.value else {
            panic!("content should be a stream");
        };

        assert_eq!(stream.decode().expect("decrypted stream"), expected);
        assert_eq!(document.page_tree().expect("page tree").page_count(), 1);
    }

    #[test]
    fn load_classic_document_should_open_empty_password_aes256_permissions_pdf() {
        let expected = b"BT /F1 12 Tf 20 40 Td (AES256 permissions) Tj ET";
        let pdf = build_permissions_encrypted_pdf(TestEncryption::AesV3R5, expected);

        let document = load_classic_document(PdfBytes::new(&pdf)).expect("encrypted document");
        let content = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(4).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("content object");
        let ObjectValue::Stream(stream) = &content.value else {
            panic!("content should be a stream");
        };

        assert_eq!(stream.decode().expect("decrypted stream"), expected);
        assert_eq!(document.page_tree().expect("page tree").page_count(), 1);
    }

    #[test]
    fn load_classic_document_should_reject_encrypted_catalog() {
        let pdf = build_encrypted_catalog_pdf();

        let error = load_classic_document(PdfBytes::new(&pdf)).expect_err("encrypted catalog");

        assert_eq!(error, ObjectError::Encrypted);
    }

    #[test]
    fn load_classic_document_should_require_startxref() {
        let error =
            load_classic_document(PdfBytes::new(b"%PDF-1.7\n")).expect_err("missing startxref");

        assert_eq!(error.offset(), Some(ByteOffset::new(0)));
    }

    #[test]
    fn load_modern_document_should_load_xref_stream_and_object_stream() {
        let pdf = build_modern_pdf(false);

        let document = load_modern_document(PdfBytes::new(&pdf)).expect("modern document");

        assert_eq!(document.xref.entries().len(), 6);
        assert_eq!(document.objects.len(), 5);
        assert_eq!(document.compressed_object_index.len(), 1);
        assert_eq!(
            document.compressed_object_index[0].id,
            ObjectId::new(
                ObjectNumber::new(3).expect("object number"),
                GenerationNumber::new(0)
            )
        );
        assert!(document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(3).expect("object number"),
                GenerationNumber::new(0)
            ))
            .is_none());
        let page = document
            .get_object(ObjectId::new(
                ObjectNumber::new(3).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("resolve compressed object")
            .expect("compressed page object");
        assert!(matches!(
            page.value,
            ObjectValue::Primitive(PdfPrimitive::Dictionary(_))
        ));
        let repeated_page = document
            .get_object(ObjectId::new(
                ObjectNumber::new(3).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("resolve compressed object again")
            .expect("compressed page object");
        assert_eq!(repeated_page.id, page.id);
        let contents = document
            .get_object(ObjectId::new(
                ObjectNumber::new(4).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("resolve direct object")
            .expect("direct content object");
        let ObjectValue::Stream(stream) = contents.value else {
            panic!("expected content stream");
        };
        assert_eq!(stream.decode().expect("decoded content stream"), b"q");
    }

    #[test]
    fn page_tree_should_resolve_classic_inherited_metadata() {
        let pdf = build_classic_page_tree_pdf(false, false);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let page_tree = document.page_tree().expect("page tree");

        assert_eq!(page_tree.page_count(), 2);
        assert_eq!(
            page_tree.first_page_size(),
            Some(PageSize {
                width: 300.0,
                height: 160.0
            })
        );
        assert_eq!(
            page_tree.pages()[0].resources,
            Some(Reference::new(ObjectId::new(
                ObjectNumber::new(5).expect("object number"),
                GenerationNumber::new(0)
            )))
        );
        assert_eq!(
            page_tree.pages()[1].size(),
            PageSize {
                width: 100.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn page_tree_should_resolve_rotation_and_user_unit_metadata() {
        let objects = vec![
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec(),
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 160] /Rotate -270 >>\nendobj\n".to_vec(),
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /CropBox [0 0 100 50] /UserUnit 2 >>\nendobj\n".to_vec(),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let page_tree = document.page_tree().expect("page tree");
        let page = page_tree.first_page().expect("first page");

        assert_eq!(page.rotation_degrees, 90);
        assert_eq!(page.user_unit, 2.0);
        assert_eq!(
            page.size(),
            PageSize {
                width: 50.0,
                height: 100.0,
            }
        );
    }

    #[test]
    fn page_tree_should_reject_invalid_rotation() {
        let objects = vec![
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec(),
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 160] >>\nendobj\n"
                .to_vec(),
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /Rotate 45 >>\nendobj\n".to_vec(),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let error = document.page_tree().expect_err("invalid rotation");

        assert_eq!(error, ObjectError::InvalidPageRotation);
    }

    #[test]
    fn page_tree_should_reject_invalid_user_unit() {
        let objects = vec![
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec(),
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 160] >>\nendobj\n"
                .to_vec(),
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /UserUnit 0 >>\nendobj\n".to_vec(),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let error = document.page_tree().expect_err("invalid user unit");

        assert_eq!(error, ObjectError::InvalidUserUnit);
    }

    #[test]
    fn page_tree_should_resolve_modern_compressed_page_metadata() {
        let pdf = build_modern_pdf(false);
        let document = load_modern_document(PdfBytes::new(&pdf)).expect("modern document");

        let page_tree = document.page_tree().expect("page tree");

        assert_eq!(page_tree.page_count(), 1);
        assert_eq!(
            page_tree.first_page_size(),
            Some(PageSize {
                width: 300.0,
                height: 160.0
            })
        );
    }

    #[test]
    fn page_tree_should_reject_missing_media_box() {
        let pdf = build_classic_page_tree_pdf(true, false);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let error = document.page_tree().expect_err("missing media box");

        assert_eq!(
            error,
            ObjectError::MissingPageTreeField { field: "MediaBox" }
        );
    }

    #[test]
    fn page_tree_should_reject_cycles() {
        let pdf = build_classic_page_tree_pdf(false, true);
        let document = load_classic_document(PdfBytes::new(&pdf)).expect("classic document");

        let error = document.page_tree().expect_err("cycle");

        assert!(matches!(error, ObjectError::PageTreeCycle { .. }));
    }

    #[test]
    fn load_modern_document_should_reject_bad_object_stream_index() {
        let pdf = build_modern_pdf(true);

        let error = load_modern_document(PdfBytes::new(&pdf)).expect_err("bad object stream index");

        assert!(matches!(error, ObjectError::ObjectStreamMismatch { .. }));
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

    fn build_classic_pdf_with_first_offset_delta(delta: usize) -> Vec<u8> {
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
        let object_1_xref = object_1 + delta;
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1_xref:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_incremental_classic_pdf() -> Vec<u8> {
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
        let first_xref = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{first_xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        let updated_object_3 = append_object(
            &mut pdf,
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
        );
        let update_xref = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n3 1\n{updated_object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R /Prev {first_xref} >>\nstartxref\n{update_xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_hybrid_reference_pdf() -> Vec<u8> {
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
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 160] /Contents 4 0 R >>\nendobj\n",
        );
        let object_4 = append_object(
            &mut pdf,
            b"4 0 obj\n<< /Length 1 >>\nstream\nq\nendstream\nendobj\n",
        );
        let xref_stream_offset = pdf.len();
        let mut xref_data = Vec::new();
        push_xref_entry(&mut xref_data, 1, object_4, 0);
        let compressed_xref = zlib_compress(&xref_data);
        pdf.extend_from_slice(
            format!(
                "5 0 obj\n<< /Type /XRef /Size 5 /W [1 4 2] /Index [4 1] /Length {} /Filter /FlateDecode >>\nstream\n",
                compressed_xref.len()
            )
            .as_bytes(),
        );
        pdf.extend_from_slice(&compressed_xref);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");
        let classic_xref = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R /XRefStm {xref_stream_offset} >>\nstartxref\n{classic_xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_encrypted_trailer_pdf() -> Vec<u8> {
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
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R /Encrypt 4 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_encrypted_catalog_pdf() -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let object_1 = append_object(
            &mut pdf,
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /Encrypt << /Filter /Standard >> >>\nendobj\n",
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
        pdf.extend_from_slice(
            format!(
                "xref\n0 4\n0000000000 65535 f \n{object_1:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    #[derive(Debug, Clone, Copy)]
    enum TestEncryption {
        Rc4R2,
        AesV2R4,
        AesV3R5,
    }

    fn build_permissions_encrypted_pdf(encryption: TestEncryption, content: &[u8]) -> Vec<u8> {
        let file_id = *b"ferrugo-empty-id";
        let encrypted_content = match encryption {
            TestEncryption::Rc4R2 => test_rc4_encrypt(
                &object_crypt_key(
                    &standard_v2_file_key(&test_owner_key_r2(), -4, &file_id),
                    ObjectId::new(
                        ObjectNumber::new(4).expect("object number"),
                        GenerationNumber::new(0),
                    ),
                    CryptAlgorithm::Rc4,
                ),
                content,
            ),
            TestEncryption::AesV2R4 => {
                let owner = test_owner_key_r4(128);
                let file_key =
                    standard_v4_file_key(&owner, -4, &file_id, 128, true).expect("file key");
                test_aes128_object_encrypt(
                    &object_crypt_key(
                        &file_key,
                        ObjectId::new(
                            ObjectNumber::new(4).expect("object number"),
                            GenerationNumber::new(0),
                        ),
                        CryptAlgorithm::AesV2,
                    ),
                    content,
                )
            }
            TestEncryption::AesV3R5 => {
                let file_key = test_aes256_file_key();
                test_aes256_stream_encrypt(&file_key, content)
            }
        };
        let encryption_dictionary = test_encryption_dictionary(encryption, &file_id);
        let mut pdf = bytearray_pdf_header();
        let mut offsets: Vec<usize> = vec![0];
        add_pdf_object(
            &mut pdf,
            &mut offsets,
            1,
            b"<< /Type /Catalog /Pages 2 0 R >>",
        );
        add_pdf_object(
            &mut pdf,
            &mut offsets,
            2,
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        );
        add_pdf_object(
            &mut pdf,
            &mut offsets,
            3,
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 4 0 R >>",
        );
        let mut content_object =
            format!("<< /Length {} >>\nstream\n", encrypted_content.len()).into_bytes();
        content_object.extend_from_slice(&encrypted_content);
        content_object.extend_from_slice(b"\nendstream");
        add_pdf_object(&mut pdf, &mut offsets, 4, &content_object);
        add_pdf_object(&mut pdf, &mut offsets, 5, encryption_dictionary.as_bytes());
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R /Encrypt 5 0 R /ID [<{}> <{}>] >>\nstartxref\n{xref_offset}\n%%EOF\n",
                offsets.len(),
                test_hex(&file_id),
                test_hex(&file_id)
            )
            .as_bytes(),
        );
        pdf
    }

    fn bytearray_pdf_header() -> Vec<u8> {
        b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n".to_vec()
    }

    fn add_pdf_object(pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, number: u32, body: &[u8]) {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"\nendobj\n");
    }

    fn test_encryption_dictionary(encryption: TestEncryption, file_id: &[u8]) -> String {
        match encryption {
            TestEncryption::Rc4R2 => {
                let owner = test_owner_key_r2();
                let file_key = standard_v2_file_key(&owner, -4, file_id);
                let user = test_rc4_encrypt(&file_key, &PASSWORD_PADDING);
                format!(
                    "<< /Filter /Standard /V 1 /R 2 /O <{}> /U <{}> /P -4 >>",
                    test_hex(&owner),
                    test_hex(&user)
                )
            }
            TestEncryption::AesV2R4 => {
                let owner = test_owner_key_r4(128);
                let file_key =
                    standard_v4_file_key(&owner, -4, file_id, 128, true).expect("file key");
                let user = standard_v4_user_key(&file_key, file_id).expect("user key");
                format!(
                    "<< /Filter /Standard /V 4 /R 4 /Length 128 /O <{}> /U <{}> /P -4 /EncryptMetadata true /CF << /StdCF << /CFM /AESV2 /Length 16 /AuthEvent /DocOpen >> >> /StmF /StdCF /StrF /StdCF >>",
                    test_hex(&owner),
                    test_hex(&user)
                )
            }
            TestEncryption::AesV3R5 => {
                let file_key = test_aes256_file_key();
                let (user, ue) = test_aes256_user_entries(&file_key);
                let perms = test_aes256_permissions(&file_key);
                format!(
                    "<< /Filter /Standard /V 5 /R 5 /Length 256 /O <{}> /U <{}> /OE <{}> /UE <{}> /Perms <{}> /P -4 /EncryptMetadata true /CF << /StdCF << /CFM /AESV3 /Length 32 /AuthEvent /DocOpen >> >> /StmF /StdCF /StrF /StdCF >>",
                    test_hex(&[0x33; 48]),
                    test_hex(&user),
                    test_hex(&[0x44; 32]),
                    test_hex(&ue),
                    test_hex(&perms)
                )
            }
        }
    }

    fn test_owner_key_r2() -> Vec<u8> {
        let owner_hash = Md5::digest(PASSWORD_PADDING);
        test_rc4_encrypt(&owner_hash[..5], &PASSWORD_PADDING)
    }

    fn test_owner_key_r4(key_bits: usize) -> Vec<u8> {
        let key_bytes = key_bits / 8;
        let mut digest = Md5::digest(PASSWORD_PADDING).to_vec();
        for _ in 0..50 {
            digest = Md5::digest(&digest[..key_bytes]).to_vec();
        }
        let mut value = test_rc4_encrypt(&digest[..key_bytes], &PASSWORD_PADDING);
        for round in 1_u8..=19 {
            let round_key: Vec<u8> = digest[..key_bytes]
                .iter()
                .map(|byte| byte ^ round)
                .collect();
            value = test_rc4_encrypt(&round_key, &value);
        }
        value
    }

    fn test_rc4_encrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
        rc4_crypt(key, data).expect("test rc4")
    }

    fn test_aes128_object_encrypt(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let iv = [0x12_u8; AES_BLOCK_BYTES];
        let mut output = iv.to_vec();
        output.extend(
            cbc::Encryptor::<aes::Aes128>::new(key.into(), (&iv).into())
                .encrypt_padded_vec_mut::<Pkcs7>(plaintext),
        );
        output
    }

    fn test_aes256_stream_encrypt(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let iv = [0x24_u8; AES_BLOCK_BYTES];
        let mut output = iv.to_vec();
        output.extend(
            cbc::Encryptor::<aes::Aes256>::new(key.into(), (&iv).into())
                .encrypt_padded_vec_mut::<Pkcs7>(plaintext),
        );
        output
    }

    fn test_aes256_file_key() -> Vec<u8> {
        (0..PDF_AES256_KEY_BYTES).map(|value| value as u8).collect()
    }

    fn test_aes256_user_entries(file_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let validation_salt = [1_u8, 2, 3, 4, 5, 6, 7, 8];
        let key_salt = [9_u8, 10, 11, 12, 13, 14, 15, 16];
        let mut user = Sha256::digest(validation_salt).to_vec();
        user.extend_from_slice(&validation_salt);
        user.extend_from_slice(&key_salt);
        let key = Sha256::digest(key_salt);
        let iv = [0_u8; AES_BLOCK_BYTES];
        let ue = cbc::Encryptor::<aes::Aes256>::new((&key[..]).into(), (&iv).into())
            .encrypt_padded_vec_mut::<NoPadding>(file_key);
        (user, ue)
    }

    fn test_aes256_permissions(file_key: &[u8]) -> Vec<u8> {
        let mut perms = Vec::new();
        perms.extend_from_slice(&permission_bytes(-4));
        perms.extend_from_slice(&[0xff; 4]);
        perms.push(b'T');
        perms.extend_from_slice(b"adb");
        perms.extend_from_slice(&[0x55; 4]);
        let iv = [0_u8; AES_BLOCK_BYTES];
        cbc::Encryptor::<aes::Aes256>::new(file_key.into(), (&iv).into())
            .encrypt_padded_vec_mut::<NoPadding>(&perms)
    }

    fn test_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push(HEX[usize::from(byte >> 4)] as char);
            output.push(HEX[usize::from(byte & 0x0f)] as char);
        }
        output
    }

    fn build_cyclic_incremental_xref_pdf() -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let xref_offset = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n0 1\n0000000000 65535 f \ntrailer\n<< /Size 1 /Prev {xref_offset} >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_incremental_deleted_object_pdf() -> Vec<u8> {
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
        let object_4 = append_object(&mut pdf, b"4 0 obj\n<< /Deleted true >>\nendobj\n");
        let first_xref = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n0 5\n0000000000 65535 f \n{object_1:010} 00000 n \n{object_2:010} 00000 n \n{object_3:010} 00000 n \n{object_4:010} 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{first_xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        let second_xref = pdf.len();
        pdf.extend_from_slice(
            format!(
                "xref\n4 1\n0000000000 00001 f \ntrailer\n<< /Size 5 /Root 1 0 R /Prev {first_xref} >>\nstartxref\n{second_xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_incremental_depth_overflow_pdf() -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let mut previous = None;
        for _ in 0..=DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT {
            let xref_offset = pdf.len();
            pdf.extend_from_slice(b"xref\n0 1\n0000000000 65535 f \ntrailer\n<< /Size 1");
            if let Some(previous) = previous {
                pdf.extend_from_slice(format!(" /Prev {previous}").as_bytes());
            }
            pdf.extend_from_slice(format!(" >>\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes());
            previous = Some(xref_offset);
        }
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

    fn pack_lzw_codes(codes: &[u16], width: u8) -> Vec<u8> {
        let mut packed = Vec::new();
        let mut buffer = 0_u32;
        let mut bit_count = 0_u8;
        for code in codes {
            pack_lzw_code(&mut packed, &mut buffer, &mut bit_count, *code, width);
        }
        if bit_count > 0 {
            packed.push((buffer << (8 - bit_count)) as u8);
        }
        packed
    }

    fn pack_lzw_literal_bytes(bytes: &[u8], early_change: usize) -> Vec<u8> {
        let mut packed = Vec::new();
        let mut buffer = 0_u32;
        let mut bit_count = 0_u8;
        let mut width = 9_u8;
        let mut next_code = 258_usize;
        pack_lzw_code(&mut packed, &mut buffer, &mut bit_count, 256, width);
        for (index, byte) in bytes.iter().enumerate() {
            pack_lzw_code(
                &mut packed,
                &mut buffer,
                &mut bit_count,
                u16::from(*byte),
                width,
            );
            if index > 0 && next_code < 4096 {
                next_code += 1;
                if next_code.saturating_add(early_change) >= (1_usize << width) && width < 12 {
                    width += 1;
                }
            }
        }
        pack_lzw_code(&mut packed, &mut buffer, &mut bit_count, 257, width);
        if bit_count > 0 {
            packed.push((buffer << (8 - bit_count)) as u8);
        }
        packed
    }

    fn pack_lzw_code(
        packed: &mut Vec<u8>,
        buffer: &mut u32,
        bit_count: &mut u8,
        code: u16,
        width: u8,
    ) {
        *buffer = (*buffer << width) | u32::from(code);
        *bit_count += width;
        while *bit_count >= 8 {
            let shift = *bit_count - 8;
            packed.push(((*buffer >> shift) & 0xff) as u8);
            *bit_count = shift;
            *buffer &= if *bit_count == 0 {
                0
            } else {
                (1_u32 << *bit_count) - 1
            };
        }
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

    fn build_classic_page_tree_pdf(missing_media_box: bool, cycle: bool) -> Vec<u8> {
        let page_tree_dictionary = if missing_media_box {
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 /Resources 5 0 R >>".to_vec()
        } else if cycle {
            b"<< /Type /Pages /Kids [2 0 R] /Count 1 /MediaBox [0 0 300 160] >>".to_vec()
        } else {
            b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 /MediaBox [0 0 300 160] /Resources 5 0 R >>".to_vec()
        };
        let objects = vec![
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec(),
            indirect_object_bytes(2, &page_tree_dictionary),
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n".to_vec(),
            b"4 0 obj\n<< /Type /Page /Parent 2 0 R /CropBox [10 20 110 120] >>\nendobj\n".to_vec(),
            b"5 0 obj\n<< /Font << >> >>\nendobj\n".to_vec(),
        ];
        build_classic_pdf_from_objects(&objects)
    }

    fn indirect_object_bytes(number: u32, dictionary: &[u8]) -> Vec<u8> {
        let mut object = format!("{number} 0 obj\n").into_bytes();
        object.extend_from_slice(dictionary);
        object.extend_from_slice(b"\nendobj\n");
        object
    }

    fn build_classic_pdf_from_objects(objects: &[Vec<u8>]) -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::with_capacity(objects.len());
        for object in objects {
            offsets.push(pdf.len());
            pdf.extend_from_slice(object);
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
        );
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }

    fn build_modern_pdf(use_bad_compressed_index: bool) -> Vec<u8> {
        let mut pdf = b"%PDF-1.7\n".to_vec();
        let object_1 = append_object(
            &mut pdf,
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
        );
        let object_2 = append_object(
            &mut pdf,
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 160] >>\nendobj\n",
        );
        let object_4 = append_object(
            &mut pdf,
            b"4 0 obj\n<< /Length 1 >>\nstream\nq\nendstream\nendobj\n",
        );
        let object_stream_payload = b"3 0 << /Type /Page /Parent 2 0 R /Contents 4 0 R >>";
        let compressed_object_stream = zlib_compress(object_stream_payload);
        let object_5 = pdf.len();
        pdf.extend_from_slice(
            format!(
                "5 0 obj\n<< /Type /ObjStm /N 1 /First 4 /Length {} /Filter /FlateDecode >>\nstream\n",
                compressed_object_stream.len()
            )
            .as_bytes(),
        );
        pdf.extend_from_slice(&compressed_object_stream);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        let xref_offset = pdf.len();
        let compressed_index = if use_bad_compressed_index { 1 } else { 0 };
        let mut xref_data = Vec::new();
        push_xref_entry(&mut xref_data, 0, 0, 65_535);
        push_xref_entry(&mut xref_data, 1, object_1, 0);
        push_xref_entry(&mut xref_data, 1, object_2, 0);
        push_xref_entry(&mut xref_data, 2, 5, compressed_index);
        push_xref_entry(&mut xref_data, 1, object_4, 0);
        push_xref_entry(&mut xref_data, 1, object_5, 0);
        push_xref_entry(&mut xref_data, 1, xref_offset, 0);
        let compressed_xref = zlib_compress(&xref_data);
        pdf.extend_from_slice(
            format!(
                "6 0 obj\n<< /Type /XRef /Size 7 /Root 1 0 R /W [1 4 2] /Index [0 7] /Length {} /Filter /FlateDecode >>\nstream\n",
                compressed_xref.len()
            )
            .as_bytes(),
        );
        pdf.extend_from_slice(&compressed_xref);
        pdf.extend_from_slice(
            format!("\nendstream\nendobj\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes(),
        );
        pdf
    }

    fn push_xref_entry(output: &mut Vec<u8>, entry_type: u8, field_2: usize, field_3: usize) {
        output.push(entry_type);
        push_big_endian(output, field_2, 4);
        push_big_endian(output, field_3, 2);
    }

    fn push_big_endian(output: &mut Vec<u8>, value: usize, width: usize) {
        for shift in (0..width).rev() {
            output.push(((value >> (shift * 8)) & 0xff) as u8);
        }
    }

    fn zlib_compress(input: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(input).expect("write compressed input");
        encoder.finish().expect("finish compressed input")
    }
}
