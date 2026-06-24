//! Rust-native backend adapter for the thumbnail facade.

#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::fs;

use pdfrust_content::tokenize_content;
use pdfrust_object::{
    load_classic_document, load_modern_document, ClassicDocument, GenerationNumber, ObjectId,
    ObjectNumber, ObjectValue, PageBox, PageMetadata as ObjectPageMetadata, PageTree, Reference,
};
use pdfrust_render::{
    build_image_display_list, build_path_display_list, build_text_display_list, rasterize_images,
    rasterize_paths, rasterize_text, DisplayListOptions, FontDescriptor, FontResources,
    GraphicsError, GraphicsErrorKind, ImageResources, PageGeometry, PageRotation, PageTransform,
    PathBounds, PathRasterOptions, RasterError,
};
use pdfrust_syntax::{PdfBytes, PdfName, PdfPrimitive};
use pdfrust_thumbnail::{
    DocumentMetadata, DocumentMetadataBackend, PageMetadata as ThumbnailPageMetadata, PageSize,
    PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "native-backend";

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
        Err(_) => load_classic_document(input)
            .and_then(|document| document.page_tree())
            .map_err(|_| ThumbnailError::Malformed)
            .and_then(|page_tree| metadata_from_page_tree(&page_tree)),
    }
}

fn render_bytes(bytes: &[u8], options: &ThumbnailOptions) -> Result<Thumbnail, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    let document = load_classic_document(input).map_err(|_| ThumbnailError::Malformed)?;
    let page_tree = document
        .page_tree()
        .map_err(|_| ThumbnailError::Malformed)?;
    let page = page_tree
        .pages()
        .get(options.page_index as usize)
        .ok_or(ThumbnailError::Unsupported)?;
    let content = page_content_stream(&document, page)?;
    let display_options = DisplayListOptions::default();
    let display_list =
        build_path_display_list(tokenize_content(PdfBytes::new(&content)), display_options)
            .map_err(map_graphics_error)?;
    let transform =
        PageTransform::new(page_geometry(*page), options.max_edge).map_err(map_raster_error)?;
    let mut raster = rasterize_paths(
        &display_list,
        transform,
        options.background,
        PathRasterOptions::default(),
    )
    .map_err(map_raster_error)?;
    let image_resources = page_image_resources(&document, page)?;
    let image_list = build_image_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &image_resources,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)?;
    rasterize_images(&image_list, &mut raster, transform).map_err(map_raster_error)?;
    let font_resources = page_font_resources(&document, page)?;
    let text_list = build_text_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &font_resources,
        DisplayListOptions::default(),
    )
    .map_err(map_graphics_error)?;
    rasterize_text(&text_list, &mut raster, transform).map_err(map_raster_error)?;
    let dimensions = raster.dimensions();
    Thumbnail::rgba(dimensions.width, dimensions.height, raster.into_pixels())
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
    let contents = dictionary_value(dictionary, b"Contents").ok_or(ThumbnailError::Unsupported)?;
    decode_contents(document, contents)
}

fn page_image_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
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
    ImageResources::from_xobject_dictionary(xobjects, document, DisplayListOptions::default())
        .map_err(map_graphics_error)
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
        _ => return Ok(FontResources::empty()),
    };
    let Some(PdfPrimitive::Dictionary(fonts)) = dictionary_value(resource_dictionary, b"Font")
    else {
        return Ok(FontResources::empty());
    };
    let mut descriptors = Vec::new();
    for (name, value) in fonts {
        let base_font = match value {
            PdfPrimitive::Reference(reference) => {
                let object_number =
                    ObjectNumber::new(reference.object).map_err(|_| ThumbnailError::Malformed)?;
                let reference = Reference::new(ObjectId::new(
                    object_number,
                    GenerationNumber::new(reference.generation),
                ));
                document
                    .objects
                    .get(reference.id)
                    .and_then(|object| font_base_name(&object.value))
            }
            PdfPrimitive::Dictionary(dictionary) => dictionary_value(dictionary, b"BaseFont")
                .and_then(|value| match value {
                    PdfPrimitive::Name(name) => Some(name.as_bytes().to_vec()),
                    _ => None,
                }),
            _ => None,
        };
        descriptors.push(FontDescriptor::new(name.as_bytes().to_vec(), base_font));
    }
    Ok(FontResources::new(descriptors))
}

fn font_base_name(value: &ObjectValue<'_>) -> Option<Vec<u8>> {
    let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = value else {
        return None;
    };
    match dictionary_value(dictionary, b"BaseFont") {
        Some(PdfPrimitive::Name(name)) => Some(name.as_bytes().to_vec()),
        _ => None,
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
        _ => Err(ThumbnailError::Unsupported),
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
        return Err(ThumbnailError::Unsupported);
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

fn dictionary_value<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    key: &[u8],
) -> Option<&'a PdfPrimitive<'a>> {
    dictionary
        .iter()
        .find_map(|(name, value)| (name.as_bytes() == key).then_some(value))
}

fn page_geometry(page: ObjectPageMetadata) -> PageGeometry {
    PageGeometry {
        media_box: page_box_bounds(page.media_box),
        crop_box: page.crop_box.map(page_box_bounds),
        rotation: PageRotation::Deg0,
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

fn map_graphics_error(error: GraphicsError) -> ThumbnailError {
    match error.kind() {
        GraphicsErrorKind::Content(_)
        | GraphicsErrorKind::OperandCount { .. }
        | GraphicsErrorKind::InvalidOperand { .. }
        | GraphicsErrorKind::MissingCurrentPoint { .. } => ThumbnailError::Malformed,
        _ => ThumbnailError::Unsupported,
    }
}

fn map_raster_error(error: RasterError) -> ThumbnailError {
    ThumbnailError::internal(error.to_string())
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

        assert_eq!(error, ThumbnailError::Unsupported);
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
}
