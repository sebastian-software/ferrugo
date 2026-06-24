//! Safe PDF object model for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fmt;
use std::io::Read;

use flate2::read::ZlibDecoder;
use pdfrust_syntax::{
    parse_primitive, parse_primitive_prefix, ByteCursor, ByteOffset, PdfBytes, PdfName, PdfNumber,
    PdfPrimitive, SyntaxError,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "object";

/// Maximum bytes scanned around a declared xref object offset for local repair.
pub const DEFAULT_XREF_OFFSET_RECOVERY_SCAN_BYTES: usize = 64;

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

/// Default maximum number of classic trailer revisions followed through `/Prev`.
pub const DEFAULT_INCREMENTAL_UPDATE_DEPTH_LIMIT: usize = 16;

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
        for object_stream in &self.object_streams {
            if let Some(object) = object_stream.parse_object(id)? {
                return Ok(Some(object));
            }
        }
        Ok(None)
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
    /// Resource dictionary reference inherited or declared on the page.
    pub resources: Option<Reference>,
}

impl PageMetadata {
    /// Returns the visible page size using `CropBox` when present, otherwise
    /// `MediaBox`.
    #[must_use]
    pub fn size(&self) -> PageSize {
        self.crop_box.unwrap_or(self.media_box).size()
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
    let (mut xref, trailer) = parse_classic_xref_chain(bytes, startxref)?;
    extend_classic_xref_with_hybrid_stream(bytes, &mut xref, &trailer)?;
    reject_encrypted_trailer(&trailer)?;
    let mut objects = ObjectTable::new();

    for entry in xref.entries() {
        let object = parse_object_with_xref_recovery(bytes, *entry)?;
        objects.insert(object)?;
    }

    let document = ClassicDocument {
        xref,
        trailer,
        objects,
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
    reject_encrypted_trailer(&trailer)?;
    let mut objects = ObjectTable::new();

    for entry in xref.entries() {
        if let XrefStreamEntry::InUse { id, offset } = *entry {
            let object = parse_object_with_xref_recovery(bytes, ClassicXrefEntry { id, offset })?;
            objects.insert(object)?;
        }
    }

    let object_streams = load_referenced_object_streams(&objects, xref.entries())?;
    validate_compressed_entries(&object_streams, xref.entries())?;

    let document = ModernDocument {
        xref,
        trailer,
        objects,
        object_streams,
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

#[derive(Debug, Clone, Copy, Default)]
struct InheritedPageState {
    media_box: Option<PageBox>,
    crop_box: Option<PageBox>,
    resources: Option<Reference>,
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

fn reject_encrypted_trailer(trailer: &Trailer<'_>) -> ObjectResult<()> {
    if dictionary_value(trailer.entries(), b"Encrypt").is_some() {
        return Err(ObjectError::Encrypted);
    }
    Ok(())
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

fn parse_object_with_xref_recovery<'a>(
    bytes: &'a [u8],
    entry: ClassicXrefEntry,
) -> ObjectResult<IndirectObject<'a>> {
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
) -> Option<ObjectResult<IndirectObject<'a>>> {
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
    for table in &tables {
        for entry in table.entries() {
            if !entries
                .iter()
                .any(|existing: &ClassicXrefEntry| existing.id == entry.id)
            {
                entries.push(*entry);
            }
        }
    }
    let trailer = trailers.remove(0);
    Ok((ClassicXrefTable { startxref, entries }, trailer))
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
    fn parse_object(&self, id: ObjectId) -> ObjectResult<Option<IndirectObject<'_>>> {
        let Some(object) = self.objects.iter().find(|object| object.id == id) else {
            return Ok(None);
        };
        let value = parse_primitive(PdfBytes::new(
            &self.decoded[object.value_start..object.value_end],
        ))?;
        Ok(Some(IndirectObject {
            id,
            value: ObjectValue::Primitive(value),
        }))
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
