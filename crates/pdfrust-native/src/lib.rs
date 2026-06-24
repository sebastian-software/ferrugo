//! Rust-native backend adapter for the thumbnail facade.

#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::fs;
use std::thread;

use pdfrust_content::{tokenize_content, ContentToken};
use pdfrust_object::{
    load_classic_document, load_modern_document, ClassicDocument, GenerationNumber, ObjectError,
    ObjectId, ObjectNumber, ObjectValue, PageBox, PageMetadata as ObjectPageMetadata, PageTree,
    Reference,
};
use pdfrust_render::{
    build_form_display_list_with_graphics_resources, build_image_display_list,
    build_path_display_list_with_graphics_resources, build_text_display_list,
    decode_tiling_pattern, rasterize_display_list_into, rasterize_images, rasterize_paths_into,
    rasterize_text, DisplayItem, DisplayList, DisplayListOptions, ExtGraphicsStateResources,
    FontResources, FormResources, GraphicsError, GraphicsErrorKind, ImageResources, PageGeometry,
    PageRotation, PageTransform, PageTransformOptions, PathBounds, PathRasterOptions, RasterError,
    RasterErrorKind, ShadingResources, TilingPatternResources,
};
use pdfrust_syntax::{PdfBytes, PdfName, PdfNumber, PdfPrimitive, PdfReference};
use pdfrust_thumbnail::{
    DocumentMetadata, DocumentMetadataBackend, PageMetadata as ThumbnailPageMetadata, PageSize,
    PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "native-backend";

const BUCKET_GRAPHICS_OPTIONAL_CONTENT: &str = "graphics.optional-content";
const BUCKET_GRAPHICS_PATTERN_SHADING: &str = "graphics.pattern-shading";
const BUCKET_GRAPHICS_STROKE_CLIP: &str = "graphics.stroke-clip";
const BUCKET_GRAPHICS_TRANSPARENCY: &str = "graphics.transparency";
const BUCKET_IMAGE_COLOR_SPACE: &str = "image.color-space";
const BUCKET_IMAGE_FILTER: &str = "image.filter";
const BUCKET_RENDERER_FORM_XOBJECT: &str = "renderer.form-xobject-composition";
const BUCKET_RENDERER_MEMORY_BUDGET: &str = "renderer.memory-budget";
const BUCKET_RENDERER_UNSUPPORTED: &str = "native.unsupported";
const BUCKET_TEXT_CMAP_TOUNICODE: &str = "text.cmap-tounicode";
const BUCKET_TEXT_FONT_PROGRAM: &str = "text.font-program";
const BUCKET_TEXT_GLYPH_OUTLINE: &str = "text.glyph-outline";

/// Rust-native thumbnail backend.
///
/// The backend is intentionally a placeholder until the parser, object model,
/// content interpreter, and rasterizer land in later milestones.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NativeBackend;

impl NativeBackend {
    /// Creates a new Rust-native backend placeholder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the stable backend name.
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        "rust-native"
    }

    /// Returns the current default memory and cache budget snapshot.
    #[must_use]
    pub fn memory_diagnostics(&self) -> NativeMemoryDiagnostics {
        NativeMemoryDiagnostics::default()
    }
}

/// Default Rust-native renderer memory and cache budget diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMemoryDiagnostics {
    /// Maximum pixels accepted in one page raster buffer.
    pub max_page_pixels: usize,
    /// Maximum decoded bytes accepted for one image XObject.
    pub max_image_bytes: usize,
    /// Maximum decoded bytes accepted for one embedded font program.
    pub max_font_program_bytes: usize,
    /// Maximum decoded bytes accepted for one ToUnicode CMap stream.
    pub max_cmap_bytes: usize,
    /// Maximum bytes accepted in one decoded text run.
    pub max_text_run_bytes: usize,
    /// Maximum display items accepted in one display list.
    pub max_display_items: usize,
}

/// Bounded multi-page native render scheduler configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParallelRenderOptions {
    /// Maximum number of page render workers to run at once.
    pub max_workers: usize,
    /// Maximum estimated output pixels allowed across simultaneously rendered
    /// pages.
    pub max_in_flight_pixels: usize,
}

impl Default for ParallelRenderOptions {
    fn default() -> Self {
        Self {
            max_workers: thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
            max_in_flight_pixels: NativeMemoryDiagnostics::default().max_page_pixels,
        }
    }
}

/// Ordered thumbnails rendered by the bounded parallel scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelRenderResult {
    /// Rendered pages in the same order as requested page indices.
    pub pages: Vec<Thumbnail>,
    /// Effective worker count after applying worker and memory budgets.
    pub workers: usize,
}

impl Default for NativeMemoryDiagnostics {
    fn default() -> Self {
        let display = DisplayListOptions::default();
        let page = PageTransformOptions::default();
        Self {
            max_page_pixels: page.max_page_pixels,
            max_image_bytes: display.max_image_bytes,
            max_font_program_bytes: display.max_font_program_bytes,
            max_cmap_bytes: display.max_cmap_bytes,
            max_text_run_bytes: display.max_text_run_bytes,
            max_display_items: display.max_display_items,
        }
    }
}

/// Renders multiple pages with a bounded native worker scheduler.
///
/// Results preserve the requested page order. When a batch contains failures,
/// the error for the earliest requested page in that batch is returned after
/// all already-started workers have joined.
///
/// # Errors
///
/// Returns [`ThumbnailError`] when the input cannot be loaded, the scheduler
/// configuration is invalid, a memory budget prevents even one page from being
/// scheduled, or any requested page fails to render.
pub fn render_pages_parallel(
    source: PdfSource<'_>,
    page_indices: &[u32],
    options: &ThumbnailOptions,
    parallel_options: ParallelRenderOptions,
) -> Result<ParallelRenderResult, ThumbnailError> {
    let source_bytes = load_source(source)?;
    let bytes = source_bytes.as_ref();
    let worker_count = effective_worker_count(options, parallel_options)?;
    let mut pages = Vec::with_capacity(page_indices.len());

    for chunk in page_indices.chunks(worker_count) {
        let batch = thread::scope(|scope| {
            let handles = chunk
                .iter()
                .copied()
                .map(|page_index| {
                    scope.spawn(move || {
                        let mut page_options = *options;
                        page_options.page_index = page_index;
                        render_bytes(bytes, &page_options)
                    })
                })
                .collect::<Vec<_>>();

            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| ThumbnailError::internal("parallel render worker panicked"))?
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        pages.extend(batch);
    }

    Ok(ParallelRenderResult {
        pages,
        workers: worker_count,
    })
}

fn effective_worker_count(
    options: &ThumbnailOptions,
    parallel_options: ParallelRenderOptions,
) -> Result<usize, ThumbnailError> {
    if parallel_options.max_workers == 0 {
        return Err(unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET));
    }
    let pixels_per_page = (options.max_edge as usize)
        .checked_mul(options.max_edge as usize)
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET))?;
    if pixels_per_page == 0 {
        return Err(unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET));
    }
    let memory_limited_workers = parallel_options.max_in_flight_pixels / pixels_per_page;
    if memory_limited_workers == 0 {
        return Err(unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET));
    }
    Ok(parallel_options
        .max_workers
        .min(memory_limited_workers)
        .max(1))
}

impl ThumbnailBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        Self::backend_name(self)
    }

    fn render(
        &self,
        source: PdfSource<'_>,
        options: &ThumbnailOptions,
    ) -> Result<Thumbnail, ThumbnailError> {
        let bytes = load_source(source)?;
        render_bytes(&bytes, options)
    }
}

impl DocumentMetadataBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        Self::backend_name(self)
    }

    fn inspect(&self, source: PdfSource<'_>) -> Result<DocumentMetadata, ThumbnailError> {
        let bytes = load_source(source)?;
        inspect_bytes(&bytes)
    }
}

fn load_source(source: PdfSource<'_>) -> Result<Cow<'_, [u8]>, ThumbnailError> {
    match source {
        PdfSource::Bytes(bytes) => Ok(Cow::Borrowed(bytes)),
        PdfSource::File(path) => fs::read(path)
            .map(Cow::Owned)
            .map_err(|_| ThumbnailError::Malformed),
    }
}

fn inspect_bytes(bytes: &[u8]) -> Result<DocumentMetadata, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    match load_modern_document(input).and_then(|document| document.page_tree()) {
        Ok(page_tree) => metadata_from_page_tree(&page_tree),
        Err(ObjectError::Encrypted) => Err(ThumbnailError::Encrypted),
        Err(_) => load_classic_document(input)
            .and_then(|document| document.page_tree())
            .map_err(map_object_error)
            .and_then(|page_tree| metadata_from_page_tree(&page_tree)),
    }
}

fn render_bytes(bytes: &[u8], options: &ThumbnailOptions) -> Result<Thumbnail, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    let document = load_classic_document(input).map_err(map_object_error)?;
    let page_tree = document.page_tree().map_err(map_object_error)?;
    let page = page_tree
        .pages()
        .get(options.page_index as usize)
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_UNSUPPORTED))?;
    let content = page_content_stream(&document, page)?;
    let optional_content = page_optional_content_properties(&document, page)?;
    let optional_content_state = document_optional_content_state(&document)?;
    let content = filter_optional_content(&content, &optional_content, &optional_content_state)?;
    let xobject_invocations = xobject_invocation_names(&content)?;
    let display_options = DisplayListOptions::default();
    let ext_graphics_states = page_ext_graphics_state_resources(&document, page)?;
    let shadings = page_shading_resources(&document, page)?;
    let patterns = page_tiling_pattern_resources(&document, page)?;
    let display_list = build_path_display_list_with_graphics_resources(
        tokenize_content(PdfBytes::new(&content)),
        &ext_graphics_states,
        &shadings,
        &patterns,
        display_options,
    )
    .map_err(map_graphics_error)?;
    let transform =
        PageTransform::new(page_geometry(*page), options.max_edge).map_err(map_raster_error)?;
    let form_resources = page_form_resources(&document, page, &xobject_invocations)?;
    let form_list = build_form_display_list_with_graphics_resources(
        tokenize_content(PdfBytes::new(&content)),
        &form_resources,
        &ext_graphics_states,
        &shadings,
        &patterns,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)?;
    let image_resources = page_image_resources(&document, page, &xobject_invocations)?;
    let image_list = build_image_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &image_resources,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)?;
    let font_resources = page_font_resources(&document, page)?;
    let text_list = build_text_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &font_resources,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)?;
    let mut raster = transform
        .create_device(options.background)
        .map_err(map_raster_error)?;
    let paint_order = should_scan_content_order(&display_list, &image_list, &text_list)
        .then(|| page_paint_order(&content, &image_resources, &form_resources))
        .transpose()?;
    if let Some(paint_order) = paint_order.filter(|paint_order| {
        should_rasterize_in_content_order(
            paint_order,
            &display_list,
            &form_list,
            &image_list,
            &text_list,
        )
    }) {
        let ordered_list = ordered_display_list(
            &paint_order,
            &display_list,
            &form_list,
            &image_list,
            &text_list,
        );
        rasterize_display_list_into(
            &ordered_list,
            &mut raster,
            transform,
            PathRasterOptions::default(),
        )
        .map_err(map_raster_error)?;
    } else {
        rasterize_paths_into(
            &display_list,
            &mut raster,
            transform,
            PathRasterOptions::default(),
        )
        .map_err(map_raster_error)?;
        rasterize_paths_into(
            &form_list,
            &mut raster,
            transform,
            PathRasterOptions::default(),
        )
        .map_err(map_raster_error)?;
        rasterize_images(&image_list, &mut raster, transform).map_err(map_raster_error)?;
        rasterize_text(&text_list, &mut raster, transform).map_err(map_raster_error)?;
    }
    let (annotation_forms, annotation_content) =
        page_annotation_appearance_resources(&document, page)?;
    if !annotation_content.is_empty() {
        let annotation_list = build_form_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(&annotation_content)),
            &annotation_forms,
            &ext_graphics_states,
            &shadings,
            &patterns,
            DisplayListOptions::default(),
        )
        .map_err(map_graphics_error)?;
        rasterize_paths_into(
            &annotation_list,
            &mut raster,
            transform,
            PathRasterOptions::default(),
        )
        .map_err(map_raster_error)?;
    }
    let dimensions = raster.dimensions();
    Thumbnail::rgba(dimensions.width, dimensions.height, raster.into_pixels())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PagePaintKind {
    PagePathLike,
    FormPathLike,
    Image,
    Text,
}

fn should_scan_content_order(
    page_paths: &DisplayList,
    images: &DisplayList,
    text: &DisplayList,
) -> bool {
    let categories = [
        has_visible_path_like_item(page_paths),
        !images.is_empty(),
        !text.is_empty(),
    ];
    categories
        .into_iter()
        .filter(|has_items| *has_items)
        .count()
        > 1
}

fn should_rasterize_in_content_order(
    paint_order: &[PagePaintKind],
    page_paths: &DisplayList,
    form_paths: &DisplayList,
    images: &DisplayList,
    text: &DisplayList,
) -> bool {
    let has_form_invocations = paint_order.contains(&PagePaintKind::FormPathLike);
    let categories = [
        has_visible_path_like_item(page_paths),
        has_form_invocations && has_visible_path_like_item(form_paths),
        !images.is_empty(),
        !text.is_empty(),
    ];
    categories
        .into_iter()
        .filter(|has_items| *has_items)
        .count()
        > 1
}

fn has_visible_path_like_item(display_list: &DisplayList) -> bool {
    display_list.items().iter().any(|item| {
        matches!(
            item,
            DisplayItem::Path(_) | DisplayItem::TransparencyGroup(_) | DisplayItem::Shading(_)
        )
    })
}

fn page_paint_order(
    content: &[u8],
    image_resources: &ImageResources,
    form_resources: &FormResources,
) -> Result<Vec<PagePaintKind>, ThumbnailError> {
    let mut paint_order = Vec::new();
    let mut operands = Vec::new();
    for token in spanned_content_tokens(content)? {
        match token.kind {
            SpannedContentTokenKind::Operand(value) => operands.push(value),
            SpannedContentTokenKind::Operator(name) => {
                match name.as_slice() {
                    b"W" | b"W*" | b"S" | b"s" | b"f" | b"F" | b"f*" | b"B" | b"B*" | b"b"
                    | b"b*" | b"sh" => paint_order.push(PagePaintKind::PagePathLike),
                    b"Do" => {
                        if let [PdfPrimitive::Name(resource)] = operands.as_slice() {
                            let name = PdfName::new(resource.as_bytes());
                            if image_resources.get(name).is_some() {
                                paint_order.push(PagePaintKind::Image);
                            } else if form_resources.get(name).is_some() {
                                paint_order.push(PagePaintKind::FormPathLike);
                            }
                        }
                    }
                    b"Tj" | b"TJ" => paint_order.push(PagePaintKind::Text),
                    _ => {}
                }
                operands.clear();
            }
            SpannedContentTokenKind::InlineImage => {
                paint_order.push(PagePaintKind::Image);
                operands.clear();
            }
        }
    }
    Ok(paint_order)
}

fn xobject_invocation_names(content: &[u8]) -> Result<Vec<Vec<u8>>, ThumbnailError> {
    let mut names = Vec::new();
    let mut operands = Vec::new();
    for token in spanned_content_tokens(content)? {
        match token.kind {
            SpannedContentTokenKind::Operand(value) => operands.push(value),
            SpannedContentTokenKind::Operator(name) => {
                if name.as_slice() == b"Do" {
                    if let [PdfPrimitive::Name(resource)] = operands.as_slice() {
                        let resource = resource.as_bytes();
                        if !names
                            .iter()
                            .any(|name: &Vec<u8>| name.as_slice() == resource)
                        {
                            names.push(resource.to_vec());
                        }
                    }
                }
                operands.clear();
            }
            SpannedContentTokenKind::InlineImage => {
                operands.clear();
            }
        }
    }
    Ok(names)
}

fn filter_invoked_resources<'a>(
    resources: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    invocations: &[Vec<u8>],
) -> Vec<(PdfName<'a>, PdfPrimitive<'a>)> {
    resources
        .iter()
        .filter(|(name, _)| {
            invocations
                .iter()
                .any(|invocation| invocation.as_slice() == name.as_bytes())
        })
        .map(|(name, value)| (*name, value.clone()))
        .collect()
}

fn ordered_display_list(
    paint_order: &[PagePaintKind],
    page_paths: &DisplayList,
    form_paths: &DisplayList,
    images: &DisplayList,
    text: &DisplayList,
) -> DisplayList {
    let mut items =
        Vec::with_capacity(page_paths.len() + form_paths.len() + images.len() + text.len());
    let mut page_index = 0;
    let mut form_index = 0;
    let mut image_index = 0;
    let mut text_index = 0;
    let form_invocations = paint_order
        .iter()
        .filter(|kind| **kind == PagePaintKind::FormPathLike)
        .count();

    for kind in paint_order {
        match kind {
            PagePaintKind::PagePathLike => {
                append_next_item(&mut items, page_paths.items(), &mut page_index);
            }
            PagePaintKind::FormPathLike if form_invocations == 1 => {
                items.extend(form_paths.items()[form_index..].iter().cloned());
                form_index = form_paths.len();
            }
            PagePaintKind::FormPathLike => {
                append_next_item(&mut items, form_paths.items(), &mut form_index);
            }
            PagePaintKind::Image => {
                append_next_item(&mut items, images.items(), &mut image_index);
            }
            PagePaintKind::Text => {
                append_next_item(&mut items, text.items(), &mut text_index);
            }
        }
    }

    items.extend(page_paths.items()[page_index..].iter().cloned());
    if form_invocations > 0 {
        items.extend(form_paths.items()[form_index..].iter().cloned());
    }
    items.extend(images.items()[image_index..].iter().cloned());
    items.extend(text.items()[text_index..].iter().cloned());

    DisplayList::from_items(items)
}

fn append_next_item(items: &mut Vec<DisplayItem>, source: &[DisplayItem], index: &mut usize) {
    if let Some(item) = source.get(*index) {
        items.push(item.clone());
        *index += 1;
    }
}

fn page_content_stream(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<Vec<u8>, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let contents = dictionary_value(dictionary, b"Contents")
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_UNSUPPORTED))?;
    decode_contents(document, contents)
}

fn page_ext_graphics_state_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<ExtGraphicsStateResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(ExtGraphicsStateResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(ext_graphics_states)) =
        dictionary_value(resource_dictionary, b"ExtGState")
    else {
        return Ok(ExtGraphicsStateResources::empty());
    };
    ExtGraphicsStateResources::from_extgstate_dictionary(ext_graphics_states)
        .map_err(map_graphics_error)
}

fn page_shading_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<ShadingResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(ShadingResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(shadings)) =
        dictionary_value(resource_dictionary, b"Shading")
    else {
        return Ok(ShadingResources::empty());
    };
    ShadingResources::from_shading_dictionary(shadings).map_err(map_graphics_error)
}

fn page_tiling_pattern_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<TilingPatternResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(TilingPatternResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(patterns)) =
        dictionary_value(resource_dictionary, b"Pattern")
    else {
        return Ok(TilingPatternResources::empty());
    };
    let mut decoded = Vec::new();
    for (name, value) in patterns {
        let PdfPrimitive::Reference(reference) = value else {
            return Err(unsupported_feature(BUCKET_GRAPHICS_PATTERN_SHADING));
        };
        let object_number =
            ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
        let reference = Reference::new(ObjectId::new(
            object_number,
            GenerationNumber::new(reference.generation),
        ));
        let object = document
            .objects
            .get(reference.id)
            .ok_or(ThumbnailError::Malformed)?;
        let ObjectValue::Stream(stream) = &object.value else {
            return Err(ThumbnailError::Malformed);
        };
        let content = stream
            .decode()
            .map_err(|_| unsupported_feature(BUCKET_GRAPHICS_PATTERN_SHADING))?;
        decoded.push(
            decode_tiling_pattern(
                name.as_bytes().to_vec(),
                stream.dictionary(),
                &content,
                DisplayListOptions::default(),
            )
            .map_err(map_graphics_error)?,
        );
    }
    Ok(TilingPatternResources::new(decoded))
}

fn page_image_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
    xobject_invocations: &[Vec<u8>],
) -> Result<ImageResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(ImageResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(xobjects)) =
        dictionary_value(resource_dictionary, b"XObject")
    else {
        return Ok(ImageResources::empty());
    };
    let xobjects = filter_invoked_resources(xobjects, xobject_invocations);
    ImageResources::from_xobject_dictionary(
        xobjects.as_slice(),
        document,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)
}

fn page_form_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
    xobject_invocations: &[Vec<u8>],
) -> Result<FormResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(FormResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(xobjects)) =
        dictionary_value(resource_dictionary, b"XObject")
    else {
        return Ok(FormResources::empty());
    };
    let xobjects = filter_invoked_resources(xobjects, xobject_invocations);
    FormResources::from_xobject_dictionary(xobjects.as_slice(), document)
        .map_err(map_graphics_error)
}

fn page_annotation_appearance_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<(FormResources, Vec<u8>), ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(annots) = dictionary_value(dictionary, b"Annots") else {
        return Ok((FormResources::empty(), Vec::new()));
    };
    let annotations = annotation_array(document, annots)?;
    let mut names = Vec::new();
    let mut references = Vec::new();
    let mut rects = Vec::new();

    for annotation in annotations {
        let Some(dictionary) = annotation_dictionary(document, annotation)? else {
            continue;
        };
        let Some(reference) = normal_appearance_reference(dictionary) else {
            continue;
        };
        let Some(rect) = annotation_rect(dictionary) else {
            continue;
        };
        if !document_object_exists(document, reference)? {
            continue;
        }
        let name = format!("Ann{}", names.len()).into_bytes();
        names.push(name);
        references.push(reference);
        rects.push(rect);
    }

    if names.is_empty() {
        return Ok((FormResources::empty(), Vec::new()));
    }

    let xobjects = names
        .iter()
        .zip(references)
        .map(|(name, reference)| {
            (
                PdfName::new(name.as_slice()),
                PdfPrimitive::Reference(reference),
            )
        })
        .collect::<Vec<_>>();
    let resources =
        FormResources::from_xobject_dictionary(&xobjects, document).map_err(map_graphics_error)?;
    let mut content = Vec::new();
    for (name, rect) in names.iter().zip(rects) {
        let Some(form) = resources.get(PdfName::new(name.as_slice())) else {
            continue;
        };
        append_annotation_form_invocation(&mut content, name, rect, form.bbox);
    }
    Ok((resources, content))
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct OptionalContentState {
    base_visible: bool,
    on: Vec<PdfReference>,
    off: Vec<PdfReference>,
}

impl OptionalContentState {
    fn visible(&self, reference: PdfReference) -> bool {
        if self.off.contains(&reference) {
            return false;
        }
        if self.on.contains(&reference) {
            return true;
        }
        self.base_visible
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionalContentProperty {
    name: Vec<u8>,
    policy: OptionalContentPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionalContentPolicy {
    Group(PdfReference),
    Unsupported,
}

fn document_optional_content_state(
    document: &ClassicDocument<'_>,
) -> Result<OptionalContentState, ThumbnailError> {
    let Some(catalog) = document_catalog(document)? else {
        return Ok(OptionalContentState {
            base_visible: true,
            ..OptionalContentState::default()
        });
    };
    let Some(PdfPrimitive::Dictionary(properties)) = dictionary_value(catalog, b"OCProperties")
    else {
        return Ok(OptionalContentState {
            base_visible: true,
            ..OptionalContentState::default()
        });
    };
    let Some(PdfPrimitive::Dictionary(default_config)) = dictionary_value(properties, b"D") else {
        return Ok(OptionalContentState {
            base_visible: true,
            ..OptionalContentState::default()
        });
    };
    if dictionary_value(default_config, b"AS").is_some() {
        return Err(unsupported_feature(BUCKET_GRAPHICS_OPTIONAL_CONTENT));
    }
    let base_visible = match dictionary_value(default_config, b"BaseState") {
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"OFF" => false,
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"ON" => true,
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"Unchanged" => true,
        Some(_) => return Err(ThumbnailError::Malformed),
        None => true,
    };
    Ok(OptionalContentState {
        base_visible,
        on: optional_content_reference_array(default_config, b"ON")?,
        off: optional_content_reference_array(default_config, b"OFF")?,
    })
}

fn document_catalog<'a>(
    document: &'a ClassicDocument<'a>,
) -> Result<Option<&'a [(PdfName<'a>, PdfPrimitive<'a>)]>, ThumbnailError> {
    let Some(PdfPrimitive::Reference(reference)) =
        dictionary_value(document.trailer.entries(), b"Root")
    else {
        return Ok(None);
    };
    let reference = object_reference(*reference)?;
    let object = document
        .objects
        .get(reference.id)
        .ok_or(ThumbnailError::Malformed)?;
    Ok(Some(object_dictionary(&object.value)?))
}

fn optional_content_reference_array(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> Result<Vec<PdfReference>, ThumbnailError> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(Vec::new());
    };
    let PdfPrimitive::Array(items) = value else {
        return Err(ThumbnailError::Malformed);
    };
    let mut references = Vec::new();
    for item in items {
        let PdfPrimitive::Reference(reference) = item else {
            return Err(ThumbnailError::Malformed);
        };
        references.push(*reference);
    }
    Ok(references)
}

fn page_optional_content_properties(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<Vec<OptionalContentProperty>, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(Vec::new());
    };
    let resource_dictionary = resource_dictionary(document, resources)?;
    let Some(PdfPrimitive::Dictionary(properties)) =
        dictionary_value(resource_dictionary, b"Properties")
    else {
        return Ok(Vec::new());
    };
    let mut resolved = Vec::new();
    for (name, value) in properties {
        let policy = optional_content_policy(document, value)?;
        resolved.push(OptionalContentProperty {
            name: name.as_bytes().to_vec(),
            policy,
        });
    }
    Ok(resolved)
}

fn optional_content_policy(
    document: &ClassicDocument<'_>,
    value: &PdfPrimitive<'_>,
) -> Result<OptionalContentPolicy, ThumbnailError> {
    match value {
        PdfPrimitive::Reference(reference) => {
            let reference_id = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference_id.id)
                .ok_or(ThumbnailError::Malformed)?;
            let dictionary = object_dictionary(&object.value)?;
            if dictionary_name_is(dictionary, b"Type", b"OCG") {
                return Ok(OptionalContentPolicy::Group(*reference));
            }
            if dictionary_name_is(dictionary, b"Type", b"OCMD") {
                return Ok(OptionalContentPolicy::Unsupported);
            }
            Err(ThumbnailError::Malformed)
        }
        PdfPrimitive::Dictionary(dictionary)
            if dictionary_name_is(dictionary, b"Type", b"OCMD") =>
        {
            Ok(OptionalContentPolicy::Unsupported)
        }
        PdfPrimitive::Dictionary(dictionary) if dictionary_name_is(dictionary, b"Type", b"OCG") => {
            Err(unsupported_feature(BUCKET_GRAPHICS_OPTIONAL_CONTENT))
        }
        _ => Err(ThumbnailError::Malformed),
    }
}

fn filter_optional_content(
    content: &[u8],
    properties: &[OptionalContentProperty],
    state: &OptionalContentState,
) -> Result<Vec<u8>, ThumbnailError> {
    if properties.is_empty() {
        return Ok(content.to_vec());
    }
    let tokens = spanned_content_tokens(content)?;
    let mut filtered = Vec::with_capacity(content.len());
    let mut operands = Vec::new();
    let mut visibility_stack = Vec::new();

    for token in &tokens {
        match &token.kind {
            SpannedContentTokenKind::Operand(value) => operands.push((value.clone(), token.start)),
            SpannedContentTokenKind::Operator(name) => {
                let operation_start = operands.first().map_or(token.start, |(_, start)| *start);
                match name.as_slice() {
                    b"BDC" => {
                        visibility_stack.push(optional_content_marker_visible(
                            &operands, properties, state,
                        )?);
                    }
                    b"BMC" => visibility_stack.push(true),
                    b"EMC" => {
                        if visibility_stack.pop().is_none() {
                            return Err(ThumbnailError::Malformed);
                        }
                    }
                    _ if visibility_stack.iter().all(|visible| *visible) => {
                        filtered.extend_from_slice(&content[operation_start..token.end]);
                    }
                    _ => {}
                }
                operands.clear();
            }
            SpannedContentTokenKind::InlineImage => {
                if visibility_stack.iter().all(|visible| *visible) {
                    filtered.extend_from_slice(&content[token.start..token.end]);
                }
                operands.clear();
            }
        }
    }

    if !visibility_stack.is_empty() {
        return Err(ThumbnailError::Malformed);
    }
    Ok(filtered)
}

#[derive(Debug, Clone)]
struct SpannedContentToken<'a> {
    start: usize,
    end: usize,
    kind: SpannedContentTokenKind<'a>,
}

#[derive(Debug, Clone)]
enum SpannedContentTokenKind<'a> {
    Operand(PdfPrimitive<'a>),
    Operator(Vec<u8>),
    InlineImage,
}

fn spanned_content_tokens(content: &[u8]) -> Result<Vec<SpannedContentToken<'_>>, ThumbnailError> {
    let mut raw = Vec::new();
    for token in tokenize_content(PdfBytes::new(content)) {
        let token = token.map_err(|_| ThumbnailError::Malformed)?;
        let start = match &token {
            ContentToken::Operand { offset, .. }
            | ContentToken::Operator { offset, .. }
            | ContentToken::InlineImage { offset, .. } => offset.get(),
        };
        raw.push((start, token));
    }
    let starts = raw.iter().map(|(start, _)| *start).collect::<Vec<_>>();
    let mut spanned = Vec::with_capacity(raw.len());
    for (index, (start, token)) in raw.iter().enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(content.len());
        let kind = match token {
            ContentToken::Operand { value, .. } => SpannedContentTokenKind::Operand(value.clone()),
            ContentToken::Operator { name, .. } => {
                SpannedContentTokenKind::Operator(name.as_bytes().to_vec())
            }
            ContentToken::InlineImage { .. } => SpannedContentTokenKind::InlineImage,
        };
        spanned.push(SpannedContentToken {
            start: *start,
            end,
            kind,
        });
    }
    Ok(spanned)
}

fn optional_content_marker_visible(
    operands: &[(PdfPrimitive<'_>, usize)],
    properties: &[OptionalContentProperty],
    state: &OptionalContentState,
) -> Result<bool, ThumbnailError> {
    if operands.len() != 2 {
        return Err(ThumbnailError::Malformed);
    }
    let PdfPrimitive::Name(tag) = operands[0].0 else {
        return Err(ThumbnailError::Malformed);
    };
    if tag.as_bytes() != b"OC" {
        return Ok(true);
    }
    let PdfPrimitive::Name(property_name) = operands[1].0 else {
        return Err(unsupported_feature(BUCKET_GRAPHICS_OPTIONAL_CONTENT));
    };
    let property = properties
        .iter()
        .find(|property| property.name.as_slice() == property_name.as_bytes())
        .ok_or_else(|| unsupported_feature(BUCKET_GRAPHICS_OPTIONAL_CONTENT))?;
    match property.policy {
        OptionalContentPolicy::Group(reference) => Ok(state.visible(reference)),
        OptionalContentPolicy::Unsupported => {
            Err(unsupported_feature(BUCKET_GRAPHICS_OPTIONAL_CONTENT))
        }
    }
}

fn page_font_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<FontResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(FontResources::empty());
    };
    let resource_dictionary = match resources {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let object_number =
                ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
            let reference = Reference::new(ObjectId::new(
                object_number,
                GenerationNumber::new(reference.generation),
            ));
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    let Some(PdfPrimitive::Dictionary(fonts)) = dictionary_value(resource_dictionary, b"Font")
    else {
        return Ok(FontResources::empty());
    };
    FontResources::from_font_dictionary(fonts, document, DisplayListOptions::default())
        .map_err(map_graphics_error)
}

fn annotation_array<'a>(
    document: &'a ClassicDocument<'a>,
    annots: &'a PdfPrimitive<'a>,
) -> Result<&'a [PdfPrimitive<'a>], ThumbnailError> {
    match annots {
        PdfPrimitive::Array(items) => Ok(items),
        PdfPrimitive::Reference(reference) => {
            let reference = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            let ObjectValue::Primitive(PdfPrimitive::Array(items)) = &object.value else {
                return Err(ThumbnailError::Malformed);
            };
            Ok(items)
        }
        _ => Err(ThumbnailError::Malformed),
    }
}

fn annotation_dictionary<'a>(
    document: &'a ClassicDocument<'a>,
    annotation: &'a PdfPrimitive<'a>,
) -> Result<Option<&'a [(PdfName<'a>, PdfPrimitive<'a>)]>, ThumbnailError> {
    match annotation {
        PdfPrimitive::Dictionary(dictionary) => Ok(Some(dictionary)),
        PdfPrimitive::Reference(reference) => {
            let reference = object_reference(*reference)?;
            let Some(object) = document.objects.get(reference.id) else {
                return Ok(None);
            };
            Ok(Some(object_dictionary(&object.value)?))
        }
        _ => Ok(None),
    }
}

fn normal_appearance_reference(
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Option<PdfReference> {
    let PdfPrimitive::Dictionary(appearance) = dictionary_value(annotation, b"AP")? else {
        return None;
    };
    match dictionary_value(appearance, b"N")? {
        PdfPrimitive::Reference(reference) => Some(*reference),
        PdfPrimitive::Dictionary(states) => normal_appearance_state_reference(annotation, states),
        _ => None,
    }
}

fn normal_appearance_state_reference(
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    states: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Option<PdfReference> {
    if let Some(PdfPrimitive::Name(active_state)) = dictionary_value(annotation, b"AS") {
        let PdfPrimitive::Reference(reference) = dictionary_value(states, active_state.as_bytes())?
        else {
            return None;
        };
        return Some(*reference);
    }
    states.iter().find_map(|(_, value)| match value {
        PdfPrimitive::Reference(reference) => Some(*reference),
        _ => None,
    })
}

fn annotation_rect(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> Option<PathBounds> {
    let PdfPrimitive::Array(values) = dictionary_value(annotation, b"Rect")? else {
        return None;
    };
    if values.len() != 4 {
        return None;
    }
    let left = primitive_number(&values[0])?;
    let bottom = primitive_number(&values[1])?;
    let right = primitive_number(&values[2])?;
    let top = primitive_number(&values[3])?;
    Some(PathBounds {
        min_x: left.min(right),
        min_y: bottom.min(top),
        max_x: left.max(right),
        max_y: bottom.max(top),
    })
}

fn document_object_exists(
    document: &ClassicDocument<'_>,
    reference: PdfReference,
) -> Result<bool, ThumbnailError> {
    let reference = object_reference(reference)?;
    Ok(document.objects.get(reference.id).is_some())
}

fn object_reference(reference: PdfReference) -> Result<Reference, ThumbnailError> {
    let object_number =
        ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
    Ok(Reference::new(ObjectId::new(
        object_number,
        GenerationNumber::new(reference.generation),
    )))
}

fn primitive_number(value: &PdfPrimitive<'_>) -> Option<f64> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Some(*value as f64),
        PdfPrimitive::Number(PdfNumber::Real(value)) => Some(*value),
        _ => None,
    }
}

fn append_annotation_form_invocation(
    content: &mut Vec<u8>,
    name: &[u8],
    rect: PathBounds,
    bbox: PathBounds,
) {
    let bbox_width = bbox.max_x - bbox.min_x;
    let bbox_height = bbox.max_y - bbox.min_y;
    if bbox_width <= f64::EPSILON || bbox_height <= f64::EPSILON {
        return;
    }
    let scale_x = (rect.max_x - rect.min_x) / bbox_width;
    let scale_y = (rect.max_y - rect.min_y) / bbox_height;
    let translate_x = rect.min_x - bbox.min_x * scale_x;
    let translate_y = rect.min_y - bbox.min_y * scale_y;
    content.extend_from_slice(
        format!(
            "q {} 0 0 {} {} {} cm /{} Do Q\n",
            format_pdf_number(scale_x),
            format_pdf_number(scale_y),
            format_pdf_number(translate_x),
            format_pdf_number(translate_y),
            String::from_utf8_lossy(name)
        )
        .as_bytes(),
    );
}

fn format_pdf_number(value: f64) -> String {
    if value.fract().abs() <= f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.6}")
    }
}

fn decode_contents(
    document: &ClassicDocument<'_>,
    contents: &PdfPrimitive<'_>,
) -> Result<Vec<u8>, ThumbnailError> {
    match contents {
        PdfPrimitive::Reference(reference) => decode_content_reference(document, *reference),
        PdfPrimitive::Array(items) => {
            let mut decoded = Vec::new();
            for item in items {
                if !decoded.is_empty() {
                    decoded.push(b'\n');
                }
                decoded.extend_from_slice(&decode_contents(document, item)?);
            }
            Ok(decoded)
        }
        _ => Err(unsupported_feature(BUCKET_RENDERER_UNSUPPORTED)),
    }
}

fn decode_content_reference(
    document: &ClassicDocument<'_>,
    reference: pdfrust_syntax::PdfReference,
) -> Result<Vec<u8>, ThumbnailError> {
    let object_number =
        ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
    let reference = Reference::new(ObjectId::new(
        object_number,
        GenerationNumber::new(reference.generation),
    ));
    let object = document
        .objects
        .get(reference.id)
        .ok_or(ThumbnailError::Malformed)?;
    let ObjectValue::Stream(stream) = &object.value else {
        return Err(unsupported_feature(BUCKET_RENDERER_UNSUPPORTED));
    };
    stream.decode().map_err(|_| ThumbnailError::Malformed)
}

fn object_dictionary<'a>(
    value: &'a ObjectValue<'a>,
) -> Result<&'a [(PdfName<'a>, PdfPrimitive<'a>)], ThumbnailError> {
    let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = value else {
        return Err(ThumbnailError::Malformed);
    };
    Ok(dictionary)
}

fn resource_dictionary<'a>(
    document: &'a ClassicDocument<'a>,
    value: &'a PdfPrimitive<'a>,
) -> Result<&'a [(PdfName<'a>, PdfPrimitive<'a>)], ThumbnailError> {
    match value {
        PdfPrimitive::Dictionary(dictionary) => Ok(dictionary),
        PdfPrimitive::Reference(reference) => {
            let reference = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)
        }
        _ => Err(ThumbnailError::Malformed),
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

fn page_geometry(page: ObjectPageMetadata) -> PageGeometry {
    PageGeometry {
        media_box: page_box_bounds(page.media_box),
        crop_box: page.crop_box.map(page_box_bounds),
        rotation: page_rotation(page.rotation_degrees),
    }
}

fn page_box_bounds(page_box: PageBox) -> PathBounds {
    PathBounds {
        min_x: page_box.left.min(page_box.right),
        min_y: page_box.bottom.min(page_box.top),
        max_x: page_box.left.max(page_box.right),
        max_y: page_box.bottom.max(page_box.top),
    }
}

const fn page_rotation(rotation_degrees: u16) -> PageRotation {
    match rotation_degrees {
        90 => PageRotation::Deg90,
        180 => PageRotation::Deg180,
        270 => PageRotation::Deg270,
        _ => PageRotation::Deg0,
    }
}

fn map_graphics_error(error: GraphicsError) -> ThumbnailError {
    match error.kind() {
        GraphicsErrorKind::Content(_)
        | GraphicsErrorKind::OperandCount { .. }
        | GraphicsErrorKind::InvalidOperand { .. }
        | GraphicsErrorKind::MissingCurrentPoint { .. } => ThumbnailError::Malformed,
        GraphicsErrorKind::UnsupportedPathOperator { .. }
        | GraphicsErrorKind::UnsupportedDashPattern { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_STROKE_CLIP)
        }
        GraphicsErrorKind::UnsupportedFontProgram { .. }
        | GraphicsErrorKind::UnsupportedTextEncoding
        | GraphicsErrorKind::UnsupportedTextEncodingFeature { .. }
        | GraphicsErrorKind::MissingTextMapping { .. }
        | GraphicsErrorKind::TextRunOverflow { .. } => {
            unsupported_feature(BUCKET_TEXT_FONT_PROGRAM)
        }
        GraphicsErrorKind::UnsupportedGlyphOutlineProgram { .. }
        | GraphicsErrorKind::UnsupportedGlyphOutline { .. }
        | GraphicsErrorKind::GlyphOutlineSegmentOverflow { .. }
        | GraphicsErrorKind::GlyphOutlineCacheOverflow { .. } => {
            unsupported_feature(BUCKET_TEXT_GLYPH_OUTLINE)
        }
        GraphicsErrorKind::UnsupportedCMap { .. }
        | GraphicsErrorKind::CMapBytesOverflow { .. }
        | GraphicsErrorKind::CMapEntriesOverflow { .. } => {
            unsupported_feature(BUCKET_TEXT_CMAP_TOUNICODE)
        }
        GraphicsErrorKind::UnsupportedImageColorSpace { .. } => {
            unsupported_feature(BUCKET_IMAGE_COLOR_SPACE)
        }
        GraphicsErrorKind::UnsupportedImageFilter { .. } => {
            unsupported_feature(BUCKET_IMAGE_FILTER)
        }
        GraphicsErrorKind::ImageBytesOverflow { .. } => {
            unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET)
        }
        GraphicsErrorKind::UnsupportedSoftMask { .. }
        | GraphicsErrorKind::UnsupportedTransparencyGroup { .. }
        | GraphicsErrorKind::UnsupportedBlendMode { .. }
        | GraphicsErrorKind::UnsupportedOverprint { .. }
        | GraphicsErrorKind::SoftMaskDepthOverflow { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_TRANSPARENCY)
        }
        GraphicsErrorKind::UnsupportedShading { .. }
        | GraphicsErrorKind::UnsupportedPattern { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_PATTERN_SHADING)
        }
        GraphicsErrorKind::FormRecursionOverflow { .. } => {
            unsupported_feature(BUCKET_RENDERER_FORM_XOBJECT)
        }
        _ => unsupported_feature(BUCKET_RENDERER_UNSUPPORTED),
    }
}

fn map_raster_error(error: RasterError) -> ThumbnailError {
    match error.kind() {
        RasterErrorKind::PageRasterPixelsOverflow { .. }
        | RasterErrorKind::PathComplexityOverflow { .. }
        | RasterErrorKind::TransparencyGroupPixelsOverflow { .. } => {
            unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET)
        }
        RasterErrorKind::PatternTileOverflow { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_PATTERN_SHADING)
        }
        _ => ThumbnailError::internal(error.to_string()),
    }
}

const fn unsupported_feature(bucket: &'static str) -> ThumbnailError {
    ThumbnailError::unsupported_feature(bucket)
}

fn map_object_error(error: ObjectError) -> ThumbnailError {
    match error {
        ObjectError::Encrypted => ThumbnailError::Encrypted,
        _ => ThumbnailError::Malformed,
    }
}

fn metadata_from_page_tree(page_tree: &PageTree) -> Result<DocumentMetadata, ThumbnailError> {
    let pages = page_tree
        .pages()
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let index = u32::try_from(index)
                .map_err(|_| ThumbnailError::internal("page index exceeds u32"))?;
            let size = page.size();
            Ok(ThumbnailPageMetadata {
                index,
                size: PageSize {
                    width: size.width,
                    height: size.height,
                },
            })
        })
        .collect::<Result<Vec<_>, ThumbnailError>>()?;
    Ok(DocumentMetadata::new(pages))
}

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the object-model dependency.
#[must_use]
pub fn object_role() -> &'static str {
    pdfrust_object::crate_role()
}

/// Returns the role of the render dependency.
#[must_use]
pub fn render_role() -> &'static str {
    pdfrust_render::crate_role()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "native-backend");
    }

    #[test]
    fn native_backend_name_should_be_backend_neutral() {
        assert_eq!(NativeBackend::new().backend_name(), "rust-native");
    }

    #[test]
    fn native_backend_should_expose_memory_diagnostics() {
        let diagnostics = NativeBackend::new().memory_diagnostics();

        assert_eq!(diagnostics.max_page_pixels, 16 * 1024 * 1024);
        assert_eq!(diagnostics.max_image_bytes, 32 * 1024 * 1024);
        assert_eq!(diagnostics.max_display_items, 8_192);
    }

    #[test]
    fn raster_budget_errors_should_map_to_unsupported() {
        let error = map_raster_error(RasterError::new(
            RasterErrorKind::PageRasterPixelsOverflow { limit: 1 },
        ));

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_RENDERER_MEMORY_BUDGET)
        );
    }

    #[test]
    fn native_backend_should_render_generated_vector_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/vector-paths.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated vector fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 180);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_vector_stress_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/vector-stress.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated vector stress fixture should render through native backend");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 120);
        assert!(
            thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count()
                > 8_000
        );
    }

    #[test]
    fn native_backend_should_render_generated_image_xobject_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/image-xobject.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated image XObject fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_cmyk_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/cmyk-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated CMYK image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_output_intent_rgb_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/output-intent-rgb.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated OutputIntent RGB fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 40, 40), [26, 115, 217, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_indexed_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/indexed-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Indexed image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_dct_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/dct-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated DCT image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        let center = rgba_at(&thumbnail, 60, 60);
        assert!(center[0] > 240);
        assert!(center[1] < 20);
        assert!(center[2] < 20);
        assert_eq!(center[3], 255);
    }

    #[test]
    fn native_backend_should_render_generated_predictor_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/predictor-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated predictor image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_soft_mask_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/soft-mask-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated soft-mask image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [127, 255, 127, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_scanned_page_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/scanned-page.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 200,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated scan-like fixture should render through native backend");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 200);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_cropped_scan_page_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/cropped-scan-page.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 200,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated cropped scan fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_rotated_office_export_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/rotated-office-export.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 200,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated rotated office fixture should render through native backend");

        assert_eq!(thumbnail.width, 100);
        assert_eq!(thumbnail.height, 160);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_user_unit_page_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/user-unit-page.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 200,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated user-unit fixture should render through native backend");

        assert_eq!(thumbnail.width, 80);
        assert_eq!(thumbnail.height, 60);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_inline_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/inline-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated inline image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_form_xobject_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/form-xobject.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Form XObject fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 30, 30), [51, 179, 77, 255]);
        assert_eq!(rgba_at(&thumbnail, 88, 24), [51, 179, 77, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_transparency_group_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/transparency-group.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated transparency group fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 20, 100), [255, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_blend_modes_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/blend-modes.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated blend mode fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 60, 60), [128, 128, 128, 255]);
        assert_eq!(rgba_at(&thumbnail, 20, 100), [128, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 80, 100), [128, 128, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_transparency_alpha_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/transparency-alpha.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated transparency alpha fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 20, 100), [192, 64, 64, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 100), [64, 64, 192, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_axial_gradient_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/axial-gradient.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated axial-gradient fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        let left = rgba_at(&thumbnail, 15, 60);
        let center = rgba_at(&thumbnail, 60, 60);
        let right = rgba_at(&thumbnail, 105, 60);
        assert!(left[0] > 200);
        assert!(left[2] < 60);
        assert!(center[0].abs_diff(center[2]) <= 5);
        assert!(right[0] < 60);
        assert!(right[2] > 200);
    }

    #[test]
    fn native_backend_should_render_generated_radial_gradient_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/radial-gradient.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated radial-gradient fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        let center = rgba_at(&thumbnail, 60, 60);
        let mid = rgba_at(&thumbnail, 90, 60);
        let corner = rgba_at(&thumbnail, 0, 0);
        assert!(center[0] > 240);
        assert!(center[1] > 240);
        assert!(mid[0] > 100 && mid[0] < 160);
        assert!(mid[1] > 100 && mid[1] < 160);
        assert!(corner[0] < 10);
        assert!(corner[1] < 10);
        assert!(corner[2] > 240);
    }

    #[test]
    fn native_backend_should_render_generated_tiling_pattern_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/tiling-pattern.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated tiling-pattern fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 5, 60), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 15, 60), [0, 0, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 25, 60), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 35, 60), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_dashed_stroke_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/dashed-stroke.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated dashed stroke fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 15, 60), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 25, 60), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 35, 60), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 45, 60), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_line_caps_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/line-caps.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated line-caps fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 18, 30), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 18, 60), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 18, 90), [0, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_line_joins_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/line-joins.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated line-joins fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 53, 91), [255, 255, 255, 255]);
        assert_dark(rgba_at(&thumbnail, 53, 46));
        assert_eq!(rgba_at(&thumbnail, 113, 91), [0, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_clipped_paths_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/clipped-paths.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated clipped-paths fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 20, 20), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 60), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 2, 60), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_annotation_appearance_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/annotation-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated annotation-appearance fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 30, 30), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 55, 35), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 30, 50), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_ignore_annotation_without_appearance() {
        let bytes = include_bytes!("../../../fixtures/generated/annotation-missing-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("missing annotation appearance should not abort native rendering");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 15, 95), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 45), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_link_annotation_appearance_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/link-annotation-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated link-annotation appearance fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 72, 81), [0, 0, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 90, 90), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 90, 99), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_highlight_annotation_appearance_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/highlight-annotation-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated highlight annotation appearance fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 25, 53), [255, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 65, 58), [255, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 75, 58), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_widget_annotation_appearance_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/widget-annotation-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated widget annotation appearance fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 25, 86), [230, 230, 230, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 20, 77), 96);
        assert_eq!(rgba_at(&thumbnail, 75, 86), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_acroform_text_field_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/acroform-text-field.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm text field fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 40, 40), [217, 235, 255, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 30, 30), 96);
        assert_eq!(rgba_at(&thumbnail, 95, 40), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_acroform_checkbox_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/acroform-checkbox.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 80,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm checkbox fixture should render");

        assert_eq!(thumbnail.width, 80);
        assert_eq!(thumbnail.height, 80);
        assert_low_intensity(rgba_at(&thumbnail, 30, 40), 96);
        assert_low_intensity(rgba_at(&thumbnail, 20, 30), 96);
        assert_eq!(rgba_at(&thumbnail, 45, 40), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_acroform_radio_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/acroform-radio.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 100,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm radio fixture should render");

        assert_eq!(thumbnail.width, 100);
        assert_eq!(thumbnail.height, 80);
        assert_low_intensity(rgba_at(&thumbnail, 30, 28), 96);
        assert_low_intensity(rgba_at(&thumbnail, 20, 28), 96);
    }

    #[test]
    fn native_backend_should_render_generated_acroform_radio_off_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/acroform-radio-off.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 100,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm radio off fixture should render");

        assert_eq!(thumbnail.width, 100);
        assert_eq!(thumbnail.height, 80);
        assert_low_intensity(rgba_at(&thumbnail, 20, 28), 96);
        assert_eq!(rgba_at(&thumbnail, 30, 28), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_acroform_signature_placeholder_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-signature-placeholder.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm signature placeholder fixture should render");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 30, 35), [240, 240, 240, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 20, 25), 96);
        assert_eq!(rgba_at(&thumbnail, 130, 45), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_optional_content_layer_on_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/optional-content-layer-on.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated optional-content layer-on fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 20, 50), [0, 153, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 50), [230, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_hide_generated_optional_content_layer_off_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/optional-content-layer-off.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated optional-content layer-off fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 20, 50), [0, 153, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 50), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_report_unsupported_optional_content_membership_policy() {
        let bytes = include_bytes!("../../../fixtures/generated/optional-content-ocmd.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 100,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("OCMD policy should not render silently");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_GRAPHICS_OPTIONAL_CONTENT)
        );
    }

    #[test]
    fn native_backend_should_render_latest_generated_incremental_update_revision() {
        let bytes = include_bytes!("../../../fixtures/generated/incremental-update.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated incremental-update fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 20, 50), [230, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 50), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_hybrid_reference_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/hybrid-reference.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated hybrid-reference fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 20, 50), [0, 0, 230, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 50), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_malformed_xref_offset_drift_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/malformed-xref-offset-drift.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated xref-offset-drift fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 20, 50), [230, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_report_encrypted_generated_fixture_for_render() {
        let bytes = include_bytes!("../../../fixtures/generated/encrypted-placeholder.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("encrypted fixture should not render");

        assert_eq!(error, ThumbnailError::Encrypted);
    }

    #[test]
    fn native_backend_should_report_encrypted_generated_fixture_for_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/encrypted-placeholder.pdf");
        let error =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect_err("encrypted fixture should not inspect");

        assert_eq!(error, ThumbnailError::Encrypted);
    }

    #[test]
    fn native_backend_should_render_generated_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 300,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated text fixture should render through native backend");

        assert_eq!(thumbnail.width, 300);
        assert_eq!(thumbnail.height, 160);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_apply_custom_background() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let background = pdfrust_thumbnail::Rgba {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        };
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 300,
                background,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated text fixture should render with custom background");

        assert_eq!(rgba_at(&thumbnail, 0, 0), [10, 20, 30, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_embedded_font_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/embedded-font.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated embedded-font fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_tounicode_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/tounicode-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ToUnicode text fixture should render through native backend");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_cid_font_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/cid-font-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type0 CID font fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_vertical_cjk_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/vertical-cjk-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated vertical CJK fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_shaped_rtl_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/shaped-rtl-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated pre-positioned RTL fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_encoding_differences_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/encoding-differences.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated encoding differences fixture should render through native backend");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_text_spacing_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/text-spacing.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated text spacing fixture should render through native backend");

        assert_eq!(thumbnail.width, 260);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_office_table_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/office-table.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated office table fixture should render through native backend");

        assert_eq!(thumbnail.width, 260);
        assert_eq!(thumbnail.height, 160);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_multi_page_report_first_page() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated multi-page report fixture should render through native backend");

        assert_eq!(thumbnail.width, 260);
        assert_eq!(thumbnail.height, 160);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_not_decode_unrelated_page_streams() {
        let bytes = include_bytes!("../../../fixtures/generated/page-targeted-stream.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("first page should render without decoding second-page content");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 40, 40), [26, 153, 51, 255]);
    }

    #[test]
    fn native_backend_should_decode_requested_page_streams_only() {
        let bytes = include_bytes!("../../../fixtures/generated/page-targeted-stream.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                page_index: 1,
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("second page should fail when its own content stream is decoded");

        assert_eq!(error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_inspect_generated_multi_page_report_order() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated multi-page report should inspect");

        assert_eq!(metadata.page_count(), 2);
        assert_eq!(metadata.pages[0].index, 0);
        assert_eq!(
            metadata.pages[0].size,
            PageSize {
                width: 260.0,
                height: 160.0,
            }
        );
        assert_eq!(metadata.pages[1].index, 1);
        assert_eq!(
            metadata.pages[1].size,
            PageSize {
                width: 240.0,
                height: 180.0,
            }
        );
    }

    #[test]
    fn page_geometry_should_apply_rotation_and_user_unit() {
        let page = ObjectPageMetadata {
            id: ObjectId::new(
                ObjectNumber::new(3).expect("object number"),
                GenerationNumber::new(0),
            ),
            media_box: PageBox {
                left: 0.0,
                bottom: 0.0,
                right: 300.0,
                top: 160.0,
            },
            crop_box: Some(PageBox {
                left: 10.0,
                bottom: 20.0,
                right: 60.0,
                top: 120.0,
            }),
            rotation_degrees: 90,
            user_unit: 2.0,
            resources: None,
        };

        let geometry = page_geometry(page);
        let transform = PageTransform::new(geometry, 120).expect("valid transform");

        assert_eq!(geometry.rotation, PageRotation::Deg90);
        assert_eq!(
            geometry.visible_box(),
            PathBounds {
                min_x: 10.0,
                min_y: 20.0,
                max_x: 60.0,
                max_y: 120.0,
            }
        );
        assert_eq!(transform.dimensions.width, 100);
        assert_eq!(transform.dimensions.height, 50);
    }

    #[test]
    fn native_parallel_renderer_should_preserve_requested_page_order() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[1, 0],
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                ..ParallelRenderOptions::default()
            },
        )
        .expect("multi-page report should render through parallel scheduler");

        assert_eq!(result.workers, 2);
        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.pages[0].width, 240);
        assert_eq!(result.pages[0].height, 180);
        assert_eq!(result.pages[1].width, 260);
        assert_eq!(result.pages[1].height, 160);
    }

    #[test]
    fn native_parallel_renderer_should_match_sequential_page_outputs() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let options = ThumbnailOptions {
            max_edge: 260,
            ..ThumbnailOptions::default()
        };
        let page_1 = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                page_index: 1,
                ..options
            },
        )
        .expect("page 1 should render sequentially");
        let page_0 = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                page_index: 0,
                ..options
            },
        )
        .expect("page 0 should render sequentially");
        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[1, 0],
            &options,
            ParallelRenderOptions {
                max_workers: 2,
                ..ParallelRenderOptions::default()
            },
        )
        .expect("multi-page report should render through parallel scheduler");

        assert_eq!(result.pages, vec![page_1, page_0]);
    }

    #[test]
    fn native_parallel_renderer_should_back_off_workers_for_pixel_budget() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0, 1],
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 4,
                max_in_flight_pixels: 120 * 120,
            },
        )
        .expect("scheduler should fall back to one worker under tight pixel budget");

        assert_eq!(result.workers, 1);
        assert_eq!(result.pages.len(), 2);
    }

    #[test]
    fn native_parallel_renderer_should_fail_when_budget_cannot_schedule_one_page() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let error = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0],
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 1,
            },
        )
        .expect_err("budget too small for one page should fail predictably");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_RENDERER_MEMORY_BUDGET)
        );
    }

    #[test]
    fn native_parallel_renderer_should_report_first_requested_page_error() {
        let bytes = include_bytes!("../../../fixtures/generated/page-targeted-stream.pdf");
        let error = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[1, 0],
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                ..ParallelRenderOptions::default()
            },
        )
        .expect_err("first requested page should fail deterministically");

        assert_eq!(error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_render_generated_mixed_text_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/mixed-text-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated mixed text/image fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 160);
        assert_eq!(rgba_at(&thumbnail, 160, 64), [180, 210, 245, 255]);
        assert_eq!(rgba_at(&thumbnail, 160, 96), [230, 51, 26, 255]);
    }

    #[test]
    fn native_backend_should_report_unsupported_missing_page() {
        let bytes = include_bytes!("../../../fixtures/generated/vector-paths.pdf");
        let error = NativeBackend::new()
            .render(
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    page_index: 1,
                    ..ThumbnailOptions::default()
                },
            )
            .expect_err("missing page should be unsupported");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_RENDERER_UNSUPPORTED)
        );
    }

    #[test]
    fn native_backend_should_inspect_generated_fixture_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated fixture should inspect");

        assert_eq!(metadata.page_count(), 1);
        assert_eq!(
            metadata.first_page_size(),
            Some(PageSize {
                width: 300.0,
                height: 160.0,
            })
        );
    }

    #[test]
    fn native_backend_should_map_invalid_input_to_malformed() {
        let error =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(b"nope"))
                .expect_err("invalid PDF should fail");

        assert_eq!(error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_depend_on_object_and_render_layers() {
        assert_eq!(object_role(), "object");
        assert_eq!(render_role(), "render");
    }

    fn rgba_at(thumbnail: &Thumbnail, x: u32, y: u32) -> [u8; 4] {
        let offset = y as usize * thumbnail.stride + x as usize * 4;
        [
            thumbnail.bytes[offset],
            thumbnail.bytes[offset + 1],
            thumbnail.bytes[offset + 2],
            thumbnail.bytes[offset + 3],
        ]
    }

    fn assert_dark(rgba: [u8; 4]) {
        assert!(rgba[0] < 32, "red channel should be dark: {rgba:?}");
        assert!(rgba[1] < 32, "green channel should be dark: {rgba:?}");
        assert!(rgba[2] < 32, "blue channel should be dark: {rgba:?}");
        assert_eq!(rgba[3], 255);
    }

    fn assert_low_intensity(rgba: [u8; 4], maximum: u8) {
        assert!(
            rgba[0] <= maximum,
            "red channel should be below {maximum}: {rgba:?}"
        );
        assert!(
            rgba[1] <= maximum,
            "green channel should be below {maximum}: {rgba:?}"
        );
        assert!(
            rgba[2] <= maximum,
            "blue channel should be below {maximum}: {rgba:?}"
        );
        assert_eq!(rgba[3], 255);
    }
}
