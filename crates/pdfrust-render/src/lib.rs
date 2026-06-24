//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;

use pdfrust_content::{
    tokenize_content, ContentErrorKind, ContentResult, ContentToken, InlineImage, OperatorName,
};
use pdfrust_object::{
    ClassicDocument, GenerationNumber, IndirectObject, ModernDocument, ObjectId, ObjectNumber,
    ObjectValue, Reference, StreamDecodeOptions, StreamObject,
};
use pdfrust_syntax::{ByteOffset, PdfBytes, PdfName, PdfNumber, PdfPrimitive, PdfString};
use pdfrust_thumbnail::{PixelFormat, Rgba};
use zune_jpeg::{
    zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions},
    JpegDecoder,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "render";

/// Default maximum graphics-state stack depth.
pub const DEFAULT_GRAPHICS_STATE_STACK_LIMIT: usize = 64;

/// Default maximum path segment count for one current path.
pub const DEFAULT_PATH_SEGMENT_LIMIT: usize = 16_384;

/// Default maximum display items for one content stream.
pub const DEFAULT_DISPLAY_ITEM_LIMIT: usize = 8_192;

/// Default maximum bytes in one decoded text run.
pub const DEFAULT_TEXT_RUN_BYTES_LIMIT: usize = 64 * 1024;

/// Default maximum decoded bytes for one ToUnicode CMap stream.
pub const DEFAULT_CMAP_BYTES_LIMIT: usize = 1024 * 1024;

/// Default maximum entries accepted in one parsed ToUnicode CMap.
pub const DEFAULT_CMAP_ENTRIES_LIMIT: usize = 4_096;

/// Default maximum path segments accepted in one decoded glyph outline.
pub const DEFAULT_GLYPH_OUTLINE_SEGMENT_LIMIT: usize = 2_048;

/// Default maximum cached glyph outlines per outline cache.
pub const DEFAULT_GLYPH_OUTLINE_CACHE_LIMIT: usize = 4_096;

/// Default maximum decoded bytes for one embedded font program.
pub const DEFAULT_FONT_PROGRAM_BYTES_LIMIT: usize = 16 * 1024 * 1024;

/// Default maximum decoded bytes for one image XObject.
pub const DEFAULT_IMAGE_BYTES_LIMIT: usize = 32 * 1024 * 1024;

/// Default maximum nested soft-mask image depth.
pub const DEFAULT_SOFT_MASK_DEPTH_LIMIT: usize = 1;

/// Default maximum Form XObject recursion depth.
pub const DEFAULT_FORM_RECURSION_DEPTH_LIMIT: usize = 16;

/// Default maximum flattened path line segments for one rasterization pass.
pub const DEFAULT_FLATTENED_PATH_SEGMENT_LIMIT: usize = 65_536;

/// Result alias for graphics-state interpretation.
pub type GraphicsResult<T> = Result<T, GraphicsError>;

/// Result alias for raster-device setup.
pub type RasterResult<T> = Result<T, RasterError>;

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the lower-level content dependency.
#[must_use]
pub fn content_role() -> &'static str {
    pdfrust_content::crate_role()
}

/// Returns the bytes per pixel for the facade's initial RGBA output format.
#[must_use]
pub const fn facade_rgba_bytes_per_pixel() -> usize {
    pdfrust_thumbnail::PixelFormat::Rgba8.bytes_per_pixel()
}

/// Raster output dimensions and row layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterDimensions {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bytes between adjacent rows.
    pub stride: usize,
}

impl RasterDimensions {
    /// Creates checked RGBA raster dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`RasterError`] when dimensions are empty or byte counts
    /// overflow.
    pub fn new(width: u32, height: u32) -> RasterResult<Self> {
        if width == 0 || height == 0 {
            return Err(RasterError::new(RasterErrorKind::InvalidDimensions));
        }
        let stride = (width as usize)
            .checked_mul(PixelFormat::Rgba8.bytes_per_pixel())
            .ok_or_else(|| RasterError::new(RasterErrorKind::StrideOverflow))?;
        stride
            .checked_mul(height as usize)
            .ok_or_else(|| RasterError::new(RasterErrorKind::BufferOverflow))?;
        Ok(Self {
            width,
            height,
            stride,
        })
    }

    fn buffer_len(self) -> RasterResult<usize> {
        self.stride
            .checked_mul(self.height as usize)
            .ok_or_else(|| RasterError::new(RasterErrorKind::BufferOverflow))
    }
}

/// Owned RGBA raster buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterDevice {
    dimensions: RasterDimensions,
    pixels: Vec<u8>,
}

impl RasterDevice {
    /// Allocates a checked RGBA raster and fills it with `background`.
    ///
    /// # Errors
    ///
    /// Returns [`RasterError`] when dimensions or buffer size overflow.
    pub fn new(width: u32, height: u32, background: Rgba) -> RasterResult<Self> {
        let dimensions = RasterDimensions::new(width, height)?;
        let mut pixels = vec![0; dimensions.buffer_len()?];
        for pixel in pixels.chunks_exact_mut(PixelFormat::Rgba8.bytes_per_pixel()) {
            pixel.copy_from_slice(&[background.r, background.g, background.b, background.a]);
        }
        Ok(Self { dimensions, pixels })
    }

    /// Returns raster dimensions and stride.
    #[must_use]
    pub const fn dimensions(&self) -> RasterDimensions {
        self.dimensions
    }

    /// Returns immutable raw RGBA bytes.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns mutable raw RGBA bytes.
    #[must_use]
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Consumes the raster and returns raw RGBA bytes.
    #[must_use]
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    /// Returns one immutable row by y coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`RasterErrorKind::OutOfBounds`] when `y` is outside the raster.
    pub fn row(&self, y: u32) -> RasterResult<&[u8]> {
        let range = self.row_range(y)?;
        Ok(&self.pixels[range])
    }

    /// Returns one mutable row by y coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`RasterErrorKind::OutOfBounds`] when `y` is outside the raster.
    pub fn row_mut(&mut self, y: u32) -> RasterResult<&mut [u8]> {
        let range = self.row_range(y)?;
        Ok(&mut self.pixels[range])
    }

    /// Returns one RGBA pixel by coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`RasterErrorKind::OutOfBounds`] when the coordinate is outside
    /// the raster.
    pub fn pixel(&self, x: u32, y: u32) -> RasterResult<Rgba> {
        let offset = self.pixel_offset(x, y)?;
        Ok(Rgba {
            r: self.pixels[offset],
            g: self.pixels[offset + 1],
            b: self.pixels[offset + 2],
            a: self.pixels[offset + 3],
        })
    }

    /// Writes one RGBA pixel by coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`RasterErrorKind::OutOfBounds`] when the coordinate is outside
    /// the raster.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgba) -> RasterResult<()> {
        let offset = self.pixel_offset(x, y)?;
        self.pixels[offset..offset + PixelFormat::Rgba8.bytes_per_pixel()]
            .copy_from_slice(&[color.r, color.g, color.b, color.a]);
        Ok(())
    }

    fn row_range(&self, y: u32) -> RasterResult<std::ops::Range<usize>> {
        if y >= self.dimensions.height {
            return Err(RasterError::new(RasterErrorKind::OutOfBounds));
        }
        let start = y as usize * self.dimensions.stride;
        Ok(start..start + self.dimensions.stride)
    }

    fn pixel_offset(&self, x: u32, y: u32) -> RasterResult<usize> {
        if x >= self.dimensions.width || y >= self.dimensions.height {
            return Err(RasterError::new(RasterErrorKind::OutOfBounds));
        }
        Ok(y as usize * self.dimensions.stride + x as usize * PixelFormat::Rgba8.bytes_per_pixel())
    }
}

/// Path rasterization configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathRasterOptions {
    /// Uniform supersampling factor per axis.
    pub supersample: u8,
    /// Maximum flattened line segments accepted in one rasterization pass.
    pub max_flattened_segments: usize,
}

impl Default for PathRasterOptions {
    fn default() -> Self {
        Self {
            supersample: 2,
            max_flattened_segments: DEFAULT_FLATTENED_PATH_SEGMENT_LIMIT,
        }
    }
}

/// PDF affine transform matrix.
///
/// The matrix is stored in PDF's six-number form:
/// `[a b c d e f]`, representing:
///
/// ```text
/// | a c e |
/// | b d f |
/// | 0 0 1 |
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix {
    /// Horizontal scale / x basis x component.
    pub a: f64,
    /// Vertical shear / x basis y component.
    pub b: f64,
    /// Horizontal shear / y basis x component.
    pub c: f64,
    /// Vertical scale / y basis y component.
    pub d: f64,
    /// Horizontal translation.
    pub e: f64,
    /// Vertical translation.
    pub f: f64,
}

impl Matrix {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// Creates a matrix from PDF's six-number form.
    #[must_use]
    pub const fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Returns a translation matrix.
    #[must_use]
    pub const fn translate(x: f64, y: f64) -> Self {
        Self {
            e: x,
            f: y,
            ..Self::IDENTITY
        }
    }

    /// Returns a scale matrix.
    #[must_use]
    pub const fn scale(x: f64, y: f64) -> Self {
        Self {
            a: x,
            d: y,
            b: 0.0,
            c: 0.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Multiplies this matrix by `rhs`.
    #[must_use]
    pub fn multiply(self, rhs: Self) -> Self {
        Self {
            a: self.a.mul_add(rhs.a, self.c * rhs.b),
            b: self.b.mul_add(rhs.a, self.d * rhs.b),
            c: self.a.mul_add(rhs.c, self.c * rhs.d),
            d: self.b.mul_add(rhs.c, self.d * rhs.d),
            e: self.a.mul_add(rhs.e, self.c.mul_add(rhs.f, self.e)),
            f: self.b.mul_add(rhs.e, self.d.mul_add(rhs.f, self.f)),
        }
    }

    /// Applies the matrix to a point.
    #[must_use]
    pub fn transform_point(self, x: f64, y: f64) -> Point {
        Point {
            x: self.a.mul_add(x, self.c.mul_add(y, self.e)),
            y: self.b.mul_add(x, self.d.mul_add(y, self.f)),
        }
    }

    /// Returns the inverse matrix when it is non-singular.
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let determinant = self.a.mul_add(self.d, -(self.b * self.c));
        if determinant.abs() <= f64::EPSILON {
            return None;
        }
        let inv = 1.0 / determinant;
        Some(Self {
            a: self.d * inv,
            b: -self.b * inv,
            c: -self.c * inv,
            d: self.a * inv,
            e: (self.c * self.f - self.d * self.e) * inv,
            f: (self.b * self.e - self.a * self.f) * inv,
        })
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Two-dimensional point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

/// Device gray color value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceGray(pub f64);

impl DeviceGray {
    /// Black.
    pub const BLACK: Self = Self(0.0);
}

/// Device color value captured at paint time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceColor {
    /// DeviceGray color.
    Gray(DeviceGray),
    /// DeviceRGB color.
    Rgb {
        /// Red channel, normalized to `0.0..=1.0`.
        r: f64,
        /// Green channel, normalized to `0.0..=1.0`.
        g: f64,
        /// Blue channel, normalized to `0.0..=1.0`.
        b: f64,
    },
}

impl DeviceColor {
    /// Black DeviceGray.
    pub const BLACK: Self = Self::Gray(DeviceGray::BLACK);
}

/// Current graphics state subset needed by early renderer milestones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphicsState {
    /// Current transformation matrix.
    pub ctm: Matrix,
    /// Current line width.
    pub line_width: f64,
    /// Current fill gray compatibility value.
    pub fill_gray: DeviceGray,
    /// Current stroke gray compatibility value.
    pub stroke_gray: DeviceGray,
    /// Current fill color.
    pub fill_color: DeviceColor,
    /// Current stroke color.
    pub stroke_color: DeviceColor,
    /// Placeholder flag set by `W` or `W*` until clipping is modeled fully.
    pub clip_path_pending: bool,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::IDENTITY,
            line_width: 1.0,
            fill_gray: DeviceGray::BLACK,
            stroke_gray: DeviceGray::BLACK,
            fill_color: DeviceColor::BLACK,
            stroke_color: DeviceColor::BLACK,
            clip_path_pending: false,
        }
    }
}

/// Path fill rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    /// Nonzero winding rule.
    Nonzero,
    /// Even-odd rule.
    EvenOdd,
}

/// Painting mode for a completed path item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintMode {
    /// Stroke the path.
    Stroke,
    /// Fill the path.
    Fill {
        /// Fill rule.
        rule: FillRule,
    },
    /// Fill and then stroke the path.
    FillStroke {
        /// Fill rule.
        rule: FillRule,
    },
}

/// One path segment in user space after the active CTM has been applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    /// Move the current point.
    MoveTo(Point),
    /// Draw a line to the point.
    LineTo(Point),
    /// Draw a cubic Bezier curve.
    CubicTo {
        /// First control point.
        c1: Point,
        /// Second control point.
        c2: Point,
        /// End point.
        to: Point,
    },
    /// Close the current subpath.
    Close,
}

/// One rendered path item captured before rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct PathDisplayItem {
    /// Path segments captured in order.
    pub segments: Vec<PathSegment>,
    /// Paint mode used for the path.
    pub paint: PaintMode,
    /// Graphics state snapshot at paint time.
    pub state: GraphicsState,
}

impl PathDisplayItem {
    /// Returns an approximate point bounds for this path item.
    #[must_use]
    pub fn bounds(&self) -> Option<PathBounds> {
        PathBounds::from_segments(&self.segments)
    }
}

/// One positioned text run captured before glyph shaping or rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct TextDisplayItem {
    /// Text decoded through the current lightweight font policy.
    pub text: String,
    /// Source character-code to Unicode mapping used for this run.
    pub glyphs: Vec<TextGlyph>,
    /// Device-space glyph origins after text and graphics transforms.
    pub glyph_origins: Vec<Point>,
    /// Font descriptor selected by `Tf`.
    pub font: FontDescriptor,
    /// Font size selected by `Tf`.
    pub font_size: f64,
    /// Text origin after text and graphics transforms are applied.
    pub origin: Point,
    /// Text matrix at paint time.
    pub text_matrix: Matrix,
    /// PDF text rendering mode active at paint time.
    pub rendering_mode: TextRenderingMode,
    /// Graphics state snapshot at paint time.
    pub state: GraphicsState,
}

/// PDF text rendering mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextRenderingMode {
    /// Fill glyphs.
    #[default]
    Fill,
    /// Stroke glyphs.
    Stroke,
    /// Fill, then stroke glyphs.
    FillStroke,
    /// Do not paint glyphs.
    Invisible,
    /// Fill glyphs and add them to the clipping path.
    FillClip,
    /// Stroke glyphs and add them to the clipping path.
    StrokeClip,
    /// Fill, then stroke glyphs and add them to the clipping path.
    FillStrokeClip,
    /// Add glyphs to the clipping path without painting.
    Clip,
}

impl TextRenderingMode {
    fn from_pdf_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Fill),
            1 => Some(Self::Stroke),
            2 => Some(Self::FillStroke),
            3 => Some(Self::Invisible),
            4 => Some(Self::FillClip),
            5 => Some(Self::StrokeClip),
            6 => Some(Self::FillStrokeClip),
            7 => Some(Self::Clip),
            _ => None,
        }
    }

    fn paint_color(self, state: GraphicsState) -> Option<DeviceColor> {
        match self {
            Self::Fill | Self::FillClip | Self::FillStroke | Self::FillStrokeClip => {
                Some(state.fill_color)
            }
            Self::Stroke | Self::StrokeClip => Some(state.stroke_color),
            Self::Invisible | Self::Clip => None,
        }
    }
}

/// Decoded text glyph metadata carried before outline extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextGlyph {
    /// Source character code read from the PDF string.
    pub character_code: u32,
    /// Unicode text mapped from the source character code.
    pub unicode: String,
}

/// Supported image color-space metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageColorSpace {
    /// DeviceGray image samples.
    DeviceGray,
    /// DeviceRGB image samples.
    DeviceRgb,
    /// DeviceCMYK image samples.
    DeviceCmyk,
    /// Indexed samples with DeviceGray lookup values.
    IndexedGray,
    /// Indexed samples with DeviceRGB lookup values.
    IndexedRgb,
}

impl ImageColorSpace {
    /// Returns bytes per pixel for the supported 8-bit color spaces.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::DeviceGray => 1,
            Self::DeviceRgb => 3,
            Self::DeviceCmyk => 4,
            Self::IndexedGray | Self::IndexedRgb => 1,
        }
    }

    const fn indexed_components(self) -> Option<usize> {
        match self {
            Self::IndexedGray => Some(1),
            Self::IndexedRgb => Some(3),
            Self::DeviceGray | Self::DeviceRgb | Self::DeviceCmyk => None,
        }
    }
}

/// Decoded image XObject resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageXObject {
    /// Resource name used by content streams, without the leading slash.
    pub resource_name: Vec<u8>,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Bits per color component.
    pub bits_per_component: u8,
    /// Supported color space.
    pub color_space: ImageColorSpace,
    /// Decoded image samples.
    pub samples: Arc<[u8]>,
    /// Indexed color lookup bytes for Indexed images.
    pub indexed_lookup: Option<Arc<[u8]>>,
    /// Optional 8-bit alpha mask samples matching the image dimensions.
    pub soft_mask: Option<Arc<[u8]>>,
}

/// One placed image item captured before rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageDisplayItem {
    /// Decoded image resource.
    pub image: ImageXObject,
    /// Current transformation matrix at placement time.
    pub transform: Matrix,
    /// Approximate placement bounds from transforming the unit square.
    pub bounds: PathBounds,
    /// Graphics state snapshot at placement time.
    pub state: GraphicsState,
}

/// Decoded Form XObject resource.
#[derive(Debug, Clone, PartialEq)]
pub struct FormXObject {
    /// Primary resource name used by content streams, without the leading slash.
    pub resource_name: Vec<u8>,
    /// Indirect reference for identity-based nested resource resolution.
    pub reference: Reference,
    /// Decoded form content stream bytes.
    pub content: Arc<[u8]>,
    /// Form matrix applied before executing the form content.
    pub matrix: Matrix,
    /// Form bounding box in form coordinates.
    pub bbox: PathBounds,
    /// Local `/XObject` resource references declared by the form.
    pub xobject_references: Vec<XObjectReference>,
    /// Whether omitted form resources inherit the caller resource scope.
    pub inherits_parent_resources: bool,
}

/// Owned XObject resource reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XObjectReference {
    /// Resource name without the leading slash.
    pub name: Vec<u8>,
    /// Indirect object reference.
    pub reference: Reference,
}

/// Display-list item.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayItem {
    /// Painted path.
    Path(PathDisplayItem),
    /// Clipping placeholder for a path.
    ClipPlaceholder {
        /// Path segments used to define the clip.
        segments: Vec<PathSegment>,
        /// Clip fill rule.
        rule: FillRule,
        /// Graphics state snapshot at clip time.
        state: GraphicsState,
    },
    /// Positioned text run.
    Text(TextDisplayItem),
    /// Placed image XObject.
    Image(ImageDisplayItem),
}

/// Display list produced from content streams before rasterization.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DisplayList {
    items: Vec<DisplayItem>,
}

impl DisplayList {
    /// Creates an empty display list.
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Returns display items in content order.
    #[must_use]
    pub fn items(&self) -> &[DisplayItem] {
        &self.items
    }

    /// Returns the item count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true when no display items were produced.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns approximate bounds across all path-like items.
    #[must_use]
    pub fn bounds(&self) -> Option<PathBounds> {
        let mut bounds: Option<PathBounds> = None;
        for item in &self.items {
            let item_bounds = match item {
                DisplayItem::Path(path) => path.bounds(),
                DisplayItem::ClipPlaceholder { segments, .. } => {
                    PathBounds::from_segments(segments)
                }
                DisplayItem::Text(text) => Some(PathBounds {
                    min_x: text.origin.x,
                    min_y: text.origin.y,
                    max_x: text.origin.x,
                    max_y: text.origin.y,
                }),
                DisplayItem::Image(image) => Some(image.bounds),
            };
            if let Some(item_bounds) = item_bounds {
                bounds = Some(match bounds {
                    Some(existing) => existing.union(item_bounds),
                    None => item_bounds,
                });
            }
        }
        bounds
    }

    fn push(&mut self, item: DisplayItem, limit: usize, offset: ByteOffset) -> GraphicsResult<()> {
        if self.items.len() >= limit {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::DisplayListOverflow { limit },
            ));
        }
        self.items.push(item);
        Ok(())
    }
}

/// Image XObject resource map.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImageResources {
    images: Vec<ImageXObject>,
    non_image_names: Vec<Vec<u8>>,
}

impl ImageResources {
    /// Creates an empty image resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            images: Vec::new(),
            non_image_names: Vec::new(),
        }
    }

    /// Creates an image resource map from decoded images.
    #[must_use]
    pub fn new(images: Vec<ImageXObject>) -> Self {
        Self {
            images,
            non_image_names: Vec::new(),
        }
    }

    /// Resolves image XObjects from a PDF `/XObject` resource dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when an image resource is malformed,
    /// references a missing object, uses an unsupported color space or filter,
    /// or decodes beyond the configured image byte budget.
    pub fn from_xobject_dictionary<'a, R>(
        dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
        resolver: &'a R,
        options: DisplayListOptions,
    ) -> GraphicsResult<Self>
    where
        R: ImageObjectResolver<'a> + ?Sized,
    {
        let mut images = Vec::new();
        let mut non_image_names = Vec::new();
        for (name, value) in dictionary {
            let Some(reference) = reference_from_primitive(value) else {
                continue;
            };
            let object = resolver.resolve_image_object(reference)?.ok_or_else(|| {
                GraphicsError::new(
                    None,
                    GraphicsErrorKind::MissingImageObject {
                        name: name.as_bytes().to_vec(),
                    },
                )
            })?;
            let ObjectValue::Stream(stream) = &object.value else {
                return Err(invalid_image_resource(name.as_bytes()));
            };
            if !dictionary_name_is(stream.dictionary(), b"Subtype", b"Image") {
                non_image_names.push(name.as_bytes().to_vec());
                continue;
            }
            images.push(decode_image_xobject(
                *name,
                stream,
                resolver,
                options.max_image_bytes,
                options.max_soft_mask_depth,
            )?);
        }
        Ok(Self {
            images,
            non_image_names,
        })
    }

    /// Returns the image matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&ImageXObject> {
        self.images
            .iter()
            .find(|image| image.resource_name.as_slice() == name.as_bytes())
    }

    fn is_known_non_image(&self, name: PdfName<'_>) -> bool {
        self.non_image_names
            .iter()
            .any(|non_image| non_image.as_slice() == name.as_bytes())
    }
}

/// Form XObject resource map.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FormResources {
    forms: Vec<FormXObject>,
    aliases: Vec<XObjectReference>,
    non_form_names: Vec<Vec<u8>>,
    non_form_references: Vec<Reference>,
}

impl FormResources {
    /// Creates an empty form resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            forms: Vec::new(),
            aliases: Vec::new(),
            non_form_names: Vec::new(),
            non_form_references: Vec::new(),
        }
    }

    /// Creates a form resource map from decoded forms.
    #[must_use]
    pub fn new(forms: Vec<FormXObject>, aliases: Vec<XObjectReference>) -> Self {
        Self {
            forms,
            aliases,
            non_form_names: Vec::new(),
            non_form_references: Vec::new(),
        }
    }

    /// Resolves Form XObjects from a PDF `/XObject` resource dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a form resource is malformed, references a
    /// missing object, or its content stream cannot be decoded.
    pub fn from_xobject_dictionary<'a, R>(
        dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
        resolver: &'a R,
    ) -> GraphicsResult<Self>
    where
        R: FormObjectResolver<'a> + ?Sized,
    {
        let mut forms = Vec::new();
        let mut aliases = Vec::new();
        let mut non_form_names = Vec::new();
        let mut non_form_references = Vec::new();
        let mut queue = xobject_references(dictionary);

        while let Some(alias) = queue.pop() {
            if !aliases.iter().any(|existing| existing == &alias) {
                aliases.push(alias.clone());
            }
            if forms
                .iter()
                .any(|form: &FormXObject| form.reference == alias.reference)
            {
                continue;
            }
            let Some(object) = resolver.resolve_form_object(alias.reference)? else {
                return Err(GraphicsError::new(
                    None,
                    GraphicsErrorKind::MissingFormObject { name: alias.name },
                ));
            };
            let ObjectValue::Stream(stream) = &object.value else {
                continue;
            };
            if !dictionary_name_is(stream.dictionary(), b"Subtype", b"Form") {
                if !non_form_names
                    .iter()
                    .any(|existing| existing == &alias.name)
                {
                    non_form_names.push(alias.name.clone());
                }
                if !non_form_references
                    .iter()
                    .any(|reference| reference == &alias.reference)
                {
                    non_form_references.push(alias.reference);
                }
                continue;
            }
            let form = decode_form_xobject(alias.name, alias.reference, stream)?;
            queue.extend(form.xobject_references.iter().cloned());
            forms.push(form);
        }

        Ok(Self {
            forms,
            aliases,
            non_form_names,
            non_form_references,
        })
    }

    /// Returns the form matching a page-level PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&FormXObject> {
        let reference = self.aliases.iter().find_map(|alias| {
            (alias.name.as_slice() == name.as_bytes()).then_some(alias.reference)
        })?;
        self.get_by_reference(reference)
    }

    fn get_by_reference(&self, reference: Reference) -> Option<&FormXObject> {
        self.forms.iter().find(|form| form.reference == reference)
    }

    fn resolve_invocation(
        &self,
        name: PdfName<'_>,
        local_xobjects: Option<&[XObjectReference]>,
        inherits_parent_resources: bool,
    ) -> Option<&FormXObject> {
        if let Some(local_xobjects) = local_xobjects {
            if let Some(reference) = local_xobjects.iter().find_map(|resource| {
                (resource.name.as_slice() == name.as_bytes()).then_some(resource.reference)
            }) {
                return self.get_by_reference(reference);
            }
            if !inherits_parent_resources {
                return None;
            }
        }
        self.get(name)
    }

    fn is_known_non_form_invocation(
        &self,
        name: PdfName<'_>,
        local_xobjects: Option<&[XObjectReference]>,
        inherits_parent_resources: bool,
    ) -> bool {
        if let Some(local_xobjects) = local_xobjects {
            if let Some(reference) = local_xobjects.iter().find_map(|resource| {
                (resource.name.as_slice() == name.as_bytes()).then_some(resource.reference)
            }) {
                return self
                    .non_form_references
                    .iter()
                    .any(|non_form| non_form == &reference);
            }
            if !inherits_parent_resources {
                return false;
            }
        }
        self.non_form_names
            .iter()
            .any(|non_form| non_form.as_slice() == name.as_bytes())
    }
}

/// Resolves Form XObject references from a loaded PDF document.
pub trait FormObjectResolver<'a> {
    /// Resolves an indirect object reference.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when object-stream parsing fails.
    fn resolve_form_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>>;
}

impl<'a> FormObjectResolver<'a> for ClassicDocument<'a> {
    fn resolve_form_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        Ok(self.objects.get(reference.id).cloned())
    }
}

impl<'a> FormObjectResolver<'a> for ModernDocument<'a> {
    fn resolve_form_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        self.get_object(reference.id).map_err(|error| {
            GraphicsError::new(
                error.offset(),
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            )
        })
    }
}

/// Resolves image XObject references from a loaded PDF document.
pub trait ImageObjectResolver<'a> {
    /// Resolves an indirect object reference.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when object-stream parsing fails.
    fn resolve_image_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>>;
}

impl<'a> ImageObjectResolver<'a> for ClassicDocument<'a> {
    fn resolve_image_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        Ok(self.objects.get(reference.id).cloned())
    }
}

impl<'a> ImageObjectResolver<'a> for ModernDocument<'a> {
    fn resolve_image_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        self.get_object(reference.id).map_err(|error| {
            GraphicsError::new(
                error.offset(),
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            )
        })
    }
}

/// Supported high-level PDF font subtype metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSubtype {
    /// Simple Type 1 font.
    Type1,
    /// Simple TrueType font.
    TrueType,
    /// Type 0 composite font.
    Type0,
    /// Type 3 PDF content font.
    Type3,
    /// CIDFontType0 descendant font.
    CidFontType0,
    /// CIDFontType2 descendant font.
    CidFontType2,
}

/// Loaded embedded font program kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontProgramKind {
    /// Type 1 font program from `/FontFile`.
    Type1,
    /// TrueType font program from `/FontFile2`.
    TrueType,
    /// Compact Font Format program from `/FontFile3`.
    Cff,
}

/// Cache key for an embedded font program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontProgramKey {
    /// Indirect object reference containing the font program stream.
    pub reference: Reference,
    /// Decoded font program kind.
    pub kind: FontProgramKind,
}

/// Loaded embedded font program bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontProgram {
    /// Cache key identifying this program.
    pub key: FontProgramKey,
    /// Decoded font program bytes.
    pub bytes: Arc<[u8]>,
}

/// Glyph outline extraction options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphOutlineOptions {
    /// Maximum path segments accepted in one decoded glyph outline.
    pub max_segments: usize,
    /// Maximum cached glyph outlines.
    pub max_cache_entries: usize,
}

impl Default for GlyphOutlineOptions {
    fn default() -> Self {
        Self {
            max_segments: DEFAULT_GLYPH_OUTLINE_SEGMENT_LIMIT,
            max_cache_entries: DEFAULT_GLYPH_OUTLINE_CACHE_LIMIT,
        }
    }
}

/// Decoded glyph metrics and outline path.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphOutline {
    /// Glyph code requested from the font program.
    pub glyph_code: u32,
    /// Advance width in font units.
    pub advance_width: f64,
    /// Left side bearing in font units.
    pub left_side_bearing: f64,
    /// Outline path segments in font units.
    pub segments: Vec<PathSegment>,
}

/// Small glyph outline cache keyed by font program identity and glyph code.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GlyphOutlineCache {
    outlines: Vec<CachedGlyphOutline>,
}

impl GlyphOutlineCache {
    /// Returns a cached outline or extracts and caches it.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when the font program is unsupported,
    /// malformed, exceeds the configured segment budget, or the cache is full.
    pub fn outline_for(
        &mut self,
        program: &FontProgram,
        glyph_code: u32,
        options: GlyphOutlineOptions,
    ) -> GraphicsResult<Option<GlyphOutline>> {
        if let Some(entry) = self
            .outlines
            .iter()
            .find(|entry| entry.key == program.key && entry.glyph_code == glyph_code)
        {
            return Ok(entry.outline.clone());
        }
        if self.outlines.len() >= options.max_cache_entries {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::GlyphOutlineCacheOverflow {
                    limit: options.max_cache_entries,
                },
            ));
        }
        let outline = extract_glyph_outline(program, glyph_code, options)?;
        self.outlines.push(CachedGlyphOutline {
            key: program.key,
            glyph_code,
            outline: outline.clone(),
        });
        Ok(outline)
    }

    /// Returns the number of cached glyph outline lookups.
    #[must_use]
    pub fn len(&self) -> usize {
        self.outlines.len()
    }

    /// Returns true when no glyph outlines are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.outlines.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CachedGlyphOutline {
    key: FontProgramKey,
    glyph_code: u32,
    outline: Option<GlyphOutline>,
}

/// Extracts a glyph outline from a loaded font program.
///
/// # Errors
///
/// Returns [`GraphicsError`] when the font program kind is unsupported by the
/// current outline layer, the font bytes are malformed, or the outline exceeds
/// the configured segment budget.
pub fn extract_glyph_outline(
    program: &FontProgram,
    glyph_code: u32,
    options: GlyphOutlineOptions,
) -> GraphicsResult<Option<GlyphOutline>> {
    match program.key.kind {
        FontProgramKind::TrueType => extract_truetype_glyph_outline(program, glyph_code, options),
        FontProgramKind::Cff => extract_cff_glyph_outline(program, glyph_code, options),
        FontProgramKind::Type1 => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedGlyphOutlineProgram {
                kind: program.key.kind,
            },
        )),
    }
}

/// Single-byte font encoding metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FontEncoding {
    differences: Vec<(u8, char)>,
}

impl FontEncoding {
    /// Creates an encoding with no differences from the ASCII-compatible base.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            differences: Vec::new(),
        }
    }

    fn with_differences(differences: Vec<(u8, char)>) -> Self {
        Self { differences }
    }

    fn decode_byte(&self, byte: u8) -> Option<char> {
        self.differences
            .iter()
            .find_map(|(code, character)| (*code == byte).then_some(*character))
            .or_else(|| byte.is_ascii().then_some(byte as char))
    }
}

/// Parsed ToUnicode CMap entries for character-code mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToUnicodeMap {
    entries: Vec<ToUnicodeEntry>,
    max_code_len: usize,
}

impl ToUnicodeMap {
    fn new(entries: Vec<ToUnicodeEntry>) -> Self {
        let max_code_len = entries
            .iter()
            .map(|entry| entry.code.len())
            .max()
            .unwrap_or(0);
        Self {
            entries,
            max_code_len,
        }
    }

    fn match_code(&self, bytes: &[u8], offset: usize) -> Option<(&str, usize, u32)> {
        let remaining = bytes.len().saturating_sub(offset);
        let max_width = self.max_code_len.min(remaining);
        for width in (1..=max_width).rev() {
            let code = &bytes[offset..offset + width];
            if let Some(entry) = self.entries.iter().find(|entry| entry.code == code) {
                return Some((entry.text.as_str(), width, bytes_to_u32(code)));
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToUnicodeEntry {
    code: Vec<u8>,
    text: String,
}

/// Font descriptor used by text display-list construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDescriptor {
    /// Resource name used by content streams, without the leading slash.
    pub resource_name: Vec<u8>,
    /// Optional base font name from page resources.
    pub base_font: Option<Vec<u8>>,
    /// Optional PDF font subtype.
    pub subtype: Option<FontSubtype>,
    /// Optional indirect `/FontDescriptor` object reference.
    pub descriptor_reference: Option<Reference>,
    /// Optional loaded embedded font program.
    pub program: Option<FontProgram>,
    /// Single-byte encoding metadata used when no ToUnicode CMap is present.
    pub encoding: FontEncoding,
    /// Optional ToUnicode character-code mapping.
    pub to_unicode: Option<ToUnicodeMap>,
}

impl FontDescriptor {
    /// Creates a lightweight fallback font descriptor.
    #[must_use]
    pub fn new(resource_name: impl Into<Vec<u8>>, base_font: Option<impl Into<Vec<u8>>>) -> Self {
        Self {
            resource_name: resource_name.into(),
            base_font: base_font.map(Into::into),
            subtype: None,
            descriptor_reference: None,
            program: None,
            encoding: FontEncoding::new(),
            to_unicode: None,
        }
    }

    fn loaded(
        resource_name: impl Into<Vec<u8>>,
        base_font: Option<Vec<u8>>,
        subtype: Option<FontSubtype>,
        descriptor_reference: Option<Reference>,
        program: Option<FontProgram>,
        encoding: FontEncoding,
        to_unicode: Option<ToUnicodeMap>,
    ) -> Self {
        Self {
            resource_name: resource_name.into(),
            base_font,
            subtype,
            descriptor_reference,
            program,
            encoding,
            to_unicode,
        }
    }
}

/// Lightweight font resource map.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FontResources {
    fonts: Vec<FontDescriptor>,
}

impl FontResources {
    /// Creates an empty font resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self { fonts: Vec::new() }
    }

    /// Creates a font resource map from descriptors.
    #[must_use]
    pub fn new(fonts: Vec<FontDescriptor>) -> Self {
        Self { fonts }
    }

    /// Resolves font resources from a PDF `/Font` resource dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a font resource is malformed, references a
    /// missing object, uses an unsupported embedded program kind, or decodes
    /// beyond the configured font byte budget.
    pub fn from_font_dictionary<'a, R>(
        dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
        resolver: &'a R,
        options: DisplayListOptions,
    ) -> GraphicsResult<Self>
    where
        R: FontObjectResolver<'a> + ?Sized,
    {
        let mut cache = FontProgramCache::default();
        let mut fonts = Vec::new();
        for (name, value) in dictionary {
            fonts.push(decode_font_resource(
                *name, value, resolver, &mut cache, options,
            )?);
        }
        Ok(Self { fonts })
    }

    /// Returns the font matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&FontDescriptor> {
        self.fonts
            .iter()
            .find(|font| font.resource_name.as_slice() == name.as_bytes())
    }
}

/// Resolves font and font descriptor references from a loaded PDF document.
pub trait FontObjectResolver<'a> {
    /// Resolves an indirect object reference.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when object-stream parsing fails.
    fn resolve_font_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>>;
}

impl<'a> FontObjectResolver<'a> for ClassicDocument<'a> {
    fn resolve_font_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        Ok(self.objects.get(reference.id).cloned())
    }
}

impl<'a> FontObjectResolver<'a> for ModernDocument<'a> {
    fn resolve_font_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        self.get_object(reference.id).map_err(|error| {
            GraphicsError::new(
                error.offset(),
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            )
        })
    }
}

#[derive(Debug, Default)]
struct FontProgramCache {
    programs: Vec<FontProgram>,
}

impl FontProgramCache {
    fn load(
        &mut self,
        key: FontProgramKey,
        stream: &StreamObject<'_>,
        max_font_program_bytes: usize,
    ) -> GraphicsResult<FontProgram> {
        if let Some(program) = self.programs.iter().find(|program| program.key == key) {
            return Ok(program.clone());
        }
        let decoded = stream
            .decode_with_options(StreamDecodeOptions {
                max_decoded_len: max_font_program_bytes,
            })
            .map_err(|error| match error {
                pdfrust_object::ObjectError::StreamLimitExceeded { .. } => GraphicsError::new(
                    error.offset(),
                    GraphicsErrorKind::FontProgramBytesOverflow {
                        limit: max_font_program_bytes,
                    },
                ),
                _ => GraphicsError::new(
                    error.offset(),
                    GraphicsErrorKind::ObjectModel {
                        message: error.to_string(),
                    },
                ),
            })?;
        if decoded.len() > max_font_program_bytes {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::FontProgramBytesOverflow {
                    limit: max_font_program_bytes,
                },
            ));
        }
        let program = FontProgram {
            key,
            bytes: Arc::from(decoded),
        };
        self.programs.push(program.clone());
        Ok(program)
    }
}

fn decode_font_resource<'a, R>(
    resource_name: PdfName<'a>,
    value: &PdfPrimitive<'a>,
    resolver: &'a R,
    cache: &mut FontProgramCache,
    options: DisplayListOptions,
) -> GraphicsResult<FontDescriptor>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    match value {
        PdfPrimitive::Reference(_) => {
            let reference = reference_from_primitive(value)
                .ok_or_else(|| invalid_font_resource(resource_name.as_bytes()))?;
            let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
                GraphicsError::new(
                    None,
                    GraphicsErrorKind::MissingFontObject {
                        name: resource_name.as_bytes().to_vec(),
                    },
                )
            })?;
            let ObjectValue::Primitive(PdfPrimitive::Dictionary(dictionary)) = object.value else {
                return Err(invalid_font_resource(resource_name.as_bytes()));
            };
            decode_font_dictionary(resource_name, &dictionary, resolver, cache, options)
        }
        PdfPrimitive::Dictionary(dictionary) => {
            decode_font_dictionary(resource_name, dictionary, resolver, cache, options)
        }
        _ => Err(invalid_font_resource(resource_name.as_bytes())),
    }
}

fn decode_font_dictionary<'a, R>(
    resource_name: PdfName<'a>,
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    cache: &mut FontProgramCache,
    options: DisplayListOptions,
) -> GraphicsResult<FontDescriptor>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let base_font = optional_name(dictionary, b"BaseFont");
    let subtype = optional_font_subtype(dictionary);
    let (descriptor_reference, program) =
        load_font_descriptor_program(dictionary, resolver, cache, options.max_font_program_bytes)?;
    let encoding = font_encoding(dictionary)?;
    let to_unicode = load_to_unicode_map(dictionary, resolver, options)?;
    Ok(FontDescriptor::loaded(
        resource_name.as_bytes().to_vec(),
        base_font,
        subtype,
        descriptor_reference,
        program,
        encoding,
        to_unicode,
    ))
}

fn load_font_descriptor_program<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    cache: &mut FontProgramCache,
    max_font_program_bytes: usize,
) -> GraphicsResult<(Option<Reference>, Option<FontProgram>)>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let Some(descriptor) = dictionary_value(dictionary, b"FontDescriptor") else {
        return Ok((None, None));
    };
    match descriptor {
        PdfPrimitive::Reference(_) => {
            let reference = reference_from_primitive(descriptor)
                .ok_or_else(|| invalid_font_resource(b"FontDescriptor"))?;
            let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
                GraphicsError::new(
                    None,
                    GraphicsErrorKind::MissingFontObject {
                        name: b"FontDescriptor".to_vec(),
                    },
                )
            })?;
            let ObjectValue::Primitive(PdfPrimitive::Dictionary(descriptor_dictionary)) =
                object.value
            else {
                return Err(invalid_font_resource(b"FontDescriptor"));
            };
            let program = load_font_program_from_descriptor(
                &descriptor_dictionary,
                resolver,
                cache,
                max_font_program_bytes,
            )?;
            Ok((Some(reference), program))
        }
        PdfPrimitive::Dictionary(descriptor_dictionary) => {
            let program = load_font_program_from_descriptor(
                descriptor_dictionary,
                resolver,
                cache,
                max_font_program_bytes,
            )?;
            Ok((None, program))
        }
        _ => Err(invalid_font_resource(b"FontDescriptor")),
    }
}

fn load_font_program_from_descriptor<'a, R>(
    descriptor: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    cache: &mut FontProgramCache,
    max_font_program_bytes: usize,
) -> GraphicsResult<Option<FontProgram>>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let Some((field, value, fixed_kind)) = embedded_font_program_entry(descriptor) else {
        return Ok(None);
    };
    let reference = reference_from_primitive(value).ok_or_else(|| invalid_font_resource(field))?;
    let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::MissingFontObject {
                name: field.to_vec(),
            },
        )
    })?;
    let ObjectValue::Stream(stream) = object.value else {
        return Err(invalid_font_resource(field));
    };
    let kind = match fixed_kind {
        Some(kind) => kind,
        None => font_file3_program_kind(stream.dictionary())?,
    };
    let key = FontProgramKey { reference, kind };
    cache.load(key, &stream, max_font_program_bytes).map(Some)
}

fn load_to_unicode_map<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    options: DisplayListOptions,
) -> GraphicsResult<Option<ToUnicodeMap>>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let Some(value) = dictionary_value(dictionary, b"ToUnicode") else {
        return Ok(None);
    };
    let reference = reference_from_primitive(value).ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedCMap {
                feature: b"direct ToUnicode CMap".to_vec(),
            },
        )
    })?;
    let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::MissingFontObject {
                name: b"ToUnicode".to_vec(),
            },
        )
    })?;
    let ObjectValue::Stream(stream) = object.value else {
        return Err(invalid_font_resource(b"ToUnicode"));
    };
    let decoded = stream
        .decode_with_options(StreamDecodeOptions {
            max_decoded_len: options.max_cmap_bytes,
        })
        .map_err(|error| match error {
            pdfrust_object::ObjectError::StreamLimitExceeded { .. } => GraphicsError::new(
                error.offset(),
                GraphicsErrorKind::CMapBytesOverflow {
                    limit: options.max_cmap_bytes,
                },
            ),
            _ => GraphicsError::new(
                error.offset(),
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            ),
        })?;
    if decoded.len() > options.max_cmap_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::CMapBytesOverflow {
                limit: options.max_cmap_bytes,
            },
        ));
    }
    parse_to_unicode_cmap(&decoded, options.max_cmap_entries).map(Some)
}

fn extract_truetype_glyph_outline(
    program: &FontProgram,
    glyph_code: u32,
    options: GlyphOutlineOptions,
) -> GraphicsResult<Option<GlyphOutline>> {
    let glyph_id =
        ttf_parser::GlyphId(u16::try_from(glyph_code).map_err(|_| invalid_glyph_outline())?);
    let face = ttf_parser::Face::parse(&program.bytes, 0).map_err(|_| invalid_glyph_outline())?;
    let mut builder = TtfOutlineBuilder {
        segments: Vec::new(),
        current: None,
        max_segments: options.max_segments,
        overflowed: false,
    };
    let Some(_) = face.outline_glyph(glyph_id, &mut builder) else {
        return Ok(None);
    };
    if builder.overflowed {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::GlyphOutlineSegmentOverflow {
                limit: options.max_segments,
            },
        ));
    }
    Ok(Some(GlyphOutline {
        glyph_code,
        advance_width: f64::from(face.glyph_hor_advance(glyph_id).unwrap_or(0)),
        left_side_bearing: f64::from(face.glyph_hor_side_bearing(glyph_id).unwrap_or(0)),
        segments: builder.segments,
    }))
}

const SYNTHETIC_CFF_MAX_GLYPHS: u16 = u16::MAX;

fn extract_cff_glyph_outline(
    program: &FontProgram,
    glyph_code: u32,
    options: GlyphOutlineOptions,
) -> GraphicsResult<Option<GlyphOutline>> {
    let glyph_id =
        ttf_parser::GlyphId(u16::try_from(glyph_code).map_err(|_| invalid_glyph_outline())?);
    let head = synthetic_cff_head_table();
    let hhea = synthetic_cff_hhea_table();
    let maxp = synthetic_cff_maxp_table();
    let face = ttf_parser::Face::from_raw_tables(ttf_parser::RawFaceTables {
        head: &head,
        hhea: &hhea,
        maxp: &maxp,
        cff: Some(program.bytes.as_ref()),
        ..ttf_parser::RawFaceTables::default()
    })
    .map_err(|_| invalid_glyph_outline())?;
    let mut builder = TtfOutlineBuilder {
        segments: Vec::new(),
        current: None,
        max_segments: options.max_segments,
        overflowed: false,
    };
    let Some(_) = face.outline_glyph(glyph_id, &mut builder) else {
        return Ok(None);
    };
    if builder.overflowed {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::GlyphOutlineSegmentOverflow {
                limit: options.max_segments,
            },
        ));
    }
    Ok(Some(GlyphOutline {
        glyph_code,
        advance_width: f64::from(face.glyph_hor_advance(glyph_id).unwrap_or(0)),
        left_side_bearing: f64::from(face.glyph_hor_side_bearing(glyph_id).unwrap_or(0)),
        segments: builder.segments,
    }))
}

fn synthetic_cff_head_table() -> [u8; 54] {
    let mut head = [0; 54];
    head[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    head[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    head[12..16].copy_from_slice(&0x5f0f_3cf5u32.to_be_bytes());
    head[18..20].copy_from_slice(&1000u16.to_be_bytes());
    head[36..38].copy_from_slice(&(-16_384i16).to_be_bytes());
    head[38..40].copy_from_slice(&(-16_384i16).to_be_bytes());
    head[40..42].copy_from_slice(&16_383i16.to_be_bytes());
    head[42..44].copy_from_slice(&16_383i16.to_be_bytes());
    head[46..48].copy_from_slice(&8u16.to_be_bytes());
    head
}

fn synthetic_cff_hhea_table() -> [u8; 36] {
    let mut hhea = [0; 36];
    hhea[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    hhea[4..6].copy_from_slice(&800i16.to_be_bytes());
    hhea[6..8].copy_from_slice(&(-200i16).to_be_bytes());
    hhea[34..36].copy_from_slice(&1u16.to_be_bytes());
    hhea
}

fn synthetic_cff_maxp_table() -> [u8; 6] {
    let mut maxp = [0; 6];
    maxp[0..4].copy_from_slice(&0x0000_5000u32.to_be_bytes());
    maxp[4..6].copy_from_slice(&SYNTHETIC_CFF_MAX_GLYPHS.to_be_bytes());
    maxp
}

struct TtfOutlineBuilder {
    segments: Vec<PathSegment>,
    current: Option<Point>,
    max_segments: usize,
    overflowed: bool,
}

impl TtfOutlineBuilder {
    fn push(&mut self, segment: PathSegment) {
        if self.segments.len() >= self.max_segments {
            self.overflowed = true;
            return;
        }
        self.segments.push(segment);
    }
}

impl ttf_parser::OutlineBuilder for TtfOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let point = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        self.current = Some(point);
        self.push(PathSegment::MoveTo(point));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let point = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        self.current = Some(point);
        self.push(PathSegment::LineTo(point));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let Some(from) = self.current else {
            self.overflowed = true;
            return;
        };
        let control = Point {
            x: f64::from(x1),
            y: f64::from(y1),
        };
        let to = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        let c1 = Point {
            x: from.x + (control.x - from.x) * (2.0 / 3.0),
            y: from.y + (control.y - from.y) * (2.0 / 3.0),
        };
        let c2 = Point {
            x: to.x + (control.x - to.x) * (2.0 / 3.0),
            y: to.y + (control.y - to.y) * (2.0 / 3.0),
        };
        self.current = Some(to);
        self.push(PathSegment::CubicTo { c1, c2, to });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let to = Point {
            x: f64::from(x),
            y: f64::from(y),
        };
        self.current = Some(to);
        self.push(PathSegment::CubicTo {
            c1: Point {
                x: f64::from(x1),
                y: f64::from(y1),
            },
            c2: Point {
                x: f64::from(x2),
                y: f64::from(y2),
            },
            to,
        });
    }

    fn close(&mut self) {
        self.current = None;
        self.push(PathSegment::Close);
    }
}

/// Approximate axis-aligned path bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathBounds {
    /// Minimum x coordinate.
    pub min_x: f64,
    /// Minimum y coordinate.
    pub min_y: f64,
    /// Maximum x coordinate.
    pub max_x: f64,
    /// Maximum y coordinate.
    pub max_y: f64,
}

impl PathBounds {
    /// Returns the bounds width.
    #[must_use]
    pub fn width(self) -> f64 {
        self.max_x - self.min_x
    }

    /// Returns the bounds height.
    #[must_use]
    pub fn height(self) -> f64 {
        self.max_y - self.min_y
    }

    fn from_segments(segments: &[PathSegment]) -> Option<Self> {
        let mut bounds = None;
        for segment in segments {
            match *segment {
                PathSegment::MoveTo(point) | PathSegment::LineTo(point) => {
                    bounds = Some(include_point(bounds, point));
                }
                PathSegment::CubicTo { c1, c2, to } => {
                    bounds = Some(include_point(bounds, c1));
                    bounds = Some(include_point(bounds, c2));
                    bounds = Some(include_point(bounds, to));
                }
                PathSegment::Close => {}
            }
        }
        bounds
    }

    fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

/// Supported page rotation values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageRotation {
    /// No rotation.
    Deg0,
    /// 90 degrees clockwise.
    Deg90,
    /// 180 degrees.
    Deg180,
    /// 270 degrees clockwise.
    Deg270,
}

impl PageRotation {
    /// Returns true when output dimensions are swapped.
    #[must_use]
    pub const fn swaps_axes(self) -> bool {
        matches!(self, Self::Deg90 | Self::Deg270)
    }
}

/// Page boxes and rotation used for raster target setup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageGeometry {
    /// Media box in PDF user-space coordinates.
    pub media_box: PathBounds,
    /// Optional crop box in PDF user-space coordinates.
    pub crop_box: Option<PathBounds>,
    /// Page rotation.
    pub rotation: PageRotation,
}

impl PageGeometry {
    /// Returns the visible page box using `CropBox` when present.
    #[must_use]
    pub fn visible_box(self) -> PathBounds {
        self.crop_box.unwrap_or(self.media_box)
    }
}

/// Page-to-raster transform and checked target dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageTransform {
    /// Visible source box used for rendering.
    pub source_box: PathBounds,
    /// Page rotation used by the transform.
    pub rotation: PageRotation,
    /// User-space to pixel scale.
    pub scale: f64,
    /// Target raster dimensions.
    pub dimensions: RasterDimensions,
    /// Matrix mapping PDF user-space points to pixel-space points.
    pub matrix: Matrix,
}

impl PageTransform {
    /// Builds a page-to-raster transform using the same `max_edge` scaling
    /// policy as the PDFium backend.
    ///
    /// # Errors
    ///
    /// Returns [`RasterError`] for invalid page boxes, zero `max_edge`, or
    /// overflowing output dimensions.
    pub fn new(geometry: PageGeometry, max_edge: u32) -> RasterResult<Self> {
        if max_edge == 0 {
            return Err(RasterError::new(RasterErrorKind::InvalidMaxEdge));
        }
        let source_box = geometry.visible_box();
        let page_width = source_box.width();
        let page_height = source_box.height();
        if !page_width.is_finite()
            || !page_height.is_finite()
            || page_width <= 0.0
            || page_height <= 0.0
        {
            return Err(RasterError::new(RasterErrorKind::InvalidPageBox));
        }

        let (rotated_width, rotated_height) = if geometry.rotation.swaps_axes() {
            (page_height, page_width)
        } else {
            (page_width, page_height)
        };
        let page_max = rotated_width.max(rotated_height);
        let scale = if page_max > f64::from(max_edge) {
            f64::from(max_edge) / page_max
        } else {
            1.0
        };
        let width = scaled_dimension(rotated_width, scale, max_edge)?;
        let height = scaled_dimension(rotated_height, scale, max_edge)?;
        let dimensions = RasterDimensions::new(width, height)?;
        let matrix = page_to_pixel_matrix(source_box, geometry.rotation, scale);
        Ok(Self {
            source_box,
            rotation: geometry.rotation,
            scale,
            dimensions,
            matrix,
        })
    }

    /// Allocates a raster device matching this transform.
    ///
    /// # Errors
    ///
    /// Returns [`RasterError`] when the target buffer cannot be allocated.
    pub fn create_device(self, background: Rgba) -> RasterResult<RasterDevice> {
        RasterDevice::new(self.dimensions.width, self.dimensions.height, background)
    }
}

/// Graphics-state interpreter configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphicsStateOptions {
    /// Maximum allowed depth for `q` save operations.
    pub max_stack_depth: usize,
}

impl Default for GraphicsStateOptions {
    fn default() -> Self {
        Self {
            max_stack_depth: DEFAULT_GRAPHICS_STATE_STACK_LIMIT,
        }
    }
}

/// Display-list builder configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayListOptions {
    /// Maximum allowed depth for `q` save operations.
    pub max_stack_depth: usize,
    /// Maximum number of segments allowed in one current path.
    pub max_path_segments: usize,
    /// Maximum number of display items allowed in one list.
    pub max_display_items: usize,
    /// Maximum bytes accepted in one decoded text run.
    pub max_text_run_bytes: usize,
    /// Maximum decoded bytes accepted for one ToUnicode CMap stream.
    pub max_cmap_bytes: usize,
    /// Maximum parsed entries accepted for one ToUnicode CMap.
    pub max_cmap_entries: usize,
    /// Maximum decoded bytes accepted for one embedded font program.
    pub max_font_program_bytes: usize,
    /// Maximum decoded bytes accepted for one image XObject.
    pub max_image_bytes: usize,
    /// Maximum nested soft-mask image depth.
    pub max_soft_mask_depth: usize,
    /// Maximum allowed Form XObject recursion depth.
    pub max_form_recursion_depth: usize,
}

impl Default for DisplayListOptions {
    fn default() -> Self {
        Self {
            max_stack_depth: DEFAULT_GRAPHICS_STATE_STACK_LIMIT,
            max_path_segments: DEFAULT_PATH_SEGMENT_LIMIT,
            max_display_items: DEFAULT_DISPLAY_ITEM_LIMIT,
            max_text_run_bytes: DEFAULT_TEXT_RUN_BYTES_LIMIT,
            max_cmap_bytes: DEFAULT_CMAP_BYTES_LIMIT,
            max_cmap_entries: DEFAULT_CMAP_ENTRIES_LIMIT,
            max_font_program_bytes: DEFAULT_FONT_PROGRAM_BYTES_LIMIT,
            max_image_bytes: DEFAULT_IMAGE_BYTES_LIMIT,
            max_soft_mask_depth: DEFAULT_SOFT_MASK_DEPTH_LIMIT,
            max_form_recursion_depth: DEFAULT_FORM_RECURSION_DEPTH_LIMIT,
        }
    }
}

/// Interprets supported graphics-state operators from a content-token stream.
///
/// Unsupported operators are ignored after clearing their operands. This keeps
/// early renderer layers able to scan mixed text/vector streams before later
/// milestones implement text, path, and image execution.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, graphics-state stack
/// operations underflow or overflow, or supported operators receive malformed
/// operands.
pub fn interpret_graphics_state<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    options: GraphicsStateOptions,
) -> GraphicsResult<GraphicsState> {
    let mut interpreter = GraphicsStateInterpreter::new(options);
    interpreter.interpret(tokens)?;
    Ok(interpreter.current)
}

/// Builds a path display list from supported content-stream operators.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, path or display-list
/// limits are exceeded, or supported operators receive malformed operands.
pub fn build_path_display_list<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let mut interpreter = DisplayListInterpreter::new(options);
    interpreter.interpret(tokens)?;
    Ok(interpreter.display_list)
}

/// Builds positioned text display-list items from supported text operators.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, text object state is
/// invalid, a selected font is missing, an encoding is unsupported, or display
/// limits are exceeded.
pub fn build_text_display_list<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    fonts: &FontResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let mut interpreter = TextDisplayListInterpreter::new(fonts, options);
    interpreter.interpret(tokens)?;
    Ok(interpreter.display_list)
}

/// Builds placed image display-list items from `Do` operators.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, graphics-state stack
/// operations are invalid, a named image is missing, or display limits are
/// exceeded.
pub fn build_image_display_list<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    images: &ImageResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let mut interpreter = ImageDisplayListInterpreter::new(images, options);
    interpreter.interpret(tokens)?;
    Ok(interpreter.display_list)
}

/// Builds display-list items from Form XObject invocations and nested paths.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, a named form is missing,
/// form recursion exceeds the configured limit, or display limits are exceeded.
pub fn build_form_display_list<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    forms: &FormResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let mut interpreter = DisplayListInterpreter::new_with_forms(
        GraphicsState::default(),
        options,
        forms,
        FormResourceScope::page(),
        0,
    );
    interpreter.interpret(tokens)?;
    Ok(interpreter.display_list)
}

/// Rasterizes path display-list items into an RGBA raster device.
///
/// # Errors
///
/// Returns [`RasterError`] when target allocation fails, supersampling is
/// invalid, or flattened path complexity exceeds configured limits.
pub fn rasterize_paths(
    display_list: &DisplayList,
    transform: PageTransform,
    background: Rgba,
    options: PathRasterOptions,
) -> RasterResult<RasterDevice> {
    let mut device = transform.create_device(background)?;
    rasterize_paths_into(display_list, &mut device, transform, options)?;
    Ok(device)
}

/// Rasterizes path display-list items into an existing RGBA raster device.
///
/// # Errors
///
/// Returns [`RasterError`] when supersampling is invalid or flattened path
/// complexity exceeds configured limits.
pub fn rasterize_paths_into(
    display_list: &DisplayList,
    device: &mut RasterDevice,
    transform: PageTransform,
    options: PathRasterOptions,
) -> RasterResult<()> {
    if options.supersample == 0 {
        return Err(RasterError::new(RasterErrorKind::InvalidSupersampling));
    }
    for item in display_list.items() {
        let DisplayItem::Path(path) = item else {
            continue;
        };
        let flattened = flatten_path_segments(
            &path.segments,
            transform.matrix,
            options.max_flattened_segments,
        )?;
        match path.paint {
            PaintMode::Fill { rule } => {
                fill_path(device, &flattened, rule, path.state.fill_color, options)?;
            }
            PaintMode::Stroke => {
                stroke_path(
                    device,
                    &flattened,
                    path.state.line_width * transform.scale,
                    path.state.stroke_color,
                    options,
                )?;
            }
            PaintMode::FillStroke { rule } => {
                fill_path(device, &flattened, rule, path.state.fill_color, options)?;
                stroke_path(
                    device,
                    &flattened,
                    path.state.line_width * transform.scale,
                    path.state.stroke_color,
                    options,
                )?;
            }
        }
    }
    Ok(())
}

/// Rasterizes image display-list items into an existing RGBA raster device.
///
/// # Errors
///
/// Returns [`RasterError`] when an image transform is singular or device access
/// fails.
pub fn rasterize_images(
    display_list: &DisplayList,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    for item in display_list.items() {
        let DisplayItem::Image(image) = item else {
            continue;
        };
        draw_image(device, image, transform)?;
    }
    Ok(())
}

/// Rasterizes text display-list items using the built-in ASCII fallback font.
///
/// # Errors
///
/// Returns [`RasterError`] when device access fails.
pub fn rasterize_text(
    display_list: &DisplayList,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    for item in display_list.items() {
        let DisplayItem::Text(text) = item else {
            continue;
        };
        draw_text_run(device, text, transform)?;
    }
    Ok(())
}

struct GraphicsStateInterpreter {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    max_stack_depth: usize,
}

struct DisplayListInterpreter<'r> {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    current_path: CurrentPath,
    display_list: DisplayList,
    options: DisplayListOptions,
    forms: Option<FormInterpreterContext<'r>>,
}

#[derive(Debug, Clone, Copy)]
struct FormInterpreterContext<'r> {
    resources: &'r FormResources,
    scope: FormResourceScope<'r>,
    recursion_depth: usize,
}

#[derive(Debug, Clone, Copy)]
struct FormResourceScope<'r> {
    local_xobjects: Option<&'r [XObjectReference]>,
    inherits_parent_resources: bool,
}

impl<'r> FormResourceScope<'r> {
    const fn page() -> Self {
        Self {
            local_xobjects: None,
            inherits_parent_resources: true,
        }
    }

    fn for_form(form: &'r FormXObject) -> Self {
        Self {
            local_xobjects: Some(form.xobject_references.as_slice()),
            inherits_parent_resources: form.inherits_parent_resources,
        }
    }
}

impl<'r> DisplayListInterpreter<'r> {
    fn new(options: DisplayListOptions) -> Self {
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            current_path: CurrentPath::default(),
            display_list: DisplayList::new(),
            options,
            forms: None,
        }
    }

    fn new_with_forms(
        current: GraphicsState,
        options: DisplayListOptions,
        forms: &'r FormResources,
        scope: FormResourceScope<'r>,
        recursion_depth: usize,
    ) -> Self {
        Self {
            current,
            stack: Vec::new(),
            current_path: CurrentPath::default(),
            display_list: DisplayList::new(),
            options,
            forms: Some(FormInterpreterContext {
                resources: forms,
                scope,
                recursion_depth,
            }),
        }
    }

    fn interpret<'a>(
        &mut self,
        tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ) -> GraphicsResult<()> {
        let mut operands = Vec::new();
        for token in tokens {
            match token.map_err(GraphicsError::from_content)? {
                ContentToken::Operand { value, .. } => operands.push(value),
                ContentToken::Operator { offset, name } => {
                    self.apply_operator(offset, name, &operands)?;
                    operands.clear();
                }
                ContentToken::InlineImage { .. } => {
                    operands.clear();
                }
            }
        }
        Ok(())
    }

    fn apply_operator(
        &mut self,
        offset: ByteOffset,
        name: OperatorName<'_>,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        match name.as_bytes() {
            b"q" => self.save_state(offset, operands),
            b"Q" => self.restore_state(offset, operands),
            b"cm" => self.concatenate_matrix(offset, operands),
            b"w" => self.set_line_width(offset, operands),
            b"g" => self.set_fill_gray(offset, operands),
            b"G" => self.set_stroke_gray(offset, operands),
            b"rg" => self.set_fill_rgb(offset, operands),
            b"RG" => self.set_stroke_rgb(offset, operands),
            b"m" => self.move_to(offset, operands),
            b"l" => self.line_to(offset, operands),
            b"c" => self.curve_to(offset, operands),
            b"h" => self.close_path(offset, operands),
            b"re" => self.rectangle(offset, operands),
            b"S" => self.paint(offset, operands, PaintMode::Stroke, false),
            b"s" => self.paint(offset, operands, PaintMode::Stroke, true),
            b"f" | b"F" => self.paint(
                offset,
                operands,
                PaintMode::Fill {
                    rule: FillRule::Nonzero,
                },
                false,
            ),
            b"f*" => self.paint(
                offset,
                operands,
                PaintMode::Fill {
                    rule: FillRule::EvenOdd,
                },
                false,
            ),
            b"B" => self.paint(
                offset,
                operands,
                PaintMode::FillStroke {
                    rule: FillRule::Nonzero,
                },
                false,
            ),
            b"B*" => self.paint(
                offset,
                operands,
                PaintMode::FillStroke {
                    rule: FillRule::EvenOdd,
                },
                false,
            ),
            b"W" => self.clip_placeholder(offset, operands, FillRule::Nonzero),
            b"W*" => self.clip_placeholder(offset, operands, FillRule::EvenOdd),
            b"Do" if self.forms.is_some() => self.invoke_form(offset, operands),
            b"n" => {
                expect_operand_count(offset, b"n", operands, 0)?;
                self.current_path.clear();
                Ok(())
            }
            b"v" | b"y" => Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::UnsupportedPathOperator {
                    operator: match name.as_bytes() {
                        b"v" => b"v",
                        _ => b"y",
                    },
                },
            )),
            _ => Ok(()),
        }
    }

    fn invoke_form(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Do", operands, 1)?;
        let name = name_operand(offset, b"Do", operands, 0)?;
        let Some(context) = self.forms else {
            return Ok(());
        };
        let Some(form) = context.resources.resolve_invocation(
            name,
            context.scope.local_xobjects,
            context.scope.inherits_parent_resources,
        ) else {
            if context.resources.is_known_non_form_invocation(
                name,
                context.scope.local_xobjects,
                context.scope.inherits_parent_resources,
            ) {
                return Ok(());
            }
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::MissingForm {
                    name: name.as_bytes().to_vec(),
                },
            ));
        };
        if context.recursion_depth >= self.options.max_form_recursion_depth {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::FormRecursionOverflow {
                    limit: self.options.max_form_recursion_depth,
                },
            ));
        }

        let mut nested_state = self.current;
        nested_state.ctm = nested_state.ctm.multiply(form.matrix);
        let mut nested = Self::new_with_forms(
            nested_state,
            self.options,
            context.resources,
            FormResourceScope::for_form(form),
            context.recursion_depth + 1,
        );
        nested.push_form_bbox_clip(form, offset)?;
        nested.interpret(tokenize_content(PdfBytes::new(&form.content)))?;
        for item in nested.display_list.items {
            self.display_list
                .push(item, self.options.max_display_items, offset)?;
        }
        Ok(())
    }

    fn push_form_bbox_clip(
        &mut self,
        form: &FormXObject,
        offset: ByteOffset,
    ) -> GraphicsResult<()> {
        let segments = transformed_box_segments(form.bbox, self.current.ctm);
        self.display_list.push(
            DisplayItem::ClipPlaceholder {
                segments,
                rule: FillRule::Nonzero,
                state: self.current,
            },
            self.options.max_display_items,
            offset,
        )
    }

    fn save_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"q", operands, 0)?;
        if self.stack.len() >= self.options.max_stack_depth {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::StackOverflow {
                    limit: self.options.max_stack_depth,
                },
            ));
        }
        self.stack.push(self.current);
        Ok(())
    }

    fn restore_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Q", operands, 0)?;
        self.current = self
            .stack
            .pop()
            .ok_or_else(|| GraphicsError::new(Some(offset), GraphicsErrorKind::StackUnderflow))?;
        Ok(())
    }

    fn concatenate_matrix(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"cm", operands, 6)?;
        self.current.ctm = self.current.ctm.multiply(Matrix::new(
            number_operand(offset, b"cm", operands, 0)?,
            number_operand(offset, b"cm", operands, 1)?,
            number_operand(offset, b"cm", operands, 2)?,
            number_operand(offset, b"cm", operands, 3)?,
            number_operand(offset, b"cm", operands, 4)?,
            number_operand(offset, b"cm", operands, 5)?,
        ));
        Ok(())
    }

    fn set_line_width(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"w", operands, 1)?;
        let line_width = number_operand(offset, b"w", operands, 0)?;
        if line_width < 0.0 {
            return Err(invalid_operand(offset, b"w"));
        }
        self.current.line_width = line_width;
        Ok(())
    }

    fn set_fill_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"g", operands, 1)?;
        let gray = DeviceGray(number_operand(offset, b"g", operands, 0)?.clamp(0.0, 1.0));
        self.current.fill_gray = gray;
        self.current.fill_color = DeviceColor::Gray(gray);
        Ok(())
    }

    fn set_stroke_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"G", operands, 1)?;
        let gray = DeviceGray(number_operand(offset, b"G", operands, 0)?.clamp(0.0, 1.0));
        self.current.stroke_gray = gray;
        self.current.stroke_color = DeviceColor::Gray(gray);
        Ok(())
    }

    fn set_fill_rgb(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"rg", operands, 3)?;
        self.current.fill_color = DeviceColor::Rgb {
            r: number_operand(offset, b"rg", operands, 0)?.clamp(0.0, 1.0),
            g: number_operand(offset, b"rg", operands, 1)?.clamp(0.0, 1.0),
            b: number_operand(offset, b"rg", operands, 2)?.clamp(0.0, 1.0),
        };
        Ok(())
    }

    fn set_stroke_rgb(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"RG", operands, 3)?;
        self.current.stroke_color = DeviceColor::Rgb {
            r: number_operand(offset, b"RG", operands, 0)?.clamp(0.0, 1.0),
            g: number_operand(offset, b"RG", operands, 1)?.clamp(0.0, 1.0),
            b: number_operand(offset, b"RG", operands, 2)?.clamp(0.0, 1.0),
        };
        Ok(())
    }

    fn move_to(&mut self, offset: ByteOffset, operands: &[PdfPrimitive<'_>]) -> GraphicsResult<()> {
        expect_operand_count(offset, b"m", operands, 2)?;
        let point = self.transform_point(
            number_operand(offset, b"m", operands, 0)?,
            number_operand(offset, b"m", operands, 1)?,
        );
        self.current_path.push(
            PathSegment::MoveTo(point),
            self.options.max_path_segments,
            offset,
        )?;
        self.current_path.current_point = Some(point);
        self.current_path.subpath_start = Some(point);
        Ok(())
    }

    fn line_to(&mut self, offset: ByteOffset, operands: &[PdfPrimitive<'_>]) -> GraphicsResult<()> {
        expect_operand_count(offset, b"l", operands, 2)?;
        self.require_current_point(offset, b"l")?;
        let point = self.transform_point(
            number_operand(offset, b"l", operands, 0)?,
            number_operand(offset, b"l", operands, 1)?,
        );
        self.current_path.push(
            PathSegment::LineTo(point),
            self.options.max_path_segments,
            offset,
        )?;
        self.current_path.current_point = Some(point);
        Ok(())
    }

    fn curve_to(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"c", operands, 6)?;
        self.require_current_point(offset, b"c")?;
        let c1 = self.transform_point(
            number_operand(offset, b"c", operands, 0)?,
            number_operand(offset, b"c", operands, 1)?,
        );
        let c2 = self.transform_point(
            number_operand(offset, b"c", operands, 2)?,
            number_operand(offset, b"c", operands, 3)?,
        );
        let to = self.transform_point(
            number_operand(offset, b"c", operands, 4)?,
            number_operand(offset, b"c", operands, 5)?,
        );
        self.current_path.push(
            PathSegment::CubicTo { c1, c2, to },
            self.options.max_path_segments,
            offset,
        )?;
        self.current_path.current_point = Some(to);
        Ok(())
    }

    fn close_path(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"h", operands, 0)?;
        let start = self
            .current_path
            .subpath_start
            .ok_or_else(|| missing_current_point(offset, b"h"))?;
        self.current_path
            .push(PathSegment::Close, self.options.max_path_segments, offset)?;
        self.current_path.current_point = Some(start);
        Ok(())
    }

    fn rectangle(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"re", operands, 4)?;
        let x = number_operand(offset, b"re", operands, 0)?;
        let y = number_operand(offset, b"re", operands, 1)?;
        let width = number_operand(offset, b"re", operands, 2)?;
        let height = number_operand(offset, b"re", operands, 3)?;
        let p0 = self.transform_point(x, y);
        let p1 = self.transform_point(x + width, y);
        let p2 = self.transform_point(x + width, y + height);
        let p3 = self.transform_point(x, y + height);
        for segment in [
            PathSegment::MoveTo(p0),
            PathSegment::LineTo(p1),
            PathSegment::LineTo(p2),
            PathSegment::LineTo(p3),
            PathSegment::Close,
        ] {
            self.current_path
                .push(segment, self.options.max_path_segments, offset)?;
        }
        self.current_path.current_point = Some(p0);
        self.current_path.subpath_start = Some(p0);
        Ok(())
    }

    fn paint(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
        paint: PaintMode,
        close_first: bool,
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"paint", operands, 0)?;
        if close_first && !self.current_path.is_empty() {
            self.close_path(offset, &[])?;
        }
        if self.current_path.is_empty() {
            return Ok(());
        }
        let segments = self.current_path.take_segments();
        self.display_list.push(
            DisplayItem::Path(PathDisplayItem {
                segments,
                paint,
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )
    }

    fn clip_placeholder(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
        rule: FillRule,
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"W", operands, 0)?;
        if self.current_path.is_empty() {
            return Ok(());
        }
        self.current.clip_path_pending = true;
        self.display_list.push(
            DisplayItem::ClipPlaceholder {
                segments: self.current_path.segments.clone(),
                rule,
                state: self.current,
            },
            self.options.max_display_items,
            offset,
        )
    }

    fn require_current_point(
        &self,
        offset: ByteOffset,
        operator: &'static [u8],
    ) -> GraphicsResult<()> {
        self.current_path
            .current_point
            .map(|_| ())
            .ok_or_else(|| missing_current_point(offset, operator))
    }

    fn transform_point(&self, x: f64, y: f64) -> Point {
        self.current.ctm.transform_point(x, y)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct FlattenedPath {
    subpaths: Vec<Vec<Point>>,
    lines: Vec<LineSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineSegment {
    from: Point,
    to: Point,
}

struct TextDisplayListInterpreter<'r> {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    text: TextState,
    fonts: &'r FontResources,
    display_list: DisplayList,
    options: DisplayListOptions,
}

impl<'r> TextDisplayListInterpreter<'r> {
    fn new(fonts: &'r FontResources, options: DisplayListOptions) -> Self {
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            text: TextState::default(),
            fonts,
            display_list: DisplayList::new(),
            options,
        }
    }

    fn interpret<'a>(
        &mut self,
        tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ) -> GraphicsResult<()> {
        let mut operands = Vec::new();
        for token in tokens {
            match token.map_err(GraphicsError::from_content)? {
                ContentToken::Operand { value, .. } => operands.push(value),
                ContentToken::Operator { offset, name } => {
                    self.apply_operator(offset, name, &operands)?;
                    operands.clear();
                }
                ContentToken::InlineImage { .. } => {
                    operands.clear();
                }
            }
        }
        Ok(())
    }

    fn apply_operator(
        &mut self,
        offset: ByteOffset,
        name: OperatorName<'_>,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        match name.as_bytes() {
            b"q" => self.save_state(offset, operands),
            b"Q" => self.restore_state(offset, operands),
            b"cm" => self.concatenate_matrix(offset, operands),
            b"BT" => self.begin_text(offset, operands),
            b"ET" => self.end_text(offset, operands),
            b"Tf" => self.set_font(offset, operands),
            b"Tc" => self.set_character_spacing(offset, operands),
            b"Tw" => self.set_word_spacing(offset, operands),
            b"Tz" => self.set_horizontal_scaling(offset, operands),
            b"Tr" => self.set_text_rendering_mode(offset, operands),
            b"Td" => self.move_text_position(offset, operands),
            b"Tm" => self.set_text_matrix(offset, operands),
            b"Tj" => self.show_text(offset, operands),
            b"TJ" => self.show_text_array(offset, operands),
            _ => Ok(()),
        }
    }

    fn save_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"q", operands, 0)?;
        if self.stack.len() >= self.options.max_stack_depth {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::StackOverflow {
                    limit: self.options.max_stack_depth,
                },
            ));
        }
        self.stack.push(self.current);
        Ok(())
    }

    fn restore_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Q", operands, 0)?;
        self.current = self
            .stack
            .pop()
            .ok_or_else(|| GraphicsError::new(Some(offset), GraphicsErrorKind::StackUnderflow))?;
        Ok(())
    }

    fn concatenate_matrix(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"cm", operands, 6)?;
        self.current.ctm = self.current.ctm.multiply(Matrix::new(
            number_operand(offset, b"cm", operands, 0)?,
            number_operand(offset, b"cm", operands, 1)?,
            number_operand(offset, b"cm", operands, 2)?,
            number_operand(offset, b"cm", operands, 3)?,
            number_operand(offset, b"cm", operands, 4)?,
            number_operand(offset, b"cm", operands, 5)?,
        ));
        Ok(())
    }

    fn begin_text(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"BT", operands, 0)?;
        if self.text.in_text_object {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::TextObjectAlreadyOpen,
            ));
        }
        self.text = TextState {
            in_text_object: true,
            ..TextState::default()
        };
        Ok(())
    }

    fn end_text(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"ET", operands, 0)?;
        self.require_text_object(offset, b"ET")?;
        self.text.in_text_object = false;
        Ok(())
    }

    fn set_font(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tf")?;
        expect_operand_count(offset, b"Tf", operands, 2)?;
        let name = name_operand(offset, b"Tf", operands, 0)?;
        let font_size = number_operand(offset, b"Tf", operands, 1)?;
        if font_size <= 0.0 {
            return Err(invalid_operand(offset, b"Tf"));
        }
        let font = self.fonts.get(name).cloned().ok_or_else(|| {
            GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::MissingFont {
                    name: name.as_bytes().to_vec(),
                },
            )
        })?;
        self.text.font = Some(font);
        self.text.font_size = font_size;
        Ok(())
    }

    fn set_character_spacing(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tc")?;
        expect_operand_count(offset, b"Tc", operands, 1)?;
        self.text.character_spacing = number_operand(offset, b"Tc", operands, 0)?;
        Ok(())
    }

    fn set_word_spacing(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tw")?;
        expect_operand_count(offset, b"Tw", operands, 1)?;
        self.text.word_spacing = number_operand(offset, b"Tw", operands, 0)?;
        Ok(())
    }

    fn set_horizontal_scaling(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tz")?;
        expect_operand_count(offset, b"Tz", operands, 1)?;
        let scaling = number_operand(offset, b"Tz", operands, 0)?;
        if !scaling.is_finite() || scaling <= 0.0 {
            return Err(invalid_operand(offset, b"Tz"));
        }
        self.text.horizontal_scaling = scaling / 100.0;
        Ok(())
    }

    fn set_text_rendering_mode(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tr")?;
        expect_operand_count(offset, b"Tr", operands, 1)?;
        let PdfPrimitive::Number(PdfNumber::Integer(value)) = operands[0] else {
            return Err(invalid_operand(offset, b"Tr"));
        };
        self.text.rendering_mode = TextRenderingMode::from_pdf_value(value)
            .ok_or_else(|| invalid_operand(offset, b"Tr"))?;
        Ok(())
    }

    fn move_text_position(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Td")?;
        expect_operand_count(offset, b"Td", operands, 2)?;
        let translation = Matrix::translate(
            number_operand(offset, b"Td", operands, 0)?,
            number_operand(offset, b"Td", operands, 1)?,
        );
        self.text.line_matrix = self.text.line_matrix.multiply(translation);
        self.text.text_matrix = self.text.line_matrix;
        Ok(())
    }

    fn set_text_matrix(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tm")?;
        expect_operand_count(offset, b"Tm", operands, 6)?;
        let matrix = Matrix::new(
            number_operand(offset, b"Tm", operands, 0)?,
            number_operand(offset, b"Tm", operands, 1)?,
            number_operand(offset, b"Tm", operands, 2)?,
            number_operand(offset, b"Tm", operands, 3)?,
            number_operand(offset, b"Tm", operands, 4)?,
            number_operand(offset, b"Tm", operands, 5)?,
        );
        self.text.text_matrix = matrix;
        self.text.line_matrix = matrix;
        Ok(())
    }

    fn show_text(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"Tj")?;
        expect_operand_count(offset, b"Tj", operands, 1)?;
        let font = self.selected_font(offset)?;
        let text = decode_pdf_text_string(
            string_operand(offset, b"Tj", operands, 0)?,
            &font,
            offset,
            self.options.max_text_run_bytes,
        )?;
        self.show_decoded_text(offset, text)
    }

    fn show_text_array(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.require_text_object(offset, b"TJ")?;
        expect_operand_count(offset, b"TJ", operands, 1)?;
        let Some(PdfPrimitive::Array(values)) = operands.first() else {
            return Err(invalid_operand(offset, b"TJ"));
        };
        let font = self.selected_font(offset)?;
        for value in values {
            match value {
                PdfPrimitive::String(string) => {
                    let chunk = decode_pdf_text_string(
                        *string,
                        &font,
                        offset,
                        self.options.max_text_run_bytes,
                    )?;
                    self.show_decoded_text(offset, chunk)?;
                }
                PdfPrimitive::Number(PdfNumber::Integer(value)) => {
                    self.advance_text(self.adjustment_advance(*value as f64));
                }
                PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => {
                    self.advance_text(self.adjustment_advance(*value));
                }
                _ => return Err(invalid_operand(offset, b"TJ")),
            }
        }
        Ok(())
    }

    fn show_decoded_text(
        &mut self,
        offset: ByteOffset,
        text: DecodedTextRun,
    ) -> GraphicsResult<()> {
        let font = self.selected_font(offset)?;
        let text_matrix = self.text.text_matrix;
        let mut glyph_origins = Vec::with_capacity(text.glyphs.len());
        for glyph in &text.glyphs {
            let origin_matrix = self.current.ctm.multiply(self.text.text_matrix);
            glyph_origins.push(origin_matrix.transform_point(0.0, 0.0));
            self.advance_text(self.glyph_advance(glyph));
        }
        let origin = glyph_origins.first().copied().unwrap_or_else(|| {
            self.current
                .ctm
                .multiply(text_matrix)
                .transform_point(0.0, 0.0)
        });
        self.display_list.push(
            DisplayItem::Text(TextDisplayItem {
                text: text.text,
                glyphs: text.glyphs,
                glyph_origins,
                font,
                font_size: self.text.font_size,
                origin,
                text_matrix,
                rendering_mode: self.text.rendering_mode,
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )?;
        Ok(())
    }

    fn selected_font(&self, offset: ByteOffset) -> GraphicsResult<FontDescriptor> {
        self.text
            .font
            .clone()
            .ok_or_else(|| GraphicsError::new(Some(offset), GraphicsErrorKind::FontNotSelected))
    }

    fn advance_text(&mut self, advance: f64) {
        self.text.text_matrix = self
            .text
            .text_matrix
            .multiply(Matrix::translate(advance, 0.0));
    }

    fn glyph_advance(&self, glyph: &TextGlyph) -> f64 {
        let word_spacing = if glyph.unicode == " " {
            self.text.word_spacing
        } else {
            0.0
        };
        (self.text.font_size * 0.5 + self.text.character_spacing + word_spacing)
            * self.text.horizontal_scaling
    }

    fn adjustment_advance(&self, adjustment: f64) -> f64 {
        -adjustment / 1000.0 * self.text.font_size * self.text.horizontal_scaling
    }

    fn require_text_object(
        &self,
        offset: ByteOffset,
        operator: &'static [u8],
    ) -> GraphicsResult<()> {
        if self.text.in_text_object {
            Ok(())
        } else {
            Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::TextOutsideObject { operator },
            ))
        }
    }
}

struct ImageDisplayListInterpreter<'r> {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    images: &'r ImageResources,
    display_list: DisplayList,
    options: DisplayListOptions,
}

impl<'r> ImageDisplayListInterpreter<'r> {
    fn new(images: &'r ImageResources, options: DisplayListOptions) -> Self {
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            images,
            display_list: DisplayList::new(),
            options,
        }
    }

    fn interpret<'a>(
        &mut self,
        tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ) -> GraphicsResult<()> {
        let mut operands = Vec::new();
        for token in tokens {
            match token.map_err(GraphicsError::from_content)? {
                ContentToken::Operand { value, .. } => operands.push(value),
                ContentToken::Operator { offset, name } => {
                    self.apply_operator(offset, name, &operands)?;
                    operands.clear();
                }
                ContentToken::InlineImage { offset, image } => {
                    self.place_inline_image(offset, &image)?;
                    operands.clear();
                }
            }
        }
        Ok(())
    }

    fn apply_operator(
        &mut self,
        offset: ByteOffset,
        name: OperatorName<'_>,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        match name.as_bytes() {
            b"q" => self.save_state(offset, operands),
            b"Q" => self.restore_state(offset, operands),
            b"cm" => self.concatenate_matrix(offset, operands),
            b"Do" => self.place_image(offset, operands),
            _ => Ok(()),
        }
    }

    fn save_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"q", operands, 0)?;
        if self.stack.len() >= self.options.max_stack_depth {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::StackOverflow {
                    limit: self.options.max_stack_depth,
                },
            ));
        }
        self.stack.push(self.current);
        Ok(())
    }

    fn restore_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Q", operands, 0)?;
        self.current = self
            .stack
            .pop()
            .ok_or_else(|| GraphicsError::new(Some(offset), GraphicsErrorKind::StackUnderflow))?;
        Ok(())
    }

    fn concatenate_matrix(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"cm", operands, 6)?;
        self.current.ctm = self.current.ctm.multiply(Matrix::new(
            number_operand(offset, b"cm", operands, 0)?,
            number_operand(offset, b"cm", operands, 1)?,
            number_operand(offset, b"cm", operands, 2)?,
            number_operand(offset, b"cm", operands, 3)?,
            number_operand(offset, b"cm", operands, 4)?,
            number_operand(offset, b"cm", operands, 5)?,
        ));
        Ok(())
    }

    fn place_image(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Do", operands, 1)?;
        let name = name_operand(offset, b"Do", operands, 0)?;
        let Some(image) = self.images.get(name).cloned() else {
            if self.images.is_known_non_image(name) {
                return Ok(());
            }
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::MissingImage {
                    name: name.as_bytes().to_vec(),
                },
            ));
        };
        let transform = self.current.ctm;
        self.display_list.push(
            DisplayItem::Image(ImageDisplayItem {
                image,
                transform,
                bounds: unit_square_bounds(transform),
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )
    }

    fn place_inline_image(
        &mut self,
        offset: ByteOffset,
        inline_image: &InlineImage<'_>,
    ) -> GraphicsResult<()> {
        let image = decode_inline_image(inline_image, self.options.max_image_bytes)?;
        let transform = self.current.ctm;
        self.display_list.push(
            DisplayItem::Image(ImageDisplayItem {
                image,
                transform,
                bounds: unit_square_bounds(transform),
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TextState {
    in_text_object: bool,
    text_matrix: Matrix,
    line_matrix: Matrix,
    font: Option<FontDescriptor>,
    font_size: f64,
    character_spacing: f64,
    word_spacing: f64,
    horizontal_scaling: f64,
    rendering_mode: TextRenderingMode,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            in_text_object: false,
            text_matrix: Matrix::default(),
            line_matrix: Matrix::default(),
            font: None,
            font_size: 0.0,
            character_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 1.0,
            rendering_mode: TextRenderingMode::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct CurrentPath {
    segments: Vec<PathSegment>,
    current_point: Option<Point>,
    subpath_start: Option<Point>,
}

impl CurrentPath {
    fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    fn push(
        &mut self,
        segment: PathSegment,
        limit: usize,
        offset: ByteOffset,
    ) -> GraphicsResult<()> {
        if self.segments.len() >= limit {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::PathSegmentOverflow { limit },
            ));
        }
        self.segments.push(segment);
        Ok(())
    }

    fn clear(&mut self) {
        self.segments.clear();
        self.current_point = None;
        self.subpath_start = None;
    }

    fn take_segments(&mut self) -> Vec<PathSegment> {
        self.current_point = None;
        self.subpath_start = None;
        std::mem::take(&mut self.segments)
    }
}

impl GraphicsStateInterpreter {
    fn new(options: GraphicsStateOptions) -> Self {
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            max_stack_depth: options.max_stack_depth,
        }
    }

    fn interpret<'a>(
        &mut self,
        tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ) -> GraphicsResult<()> {
        let mut operands = Vec::new();
        for token in tokens {
            match token.map_err(GraphicsError::from_content)? {
                ContentToken::Operand { value, .. } => operands.push(value),
                ContentToken::Operator { offset, name } => {
                    self.apply_operator(offset, name, &operands)?;
                    operands.clear();
                }
                ContentToken::InlineImage { .. } => {
                    operands.clear();
                }
            }
        }
        Ok(())
    }

    fn apply_operator(
        &mut self,
        offset: ByteOffset,
        name: OperatorName<'_>,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        match name.as_bytes() {
            b"q" => self.save_state(offset, operands),
            b"Q" => self.restore_state(offset, operands),
            b"cm" => self.concatenate_matrix(offset, operands),
            b"w" => self.set_line_width(offset, operands),
            b"g" => self.set_fill_gray(offset, operands),
            b"G" => self.set_stroke_gray(offset, operands),
            b"rg" => self.set_fill_rgb(offset, operands),
            b"RG" => self.set_stroke_rgb(offset, operands),
            b"W" | b"W*" => self.set_clip_pending(offset, operands),
            _ => Ok(()),
        }
    }

    fn save_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"q", operands, 0)?;
        if self.stack.len() >= self.max_stack_depth {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::StackOverflow {
                    limit: self.max_stack_depth,
                },
            ));
        }
        self.stack.push(self.current);
        Ok(())
    }

    fn restore_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"Q", operands, 0)?;
        self.current = self
            .stack
            .pop()
            .ok_or_else(|| GraphicsError::new(Some(offset), GraphicsErrorKind::StackUnderflow))?;
        Ok(())
    }

    fn concatenate_matrix(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"cm", operands, 6)?;
        let matrix = Matrix::new(
            number_operand(offset, b"cm", operands, 0)?,
            number_operand(offset, b"cm", operands, 1)?,
            number_operand(offset, b"cm", operands, 2)?,
            number_operand(offset, b"cm", operands, 3)?,
            number_operand(offset, b"cm", operands, 4)?,
            number_operand(offset, b"cm", operands, 5)?,
        );
        self.current.ctm = self.current.ctm.multiply(matrix);
        Ok(())
    }

    fn set_line_width(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"w", operands, 1)?;
        let line_width = number_operand(offset, b"w", operands, 0)?;
        if line_width < 0.0 {
            return Err(invalid_operand(offset, b"w"));
        }
        self.current.line_width = line_width;
        Ok(())
    }

    fn set_fill_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"g", operands, 1)?;
        let gray = DeviceGray(number_operand(offset, b"g", operands, 0)?.clamp(0.0, 1.0));
        self.current.fill_gray = gray;
        self.current.fill_color = DeviceColor::Gray(gray);
        Ok(())
    }

    fn set_stroke_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"G", operands, 1)?;
        let gray = DeviceGray(number_operand(offset, b"G", operands, 0)?.clamp(0.0, 1.0));
        self.current.stroke_gray = gray;
        self.current.stroke_color = DeviceColor::Gray(gray);
        Ok(())
    }

    fn set_fill_rgb(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"rg", operands, 3)?;
        self.current.fill_color = DeviceColor::Rgb {
            r: number_operand(offset, b"rg", operands, 0)?.clamp(0.0, 1.0),
            g: number_operand(offset, b"rg", operands, 1)?.clamp(0.0, 1.0),
            b: number_operand(offset, b"rg", operands, 2)?.clamp(0.0, 1.0),
        };
        Ok(())
    }

    fn set_stroke_rgb(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"RG", operands, 3)?;
        self.current.stroke_color = DeviceColor::Rgb {
            r: number_operand(offset, b"RG", operands, 0)?.clamp(0.0, 1.0),
            g: number_operand(offset, b"RG", operands, 1)?.clamp(0.0, 1.0),
            b: number_operand(offset, b"RG", operands, 2)?.clamp(0.0, 1.0),
        };
        Ok(())
    }

    fn set_clip_pending(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"W", operands, 0)?;
        self.current.clip_path_pending = true;
        Ok(())
    }
}

fn expect_operand_count(
    offset: ByteOffset,
    operator: &'static [u8],
    operands: &[PdfPrimitive<'_>],
    expected: usize,
) -> GraphicsResult<()> {
    if operands.len() == expected {
        Ok(())
    } else {
        Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::OperandCount {
                operator,
                expected,
                actual: operands.len(),
            },
        ))
    }
}

fn number_operand(
    offset: ByteOffset,
    operator: &'static [u8],
    operands: &[PdfPrimitive<'_>],
    index: usize,
) -> GraphicsResult<f64> {
    match operands.get(index) {
        Some(PdfPrimitive::Number(PdfNumber::Integer(value))) => Ok(*value as f64),
        Some(PdfPrimitive::Number(PdfNumber::Real(value))) if value.is_finite() => Ok(*value),
        _ => Err(invalid_operand(offset, operator)),
    }
}

fn name_operand<'a>(
    offset: ByteOffset,
    operator: &'static [u8],
    operands: &[PdfPrimitive<'a>],
    index: usize,
) -> GraphicsResult<PdfName<'a>> {
    match operands.get(index) {
        Some(PdfPrimitive::Name(name)) => Ok(*name),
        _ => Err(invalid_operand(offset, operator)),
    }
}

fn string_operand<'a>(
    offset: ByteOffset,
    operator: &'static [u8],
    operands: &[PdfPrimitive<'a>],
    index: usize,
) -> GraphicsResult<PdfString<'a>> {
    match operands.get(index) {
        Some(PdfPrimitive::String(string)) => Ok(*string),
        _ => Err(invalid_operand(offset, operator)),
    }
}

fn decode_pdf_text_string(
    string: PdfString<'_>,
    font: &FontDescriptor,
    offset: ByteOffset,
    limit: usize,
) -> GraphicsResult<DecodedTextRun> {
    let bytes = decode_pdf_string_bytes(string, offset, limit)?;
    if let Some(to_unicode) = &font.to_unicode {
        return decode_with_to_unicode(&bytes, to_unicode, offset, limit);
    }
    decode_with_font_encoding(&bytes, &font.encoding, offset, limit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedTextRun {
    text: String,
    glyphs: Vec<TextGlyph>,
}

fn decode_pdf_string_bytes(
    string: PdfString<'_>,
    offset: ByteOffset,
    limit: usize,
) -> GraphicsResult<Vec<u8>> {
    let bytes = match string {
        PdfString::Literal(bytes) => bytes.to_vec(),
        PdfString::Hex(bytes) => decode_hex_bytes(bytes)?,
    };
    if bytes.len() > limit {
        return Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::TextRunOverflow { limit },
        ));
    }
    Ok(bytes)
}

fn decode_with_to_unicode(
    bytes: &[u8],
    to_unicode: &ToUnicodeMap,
    offset: ByteOffset,
    limit: usize,
) -> GraphicsResult<DecodedTextRun> {
    let mut text = String::new();
    let mut glyphs = Vec::new();
    let mut byte_offset = 0;
    while byte_offset < bytes.len() {
        let Some((mapped, width, character_code)) = to_unicode.match_code(bytes, byte_offset)
        else {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::MissingTextMapping {
                    code: vec![bytes[byte_offset]],
                },
            ));
        };
        if text.len() + mapped.len() > limit {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::TextRunOverflow { limit },
            ));
        }
        text.push_str(mapped);
        glyphs.push(TextGlyph {
            character_code,
            unicode: mapped.to_string(),
        });
        byte_offset += width;
    }
    Ok(DecodedTextRun { text, glyphs })
}

fn decode_with_font_encoding(
    bytes: &[u8],
    encoding: &FontEncoding,
    offset: ByteOffset,
    limit: usize,
) -> GraphicsResult<DecodedTextRun> {
    let mut text = String::new();
    let mut glyphs = Vec::with_capacity(bytes.len());
    for byte in bytes {
        let Some(character) = encoding.decode_byte(*byte) else {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::UnsupportedTextEncoding,
            ));
        };
        if text.len() + character.len_utf8() > limit {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::TextRunOverflow { limit },
            ));
        }
        text.push(character);
        glyphs.push(TextGlyph {
            character_code: u32::from(*byte),
            unicode: character.to_string(),
        });
    }
    Ok(DecodedTextRun { text, glyphs })
}

fn decode_form_xobject(
    resource_name: Vec<u8>,
    reference: Reference,
    stream: &StreamObject<'_>,
) -> GraphicsResult<FormXObject> {
    let matrix = optional_matrix(stream.dictionary(), b"Matrix")?.unwrap_or(Matrix::IDENTITY);
    let bbox = required_bbox(stream.dictionary(), b"BBox").map_err(|_| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: resource_name.clone(),
            },
        )
    })?;
    let (xobject_references, inherits_parent_resources) =
        form_xobject_references(stream.dictionary());
    let content = stream.decode().map_err(|error| {
        GraphicsError::new(
            error.offset(),
            GraphicsErrorKind::ObjectModel {
                message: error.to_string(),
            },
        )
    })?;
    Ok(FormXObject {
        resource_name,
        reference,
        content: Arc::from(content),
        matrix,
        bbox,
        xobject_references,
        inherits_parent_resources,
    })
}

fn decode_image_xobject<'a, R>(
    resource_name: PdfName<'_>,
    stream: &StreamObject<'a>,
    resolver: &'a R,
    max_image_bytes: usize,
    max_soft_mask_depth: usize,
) -> GraphicsResult<ImageXObject>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    decode_image_xobject_at_depth(
        resource_name,
        stream,
        resolver,
        max_image_bytes,
        max_soft_mask_depth,
        0,
    )
}

fn decode_image_xobject_at_depth<'a, R>(
    resource_name: PdfName<'_>,
    stream: &StreamObject<'a>,
    resolver: &'a R,
    max_image_bytes: usize,
    max_soft_mask_depth: usize,
    soft_mask_depth: usize,
) -> GraphicsResult<ImageXObject>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    let width = required_u32(stream.dictionary(), b"Width")
        .or_else(|_| required_u32(stream.dictionary(), b"W"))
        .map_err(|_| invalid_image_resource(resource_name.as_bytes()))?;
    let height = required_u32(stream.dictionary(), b"Height")
        .or_else(|_| required_u32(stream.dictionary(), b"H"))
        .map_err(|_| invalid_image_resource(resource_name.as_bytes()))?;
    let bits_per_component = required_u8(stream.dictionary(), b"BitsPerComponent")
        .or_else(|_| required_u8(stream.dictionary(), b"BPC"))
        .map_err(|_| invalid_image_resource(resource_name.as_bytes()))?;
    if bits_per_component != 8 {
        return Err(invalid_image_resource(resource_name.as_bytes()));
    }
    let color_space = image_color_space(stream.dictionary())?;
    let image_filter = image_filter(stream.dictionary())?;
    let decoded = decode_image_samples(
        stream,
        image_filter,
        width,
        height,
        color_space.kind,
        bits_per_component,
        max_image_bytes,
    )?;
    if decoded.len() > max_image_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        ));
    }
    let expected_len = expected_image_len(width, height, color_space.kind)?;
    if decoded.len() != expected_len {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: expected_len,
                actual: decoded.len(),
            },
        ));
    }
    let mut decoded = decoded;
    apply_image_decode(
        &mut decoded,
        color_space.kind,
        image_decode_ranges(stream.dictionary())?,
    )?;
    let soft_mask = soft_mask_samples(
        stream.dictionary(),
        resolver,
        width,
        height,
        max_image_bytes,
        max_soft_mask_depth,
        soft_mask_depth,
    )?;
    Ok(ImageXObject {
        resource_name: resource_name.as_bytes().to_vec(),
        width,
        height,
        bits_per_component,
        color_space: color_space.kind,
        samples: Arc::from(decoded),
        indexed_lookup: color_space.indexed_lookup,
        soft_mask,
    })
}

fn soft_mask_samples<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    width: u32,
    height: u32,
    max_image_bytes: usize,
    max_soft_mask_depth: usize,
    soft_mask_depth: usize,
) -> GraphicsResult<Option<Arc<[u8]>>>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    let Some(value) = dictionary_value(dictionary, b"SMask") else {
        return Ok(None);
    };
    if matches!(value, PdfPrimitive::Name(name) if name.as_bytes() == b"None") {
        return Ok(None);
    }
    if soft_mask_depth >= max_soft_mask_depth {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::SoftMaskDepthOverflow {
                limit: max_soft_mask_depth,
            },
        ));
    }
    let reference = reference_from_primitive(value).ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedSoftMask {
                feature: b"SMask".to_vec(),
            },
        )
    })?;
    let object = resolver.resolve_image_object(reference)?.ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::MissingImageObject {
                name: b"SMask".to_vec(),
            },
        )
    })?;
    let ObjectValue::Stream(stream) = &object.value else {
        return Err(invalid_image_resource(b"SMask"));
    };
    if !dictionary_name_is(stream.dictionary(), b"Subtype", b"Image") {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedSoftMask {
                feature: b"Subtype".to_vec(),
            },
        ));
    }
    let mask = decode_image_xobject_at_depth(
        PdfName::new(b"SMask"),
        stream,
        resolver,
        max_image_bytes,
        max_soft_mask_depth,
        soft_mask_depth + 1,
    )?;
    if mask.width != width || mask.height != height {
        return Err(invalid_image_resource(b"SMask"));
    }
    if mask.color_space != ImageColorSpace::DeviceGray {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedSoftMask {
                feature: b"ColorSpace".to_vec(),
            },
        ));
    }
    Ok(Some(mask.samples))
}

fn decode_inline_image(
    image: &InlineImage<'_>,
    max_image_bytes: usize,
) -> GraphicsResult<ImageXObject> {
    let attributes = image.attributes();
    require_unfiltered_inline_image(attributes)?;
    let width = required_u32(attributes, b"Width")
        .or_else(|_| required_u32(attributes, b"W"))
        .map_err(|_| invalid_image_resource(b"inline-image"))?;
    let height = required_u32(attributes, b"Height")
        .or_else(|_| required_u32(attributes, b"H"))
        .map_err(|_| invalid_image_resource(b"inline-image"))?;
    let bits_per_component = required_u8(attributes, b"BitsPerComponent")
        .or_else(|_| required_u8(attributes, b"BPC"))
        .map_err(|_| invalid_image_resource(b"inline-image"))?;
    if bits_per_component != 8 {
        return Err(invalid_image_resource(b"inline-image"));
    }
    let color_space = image_color_space(attributes)?;
    let data = image.data();
    if data.len() > max_image_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        ));
    }
    let expected_len = expected_image_len(width, height, color_space.kind)?;
    if data.len() != expected_len {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: expected_len,
                actual: data.len(),
            },
        ));
    }
    let mut samples = data.to_vec();
    apply_image_decode(
        &mut samples,
        color_space.kind,
        image_decode_ranges(attributes)?,
    )?;
    Ok(ImageXObject {
        resource_name: b"inline-image".to_vec(),
        width,
        height,
        bits_per_component,
        color_space: color_space.kind,
        samples: Arc::from(samples),
        indexed_lookup: color_space.indexed_lookup,
        soft_mask: None,
    })
}

fn decode_image_samples(
    stream: &StreamObject<'_>,
    image_filter: ImageFilter,
    width: u32,
    height: u32,
    color_space: ImageColorSpace,
    bits_per_component: u8,
    max_image_bytes: usize,
) -> GraphicsResult<Vec<u8>> {
    match image_filter {
        ImageFilter::Raw => {
            if stream.raw().len() > max_image_bytes {
                return Err(GraphicsError::new(
                    None,
                    GraphicsErrorKind::ImageBytesOverflow {
                        limit: max_image_bytes,
                    },
                ));
            }
            Ok(stream.raw().to_vec())
        }
        ImageFilter::StreamDecoded => {
            let predictor = image_predictor(
                stream.dictionary(),
                width,
                height,
                color_space,
                bits_per_component,
            )?;
            let max_decoded_len = predictor
                .map(|predictor| predictor.encoded_len())
                .unwrap_or(max_image_bytes)
                .max(if predictor.is_some() {
                    stream.raw().len()
                } else {
                    max_image_bytes
                });
            let decoded = stream
                .decode_with_options(StreamDecodeOptions { max_decoded_len })
                .map_err(|error| {
                    GraphicsError::new(
                        error.offset(),
                        GraphicsErrorKind::ObjectModel {
                            message: error.to_string(),
                        },
                    )
                })?;
            let Some(predictor) = predictor else {
                return Ok(decoded);
            };
            if predictor.decoded_len() > max_image_bytes {
                return Err(GraphicsError::new(
                    None,
                    GraphicsErrorKind::ImageBytesOverflow {
                        limit: max_image_bytes,
                    },
                ));
            }
            apply_png_predictor(&decoded, predictor)
        }
        ImageFilter::DctDecode => {
            decode_dct_image(stream.raw(), width, height, color_space, max_image_bytes)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImagePredictor {
    kind: PngPredictorKind,
    row_count: usize,
    row_len: usize,
    bytes_per_pixel: usize,
    decoded_len: usize,
    encoded_len: usize,
}

impl ImagePredictor {
    const fn decoded_len(self) -> usize {
        self.decoded_len
    }

    const fn encoded_len(self) -> usize {
        self.encoded_len
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PngPredictorKind {
    Fixed(PngFilter),
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PngFilter {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

fn image_predictor(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    width: u32,
    height: u32,
    color_space: ImageColorSpace,
    bits_per_component: u8,
) -> GraphicsResult<Option<ImagePredictor>> {
    let Some(params) = image_decode_parms(dictionary)? else {
        return Ok(None);
    };
    let predictor = decode_parms_u32(params, b"Predictor")?.unwrap_or(1);
    if predictor == 1 {
        return Ok(None);
    }
    let colors = decode_parms_u32(params, b"Colors")?.unwrap_or(1);
    let columns = decode_parms_u32(params, b"Columns")?.unwrap_or(1);
    let bits = decode_parms_u32(params, b"BitsPerComponent")?.unwrap_or(8);
    if colors != color_space.bytes_per_pixel() as u32
        || columns != width
        || bits != u32::from(bits_per_component)
        || bits != 8
    {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource {
                name: b"DecodeParms".to_vec(),
            },
        ));
    }
    let row_len = (columns as usize)
        .checked_mul(colors as usize)
        .ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
            )
        })?;
    let row_count = usize::try_from(height).map_err(|_| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
        )
    })?;
    let bytes_per_pixel = usize::try_from(colors).map_err(|_| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
        )
    })?;
    let kind = match predictor {
        10 => PngPredictorKind::Fixed(PngFilter::None),
        11 => PngPredictorKind::Fixed(PngFilter::Sub),
        12 => PngPredictorKind::Fixed(PngFilter::Up),
        13 => PngPredictorKind::Fixed(PngFilter::Average),
        14 => PngPredictorKind::Fixed(PngFilter::Paeth),
        15 => PngPredictorKind::Adaptive,
        _ => {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedImageFilter {
                    filter: b"Predictor".to_vec(),
                },
            ));
        }
    };
    let decoded_len = row_count.checked_mul(row_len).ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
        )
    })?;
    let encoded_len = match kind {
        PngPredictorKind::Fixed(_) => decoded_len,
        PngPredictorKind::Adaptive => decoded_len.checked_add(row_count).ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
            )
        })?,
    };
    Ok(Some(ImagePredictor {
        kind,
        row_count,
        row_len,
        bytes_per_pixel,
        decoded_len,
        encoded_len,
    }))
}

fn image_decode_parms<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
) -> GraphicsResult<Option<&'a [(PdfName<'a>, PdfPrimitive<'a>)]>> {
    let Some(value) = dictionary_value(dictionary, b"DecodeParms")
        .or_else(|| dictionary_value(dictionary, b"DP"))
    else {
        return Ok(None);
    };
    match value {
        PdfPrimitive::Dictionary(params) => Ok(Some(params.as_slice())),
        PdfPrimitive::Array(values) if values.len() == 1 => {
            let PdfPrimitive::Dictionary(params) = &values[0] else {
                return Err(invalid_image_resource(b"DecodeParms"));
            };
            Ok(Some(params.as_slice()))
        }
        _ => Err(invalid_image_resource(b"DecodeParms")),
    }
}

fn decode_parms_u32(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<u32>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    let PdfPrimitive::Number(PdfNumber::Integer(value)) = value else {
        return Err(invalid_image_resource(b"DecodeParms"));
    };
    u32::try_from(*value)
        .map(Some)
        .map_err(|_| invalid_image_resource(b"DecodeParms"))
}

fn apply_png_predictor(encoded: &[u8], predictor: ImagePredictor) -> GraphicsResult<Vec<u8>> {
    if encoded.len() != predictor.encoded_len() {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: predictor.encoded_len(),
                actual: encoded.len(),
            },
        ));
    }
    let mut decoded = Vec::with_capacity(predictor.decoded_len());
    for row_index in 0..predictor.row_count {
        let (filter, row) = predictor_row(encoded, predictor, row_index)?;
        apply_png_predictor_row(&mut decoded, row, filter, predictor);
    }
    Ok(decoded)
}

fn predictor_row(
    encoded: &[u8],
    predictor: ImagePredictor,
    row_index: usize,
) -> GraphicsResult<(PngFilter, &[u8])> {
    match predictor.kind {
        PngPredictorKind::Fixed(filter) => {
            let start = row_index * predictor.row_len;
            Ok((filter, &encoded[start..start + predictor.row_len]))
        }
        PngPredictorKind::Adaptive => {
            let encoded_row_len = predictor.row_len + 1;
            let start = row_index * encoded_row_len;
            let filter = match encoded[start] {
                0 => PngFilter::None,
                1 => PngFilter::Sub,
                2 => PngFilter::Up,
                3 => PngFilter::Average,
                4 => PngFilter::Paeth,
                _ => {
                    return Err(GraphicsError::new(
                        None,
                        GraphicsErrorKind::InvalidImageResource {
                            name: b"DecodeParms".to_vec(),
                        },
                    ));
                }
            };
            Ok((filter, &encoded[start + 1..start + encoded_row_len]))
        }
    }
}

fn apply_png_predictor_row(
    decoded: &mut Vec<u8>,
    row: &[u8],
    filter: PngFilter,
    predictor: ImagePredictor,
) {
    let row_start = decoded.len();
    for (index, sample) in row.iter().copied().enumerate() {
        let left = if index >= predictor.bytes_per_pixel {
            decoded[row_start + index - predictor.bytes_per_pixel]
        } else {
            0
        };
        let up = if row_start >= predictor.row_len {
            decoded[row_start - predictor.row_len + index]
        } else {
            0
        };
        let up_left = if row_start >= predictor.row_len && index >= predictor.bytes_per_pixel {
            decoded[row_start - predictor.row_len + index - predictor.bytes_per_pixel]
        } else {
            0
        };
        let predicted = match filter {
            PngFilter::None => 0,
            PngFilter::Sub => left,
            PngFilter::Up => up,
            PngFilter::Average => ((u16::from(left) + u16::from(up)) / 2) as u8,
            PngFilter::Paeth => paeth_predictor(left, up, up_left),
        };
        decoded.push(sample.wrapping_add(predicted));
    }
}

fn paeth_predictor(left: u8, up: u8, up_left: u8) -> u8 {
    let left = i16::from(left);
    let up = i16::from(up);
    let up_left = i16::from(up_left);
    let estimate = left + up - up_left;
    let left_distance = (estimate - left).abs();
    let up_distance = (estimate - up).abs();
    let up_left_distance = (estimate - up_left).abs();
    if left_distance <= up_distance && left_distance <= up_left_distance {
        left as u8
    } else if up_distance <= up_left_distance {
        up as u8
    } else {
        up_left as u8
    }
}

fn decode_dct_image(
    encoded: &[u8],
    width: u32,
    height: u32,
    color_space: ImageColorSpace,
    max_image_bytes: usize,
) -> GraphicsResult<Vec<u8>> {
    let output_color_space = match color_space {
        ImageColorSpace::DeviceGray => ColorSpace::Luma,
        ImageColorSpace::DeviceRgb => ColorSpace::RGB,
        ImageColorSpace::DeviceCmyk
        | ImageColorSpace::IndexedGray
        | ImageColorSpace::IndexedRgb => {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedImageFilter {
                    filter: b"DCTDecode-color-space".to_vec(),
                },
            ));
        }
    };
    let mut decoder = JpegDecoder::new_with_options(
        ZCursor::new(encoded),
        DecoderOptions::default().jpeg_set_out_colorspace(output_color_space),
    );
    decoder.decode_headers().map_err(|error| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ObjectModel {
                message: format!("DCTDecode header error: {error}"),
            },
        )
    })?;
    let info = decoder.info().ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ObjectModel {
                message: "DCTDecode did not report image dimensions".to_string(),
            },
        )
    })?;
    if u32::from(info.width) != width || u32::from(info.height) != height {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: expected_image_len(width, height, color_space)?,
                actual: decoder.output_buffer_size().unwrap_or(0),
            },
        ));
    }
    let output_size = decoder.output_buffer_size().ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        )
    })?;
    if output_size > max_image_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        ));
    }
    decoder.decode().map_err(|error| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::ObjectModel {
                message: format!("DCTDecode data error: {error}"),
            },
        )
    })
}

fn expected_image_len(
    width: u32,
    height: u32,
    color_space: ImageColorSpace,
) -> GraphicsResult<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(color_space.bytes_per_pixel()))
        .ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
            )
        })
}

fn flatten_path_segments(
    segments: &[PathSegment],
    transform: Matrix,
    limit: usize,
) -> RasterResult<FlattenedPath> {
    let mut flattened = FlattenedPath::default();
    let mut current_subpath: Vec<Point> = Vec::new();
    let mut current_point: Option<Point> = None;
    let mut subpath_start: Option<Point> = None;

    for segment in segments {
        match *segment {
            PathSegment::MoveTo(point) => {
                finish_subpath(&mut flattened, &mut current_subpath);
                let point = transform.transform_point(point.x, point.y);
                current_subpath.push(point);
                current_point = Some(point);
                subpath_start = Some(point);
            }
            PathSegment::LineTo(point) => {
                let Some(from) = current_point else {
                    continue;
                };
                let to = transform.transform_point(point.x, point.y);
                push_flattened_line(&mut flattened, from, to, limit)?;
                current_subpath.push(to);
                current_point = Some(to);
            }
            PathSegment::CubicTo { c1, c2, to } => {
                let Some(from) = current_point else {
                    continue;
                };
                let c1 = transform.transform_point(c1.x, c1.y);
                let c2 = transform.transform_point(c2.x, c2.y);
                let to = transform.transform_point(to.x, to.y);
                let mut previous = from;
                for step in 1..=16 {
                    let t = f64::from(step) / 16.0;
                    let point = cubic_point(from, c1, c2, to, t);
                    push_flattened_line(&mut flattened, previous, point, limit)?;
                    current_subpath.push(point);
                    previous = point;
                }
                current_point = Some(to);
            }
            PathSegment::Close => {
                if let (Some(from), Some(to)) = (current_point, subpath_start) {
                    push_flattened_line(&mut flattened, from, to, limit)?;
                    current_point = Some(to);
                }
            }
        }
    }
    finish_subpath(&mut flattened, &mut current_subpath);
    Ok(flattened)
}

fn finish_subpath(flattened: &mut FlattenedPath, current_subpath: &mut Vec<Point>) {
    if current_subpath.len() >= 2 {
        flattened.subpaths.push(std::mem::take(current_subpath));
    } else {
        current_subpath.clear();
    }
}

fn push_flattened_line(
    flattened: &mut FlattenedPath,
    from: Point,
    to: Point,
    limit: usize,
) -> RasterResult<()> {
    if flattened.lines.len() >= limit {
        return Err(RasterError::new(RasterErrorKind::PathComplexityOverflow {
            limit,
        }));
    }
    flattened.lines.push(LineSegment { from, to });
    Ok(())
}

fn cubic_point(from: Point, c1: Point, c2: Point, to: Point, t: f64) -> Point {
    let inv = 1.0 - t;
    let a = inv * inv * inv;
    let b = 3.0 * inv * inv * t;
    let c = 3.0 * inv * t * t;
    let d = t * t * t;
    Point {
        x: a.mul_add(from.x, b.mul_add(c1.x, c.mul_add(c2.x, d * to.x))),
        y: a.mul_add(from.y, b.mul_add(c1.y, c.mul_add(c2.y, d * to.y))),
    }
}

fn fill_path(
    device: &mut RasterDevice,
    path: &FlattenedPath,
    rule: FillRule,
    color: DeviceColor,
    options: PathRasterOptions,
) -> RasterResult<()> {
    let source = device_color_to_rgba(color);
    let samples = u32::from(options.supersample);
    let sample_count = samples * samples;
    let dimensions = device.dimensions();
    for y in 0..dimensions.height {
        for x in 0..dimensions.width {
            let mut covered = 0;
            for sample_y in 0..samples {
                for sample_x in 0..samples {
                    let point = sample_point(x, y, sample_x, sample_y, samples);
                    if point_in_path(point, path, rule) {
                        covered += 1;
                    }
                }
            }
            if covered > 0 {
                blend_pixel(
                    device,
                    x,
                    y,
                    source,
                    f64::from(covered) / f64::from(sample_count),
                )?;
            }
        }
    }
    Ok(())
}

fn stroke_path(
    device: &mut RasterDevice,
    path: &FlattenedPath,
    line_width: f64,
    color: DeviceColor,
    options: PathRasterOptions,
) -> RasterResult<()> {
    let source = device_color_to_rgba(color);
    let radius = if line_width <= 0.0 {
        0.5
    } else {
        line_width / 2.0
    };
    let samples = u32::from(options.supersample);
    let sample_count = samples * samples;
    let dimensions = device.dimensions();
    for y in 0..dimensions.height {
        for x in 0..dimensions.width {
            let mut covered = 0;
            for sample_y in 0..samples {
                for sample_x in 0..samples {
                    let point = sample_point(x, y, sample_x, sample_y, samples);
                    if point_in_stroke(point, &path.lines, radius) {
                        covered += 1;
                    }
                }
            }
            if covered > 0 {
                blend_pixel(
                    device,
                    x,
                    y,
                    source,
                    f64::from(covered) / f64::from(sample_count),
                )?;
            }
        }
    }
    Ok(())
}

fn sample_point(x: u32, y: u32, sample_x: u32, sample_y: u32, samples: u32) -> Point {
    Point {
        x: f64::from(x) + (f64::from(sample_x) + 0.5) / f64::from(samples),
        y: f64::from(y) + (f64::from(sample_y) + 0.5) / f64::from(samples),
    }
}

fn point_in_path(point: Point, path: &FlattenedPath, rule: FillRule) -> bool {
    match rule {
        FillRule::EvenOdd => {
            path.subpaths
                .iter()
                .filter(|subpath| point_in_polygon_even_odd(point, subpath))
                .count()
                % 2
                == 1
        }
        FillRule::Nonzero => {
            path.subpaths
                .iter()
                .map(|subpath| polygon_winding(point, subpath))
                .sum::<i32>()
                != 0
        }
    }
}

fn point_in_polygon_even_odd(point: Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    for edge in polygon_edges(polygon) {
        if (edge.from.y > point.y) != (edge.to.y > point.y) {
            let intersection_x = (edge.to.x - edge.from.x) * (point.y - edge.from.y)
                / (edge.to.y - edge.from.y)
                + edge.from.x;
            if point.x < intersection_x {
                inside = !inside;
            }
        }
    }
    inside
}

fn polygon_winding(point: Point, polygon: &[Point]) -> i32 {
    let mut winding = 0;
    for edge in polygon_edges(polygon) {
        if edge.from.y <= point.y {
            if edge.to.y > point.y && is_left(edge.from, edge.to, point) > 0.0 {
                winding += 1;
            }
        } else if edge.to.y <= point.y && is_left(edge.from, edge.to, point) < 0.0 {
            winding -= 1;
        }
    }
    winding
}

fn polygon_edges(polygon: &[Point]) -> impl Iterator<Item = LineSegment> + '_ {
    polygon.iter().enumerate().map(|(index, from)| LineSegment {
        from: *from,
        to: polygon[(index + 1) % polygon.len()],
    })
}

fn is_left(from: Point, to: Point, point: Point) -> f64 {
    (to.x - from.x).mul_add(point.y - from.y, -((point.x - from.x) * (to.y - from.y)))
}

fn point_in_stroke(point: Point, lines: &[LineSegment], radius: f64) -> bool {
    let radius_squared = radius * radius;
    lines
        .iter()
        .any(|line| distance_to_line_segment_squared(point, *line) <= radius_squared)
}

fn distance_to_line_segment_squared(point: Point, line: LineSegment) -> f64 {
    let dx = line.to.x - line.from.x;
    let dy = line.to.y - line.from.y;
    let len_squared = dx.mul_add(dx, dy * dy);
    if len_squared <= f64::EPSILON {
        let px = point.x - line.from.x;
        let py = point.y - line.from.y;
        return px.mul_add(px, py * py);
    }
    let t = (((point.x - line.from.x) * dx + (point.y - line.from.y) * dy) / len_squared)
        .clamp(0.0, 1.0);
    let projection = Point {
        x: line.from.x + t * dx,
        y: line.from.y + t * dy,
    };
    let px = point.x - projection.x;
    let py = point.y - projection.y;
    px.mul_add(px, py * py)
}

fn device_color_to_rgba(color: DeviceColor) -> Rgba {
    match color {
        DeviceColor::Gray(DeviceGray(value)) => {
            let channel = normalized_to_u8(value);
            Rgba {
                r: channel,
                g: channel,
                b: channel,
                a: 255,
            }
        }
        DeviceColor::Rgb { r, g, b } => Rgba {
            r: normalized_to_u8(r),
            g: normalized_to_u8(g),
            b: normalized_to_u8(b),
            a: 255,
        },
    }
}

fn normalized_to_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn blend_pixel(
    device: &mut RasterDevice,
    x: u32,
    y: u32,
    source: Rgba,
    coverage: f64,
) -> RasterResult<()> {
    let dest = device.pixel(x, y)?;
    let inv = 1.0 - coverage;
    device.set_pixel(
        x,
        y,
        Rgba {
            r: blend_channel(source.r, dest.r, coverage, inv),
            g: blend_channel(source.g, dest.g, coverage, inv),
            b: blend_channel(source.b, dest.b, coverage, inv),
            a: 255,
        },
    )
}

fn blend_channel(source: u8, dest: u8, coverage: f64, inv_coverage: f64) -> u8 {
    (f64::from(source).mul_add(coverage, f64::from(dest) * inv_coverage)).round() as u8
}

fn draw_image(
    device: &mut RasterDevice,
    image: &ImageDisplayItem,
    page_transform: PageTransform,
) -> RasterResult<()> {
    let image_to_device = page_transform.matrix.multiply(image.transform);
    let inverse = image_to_device
        .inverse()
        .ok_or_else(|| RasterError::new(RasterErrorKind::SingularImageTransform))?;
    let bounds = transformed_image_bounds(image_to_device);
    let dimensions = device.dimensions();
    let min_x = bounds.min_x.floor().max(0.0) as u32;
    let min_y = bounds.min_y.floor().max(0.0) as u32;
    let max_x = bounds.max_x.ceil().min(f64::from(dimensions.width)) as u32;
    let max_y = bounds.max_y.ceil().min(f64::from(dimensions.height)) as u32;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample = inverse.transform_point(f64::from(x) + 0.5, f64::from(y) + 0.5);
            if !(0.0..1.0).contains(&sample.x) || !(0.0..1.0).contains(&sample.y) {
                continue;
            }
            let pixel = sample_image(&image.image, sample.x, sample.y);
            composite_image_pixel(device, x, y, pixel)?;
        }
    }
    Ok(())
}

fn transformed_image_bounds(transform: Matrix) -> PathBounds {
    let p0 = transform.transform_point(0.0, 0.0);
    let p1 = transform.transform_point(1.0, 0.0);
    let p2 = transform.transform_point(1.0, 1.0);
    let p3 = transform.transform_point(0.0, 1.0);
    let bounds = include_point(None, p0);
    let bounds = include_point(Some(bounds), p1);
    let bounds = include_point(Some(bounds), p2);
    include_point(Some(bounds), p3)
}

fn sample_image(image: &ImageXObject, x: f64, y: f64) -> Rgba {
    let sample_x = ((x * f64::from(image.width)).floor() as u32).min(image.width - 1);
    let sample_y = (((1.0 - y) * f64::from(image.height)).floor() as u32).min(image.height - 1);
    let index = (sample_y as usize * image.width as usize + sample_x as usize)
        * image.color_space.bytes_per_pixel();
    let alpha = sample_soft_mask(image, sample_x, sample_y);
    match image.color_space {
        ImageColorSpace::DeviceGray => {
            let channel = image.samples[index];
            Rgba {
                r: channel,
                g: channel,
                b: channel,
                a: alpha,
            }
        }
        ImageColorSpace::DeviceRgb => Rgba {
            r: image.samples[index],
            g: image.samples[index + 1],
            b: image.samples[index + 2],
            a: alpha,
        },
        ImageColorSpace::DeviceCmyk => {
            let mut rgba = cmyk_to_rgba(
                image.samples[index],
                image.samples[index + 1],
                image.samples[index + 2],
                image.samples[index + 3],
            );
            rgba.a = alpha;
            rgba
        }
        ImageColorSpace::IndexedGray => {
            let lookup = image.indexed_lookup.as_deref().unwrap_or_default();
            let index = usize::from(image.samples[index]).min(lookup.len().saturating_sub(1));
            let channel = lookup.get(index).copied().unwrap_or(0);
            Rgba {
                r: channel,
                g: channel,
                b: channel,
                a: alpha,
            }
        }
        ImageColorSpace::IndexedRgb => {
            let lookup = image.indexed_lookup.as_deref().unwrap_or_default();
            let lookup_index = usize::from(image.samples[index])
                .saturating_mul(3)
                .min(lookup.len());
            let rgb = lookup
                .get(lookup_index..lookup_index + 3)
                .unwrap_or(&[0, 0, 0]);
            Rgba {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: alpha,
            }
        }
    }
}

fn sample_soft_mask(image: &ImageXObject, sample_x: u32, sample_y: u32) -> u8 {
    let Some(mask) = &image.soft_mask else {
        return 255;
    };
    let index = sample_y as usize * image.width as usize + sample_x as usize;
    mask.get(index).copied().unwrap_or(255)
}

fn composite_image_pixel(
    device: &mut RasterDevice,
    x: u32,
    y: u32,
    source: Rgba,
) -> RasterResult<()> {
    if source.a == 255 {
        return device.set_pixel(x, y, source);
    }
    if source.a == 0 {
        return Ok(());
    }
    let dest = device.pixel(x, y)?;
    let coverage = f64::from(source.a) / 255.0;
    let inv = 1.0 - coverage;
    device.set_pixel(
        x,
        y,
        Rgba {
            r: blend_channel(source.r, dest.r, coverage, inv),
            g: blend_channel(source.g, dest.g, coverage, inv),
            b: blend_channel(source.b, dest.b, coverage, inv),
            a: f64::from(source.a)
                .mul_add(1.0, f64::from(dest.a) * inv)
                .round()
                .min(255.0) as u8,
        },
    )
}

fn cmyk_to_rgba(cyan: u8, magenta: u8, yellow: u8, key: u8) -> Rgba {
    Rgba {
        r: subtractive_channel_to_rgb(cyan, key),
        g: subtractive_channel_to_rgb(magenta, key),
        b: subtractive_channel_to_rgb(yellow, key),
        a: 255,
    }
}

fn subtractive_channel_to_rgb(channel: u8, key: u8) -> u8 {
    255u8.saturating_sub(channel.saturating_add(key))
}

fn draw_text_run(
    device: &mut RasterDevice,
    text: &TextDisplayItem,
    page_transform: PageTransform,
) -> RasterResult<()> {
    let Some(color) = text
        .rendering_mode
        .paint_color(text.state)
        .map(device_color_to_rgba)
    else {
        return Ok(());
    };
    let cell = text.font_size / 7.0;
    for (glyph, origin) in text.glyphs.iter().zip(text.glyph_origins.iter()) {
        let Some(character) = glyph.unicode.chars().next() else {
            continue;
        };
        if character == ' ' {
            continue;
        }
        draw_ascii_glyph(
            device,
            page_transform,
            character,
            origin.x,
            origin.y,
            cell,
            color,
        )?;
    }
    Ok(())
}

fn draw_ascii_glyph(
    device: &mut RasterDevice,
    page_transform: PageTransform,
    character: char,
    x: f64,
    baseline_y: f64,
    cell: f64,
    color: Rgba,
) -> RasterResult<()> {
    let glyph = ascii_glyph(character);
    for (row, pattern) in glyph.iter().enumerate() {
        for (col, byte) in pattern.as_bytes().iter().enumerate() {
            if *byte != b'#' {
                continue;
            }
            let left = x + col as f64 * cell;
            let right = left + cell;
            let top = baseline_y + (7 - row) as f64 * cell;
            let bottom = top - cell;
            fill_device_rect(
                device,
                page_transform.matrix.transform_point(left, top),
                page_transform.matrix.transform_point(right, bottom),
                color,
            )?;
        }
    }
    Ok(())
}

fn fill_device_rect(
    device: &mut RasterDevice,
    p0: Point,
    p1: Point,
    color: Rgba,
) -> RasterResult<()> {
    let dimensions = device.dimensions();
    let min_x = p0.x.min(p1.x).floor().max(0.0) as u32;
    let max_x = p0.x.max(p1.x).ceil().min(f64::from(dimensions.width)) as u32;
    let min_y = p0.y.min(p1.y).floor().max(0.0) as u32;
    let max_y = p0.y.max(p1.y).ceil().min(f64::from(dimensions.height)) as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            device.set_pixel(x, y, color)?;
        }
    }
    Ok(())
}

fn ascii_glyph(character: char) -> [&'static str; 7] {
    match character.to_ascii_lowercase() {
        'a' => [
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'b' => [
            "#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### ",
        ],
        'c' => [
            " ####", "#    ", "#    ", "#    ", "#    ", "#    ", " ####",
        ],
        'd' => [
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ],
        'e' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####",
        ],
        'f' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#    ",
        ],
        'g' => [
            " ####", "#    ", "#    ", "#  ##", "#   #", "#   #", " ####",
        ],
        'h' => [
            "#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'i' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####",
        ],
        'j' => [
            "#####", "    #", "    #", "    #", "#   #", "#   #", " ### ",
        ],
        'k' => [
            "#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #",
        ],
        'l' => [
            "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
        ],
        'm' => [
            "#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #",
        ],
        'n' => [
            "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #",
        ],
        'o' => [
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'p' => [
            "#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    ",
        ],
        'q' => [
            " ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #",
        ],
        'r' => [
            "#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #",
        ],
        's' => [
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ],
        't' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'u' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'v' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", " # # ", "  #  ",
        ],
        'w' => [
            "#   #", "#   #", "#   #", "# # #", "# # #", "## ##", "#   #",
        ],
        'x' => [
            "#   #", "#   #", " # # ", "  #  ", " # # ", "#   #", "#   #",
        ],
        'y' => [
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'z' => [
            "#####", "    #", "   # ", "  #  ", " #   ", "#    ", "#####",
        ],
        '0' => [
            " ### ", "#   #", "#  ##", "# # #", "##  #", "#   #", " ### ",
        ],
        '1' => [
            "  #  ", " ##  ", "# #  ", "  #  ", "  #  ", "  #  ", "#####",
        ],
        '2' => [
            " ### ", "#   #", "    #", "   # ", "  #  ", " #   ", "#####",
        ],
        '3' => [
            "#### ", "    #", "    #", " ### ", "    #", "    #", "#### ",
        ],
        '4' => [
            "#   #", "#   #", "#   #", "#####", "    #", "    #", "    #",
        ],
        '5' => [
            "#####", "#    ", "#    ", "#### ", "    #", "    #", "#### ",
        ],
        '6' => [
            " ####", "#    ", "#    ", "#### ", "#   #", "#   #", " ### ",
        ],
        '7' => [
            "#####", "    #", "   # ", "  #  ", " #   ", " #   ", " #   ",
        ],
        '8' => [
            " ### ", "#   #", "#   #", " ### ", "#   #", "#   #", " ### ",
        ],
        '9' => [
            " ### ", "#   #", "#   #", " ####", "    #", "    #", " ### ",
        ],
        _ => [
            "#####", "#   #", "   # ", "  #  ", "     ", "  #  ", "  #  ",
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageFilter {
    Raw,
    StreamDecoded,
    DctDecode,
}

fn image_filter(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> GraphicsResult<ImageFilter> {
    let Some(filter) =
        dictionary_value(dictionary, b"Filter").or_else(|| dictionary_value(dictionary, b"F"))
    else {
        return Ok(ImageFilter::Raw);
    };
    match filter {
        PdfPrimitive::Name(name) if name.as_bytes() == b"FlateDecode" => {
            Ok(ImageFilter::StreamDecoded)
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"DCTDecode" => Ok(ImageFilter::DctDecode),
        PdfPrimitive::Name(name) if is_deferred_image_codec(name.as_bytes()) => {
            Err(unsupported_image_filter(name.as_bytes()))
        }
        PdfPrimitive::Name(name) => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageFilter {
                filter: name.as_bytes().to_vec(),
            },
        )),
        PdfPrimitive::Array(filters) => {
            if filters.len() == 1 {
                if let PdfPrimitive::Name(name) = filters[0] {
                    if name.as_bytes() == b"FlateDecode" {
                        return Ok(ImageFilter::StreamDecoded);
                    }
                    if name.as_bytes() == b"DCTDecode" {
                        return Ok(ImageFilter::DctDecode);
                    }
                    if is_deferred_image_codec(name.as_bytes()) {
                        return Err(unsupported_image_filter(name.as_bytes()));
                    }
                    return Err(GraphicsError::new(
                        None,
                        GraphicsErrorKind::UnsupportedImageFilter {
                            filter: name.as_bytes().to_vec(),
                        },
                    ));
                }
            }
            Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedImageFilter {
                    filter: b"filter-chain".to_vec(),
                },
            ))
        }
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageFilter {
                filter: b"malformed-filter".to_vec(),
            },
        )),
    }
}

fn is_deferred_image_codec(filter: &[u8]) -> bool {
    matches!(
        filter,
        b"CCITTFaxDecode" | b"CCF" | b"JPXDecode" | b"JBIG2Decode"
    )
}

fn require_unfiltered_inline_image(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<()> {
    let Some(filter) =
        dictionary_value(dictionary, b"Filter").or_else(|| dictionary_value(dictionary, b"F"))
    else {
        return Ok(());
    };
    let filter = match filter {
        PdfPrimitive::Name(name) => name.as_bytes().to_vec(),
        PdfPrimitive::Array(_) => b"filter-chain".to_vec(),
        _ => b"malformed-filter".to_vec(),
    };
    Err(GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedImageFilter { filter },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageColorSpaceInfo {
    kind: ImageColorSpace,
    indexed_lookup: Option<Arc<[u8]>>,
}

impl ImageColorSpaceInfo {
    fn new(kind: ImageColorSpace) -> Self {
        Self {
            kind,
            indexed_lookup: None,
        }
    }
}

fn image_color_space(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<ImageColorSpaceInfo> {
    let Some(value) =
        dictionary_value(dictionary, b"ColorSpace").or_else(|| dictionary_value(dictionary, b"CS"))
    else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: b"missing".to_vec(),
            },
        ));
    };
    match value {
        PdfPrimitive::Name(name) if name.as_bytes() == b"DeviceRGB" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceRgb))
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"RGB" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceRgb))
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"DeviceCMYK" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceCmyk))
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"CMYK" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceCmyk))
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"DeviceGray" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceGray))
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"G" => {
            Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceGray))
        }
        PdfPrimitive::Array(values) => array_color_space(values),
        PdfPrimitive::Name(name) => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: name.as_bytes().to_vec(),
            },
        )),
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: b"malformed".to_vec(),
            },
        )),
    }
}

fn array_color_space(values: &[PdfPrimitive<'_>]) -> GraphicsResult<ImageColorSpaceInfo> {
    let Some(PdfPrimitive::Name(kind)) = values.first() else {
        return Err(unsupported_color_space(b"array"));
    };
    match kind.as_bytes() {
        b"Indexed" | b"I" => indexed_color_space(values),
        b"CalRGB" => Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceRgb)),
        b"CalGray" => Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceGray)),
        b"ICCBased" => Err(unsupported_color_space(b"ICCBased")),
        other => Err(unsupported_color_space(other)),
    }
}

fn indexed_color_space(values: &[PdfPrimitive<'_>]) -> GraphicsResult<ImageColorSpaceInfo> {
    if values.len() != 4 {
        return Err(unsupported_color_space(b"Indexed"));
    }
    let PdfPrimitive::Name(kind) = values[0] else {
        return Err(unsupported_color_space(b"Indexed"));
    };
    if !matches!(kind.as_bytes(), b"Indexed" | b"I") {
        return Err(unsupported_color_space(kind.as_bytes()));
    }
    let PdfPrimitive::Name(base) = values[1] else {
        return Err(unsupported_color_space(b"Indexed"));
    };
    let base_kind = match base.as_bytes() {
        b"DeviceRGB" | b"RGB" => ImageColorSpace::IndexedRgb,
        b"DeviceGray" | b"G" => ImageColorSpace::IndexedGray,
        _ => return Err(unsupported_color_space(base.as_bytes())),
    };
    let hival = match values[2] {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => {
            u8::try_from(value).map_err(|_| unsupported_color_space(b"Indexed"))?
        }
        _ => return Err(unsupported_color_space(b"Indexed")),
    };
    let lookup = indexed_lookup_bytes(&values[3])?;
    let components = base_kind
        .indexed_components()
        .ok_or_else(|| unsupported_color_space(b"Indexed"))?;
    let expected = (usize::from(hival) + 1)
        .checked_mul(components)
        .ok_or_else(|| unsupported_color_space(b"Indexed"))?;
    if lookup.len() < expected {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource {
                name: b"Indexed".to_vec(),
            },
        ));
    }
    Ok(ImageColorSpaceInfo {
        kind: base_kind,
        indexed_lookup: Some(Arc::from(&lookup[..expected])),
    })
}

fn indexed_lookup_bytes(value: &PdfPrimitive<'_>) -> GraphicsResult<Vec<u8>> {
    let PdfPrimitive::String(string) = value else {
        return Err(unsupported_color_space(b"Indexed"));
    };
    match string {
        PdfString::Literal(bytes) => Ok(bytes.to_vec()),
        PdfString::Hex(bytes) => decode_hex_bytes(bytes),
    }
}

fn unsupported_color_space(color_space: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedImageColorSpace {
            color_space: color_space.to_vec(),
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ImageDecode {
    ranges: [(f64, f64); 4],
    len: usize,
}

fn image_decode_ranges(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<Option<ImageDecode>> {
    let Some(value) =
        dictionary_value(dictionary, b"Decode").or_else(|| dictionary_value(dictionary, b"D"))
    else {
        return Ok(None);
    };
    let PdfPrimitive::Array(values) = value else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource {
                name: b"Decode".to_vec(),
            },
        ));
    };
    let mut ranges = [(0.0, 1.0); 4];
    let mut len = 0;
    let mut pairs = values.chunks_exact(2);
    for pair in &mut pairs {
        if len >= ranges.len() {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::InvalidImageResource {
                    name: b"Decode".to_vec(),
                },
            ));
        }
        let min = primitive_number(&pair[0]).ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::InvalidImageResource {
                    name: b"Decode".to_vec(),
                },
            )
        })?;
        let max = primitive_number(&pair[1]).ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::InvalidImageResource {
                    name: b"Decode".to_vec(),
                },
            )
        })?;
        ranges[len] = (min, max);
        len += 1;
    }
    if !pairs.remainder().is_empty() {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource {
                name: b"Decode".to_vec(),
            },
        ));
    }
    Ok(Some(ImageDecode { ranges, len }))
}

fn apply_image_decode(
    samples: &mut [u8],
    color_space: ImageColorSpace,
    decode: Option<ImageDecode>,
) -> GraphicsResult<()> {
    let Some(decode) = decode else {
        return Ok(());
    };
    let components = color_space.bytes_per_pixel();
    if decode.len != components {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource {
                name: b"Decode".to_vec(),
            },
        ));
    }
    for pixel in samples.chunks_exact_mut(components) {
        for (sample, (min, max)) in pixel.iter_mut().zip(decode.ranges[..decode.len].iter()) {
            let normalized = f64::from(*sample) / 255.0;
            let decoded = min + normalized * (max - min);
            *sample = match color_space {
                ImageColorSpace::IndexedGray | ImageColorSpace::IndexedRgb => {
                    decoded.clamp(0.0, 255.0).round() as u8
                }
                ImageColorSpace::DeviceGray
                | ImageColorSpace::DeviceRgb
                | ImageColorSpace::DeviceCmyk => (decoded.clamp(0.0, 1.0) * 255.0).round() as u8,
            };
        }
    }
    Ok(())
}

fn primitive_number(value: &PdfPrimitive<'_>) -> Option<f64> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Some(*value as f64),
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => Some(*value),
        _ => None,
    }
}

fn required_u32(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)], key: &[u8]) -> GraphicsResult<u32> {
    match dictionary_value(dictionary, key) {
        Some(PdfPrimitive::Number(PdfNumber::Integer(value))) => {
            u32::try_from(*value).map_err(|_| {
                GraphicsError::new(
                    None,
                    GraphicsErrorKind::InvalidImageResource { name: key.to_vec() },
                )
            })
        }
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageResource { name: key.to_vec() },
        )),
    }
}

fn required_u8(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)], key: &[u8]) -> GraphicsResult<u8> {
    required_u32(dictionary, key).and_then(|value| {
        u8::try_from(value).map_err(|_| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::InvalidImageResource { name: key.to_vec() },
            )
        })
    })
}

fn optional_matrix(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<Matrix>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    Ok(Some(matrix_from_array(value)?))
}

fn required_bbox(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<PathBounds> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource { name: key.to_vec() },
        ));
    };
    bounds_from_array(value)
}

fn matrix_from_array(value: &PdfPrimitive<'_>) -> GraphicsResult<Matrix> {
    let PdfPrimitive::Array(values) = value else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: b"Matrix".to_vec(),
            },
        ));
    };
    if values.len() != 6 {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: b"Matrix".to_vec(),
            },
        ));
    }
    Ok(Matrix::new(
        number_primitive(&values[0], b"Matrix")?,
        number_primitive(&values[1], b"Matrix")?,
        number_primitive(&values[2], b"Matrix")?,
        number_primitive(&values[3], b"Matrix")?,
        number_primitive(&values[4], b"Matrix")?,
        number_primitive(&values[5], b"Matrix")?,
    ))
}

fn bounds_from_array(value: &PdfPrimitive<'_>) -> GraphicsResult<PathBounds> {
    let PdfPrimitive::Array(values) = value else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: b"BBox".to_vec(),
            },
        ));
    };
    if values.len() != 4 {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: b"BBox".to_vec(),
            },
        ));
    }
    let x0 = number_primitive(&values[0], b"BBox")?;
    let y0 = number_primitive(&values[1], b"BBox")?;
    let x1 = number_primitive(&values[2], b"BBox")?;
    let y1 = number_primitive(&values[3], b"BBox")?;
    Ok(PathBounds {
        min_x: x0.min(x1),
        min_y: y0.min(y1),
        max_x: x0.max(x1),
        max_y: y0.max(y1),
    })
}

fn number_primitive(value: &PdfPrimitive<'_>, field: &'static [u8]) -> GraphicsResult<f64> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Ok(*value as f64),
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => Ok(*value),
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: field.to_vec(),
            },
        )),
    }
}

fn form_xobject_references(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> (Vec<XObjectReference>, bool) {
    let Some(PdfPrimitive::Dictionary(resources)) = dictionary_value(dictionary, b"Resources")
    else {
        return (Vec::new(), true);
    };
    let Some(PdfPrimitive::Dictionary(xobjects)) = dictionary_value(resources, b"XObject") else {
        return (Vec::new(), false);
    };
    (xobject_references(xobjects), false)
}

fn xobject_references(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> Vec<XObjectReference> {
    dictionary
        .iter()
        .filter_map(|(name, value)| {
            Some(XObjectReference {
                name: name.as_bytes().to_vec(),
                reference: reference_from_primitive(value)?,
            })
        })
        .collect()
}

type EmbeddedFontEntry<'a> = (&'static [u8], &'a PdfPrimitive<'a>, Option<FontProgramKind>);

fn font_encoding(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> GraphicsResult<FontEncoding> {
    let Some(encoding) = dictionary_value(dictionary, b"Encoding") else {
        return Ok(FontEncoding::new());
    };
    match encoding {
        PdfPrimitive::Name(name)
            if matches!(
                name.as_bytes(),
                b"WinAnsiEncoding" | b"MacRomanEncoding" | b"MacExpertEncoding"
            ) =>
        {
            Ok(FontEncoding::new())
        }
        PdfPrimitive::Dictionary(dictionary) => encoding_dictionary(dictionary),
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedTextEncodingFeature {
                feature: b"font Encoding".to_vec(),
            },
        )),
    }
}

fn encoding_dictionary(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<FontEncoding> {
    if let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, b"BaseEncoding") {
        if !matches!(
            name.as_bytes(),
            b"WinAnsiEncoding" | b"MacRomanEncoding" | b"MacExpertEncoding"
        ) {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedTextEncodingFeature {
                    feature: name.as_bytes().to_vec(),
                },
            ));
        }
    }
    let Some(PdfPrimitive::Array(values)) = dictionary_value(dictionary, b"Differences") else {
        return Ok(FontEncoding::new());
    };
    let mut code: Option<u8> = None;
    let mut differences = Vec::new();
    for value in values {
        match value {
            PdfPrimitive::Number(PdfNumber::Integer(next_code))
                if (0..=255).contains(next_code) =>
            {
                code = Some(*next_code as u8);
            }
            PdfPrimitive::Name(name) => {
                let Some(current_code) = code else {
                    return Err(invalid_font_resource(b"Encoding"));
                };
                let character = glyph_name_to_char(name.as_bytes()).ok_or_else(|| {
                    GraphicsError::new(
                        None,
                        GraphicsErrorKind::UnsupportedTextEncodingFeature {
                            feature: name.as_bytes().to_vec(),
                        },
                    )
                })?;
                differences.push((current_code, character));
                code = current_code.checked_add(1);
            }
            _ => return Err(invalid_font_resource(b"Encoding")),
        }
    }
    Ok(FontEncoding::with_differences(differences))
}

fn glyph_name_to_char(name: &[u8]) -> Option<char> {
    match name {
        [b'A'..=b'Z'] | [b'a'..=b'z'] | [b'0'..=b'9'] => Some(name[0] as char),
        b"space" => Some(' '),
        b"hyphen" => Some('-'),
        b"period" => Some('.'),
        b"comma" => Some(','),
        b"colon" => Some(':'),
        b"semicolon" => Some(';'),
        b"parenleft" => Some('('),
        b"parenright" => Some(')'),
        _ => None,
    }
}

fn parse_to_unicode_cmap(bytes: &[u8], entry_limit: usize) -> GraphicsResult<ToUnicodeMap> {
    let mut entries = Vec::new();
    let mut mode = CMapSection::None;
    for raw_line in bytes.split(|byte| matches!(byte, b'\n' | b'\r')) {
        let line = trim_cmap_comment(raw_line);
        if line.is_empty() {
            continue;
        }
        if contains_word(line, b"beginbfchar") {
            mode = CMapSection::BfChar;
            continue;
        }
        if contains_word(line, b"beginbfrange") {
            mode = CMapSection::BfRange;
            continue;
        }
        if contains_word(line, b"endbfchar") || contains_word(line, b"endbfrange") {
            mode = CMapSection::None;
            continue;
        }
        match mode {
            CMapSection::None => {
                if contains_word(line, b"usecmap") {
                    return Err(GraphicsError::new(
                        None,
                        GraphicsErrorKind::UnsupportedCMap {
                            feature: b"usecmap".to_vec(),
                        },
                    ));
                }
            }
            CMapSection::BfChar => {
                let hex_values = collect_hex_strings(line)?;
                for pair in hex_values.chunks_exact(2) {
                    push_cmap_entry(&mut entries, pair[0].clone(), &pair[1], entry_limit)?;
                }
            }
            CMapSection::BfRange => {
                let hex_values = collect_hex_strings(line)?;
                if hex_values.len() >= 3 {
                    push_cmap_range(&mut entries, &hex_values, entry_limit)?;
                }
            }
        }
    }
    Ok(ToUnicodeMap::new(entries))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CMapSection {
    None,
    BfChar,
    BfRange,
}

fn push_cmap_range(
    entries: &mut Vec<ToUnicodeEntry>,
    hex_values: &[Vec<u8>],
    entry_limit: usize,
) -> GraphicsResult<()> {
    let start = bytes_to_u32(&hex_values[0]);
    let end = bytes_to_u32(&hex_values[1]);
    if end < start || hex_values[0].len() != hex_values[1].len() {
        return Err(invalid_cmap());
    }
    let count = end - start + 1;
    if hex_values.len() == 3 {
        let Some(first_char) = unicode_hex_to_string(&hex_values[2])?.chars().next() else {
            return Err(invalid_cmap());
        };
        for offset in 0..count {
            let code = u32_to_sized_bytes(start + offset, hex_values[0].len());
            let Some(character) = char::from_u32(first_char as u32 + offset) else {
                return Err(invalid_cmap());
            };
            push_cmap_entry(entries, code, &char_to_utf16be(character), entry_limit)?;
        }
    } else {
        if count as usize != hex_values.len() - 2 {
            return Err(invalid_cmap());
        }
        for (offset, destination) in hex_values[2..].iter().enumerate() {
            let code = u32_to_sized_bytes(start + offset as u32, hex_values[0].len());
            push_cmap_entry(entries, code, destination, entry_limit)?;
        }
    }
    Ok(())
}

fn push_cmap_entry(
    entries: &mut Vec<ToUnicodeEntry>,
    code: Vec<u8>,
    destination: &[u8],
    entry_limit: usize,
) -> GraphicsResult<()> {
    if entries.len() >= entry_limit {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::CMapEntriesOverflow { limit: entry_limit },
        ));
    }
    if code.is_empty() {
        return Err(invalid_cmap());
    }
    let text = unicode_hex_to_string(destination)?;
    entries.push(ToUnicodeEntry { code, text });
    Ok(())
}

fn collect_hex_strings(line: &[u8]) -> GraphicsResult<Vec<Vec<u8>>> {
    let mut values = Vec::new();
    let mut index = 0;
    while index < line.len() {
        if line[index] == b'<' && line.get(index + 1) != Some(&b'<') {
            let start = index + 1;
            let Some(end_offset) = line[start..].iter().position(|byte| *byte == b'>') else {
                return Err(invalid_cmap());
            };
            let end = start + end_offset;
            values.push(decode_hex_bytes(&line[start..end])?);
            index = end + 1;
        } else {
            index += 1;
        }
    }
    Ok(values)
}

fn decode_hex_bytes(hex: &[u8]) -> GraphicsResult<Vec<u8>> {
    let mut bytes = Vec::with_capacity(hex.len().div_ceil(2));
    let mut high_nibble = None;
    for byte in hex
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        let nibble = hex_nibble(byte).ok_or_else(invalid_cmap)?;
        if let Some(high) = high_nibble.take() {
            bytes.push((high << 4) | nibble);
        } else {
            high_nibble = Some(nibble);
        }
    }
    if let Some(high) = high_nibble {
        bytes.push(high << 4);
    }
    Ok(bytes)
}

fn unicode_hex_to_string(bytes: &[u8]) -> GraphicsResult<String> {
    if bytes.is_empty() || bytes.len() % 2 != 0 {
        return Err(invalid_cmap());
    }
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
    char::decode_utf16(units)
        .collect::<Result<String, _>>()
        .map_err(|_| invalid_cmap())
}

fn char_to_utf16be(character: char) -> Vec<u8> {
    let mut units = [0; 2];
    character
        .encode_utf16(&mut units)
        .iter()
        .flat_map(|unit| unit.to_be_bytes())
        .collect()
}

fn trim_cmap_comment(line: &[u8]) -> &[u8] {
    let uncommented = line
        .iter()
        .position(|byte| *byte == b'%')
        .map_or(line, |index| &line[..index]);
    trim_ascii(uncommented)
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn contains_word(line: &[u8], word: &[u8]) -> bool {
    line.windows(word.len()).any(|window| window == word)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |value, byte| (value << 8) | u32::from(*byte))
}

fn u32_to_sized_bytes(value: u32, size: usize) -> Vec<u8> {
    (0..size)
        .rev()
        .map(|index| ((value >> (index * 8)) & 0xff) as u8)
        .collect()
}

fn embedded_font_program_entry<'a>(
    descriptor: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
) -> Option<EmbeddedFontEntry<'a>> {
    dictionary_value(descriptor, b"FontFile")
        .map(|value| (b"FontFile".as_slice(), value, Some(FontProgramKind::Type1)))
        .or_else(|| {
            dictionary_value(descriptor, b"FontFile2").map(|value| {
                (
                    b"FontFile2".as_slice(),
                    value,
                    Some(FontProgramKind::TrueType),
                )
            })
        })
        .or_else(|| {
            dictionary_value(descriptor, b"FontFile3")
                .map(|value| (b"FontFile3".as_slice(), value, None))
        })
}

fn optional_name(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)], key: &[u8]) -> Option<Vec<u8>> {
    match dictionary_value(dictionary, key) {
        Some(PdfPrimitive::Name(name)) => Some(name.as_bytes().to_vec()),
        _ => None,
    }
}

fn optional_font_subtype(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> Option<FontSubtype> {
    let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, b"Subtype") else {
        return None;
    };
    match name.as_bytes() {
        b"Type1" => Some(FontSubtype::Type1),
        b"TrueType" => Some(FontSubtype::TrueType),
        b"Type0" => Some(FontSubtype::Type0),
        b"Type3" => Some(FontSubtype::Type3),
        b"CIDFontType0" => Some(FontSubtype::CidFontType0),
        b"CIDFontType2" => Some(FontSubtype::CidFontType2),
        _ => None,
    }
}

fn font_file3_program_kind(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<FontProgramKind> {
    let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, b"Subtype") else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedFontProgram {
                name: b"FontFile3".to_vec(),
            },
        ));
    };
    match name.as_bytes() {
        b"Type1C" | b"CIDFontType0C" => Ok(FontProgramKind::Cff),
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedFontProgram {
                name: name.as_bytes().to_vec(),
            },
        )),
    }
}

fn dictionary_value<'d, 'a>(
    dictionary: &'d [(PdfName<'a>, PdfPrimitive<'a>)],
    key: &[u8],
) -> Option<&'d PdfPrimitive<'a>> {
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

fn reference_from_primitive(value: &PdfPrimitive<'_>) -> Option<Reference> {
    let PdfPrimitive::Reference(reference) = *value else {
        return None;
    };
    let number = ObjectNumber::new(reference.object).ok()?;
    let generation = GenerationNumber::new(reference.generation);
    Some(Reference::new(ObjectId::new(number, generation)))
}

fn unit_square_bounds(transform: Matrix) -> PathBounds {
    let p0 = transform.transform_point(0.0, 0.0);
    let p1 = transform.transform_point(1.0, 0.0);
    let p2 = transform.transform_point(1.0, 1.0);
    let p3 = transform.transform_point(0.0, 1.0);
    let bounds = include_point(None, p0);
    let bounds = include_point(Some(bounds), p1);
    let bounds = include_point(Some(bounds), p2);
    include_point(Some(bounds), p3)
}

fn scaled_dimension(value: f64, scale: f64, max_edge: u32) -> RasterResult<u32> {
    let scaled = value * scale;
    if !scaled.is_finite() || scaled <= 0.0 || scaled > f64::from(u32::MAX) {
        return Err(RasterError::new(RasterErrorKind::DimensionOverflow));
    }
    Ok((scaled.round() as u32).clamp(1, max_edge))
}

fn page_to_pixel_matrix(bounds: PathBounds, rotation: PageRotation, scale: f64) -> Matrix {
    match rotation {
        PageRotation::Deg0 => Matrix::new(
            scale,
            0.0,
            0.0,
            -scale,
            -bounds.min_x * scale,
            bounds.max_y * scale,
        ),
        PageRotation::Deg90 => Matrix::new(
            0.0,
            scale,
            scale,
            0.0,
            -bounds.min_y * scale,
            -bounds.min_x * scale,
        ),
        PageRotation::Deg180 => Matrix::new(
            -scale,
            0.0,
            0.0,
            scale,
            bounds.max_x * scale,
            -bounds.min_y * scale,
        ),
        PageRotation::Deg270 => Matrix::new(
            0.0,
            -scale,
            -scale,
            0.0,
            bounds.max_y * scale,
            bounds.max_x * scale,
        ),
    }
}

fn transformed_box_segments(bounds: PathBounds, transform: Matrix) -> Vec<PathSegment> {
    let p0 = transform.transform_point(bounds.min_x, bounds.min_y);
    let p1 = transform.transform_point(bounds.max_x, bounds.min_y);
    let p2 = transform.transform_point(bounds.max_x, bounds.max_y);
    let p3 = transform.transform_point(bounds.min_x, bounds.max_y);
    vec![
        PathSegment::MoveTo(p0),
        PathSegment::LineTo(p1),
        PathSegment::LineTo(p2),
        PathSegment::LineTo(p3),
        PathSegment::Close,
    ]
}

fn invalid_operand(offset: ByteOffset, operator: &'static [u8]) -> GraphicsError {
    GraphicsError::new(Some(offset), GraphicsErrorKind::InvalidOperand { operator })
}

fn invalid_image_resource(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidImageResource {
            name: name.to_vec(),
        },
    )
}

fn unsupported_image_filter(filter: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedImageFilter {
            filter: filter.to_vec(),
        },
    )
}

fn invalid_font_resource(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidFontResource {
            name: name.to_vec(),
        },
    )
}

fn invalid_cmap() -> GraphicsError {
    GraphicsError::new(None, GraphicsErrorKind::InvalidCMap)
}

fn invalid_glyph_outline() -> GraphicsError {
    GraphicsError::new(None, GraphicsErrorKind::InvalidGlyphOutline)
}

fn missing_current_point(offset: ByteOffset, operator: &'static [u8]) -> GraphicsError {
    GraphicsError::new(
        Some(offset),
        GraphicsErrorKind::MissingCurrentPoint { operator },
    )
}

fn include_point(bounds: Option<PathBounds>, point: Point) -> PathBounds {
    match bounds {
        Some(bounds) => PathBounds {
            min_x: bounds.min_x.min(point.x),
            min_y: bounds.min_y.min(point.y),
            max_x: bounds.max_x.max(point.x),
            max_y: bounds.max_y.max(point.y),
        },
        None => PathBounds {
            min_x: point.x,
            min_y: point.y,
            max_x: point.x,
            max_y: point.y,
        },
    }
}

/// Raster-device setup error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterError {
    kind: RasterErrorKind,
}

impl RasterError {
    /// Creates a raster error.
    #[must_use]
    pub const fn new(kind: RasterErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the error kind.
    #[must_use]
    pub const fn kind(&self) -> &RasterErrorKind {
        &self.kind
    }
}

impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl std::error::Error for RasterError {}

/// Raster-device setup error category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterErrorKind {
    /// Width or height was zero.
    InvalidDimensions,
    /// `max_edge` was zero.
    InvalidMaxEdge,
    /// Supersampling factor was zero.
    InvalidSupersampling,
    /// Page box dimensions are non-finite, empty, or inverted.
    InvalidPageBox,
    /// Rounded output dimension overflowed `u32`.
    DimensionOverflow,
    /// Row stride overflowed `usize`.
    StrideOverflow,
    /// Pixel buffer length overflowed `usize`.
    BufferOverflow,
    /// Flattened path complexity exceeded the configured limit.
    PathComplexityOverflow {
        /// Configured flattened line segment limit.
        limit: usize,
    },
    /// Image transform could not be inverted for sampling.
    SingularImageTransform,
    /// Row or pixel coordinate was outside the raster.
    OutOfBounds,
}

impl fmt::Display for RasterErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("raster dimensions must be non-zero"),
            Self::InvalidMaxEdge => f.write_str("max_edge must be greater than zero"),
            Self::InvalidSupersampling => f.write_str("supersampling must be greater than zero"),
            Self::InvalidPageBox => f.write_str("page box is invalid"),
            Self::DimensionOverflow => f.write_str("raster dimension overflow"),
            Self::StrideOverflow => f.write_str("raster stride overflow"),
            Self::BufferOverflow => f.write_str("raster buffer size overflow"),
            Self::PathComplexityOverflow { limit } => {
                write!(f, "flattened path exceeds segment limit {limit}")
            }
            Self::SingularImageTransform => f.write_str("image transform is singular"),
            Self::OutOfBounds => f.write_str("raster coordinate is out of bounds"),
        }
    }
}

/// Graphics-state interpretation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsError {
    offset: Option<ByteOffset>,
    kind: GraphicsErrorKind,
}

impl GraphicsError {
    /// Creates a graphics-state error.
    #[must_use]
    pub const fn new(offset: Option<ByteOffset>, kind: GraphicsErrorKind) -> Self {
        Self { offset, kind }
    }

    /// Returns the source offset when available.
    #[must_use]
    pub const fn offset(&self) -> Option<ByteOffset> {
        self.offset
    }

    /// Returns the error kind.
    #[must_use]
    pub const fn kind(&self) -> &GraphicsErrorKind {
        &self.kind
    }

    fn from_content(error: pdfrust_content::ContentError) -> Self {
        Self {
            offset: Some(error.offset()),
            kind: GraphicsErrorKind::Content(error.kind()),
        }
    }
}

impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} at {offset}", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for GraphicsError {}

/// Graphics-state interpretation error category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsErrorKind {
    /// Underlying content tokenizer error.
    Content(ContentErrorKind),
    /// `Q` was used with no saved state.
    StackUnderflow,
    /// `q` would exceed the configured stack depth.
    StackOverflow {
        /// Configured stack depth limit.
        limit: usize,
    },
    /// Supported operator received the wrong number of operands.
    OperandCount {
        /// Operator bytes.
        operator: &'static [u8],
        /// Expected operand count.
        expected: usize,
        /// Actual operand count.
        actual: usize,
    },
    /// Supported operator received an operand with the wrong type or range.
    InvalidOperand {
        /// Operator bytes.
        operator: &'static [u8],
    },
    /// Current path exceeds the configured segment limit.
    PathSegmentOverflow {
        /// Configured path segment limit.
        limit: usize,
    },
    /// Display list exceeds the configured item limit.
    DisplayListOverflow {
        /// Configured display item limit.
        limit: usize,
    },
    /// Path operator requires a current point but no current point exists.
    MissingCurrentPoint {
        /// Operator bytes.
        operator: &'static [u8],
    },
    /// Path operator is intentionally unsupported in this milestone.
    UnsupportedPathOperator {
        /// Operator bytes.
        operator: &'static [u8],
    },
    /// Text operator appeared outside `BT`/`ET`.
    TextOutsideObject {
        /// Operator bytes.
        operator: &'static [u8],
    },
    /// `BT` appeared before the previous text object ended.
    TextObjectAlreadyOpen,
    /// `Tj` or `TJ` appeared before a font was selected.
    FontNotSelected,
    /// A selected font resource does not exist.
    MissingFont {
        /// Missing PDF font resource name.
        name: Vec<u8>,
    },
    /// Referenced font object was not present in the loaded document.
    MissingFontObject {
        /// Missing font resource name.
        name: Vec<u8>,
    },
    /// Font resource dictionary or descriptor metadata is malformed.
    InvalidFontResource {
        /// Resource or field name related to the failure.
        name: Vec<u8>,
    },
    /// Embedded font program kind is unsupported by this milestone.
    UnsupportedFontProgram {
        /// Unsupported font program subtype or field name.
        name: Vec<u8>,
    },
    /// Decoded embedded font program exceeds the configured limit.
    FontProgramBytesOverflow {
        /// Configured decoded font program byte limit.
        limit: usize,
    },
    /// Font program kind cannot produce outlines in the current extractor.
    UnsupportedGlyphOutlineProgram {
        /// Unsupported font program kind.
        kind: FontProgramKind,
    },
    /// Glyph outline format uses a feature outside the current support.
    UnsupportedGlyphOutline {
        /// Unsupported outline feature name.
        feature: Vec<u8>,
    },
    /// Glyph outline data is malformed for the supported subset.
    InvalidGlyphOutline,
    /// Decoded glyph outline exceeds the configured segment limit.
    GlyphOutlineSegmentOverflow {
        /// Configured outline segment limit.
        limit: usize,
    },
    /// Glyph outline cache reached the configured entry limit.
    GlyphOutlineCacheOverflow {
        /// Configured cache entry limit.
        limit: usize,
    },
    /// Text string uses an encoding outside the current ASCII stub policy.
    UnsupportedTextEncoding,
    /// Font encoding metadata uses a feature outside the current support.
    UnsupportedTextEncodingFeature {
        /// Unsupported encoding feature name.
        feature: Vec<u8>,
    },
    /// ToUnicode CMap uses a feature outside the current support.
    UnsupportedCMap {
        /// Unsupported CMap feature name.
        feature: Vec<u8>,
    },
    /// ToUnicode CMap syntax is malformed for the supported subset.
    InvalidCMap,
    /// Decoded ToUnicode CMap exceeds the configured byte limit.
    CMapBytesOverflow {
        /// Configured decoded CMap byte limit.
        limit: usize,
    },
    /// Parsed ToUnicode CMap exceeds the configured entry limit.
    CMapEntriesOverflow {
        /// Configured CMap entry limit.
        limit: usize,
    },
    /// No text mapping exists for a source character code.
    MissingTextMapping {
        /// Unmapped source character-code bytes.
        code: Vec<u8>,
    },
    /// Decoded text run exceeds the configured limit.
    TextRunOverflow {
        /// Configured text run byte limit.
        limit: usize,
    },
    /// Image resource name was not present in the resource map.
    MissingImage {
        /// Missing image resource name.
        name: Vec<u8>,
    },
    /// Referenced image object was not present in the loaded document.
    MissingImageObject {
        /// Missing image resource name.
        name: Vec<u8>,
    },
    /// Image resource dictionary or stream metadata is malformed.
    InvalidImageResource {
        /// Resource or field name related to the failure.
        name: Vec<u8>,
    },
    /// Image color space is unsupported by this milestone.
    UnsupportedImageColorSpace {
        /// Unsupported color-space name.
        color_space: Vec<u8>,
    },
    /// Image filter is unsupported by this milestone.
    UnsupportedImageFilter {
        /// Unsupported filter name.
        filter: Vec<u8>,
    },
    /// Decoded image data length does not match metadata.
    InvalidImageDataLength {
        /// Expected decoded byte length.
        expected: usize,
        /// Actual decoded byte length.
        actual: usize,
    },
    /// Decoded image data exceeds the configured limit.
    ImageBytesOverflow {
        /// Configured decoded image byte limit.
        limit: usize,
    },
    /// Soft-mask recursion exceeds the configured limit.
    SoftMaskDepthOverflow {
        /// Configured soft-mask depth limit.
        limit: usize,
    },
    /// Soft-mask metadata uses a feature outside the current support.
    UnsupportedSoftMask {
        /// Unsupported soft-mask feature name.
        feature: Vec<u8>,
    },
    /// Form resource name was not present in the resource map.
    MissingForm {
        /// Missing form resource name.
        name: Vec<u8>,
    },
    /// Referenced form object was not present in the loaded document.
    MissingFormObject {
        /// Missing form resource name.
        name: Vec<u8>,
    },
    /// Form resource dictionary or stream metadata is malformed.
    InvalidFormResource {
        /// Resource or field name related to the failure.
        name: Vec<u8>,
    },
    /// Form XObject recursion exceeds the configured limit.
    FormRecursionOverflow {
        /// Configured form recursion depth limit.
        limit: usize,
    },
    /// Lower object model error surfaced during resource resolution.
    ObjectModel {
        /// Stable object-model error message.
        message: String,
    },
}

impl fmt::Display for GraphicsErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(kind) => write!(f, "content tokenizer error: {kind}"),
            Self::StackUnderflow => f.write_str("graphics state stack underflow"),
            Self::StackOverflow { limit } => {
                write!(f, "graphics state stack exceeds limit {limit}")
            }
            Self::OperandCount {
                operator,
                expected,
                actual,
            } => write!(
                f,
                "operator {} expected {expected} operands but received {actual}",
                String::from_utf8_lossy(operator)
            ),
            Self::InvalidOperand { operator } => write!(
                f,
                "operator {} received invalid operands",
                String::from_utf8_lossy(operator)
            ),
            Self::PathSegmentOverflow { limit } => {
                write!(f, "current path exceeds segment limit {limit}")
            }
            Self::DisplayListOverflow { limit } => {
                write!(f, "display list exceeds item limit {limit}")
            }
            Self::MissingCurrentPoint { operator } => write!(
                f,
                "operator {} requires a current point",
                String::from_utf8_lossy(operator)
            ),
            Self::UnsupportedPathOperator { operator } => write!(
                f,
                "operator {} is unsupported for path display lists",
                String::from_utf8_lossy(operator)
            ),
            Self::TextOutsideObject { operator } => write!(
                f,
                "operator {} requires an open text object",
                String::from_utf8_lossy(operator)
            ),
            Self::TextObjectAlreadyOpen => f.write_str("text object is already open"),
            Self::FontNotSelected => f.write_str("text font has not been selected"),
            Self::MissingFont { name } => {
                write!(f, "missing font resource {}", String::from_utf8_lossy(name))
            }
            Self::MissingFontObject { name } => write!(
                f,
                "missing font object for resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidFontResource { name } => {
                write!(f, "invalid font resource {}", String::from_utf8_lossy(name))
            }
            Self::UnsupportedFontProgram { name } => write!(
                f,
                "unsupported font program {}",
                String::from_utf8_lossy(name)
            ),
            Self::FontProgramBytesOverflow { limit } => {
                write!(f, "decoded font program exceeds byte limit {limit}")
            }
            Self::UnsupportedGlyphOutlineProgram { kind } => {
                write!(f, "unsupported glyph outline font program {kind:?}")
            }
            Self::UnsupportedGlyphOutline { feature } => write!(
                f,
                "unsupported glyph outline feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::InvalidGlyphOutline => f.write_str("invalid glyph outline data"),
            Self::GlyphOutlineSegmentOverflow { limit } => {
                write!(f, "decoded glyph outline exceeds segment limit {limit}")
            }
            Self::GlyphOutlineCacheOverflow { limit } => {
                write!(f, "glyph outline cache exceeds entry limit {limit}")
            }
            Self::UnsupportedTextEncoding => f.write_str("unsupported text encoding"),
            Self::UnsupportedTextEncodingFeature { feature } => write!(
                f,
                "unsupported text encoding feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::UnsupportedCMap { feature } => {
                write!(
                    f,
                    "unsupported ToUnicode CMap feature {}",
                    String::from_utf8_lossy(feature)
                )
            }
            Self::InvalidCMap => f.write_str("invalid ToUnicode CMap"),
            Self::CMapBytesOverflow { limit } => {
                write!(f, "decoded ToUnicode CMap exceeds byte limit {limit}")
            }
            Self::CMapEntriesOverflow { limit } => {
                write!(f, "ToUnicode CMap exceeds entry limit {limit}")
            }
            Self::MissingTextMapping { code } => {
                write!(f, "missing text mapping for character code {code:x?}")
            }
            Self::TextRunOverflow { limit } => {
                write!(f, "decoded text run exceeds byte limit {limit}")
            }
            Self::MissingImage { name } => {
                write!(
                    f,
                    "missing image resource {}",
                    String::from_utf8_lossy(name)
                )
            }
            Self::MissingImageObject { name } => write!(
                f,
                "missing image object for resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidImageResource { name } => write!(
                f,
                "invalid image resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::UnsupportedImageColorSpace { color_space } => write!(
                f,
                "unsupported image color space {}",
                String::from_utf8_lossy(color_space)
            ),
            Self::UnsupportedImageFilter { filter } => {
                write!(
                    f,
                    "unsupported image filter {}",
                    String::from_utf8_lossy(filter)
                )
            }
            Self::InvalidImageDataLength { expected, actual } => write!(
                f,
                "invalid decoded image length: expected {expected} bytes but got {actual}"
            ),
            Self::ImageBytesOverflow { limit } => {
                write!(f, "decoded image exceeds byte limit {limit}")
            }
            Self::SoftMaskDepthOverflow { limit } => {
                write!(f, "soft-mask recursion exceeds depth limit {limit}")
            }
            Self::UnsupportedSoftMask { feature } => write!(
                f,
                "unsupported soft-mask feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::MissingForm { name } => {
                write!(f, "missing form resource {}", String::from_utf8_lossy(name))
            }
            Self::MissingFormObject { name } => write!(
                f,
                "missing form object for resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidFormResource { name } => {
                write!(f, "invalid form resource {}", String::from_utf8_lossy(name))
            }
            Self::FormRecursionOverflow { limit } => {
                write!(f, "form recursion exceeds depth limit {limit}")
            }
            Self::ObjectModel { message } => write!(f, "object model error: {message}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfrust_object::{
        load_classic_document, GenerationNumber, ObjectId, ObjectNumber, ObjectValue,
    };

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "render");
    }

    #[test]
    fn render_should_depend_on_content() {
        assert_eq!(content_role(), "content");
    }

    #[test]
    fn render_should_use_thumbnail_facade_pixel_layout() {
        assert_eq!(facade_rgba_bytes_per_pixel(), 4);
    }

    #[test]
    fn raster_dimensions_should_compute_stride_and_reject_empty_dimensions() {
        let dimensions = RasterDimensions::new(300, 160).expect("valid dimensions");

        assert_eq!(
            dimensions,
            RasterDimensions {
                width: 300,
                height: 160,
                stride: 1200,
            }
        );
        let error = RasterDimensions::new(0, 160).expect_err("zero width should fail");
        assert_eq!(error.kind(), &RasterErrorKind::InvalidDimensions);
    }

    #[test]
    fn raster_dimensions_should_report_buffer_overflow_without_allocating() {
        let error = RasterDimensions::new(u32::MAX, u32::MAX)
            .expect_err("huge dimensions should overflow buffer length");

        assert_eq!(error.kind(), &RasterErrorKind::BufferOverflow);
    }

    #[test]
    fn raster_device_should_fill_background_and_expose_safe_accessors() {
        let background = Rgba {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        };
        let mut device = RasterDevice::new(2, 2, background).expect("valid raster");

        assert_eq!(device.dimensions().stride, 8);
        assert_eq!(
            device.row(1).expect("second row"),
            &[10, 20, 30, 255, 10, 20, 30, 255]
        );
        device
            .set_pixel(
                1,
                0,
                Rgba {
                    r: 1,
                    g: 2,
                    b: 3,
                    a: 4,
                },
            )
            .expect("in-bounds pixel write");
        assert_eq!(
            device.pixel(1, 0).expect("in-bounds pixel read"),
            Rgba {
                r: 1,
                g: 2,
                b: 3,
                a: 4,
            }
        );
        let error = device
            .pixel(2, 0)
            .expect_err("x outside raster should fail");
        assert_eq!(error.kind(), &RasterErrorKind::OutOfBounds);
    }

    #[test]
    fn page_transform_should_match_pdfium_max_edge_scaling() {
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 300.0,
                    max_y: 160.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            256,
        )
        .expect("valid page transform");

        assert_eq!(transform.dimensions.width, 256);
        assert_eq!(transform.dimensions.height, 137);
        assert!((transform.scale - (256.0 / 300.0)).abs() < 0.000_001);
    }

    #[test]
    fn page_transform_should_apply_crop_box_to_device_mapping() {
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 300.0,
                    max_y: 160.0,
                },
                crop_box: Some(PathBounds {
                    min_x: 10.0,
                    min_y: 20.0,
                    max_x: 110.0,
                    max_y: 120.0,
                }),
                rotation: PageRotation::Deg0,
            },
            100,
        )
        .expect("valid cropped page transform");

        assert_eq!(transform.dimensions.width, 100);
        assert_eq!(transform.dimensions.height, 100);
        assert_eq!(
            transform.matrix.transform_point(10.0, 120.0),
            Point { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            transform.matrix.transform_point(110.0, 20.0),
            Point { x: 100.0, y: 100.0 }
        );
    }

    #[test]
    fn page_transform_should_swap_dimensions_for_quarter_turn_rotation() {
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 200.0,
                    max_y: 100.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg90,
            },
            200,
        )
        .expect("valid rotated page transform");

        assert_eq!(transform.dimensions.width, 100);
        assert_eq!(transform.dimensions.height, 200);
        assert_eq!(
            transform.matrix.transform_point(0.0, 0.0),
            Point { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            transform.matrix.transform_point(200.0, 100.0),
            Point { x: 100.0, y: 200.0 }
        );
    }

    #[test]
    fn page_transform_should_reject_invalid_inputs() {
        let geometry = PageGeometry {
            media_box: PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 0.0,
                max_y: 160.0,
            },
            crop_box: None,
            rotation: PageRotation::Deg0,
        };

        assert_eq!(
            PageTransform::new(geometry, 256)
                .expect_err("empty page box should fail")
                .kind(),
            &RasterErrorKind::InvalidPageBox
        );
        assert_eq!(
            PageTransform::new(
                PageGeometry {
                    media_box: PathBounds {
                        min_x: 0.0,
                        min_y: 0.0,
                        max_x: 300.0,
                        max_y: 160.0,
                    },
                    crop_box: None,
                    rotation: PageRotation::Deg0,
                },
                0,
            )
            .expect_err("zero max_edge should fail")
            .kind(),
            &RasterErrorKind::InvalidMaxEdge
        );
    }

    #[test]
    fn path_rasterizer_should_draw_generated_vector_fixture() {
        let decoded = generated_fixture_content("vector-paths.pdf");
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            DisplayListOptions::default(),
        )
        .expect("vector fixture display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 220.0,
                    max_y: 180.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            220,
        )
        .expect("vector fixture transform");
        let raster = rasterize_paths(&list, transform, Rgba::WHITE, PathRasterOptions::default())
            .expect("vector fixture should rasterize");

        assert_eq!(raster.dimensions().width, 220);
        assert_eq!(raster.dimensions().height, 180);
        assert!(raster
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
        assert_eq!(
            raster.pixel(100, 100).expect("filled rectangle pixel"),
            Rgba {
                r: 230,
                g: 51,
                b: 26,
                a: 255,
            }
        );
    }

    #[test]
    fn path_rasterizer_should_enforce_flattened_segment_limit() {
        let decoded = generated_fixture_content("vector-paths.pdf");
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            DisplayListOptions::default(),
        )
        .expect("vector fixture display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 220.0,
                    max_y: 180.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            220,
        )
        .expect("vector fixture transform");
        let error = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                max_flattened_segments: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect_err("segment limit should fail");

        assert_eq!(
            error.kind(),
            &RasterErrorKind::PathComplexityOverflow { limit: 1 }
        );
    }

    #[test]
    fn image_rasterizer_should_draw_generated_image_xobject_fixture() {
        let document = generated_fixture_document("image-xobject.pdf");
        let resources = image_resources_from_document(&document).expect("generated image resource");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("generated image display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 120.0,
                    max_y: 120.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            120,
        )
        .expect("image fixture transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("image should rasterize");

        assert_eq!(
            device.pixel(44, 44).expect("top-left sample"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(76, 44).expect("top-right sample"),
            Rgba {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(44, 76).expect("bottom-left sample"),
            Rgba {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }
        );
    }

    #[test]
    fn text_rasterizer_should_draw_generated_text_fixture() {
        let decoded = generated_fixture_content("text-page.pdf");
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("generated text display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 300.0,
                    max_y: 160.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            300,
        )
        .expect("text fixture transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_text(&list, &mut device, transform).expect("text should rasterize");

        assert!(device
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn matrix_should_transform_points_deterministically() {
        let matrix = Matrix::translate(10.0, 20.0).multiply(Matrix::scale(2.0, 3.0));

        assert_eq!(matrix.transform_point(4.0, 5.0), Point { x: 18.0, y: 35.0 });
    }

    #[test]
    fn matrix_should_invert_non_singular_transforms() {
        let matrix = Matrix::translate(10.0, 20.0).multiply(Matrix::scale(2.0, 4.0));
        let inverse = matrix.inverse().expect("matrix should invert");
        let point = matrix.transform_point(3.0, 5.0);

        assert_eq!(
            inverse.transform_point(point.x, point.y),
            Point { x: 3.0, y: 5.0 }
        );
        assert!(Matrix::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0)
            .inverse()
            .is_none());
    }

    #[test]
    fn graphics_state_should_apply_cm_and_line_width() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"2 w 1 0 0 1 10 20 cm")),
            GraphicsStateOptions::default(),
        )
        .expect("valid graphics state stream");

        assert_eq!(state.line_width, 2.0);
        assert_eq!(state.ctm, Matrix::translate(10.0, 20.0));
    }

    #[test]
    fn graphics_state_should_restore_saved_state() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"0.25 g q 0.75 g Q")),
            GraphicsStateOptions::default(),
        )
        .expect("valid graphics state stream");

        assert_eq!(state.fill_gray, DeviceGray(0.25));
    }

    #[test]
    fn graphics_state_should_track_stroke_color_and_clipping_placeholder() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"0.5 G W")),
            GraphicsStateOptions::default(),
        )
        .expect("valid graphics state stream");

        assert_eq!(state.stroke_gray, DeviceGray(0.5));
        assert!(state.clip_path_pending);
    }

    #[test]
    fn graphics_state_should_track_rgb_colors() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"0.9 0.2 0.1 rg 0.1 0.4 0.8 RG")),
            GraphicsStateOptions::default(),
        )
        .expect("valid color operators");

        assert_eq!(
            state.fill_color,
            DeviceColor::Rgb {
                r: 0.9,
                g: 0.2,
                b: 0.1,
            }
        );
        assert_eq!(
            state.stroke_color,
            DeviceColor::Rgb {
                r: 0.1,
                g: 0.4,
                b: 0.8,
            }
        );
    }

    #[test]
    fn graphics_state_should_report_stack_underflow() {
        let error = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"Q")),
            GraphicsStateOptions::default(),
        )
        .expect_err("Q without q should fail");

        assert_eq!(error.offset(), Some(ByteOffset::new(0)));
        assert_eq!(error.kind(), &GraphicsErrorKind::StackUnderflow);
    }

    #[test]
    fn graphics_state_should_report_stack_overflow() {
        let error = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"q q")),
            GraphicsStateOptions { max_stack_depth: 1 },
        )
        .expect_err("second q should exceed depth");

        assert_eq!(error.offset(), Some(ByteOffset::new(2)));
        assert_eq!(error.kind(), &GraphicsErrorKind::StackOverflow { limit: 1 });
    }

    #[test]
    fn graphics_state_should_report_invalid_cm_operands() {
        let error = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"1 0 0 cm")),
            GraphicsStateOptions::default(),
        )
        .expect_err("cm needs six operands");

        assert_eq!(error.offset(), Some(ByteOffset::new(6)));
        assert!(matches!(
            error.kind(),
            GraphicsErrorKind::OperandCount {
                operator: b"cm",
                expected: 6,
                actual: 3,
            }
        ));
    }

    #[test]
    fn graphics_state_should_ignore_text_fixture_operators() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(
                b"BT /F1 24 Tf 40 90 Td (pdfrust thumbnail fixture) Tj ET",
            )),
            GraphicsStateOptions::default(),
        )
        .expect("unknown text operators should not fail graphics scan");

        assert_eq!(state, GraphicsState::default());
    }

    #[test]
    fn display_list_should_capture_stroked_path() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"0.1 0.4 0.8 RG 4 w 30 30 m 110 150 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid path stream");

        assert_eq!(list.len(), 1);
        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.paint, PaintMode::Stroke);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.state.line_width, 4.0);
        assert_eq!(
            path.state.stroke_color,
            DeviceColor::Rgb {
                r: 0.1,
                g: 0.4,
                b: 0.8,
            }
        );
    }

    #[test]
    fn display_list_should_capture_rectangle_fill() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"0.9 0.2 0.1 rg 70 55 80 50 re f")),
            DisplayListOptions::default(),
        )
        .expect("valid rectangle stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(
            path.paint,
            PaintMode::Fill {
                rule: FillRule::Nonzero,
            }
        );
        assert_eq!(path.segments.len(), 5);
        assert_eq!(
            path.state.fill_color,
            DeviceColor::Rgb {
                r: 0.9,
                g: 0.2,
                b: 0.1,
            }
        );
    }

    #[test]
    fn display_list_should_capture_clip_placeholder() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"10 10 20 20 re W n")),
            DisplayListOptions::default(),
        )
        .expect("valid clipping stream");

        let DisplayItem::ClipPlaceholder {
            segments,
            rule,
            state,
        } = &list.items()[0]
        else {
            panic!("expected clip placeholder item");
        };
        assert_eq!(segments.len(), 5);
        assert_eq!(*rule, FillRule::Nonzero);
        assert!(state.clip_path_pending);
    }

    #[test]
    fn display_list_should_parse_generated_vector_fixture() {
        let decoded = generated_fixture_content("vector-paths.pdf");
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            DisplayListOptions::default(),
        )
        .expect("vector fixture should build display list");

        assert_eq!(list.len(), 2);
        let bounds = list.bounds().expect("vector fixture bounds");
        assert_eq!(
            bounds,
            PathBounds {
                min_x: 30.0,
                min_y: 30.0,
                max_x: 190.0,
                max_y: 150.0,
            }
        );
        assert!(bounds.max_x <= 220.0);
        assert!(bounds.max_y <= 180.0);
    }

    #[test]
    fn display_list_should_report_unsupported_path_operator() {
        let error = build_path_display_list(
            tokenize_content(PdfBytes::new(b"10 10 m 20 20 v")),
            DisplayListOptions::default(),
        )
        .expect_err("v is unsupported in this milestone");

        assert_eq!(error.offset(), Some(ByteOffset::new(14)));
        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedPathOperator { operator: b"v" }
        );
    }

    #[test]
    fn display_list_should_report_missing_current_point() {
        let error = build_path_display_list(
            tokenize_content(PdfBytes::new(b"10 10 l")),
            DisplayListOptions::default(),
        )
        .expect_err("line-to requires move-to first");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::MissingCurrentPoint { operator: b"l" }
        );
    }

    #[test]
    fn display_list_should_enforce_path_segment_limit() {
        let error = build_path_display_list(
            tokenize_content(PdfBytes::new(b"10 10 m 20 20 l")),
            DisplayListOptions {
                max_path_segments: 1,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("second segment should exceed limit");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::PathSegmentOverflow { limit: 1 }
        );
    }

    #[test]
    fn display_list_should_enforce_item_limit() {
        let error = build_path_display_list(
            tokenize_content(PdfBytes::new(b"10 10 m 20 20 l S 30 30 10 10 re f")),
            DisplayListOptions {
                max_display_items: 1,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("second item should exceed limit");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::DisplayListOverflow { limit: 1 }
        );
    }

    #[test]
    fn text_display_list_should_parse_generated_text_fixture() {
        let decoded = generated_fixture_content("text-page.pdf");
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("text fixture should build text display list");

        assert_eq!(list.len(), 1);
        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "pdfrust thumbnail fixture");
        assert_eq!(text.font.resource_name, b"F1");
        assert_eq!(text.font_size, 24.0);
        assert_eq!(text.origin, Point { x: 40.0, y: 90.0 });
        assert_eq!(
            text.glyph_origins.first().copied(),
            Some(Point { x: 40.0, y: 90.0 })
        );
    }

    #[test]
    fn text_display_list_should_parse_generated_spacing_fixture() {
        let decoded = generated_fixture_content("text-spacing.pdf");
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("spacing fixture should build text display list");

        assert_eq!(list.len(), 4);
        let DisplayItem::Text(first) = &list.items()[0] else {
            panic!("expected first text item");
        };
        let DisplayItem::Text(second) = &list.items()[1] else {
            panic!("expected second text item");
        };
        let DisplayItem::Text(third) = &list.items()[2] else {
            panic!("expected third text item");
        };
        let DisplayItem::Text(hidden) = &list.items()[3] else {
            panic!("expected hidden text item");
        };

        assert_eq!(first.text, "office");
        assert_eq!(first.origin, Point { x: 20.0, y: 76.0 });
        assert_eq!(second.text, "export");
        assert!((second.origin.x - 74.108).abs() < 0.001);
        assert_eq!(second.origin.y, 76.0);
        assert_eq!(third.text, "normal text");
        assert_eq!(third.origin, Point { x: 40.0, y: 34.0 });
        assert_eq!(hidden.text, "hidden");
        assert_eq!(hidden.origin, Point { x: 40.0, y: 54.0 });
        assert_eq!(hidden.rendering_mode, TextRenderingMode::Invisible);
    }

    #[test]
    fn text_display_list_should_apply_tm_and_ctm_to_origin() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(
                b"2 0 0 2 10 20 cm BT /F1 12 Tf 1 0 0 1 5 6 Tm (Hi) Tj ET",
            )),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("valid text transform stream");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.origin, Point { x: 20.0, y: 32.0 });
    }

    #[test]
    fn text_display_list_should_parse_tj_arrays() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(
                b"BT /F1 10 Tf 10 20 Td [(A) 120 (B)] TJ (C) Tj ET",
            )),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("valid TJ stream");

        assert_eq!(list.len(), 3);
        let DisplayItem::Text(first) = &list.items()[0] else {
            panic!("expected first text item");
        };
        let DisplayItem::Text(second) = &list.items()[1] else {
            panic!("expected second text item");
        };
        let DisplayItem::Text(third) = &list.items()[2] else {
            panic!("expected third text item");
        };
        assert_eq!(first.text, "A");
        assert_eq!(second.text, "B");
        assert_eq!(third.text, "C");
        assert_eq!(first.origin, Point { x: 10.0, y: 20.0 });
        assert!((second.origin.x - 13.8).abs() < 0.001);
        assert_eq!(second.origin.y, 20.0);
        assert!((third.origin.x - 18.8).abs() < 0.001);
        assert_eq!(third.origin.y, 20.0);
    }

    #[test]
    fn text_display_list_should_apply_spacing_state_to_glyph_origins() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(
                b"BT /F1 10 Tf 2 Tc 4 Tw 80 Tz 10 20 Td (A B) Tj ET",
            )),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("valid text spacing stream");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text item");
        };
        assert_eq!(text.text, "A B");
        assert_eq!(text.glyph_origins[0], Point { x: 10.0, y: 20.0 });
        assert!((text.glyph_origins[1].x - 15.6).abs() < 0.001);
        assert_eq!(text.glyph_origins[1].y, 20.0);
        assert!((text.glyph_origins[2].x - 24.4).abs() < 0.001);
        assert_eq!(text.glyph_origins[2].y, 20.0);
    }

    #[test]
    fn text_display_list_should_capture_text_rendering_mode() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 10 Tf 3 Tr 10 20 Td (A) Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("valid invisible text stream");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text item");
        };
        assert_eq!(text.rendering_mode, TextRenderingMode::Invisible);
    }

    #[test]
    fn text_display_list_should_report_missing_font() {
        let error = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /Missing 12 Tf (Hi) Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect_err("missing font should fail");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::MissingFont {
                name: b"Missing".to_vec(),
            }
        );
    }

    #[test]
    fn text_display_list_should_report_font_not_selected() {
        let error = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT (Hi) Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect_err("show text without Tf should fail");

        assert_eq!(error.kind(), &GraphicsErrorKind::FontNotSelected);
    }

    #[test]
    fn text_display_list_should_report_text_outside_object() {
        let error = build_text_display_list(
            tokenize_content(PdfBytes::new(b"/F1 12 Tf")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect_err("Tf outside BT/ET should fail");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::TextOutsideObject { operator: b"Tf" }
        );
    }

    #[test]
    fn text_display_list_should_decode_hex_text_with_default_encoding() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 12 Tf <4869> Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("hex strings with ASCII bytes should decode through default encoding");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "Hi");
        assert_eq!(text.glyphs[0].character_code, 0x48);
    }

    #[test]
    fn text_display_list_should_decode_tounicode_hex_text() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 beginbfchar\n<01> <005a>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("ToUnicode text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "Z");
        assert_eq!(
            text.glyphs,
            vec![TextGlyph {
                character_code: 1,
                unicode: "Z".to_string(),
            }]
        );
    }

    #[test]
    fn text_display_list_should_decode_encoding_differences() {
        let document = generated_fixture_document("text-page.pdf");
        let resources = FontResources::from_font_dictionary(
            &[(
                PdfName::new(b"F1"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"Subtype"),
                        PdfPrimitive::Name(PdfName::new(b"Type1")),
                    ),
                    (
                        PdfName::new(b"Encoding"),
                        PdfPrimitive::Dictionary(vec![(
                            PdfName::new(b"Differences"),
                            PdfPrimitive::Array(vec![
                                PdfPrimitive::Number(PdfNumber::Integer(65)),
                                PdfPrimitive::Name(PdfName::new(b"Z")),
                            ]),
                        )]),
                    ),
                ]),
            )],
            &document,
            DisplayListOptions::default(),
        )
        .expect("differences encoding should resolve");
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 12 Tf (A) Tj ET")),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("differences text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "Z");
        assert_eq!(text.glyphs[0].character_code, 65);
    }

    #[test]
    fn font_resources_should_enforce_tounicode_cmap_byte_budget() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>",
            b"1 beginbfchar\n<01> <005a>\nendbfchar",
        );
        let error = font_resources_from_document_with_options(
            &document,
            &[("F1", 4)],
            DisplayListOptions {
                max_cmap_bytes: 3,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("CMap should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::CMapBytesOverflow { limit: 3 }
        );
    }

    #[test]
    fn text_display_list_should_enforce_text_run_limit() {
        let error = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 12 Tf (abcd) Tj ET")),
            &test_font_resources(),
            DisplayListOptions {
                max_text_run_bytes: 3,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("text run should exceed limit");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::TextRunOverflow { limit: 3 }
        );
    }

    #[test]
    fn font_resources_should_load_truetype_program() {
        let document = load_font_program_pdf(
            b"<< /Type /Font /Subtype /TrueType /BaseFont /TestFont /FontDescriptor 6 0 R >>",
            b"<< /Type /FontDescriptor /FontName /TestFont /FontFile2 7 0 R >>",
            b"<< /Length 4 >>",
            b"font",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let font = resources.get(PdfName::new(b"F1")).expect("font resource");

        assert_eq!(font.subtype, Some(FontSubtype::TrueType));
        assert_eq!(font.base_font.as_deref(), Some(b"TestFont".as_slice()));
        assert_eq!(
            font.program.as_ref().map(|program| program.key.kind),
            Some(FontProgramKind::TrueType)
        );
        assert_eq!(
            font.program.as_ref().map(|program| &*program.bytes),
            Some(b"font".as_slice())
        );
    }

    #[test]
    fn font_resources_should_share_program_cache_for_repeated_references() {
        let document = load_font_program_pdf(
            b"<< /Type /Font /Subtype /TrueType /BaseFont /TestFont /FontDescriptor 6 0 R >>",
            b"<< /Type /FontDescriptor /FontName /TestFont /FontFile2 7 0 R >>",
            b"<< /Length 4 >>",
            b"font",
        );
        let resources = font_resources_from_document(&document, &[("F1", 4), ("F2", 4)])
            .expect("valid font resources");
        let first = resources
            .get(PdfName::new(b"F1"))
            .and_then(|font| font.program.as_ref())
            .expect("first program");
        let second = resources
            .get(PdfName::new(b"F2"))
            .and_then(|font| font.program.as_ref())
            .expect("second program");

        assert!(Arc::ptr_eq(&first.bytes, &second.bytes));
    }

    #[test]
    fn font_resources_should_load_cff_fontfile3_program() {
        let document = load_font_program_pdf(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /CffFont /FontDescriptor 6 0 R >>",
            b"<< /Type /FontDescriptor /FontName /CffFont /FontFile3 7 0 R >>",
            b"<< /Subtype /Type1C /Length 3 >>",
            b"cff",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let font = resources.get(PdfName::new(b"F1")).expect("font resource");

        assert_eq!(
            font.program.as_ref().map(|program| program.key.kind),
            Some(FontProgramKind::Cff)
        );
    }

    #[test]
    fn font_resources_should_enforce_program_byte_budget() {
        let document = load_font_program_pdf(
            b"<< /Type /Font /Subtype /TrueType /BaseFont /TestFont /FontDescriptor 6 0 R >>",
            b"<< /Type /FontDescriptor /FontName /TestFont /FontFile2 7 0 R >>",
            b"<< /Length 4 >>",
            b"font",
        );
        let error = font_resources_from_document_with_options(
            &document,
            &[("F1", 4)],
            DisplayListOptions {
                max_font_program_bytes: 3,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("font program should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::FontProgramBytesOverflow { limit: 3 }
        );
    }

    #[test]
    fn font_resources_should_keep_base_font_without_embedded_program() {
        let document = generated_fixture_document("text-page.pdf");
        let resources = FontResources::from_font_dictionary(
            &[(
                PdfName::new(b"F1"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"Subtype"),
                        PdfPrimitive::Name(PdfName::new(b"Type1")),
                    ),
                    (
                        PdfName::new(b"BaseFont"),
                        PdfPrimitive::Name(PdfName::new(b"Helvetica")),
                    ),
                ]),
            )],
            &document,
            DisplayListOptions::default(),
        )
        .expect("base font should be accepted as fallback");
        let font = resources.get(PdfName::new(b"F1")).expect("font resource");

        assert_eq!(font.subtype, Some(FontSubtype::Type1));
        assert_eq!(font.base_font.as_deref(), Some(b"Helvetica".as_slice()));
        assert!(font.program.is_none());
    }

    #[test]
    fn glyph_outline_should_extract_simple_truetype_contour() {
        let program = test_truetype_program();
        let outline = extract_glyph_outline(&program, 1, GlyphOutlineOptions::default())
            .expect("outline extraction should succeed")
            .expect("glyph should exist");

        assert_eq!(outline.glyph_code, 1);
        assert_eq!(outline.advance_width, 600.0);
        assert_eq!(outline.left_side_bearing, 0.0);
        assert_eq!(
            outline.segments,
            vec![
                PathSegment::MoveTo(Point { x: 0.0, y: 0.0 }),
                PathSegment::LineTo(Point { x: 100.0, y: 0.0 }),
                PathSegment::LineTo(Point { x: 100.0, y: 100.0 }),
                PathSegment::LineTo(Point { x: 0.0, y: 100.0 }),
                PathSegment::LineTo(Point { x: 0.0, y: 0.0 }),
                PathSegment::Close,
            ]
        );
    }

    #[test]
    fn glyph_outline_should_report_missing_truetype_glyph() {
        let program = test_truetype_program();
        let outline = extract_glyph_outline(&program, 0, GlyphOutlineOptions::default())
            .expect("missing glyph should not be malformed");

        assert!(outline.is_none());
    }

    #[test]
    fn glyph_outline_cache_should_reuse_decoded_outline() {
        let program = test_truetype_program();
        let mut cache = GlyphOutlineCache::default();
        let first = cache
            .outline_for(&program, 1, GlyphOutlineOptions::default())
            .expect("first extraction")
            .expect("glyph should exist");
        let second = cache
            .outline_for(&program, 1, GlyphOutlineOptions::default())
            .expect("cached extraction")
            .expect("glyph should exist");

        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn glyph_outline_should_enforce_segment_budget() {
        let program = test_truetype_program();
        let error = extract_glyph_outline(
            &program,
            1,
            GlyphOutlineOptions {
                max_segments: 2,
                ..GlyphOutlineOptions::default()
            },
        )
        .expect_err("rectangle glyph should exceed segment budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::GlyphOutlineSegmentOverflow { limit: 2 }
        );
    }

    #[test]
    fn glyph_outline_should_extract_simple_cff_charstring() {
        let program = test_cff_program();
        let outline = extract_glyph_outline(&program, 1, GlyphOutlineOptions::default())
            .expect("CFF outline extraction should succeed")
            .expect("glyph should exist");

        assert_eq!(outline.glyph_code, 1);
        assert_eq!(
            outline.segments,
            vec![
                PathSegment::MoveTo(Point { x: 0.0, y: 0.0 }),
                PathSegment::LineTo(Point { x: 100.0, y: 0.0 }),
                PathSegment::LineTo(Point { x: 100.0, y: 100.0 }),
                PathSegment::LineTo(Point { x: 0.0, y: 100.0 }),
                PathSegment::LineTo(Point { x: 0.0, y: 0.0 }),
                PathSegment::Close,
            ]
        );
    }

    #[test]
    fn image_resources_should_decode_flate_rgb_xobject() {
        let document = load_image_xobject_pdf(
            b"q 64 0 0 64 28 28 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length 16 >>",
            &[120, 156, 251, 207, 192, 192, 240, 31, 132, 129, 0, 0, 29, 238, 5, 251],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
        assert_eq!(image.color_space, ImageColorSpace::DeviceRgb);
        assert_eq!(image.samples.len(), 12);
    }

    #[test]
    fn image_resources_should_decode_dct_rgb_xobject() {
        let document = generated_fixture_document("dct-image.pdf");
        let resources = image_resources_from_document(&document).expect("valid DCT image resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.width, 4);
        assert_eq!(image.height, 4);
        assert_eq!(image.color_space, ImageColorSpace::DeviceRgb);
        assert_eq!(image.samples.len(), 48);
        assert!(image.samples.chunks_exact(3).all(|pixel| pixel[0] > 240));
    }

    #[test]
    fn image_resources_should_apply_png_predictor_xobject() {
        let document = generated_fixture_document("predictor-image.pdf");
        let resources =
            image_resources_from_document(&document).expect("valid predictor image resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
        assert_eq!(image.color_space, ImageColorSpace::DeviceRgb);
        assert_eq!(
            &*image.samples,
            &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]
        );
    }

    #[test]
    fn image_resources_should_decode_device_gray_xobject() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 2 >>",
            &[0, 255],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceGray);
        assert_eq!(&*image.samples, &[0, 255]);
    }

    #[test]
    fn image_resources_should_enforce_image_byte_budget() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>",
            &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0],
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_image_bytes: 4,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("image samples should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageBytesOverflow { limit: 4 }
        );
    }

    #[test]
    fn image_resources_should_decode_device_gray_soft_mask() {
        let document = load_image_xobject_pdf_with_soft_mask(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 6 0 R /Length 6 >>",
            &[255, 0, 0, 0, 0, 255],
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 2 >>",
            &[0, 128],
        );
        let resources = image_resources_from_document(&document).expect("image with soft mask");
        let image = resources
            .get(PdfName::new(b"Im1"))
            .expect("decoded image resource");

        assert_eq!(image.soft_mask.as_deref(), Some([0, 128].as_slice()));
    }

    #[test]
    fn image_resources_should_enforce_soft_mask_depth_budget() {
        let document = load_image_xobject_pdf_with_soft_mask(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 6 0 R /Length 3 >>",
            &[255, 0, 0],
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 1 >>",
            &[128],
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_soft_mask_depth: 0,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("soft mask should exceed configured depth budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::SoftMaskDepthOverflow { limit: 0 }
        );
    }

    #[test]
    fn rasterize_images_should_apply_soft_mask_alpha() {
        let image = ImageDisplayItem {
            image: ImageXObject {
                resource_name: b"Im1".to_vec(),
                width: 2,
                height: 1,
                bits_per_component: 8,
                color_space: ImageColorSpace::DeviceRgb,
                samples: Arc::from([255, 0, 0, 0, 0, 255].as_slice()),
                indexed_lookup: None,
                soft_mask: Some(Arc::from([0, 128].as_slice())),
            },
            transform: Matrix::scale(2.0, 1.0),
            bounds: unit_square_bounds(Matrix::scale(2.0, 1.0)),
            state: GraphicsState::default(),
        };
        let mut device = RasterDevice::new(
            2,
            1,
            Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        )
        .expect("raster device");
        let dimensions = device.dimensions();
        draw_image(
            &mut device,
            &image,
            PageTransform {
                source_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 2.0,
                    max_y: 1.0,
                },
                rotation: PageRotation::Deg0,
                scale: 1.0,
                dimensions,
                matrix: Matrix::IDENTITY,
            },
        )
        .expect("masked image draw");

        assert_eq!(
            device.pixel(0, 0).expect("left pixel"),
            Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(1, 0).expect("right pixel"),
            Rgba {
                r: 127,
                g: 127,
                b: 255,
                a: 255,
            }
        );
    }

    #[test]
    fn image_resources_should_apply_device_gray_decode_array() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Decode [1 0] /Length 2 >>",
            &[0, 255],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceGray);
        assert_eq!(&*image.samples, &[255, 0]);
    }

    #[test]
    fn image_resources_should_decode_device_cmyk_xobject() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceCMYK /BitsPerComponent 8 /Length 4 >>",
            &[0, 255, 255, 0],
        );
        let resources =
            image_resources_from_document(&document).expect("valid CMYK image resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceCmyk);
        assert_eq!(&*image.samples, &[0, 255, 255, 0]);
    }

    #[test]
    fn image_resources_should_apply_device_cmyk_decode_array() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceCMYK /BitsPerComponent 8 /Decode [1 0 1 0 1 0 0 1] /Length 4 >>",
            &[255, 0, 0, 0],
        );
        let resources =
            image_resources_from_document(&document).expect("valid CMYK image resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceCmyk);
        assert_eq!(&*image.samples, &[0, 255, 255, 0]);
    }

    #[test]
    fn image_resources_should_decode_indexed_rgb_xobject() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace [/Indexed /DeviceRGB 1 <ff00000000ff>] /BitsPerComponent 8 /Length 2 >>",
            &[0, 1],
        );
        let resources =
            image_resources_from_document(&document).expect("valid Indexed image resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::IndexedRgb);
        assert_eq!(&*image.samples, &[0, 1]);
        assert_eq!(
            image.indexed_lookup.as_deref(),
            Some([255, 0, 0, 0, 0, 255].as_slice())
        );
    }

    #[test]
    fn image_resources_should_treat_calibrated_rgb_as_device_rgb_fallback() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/CalRGB << /WhitePoint [1 1 1] >>] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
        );
        let resources =
            image_resources_from_document(&document).expect("CalRGB should use RGB fallback");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceRgb);
        assert_eq!(&*image.samples, &[10, 20, 30]);
    }

    #[test]
    fn image_resources_should_report_unsupported_icc_based_color_space() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 9 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
        );
        let error =
            image_resources_from_document(&document).expect_err("ICCBased policy is unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: b"ICCBased".to_vec(),
            }
        );
    }

    #[test]
    fn image_display_list_should_place_image_with_ctm_bounds() {
        let document = load_image_xobject_pdf(
            b"q 64 0 0 64 28 28 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length 16 >>",
            &[120, 156, 251, 207, 192, 192, 240, 31, 132, 129, 0, 0, 29, 238, 5, 251],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid image placement");

        let DisplayItem::Image(image) = &list.items()[0] else {
            panic!("expected image item");
        };
        assert_eq!(
            image.bounds,
            PathBounds {
                min_x: 28.0,
                min_y: 28.0,
                max_x: 92.0,
                max_y: 92.0,
            }
        );
        assert_eq!(image.image.resource_name, b"Im1");
    }

    #[test]
    fn image_display_list_should_place_inline_image_with_ctm_bounds() {
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(
                b"q 64 0 0 64 28 28 cm BI /W 2 /H 2 /CS /RGB /BPC 8 ID \xff\0\0\0\xff\0\0\0\xff\xff\xff\0 EI Q",
            )),
            &ImageResources::empty(),
            DisplayListOptions::default(),
        )
        .expect("valid inline image placement");

        let DisplayItem::Image(image) = &list.items()[0] else {
            panic!("expected image item");
        };
        assert_eq!(
            image.bounds,
            PathBounds {
                min_x: 28.0,
                min_y: 28.0,
                max_x: 92.0,
                max_y: 92.0,
            }
        );
        assert_eq!(image.image.resource_name, b"inline-image");
    }

    #[test]
    fn image_rasterizer_should_draw_generated_inline_image_fixture() {
        let decoded = generated_fixture_content("inline-image.pdf");
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            &ImageResources::empty(),
            DisplayListOptions::default(),
        )
        .expect("generated inline image display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 120.0,
                    max_y: 120.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            120,
        )
        .expect("inline image fixture transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("inline image should rasterize");

        assert_eq!(
            device.pixel(44, 44).expect("top-left sample"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
    }

    #[test]
    fn image_rasterizer_should_convert_cmyk_samples() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 5 5 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceCMYK /BitsPerComponent 8 /Length 4 >>",
            &[0, 255, 255, 0],
        );
        let resources =
            image_resources_from_document(&document).expect("valid CMYK image resource");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid CMYK image display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 20.0,
                    max_y: 20.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            20,
        )
        .expect("CMYK image transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("CMYK image should rasterize");

        assert_eq!(
            device.pixel(10, 10).expect("CMYK sample pixel"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
    }

    #[test]
    fn image_rasterizer_should_draw_indexed_rgb_samples() {
        let document = load_image_xobject_pdf(
            b"q 20 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace [/Indexed /DeviceRGB 1 <ff00000000ff>] /BitsPerComponent 8 /Length 2 >>",
            &[0, 1],
        );
        let resources =
            image_resources_from_document(&document).expect("valid Indexed image resource");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid Indexed image display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 20.0,
                    max_y: 10.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            20,
        )
        .expect("Indexed image transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("Indexed image should rasterize");

        assert_eq!(
            device.pixel(5, 5).expect("left Indexed sample"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(15, 5).expect("right Indexed sample"),
            Rgba {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }
        );
    }

    #[test]
    fn image_display_list_should_report_unsupported_inline_image_filter() {
        let error = build_image_display_list(
            tokenize_content(PdfBytes::new(
                b"BI /W 1 /H 1 /CS /G /BPC 8 /F /FlateDecode ID \0 EI",
            )),
            &ImageResources::empty(),
            DisplayListOptions::default(),
        )
        .expect_err("inline image filters are not decoded yet");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageFilter {
                filter: b"FlateDecode".to_vec(),
            }
        );
    }

    #[test]
    fn image_display_list_should_share_decoded_samples_across_placements() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q q 10 0 0 10 20 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 2 >>",
            &[0, 255],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid image placements");

        let (DisplayItem::Image(first), DisplayItem::Image(second)) =
            (&list.items()[0], &list.items()[1])
        else {
            panic!("expected two image items");
        };
        assert!(Arc::ptr_eq(&first.image.samples, &second.image.samples));
    }

    #[test]
    fn image_resources_should_report_malformed_dct_filter_data() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length 3 >>",
            &[0, 0, 0],
        );
        let error =
            image_resources_from_document(&document).expect_err("malformed DCT should fail");

        assert!(matches!(
            error.kind(),
            GraphicsErrorKind::ObjectModel { .. }
        ));
    }

    #[test]
    fn image_resources_should_report_unsupported_predictor() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /DecodeParms << /Predictor 2 /Colors 3 /Columns 1 /BitsPerComponent 8 >> /Length 1 >>",
            &[0],
        );
        let error =
            image_resources_from_document(&document).expect_err("TIFF predictor is unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageFilter {
                filter: b"Predictor".to_vec(),
            }
        );
    }

    #[test]
    fn image_resources_should_report_unsupported_deferred_image_codecs() {
        for filter in [b"CCITTFaxDecode".as_slice(), b"JPXDecode", b"JBIG2Decode"] {
            let dictionary = format!(
                "<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /{} /Length 1 >>",
                String::from_utf8_lossy(filter)
            );
            let document = load_image_xobject_pdf(
                b"q 10 0 0 10 0 0 cm /Im1 Do Q",
                dictionary.as_bytes(),
                &[0],
            );
            let error = image_resources_from_document(&document)
                .expect_err("deferred codec should stay unsupported");

            assert_eq!(
                error.kind(),
                &GraphicsErrorKind::UnsupportedImageFilter {
                    filter: filter.to_vec(),
                }
            );
        }
    }

    #[test]
    fn image_resources_should_report_unsupported_color_space() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /Separation /BitsPerComponent 8 /Length 1 >>",
            &[0],
        );
        let error =
            image_resources_from_document(&document).expect_err("Separation is not supported yet");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: b"Separation".to_vec(),
            }
        );
    }

    #[test]
    fn image_display_list_should_report_missing_image_resource() {
        let error = build_image_display_list(
            tokenize_content(PdfBytes::new(b"/Missing Do")),
            &ImageResources::empty(),
            DisplayListOptions::default(),
        )
        .expect_err("missing image resource should fail");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::MissingImage {
                name: b"Missing".to_vec(),
            }
        );
    }

    #[test]
    fn image_display_list_should_ignore_known_form_xobject_names() {
        let document = load_form_xobject_pdf(
            b"/Fm1 Do",
            b"0 0 10 10 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length 14 >>",
            None,
        );
        let xobjects = vec![(
            PdfName::new(b"Fm1"),
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
        )];
        let resources = ImageResources::from_xobject_dictionary(
            &xobjects,
            &document,
            DisplayListOptions::default(),
        )
        .expect("valid xobject resources");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("known form name should not fail image pass");

        assert!(list.is_empty());
    }

    #[test]
    fn form_resources_should_decode_matrix_bbox_and_local_resources() {
        let document = load_form_xobject_pdf(
            b"/Fm1 Do",
            b"0 0 10 10 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 30] /Matrix [1 0 0 1 5 6] /Resources << /XObject << /Nested 6 0 R >> >> /Length 14 >>",
            Some((
                b"0 0 5 5 re f".as_slice(),
                b"<< /Type /XObject /Subtype /Form /BBox [0 0 5 5] /Length 12 >>".as_slice(),
            )),
        );
        let resources =
            form_resources_from_document(&document, &[("Fm1", 4)]).expect("valid form resources");
        let form = resources.get(PdfName::new(b"Fm1")).expect("form resource");

        assert_eq!(form.matrix, Matrix::translate(5.0, 6.0));
        assert_eq!(
            form.bbox,
            PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 20.0,
                max_y: 30.0,
            }
        );
        assert_eq!(form.xobject_references[0].name, b"Nested");
    }

    #[test]
    fn form_display_list_should_apply_form_matrix_and_bbox_clip() {
        let document = load_form_xobject_pdf(
            b"q 2 0 0 2 10 20 cm /Fm1 Do Q",
            b"0 0 10 10 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Matrix [1 0 0 1 5 6] /Length 14 >>",
            None,
        );
        let resources =
            form_resources_from_document(&document, &[("Fm1", 4)]).expect("valid form resources");
        let content = content_stream_from_document(&document);
        let list = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid form invocation");

        assert_eq!(list.len(), 2);
        let DisplayItem::ClipPlaceholder { segments, .. } = &list.items()[0] else {
            panic!("expected form bbox clip placeholder");
        };
        assert_eq!(
            PathBounds::from_segments(segments).expect("clip bounds"),
            PathBounds {
                min_x: 20.0,
                min_y: 32.0,
                max_x: 60.0,
                max_y: 72.0,
            }
        );
        let DisplayItem::Path(path) = &list.items()[1] else {
            panic!("expected form path");
        };
        assert_eq!(
            path.bounds().expect("path bounds"),
            PathBounds {
                min_x: 20.0,
                min_y: 32.0,
                max_x: 40.0,
                max_y: 52.0,
            }
        );
    }

    #[test]
    fn form_display_list_should_use_local_form_xobject_resources() {
        let document = load_form_xobject_pdf(
            b"/Outer Do",
            b"/Inner Do",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Resources << /XObject << /Inner 6 0 R >> >> /Length 9 >>",
            Some((
                b"0 0 5 5 re f".as_slice(),
                b"<< /Type /XObject /Subtype /Form /BBox [0 0 5 5] /Length 12 >>".as_slice(),
            )),
        );
        let resources =
            form_resources_from_document(&document, &[("Outer", 4)]).expect("valid form resources");
        let content = content_stream_from_document(&document);
        let list = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid nested form invocation");

        assert_eq!(list.len(), 3);
        let DisplayItem::Path(path) = &list.items()[2] else {
            panic!("expected nested form path");
        };
        assert_eq!(
            path.bounds().expect("nested path bounds"),
            PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 5.0,
                max_y: 5.0,
            }
        );
    }

    #[test]
    fn form_display_list_should_inherit_page_xobjects_when_resources_are_omitted() {
        let document = load_form_xobject_pdf(
            b"/Outer Do",
            b"/Inner Do",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length 9 >>",
            Some((
                b"0 0 5 5 re f".as_slice(),
                b"<< /Type /XObject /Subtype /Form /BBox [0 0 5 5] /Length 12 >>".as_slice(),
            )),
        );
        let resources = form_resources_from_document(&document, &[("Outer", 4), ("Inner", 6)])
            .expect("valid form resources");
        let content = content_stream_from_document(&document);
        let list = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid inherited form invocation");

        assert_eq!(list.len(), 3);
        let DisplayItem::Path(path) = &list.items()[2] else {
            panic!("expected inherited nested form path");
        };
        assert_eq!(
            path.bounds().expect("nested path bounds"),
            PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 5.0,
                max_y: 5.0,
            }
        );
    }

    #[test]
    fn form_display_list_should_parse_generated_form_fixture() {
        let document = generated_fixture_document("form-xobject.pdf");
        let resources = form_resources_from_document(&document, &[("Fm1", 4)])
            .expect("generated form resources");
        let content = content_stream_from_document(&document);
        let list = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("generated form fixture should build display list");

        assert_eq!(list.len(), 2);
        let bounds = list.bounds().expect("generated form bounds");
        assert_eq!(
            bounds,
            PathBounds {
                min_x: 20.0,
                min_y: 32.0,
                max_x: 100.0,
                max_y: 112.0,
            }
        );
    }

    #[test]
    fn form_display_list_should_report_missing_form_resource() {
        let error = build_form_display_list(
            tokenize_content(PdfBytes::new(b"/Missing Do")),
            &FormResources::empty(),
            DisplayListOptions::default(),
        )
        .expect_err("missing form resource should fail");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::MissingForm {
                name: b"Missing".to_vec(),
            }
        );
    }

    #[test]
    fn form_display_list_should_enforce_recursion_limit() {
        let document = load_form_xobject_pdf(
            b"/Self Do",
            b"/Self Do",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 10 10] /Resources << /XObject << /Self 4 0 R >> >> /Length 8 >>",
            None,
        );
        let resources = form_resources_from_document(&document, &[("Self", 4)])
            .expect("valid recursive form resources");
        let content = content_stream_from_document(&document);
        let error = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions {
                max_form_recursion_depth: 1,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("recursive form should exceed depth limit");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::FormRecursionOverflow { limit: 1 }
        );
    }

    fn generated_fixture_content(file_name: &str) -> Vec<u8> {
        let document = generated_fixture_document(file_name);
        content_stream_from_document(&document)
    }

    fn generated_fixture_document(file_name: &str) -> ClassicDocument<'static> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/generated/{file_name}"));
        let bytes = std::fs::read(path).expect("fixture should be readable");
        let leaked = Box::leak(bytes.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("fixture should load as PDF")
    }

    fn image_resources_from_document(
        document: &ClassicDocument<'_>,
    ) -> GraphicsResult<ImageResources> {
        image_resources_from_document_with_options(document, DisplayListOptions::default())
    }

    fn image_resources_from_document_with_options(
        document: &ClassicDocument<'_>,
        options: DisplayListOptions,
    ) -> GraphicsResult<ImageResources> {
        let xobjects = vec![(
            PdfName::new(b"Im1"),
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
        )];
        ImageResources::from_xobject_dictionary(&xobjects, document, options)
    }

    fn form_resources_from_document(
        document: &ClassicDocument<'_>,
        resources: &[(&str, u32)],
    ) -> GraphicsResult<FormResources> {
        let xobjects = resources
            .iter()
            .map(|(name, object)| {
                (
                    PdfName::new(name.as_bytes()),
                    PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(*object, 0)),
                )
            })
            .collect::<Vec<_>>();
        FormResources::from_xobject_dictionary(&xobjects, document)
    }

    fn font_resources_from_document(
        document: &ClassicDocument<'_>,
        resources: &[(&str, u32)],
    ) -> GraphicsResult<FontResources> {
        font_resources_from_document_with_options(
            document,
            resources,
            DisplayListOptions::default(),
        )
    }

    fn font_resources_from_document_with_options(
        document: &ClassicDocument<'_>,
        resources: &[(&str, u32)],
        options: DisplayListOptions,
    ) -> GraphicsResult<FontResources> {
        let fonts = resources
            .iter()
            .map(|(name, object)| {
                (
                    PdfName::new(name.as_bytes()),
                    PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(*object, 0)),
                )
            })
            .collect::<Vec<_>>();
        FontResources::from_font_dictionary(&fonts, document, options)
    }

    fn content_stream_from_document(document: &ClassicDocument<'_>) -> Vec<u8> {
        let content_id = ObjectId::new(
            ObjectNumber::new(1).expect("object number"),
            GenerationNumber::new(0),
        );
        let content = document.objects.get(content_id).expect("content stream");
        let ObjectValue::Stream(stream) = &content.value else {
            panic!("content object should be a stream");
        };
        stream.decode().expect("content stream should decode")
    }

    fn load_image_xobject_pdf(
        content_stream: &[u8],
        image_dictionary: &[u8],
        image_stream: &[u8],
    ) -> ClassicDocument<'static> {
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /XObject << /Im1 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            stream_object_bytes(4, image_dictionary, image_stream),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("image XObject PDF should load")
    }

    fn load_image_xobject_pdf_with_soft_mask(
        content_stream: &[u8],
        image_dictionary: &[u8],
        image_stream: &[u8],
        mask_dictionary: &[u8],
        mask_stream: &[u8],
    ) -> ClassicDocument<'static> {
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /XObject << /Im1 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            stream_object_bytes(4, image_dictionary, image_stream),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
            stream_object_bytes(6, mask_dictionary, mask_stream),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked))
            .expect("image XObject PDF with soft mask should load")
    }

    fn load_form_xobject_pdf(
        content_stream: &[u8],
        form_stream: &[u8],
        form_dictionary: &[u8],
        nested_form: Option<(&[u8], &[u8])>,
    ) -> ClassicDocument<'static> {
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let mut objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /XObject << /Fm1 4 0 R /Outer 4 0 R /Self 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            stream_object_bytes(4, form_dictionary, form_stream),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
        ];
        if let Some((nested_stream, nested_dictionary)) = nested_form {
            objects.push(stream_object_bytes(6, nested_dictionary, nested_stream));
        }
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("form XObject PDF should load")
    }

    fn load_font_program_pdf(
        font_dictionary: &[u8],
        descriptor_dictionary: &[u8],
        font_stream_dictionary: &[u8],
        font_stream: &[u8],
    ) -> ClassicDocument<'static> {
        let content_stream = b"BT /F1 12 Tf 10 20 Td (Hi) Tj ET";
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /Font << /F1 4 0 R /F2 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            indirect_object_bytes(4, font_dictionary),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
            indirect_object_bytes(6, descriptor_dictionary),
            stream_object_bytes(7, font_stream_dictionary, font_stream),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("font program PDF should load")
    }

    fn load_tounicode_text_pdf(
        content_stream: &[u8],
        font_dictionary: &[u8],
        cmap_stream: &[u8],
    ) -> ClassicDocument<'static> {
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let cmap_dictionary = format!("<< /Length {} >>", cmap_stream.len());
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /Font << /F1 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            indirect_object_bytes(4, font_dictionary),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
            stream_object_bytes(6, cmap_dictionary.as_bytes(), cmap_stream),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("ToUnicode text PDF should load")
    }

    fn test_truetype_program() -> FontProgram {
        FontProgram {
            key: FontProgramKey {
                reference: Reference::new(ObjectId::new(
                    ObjectNumber::new(7).expect("object number"),
                    GenerationNumber::new(0),
                )),
                kind: FontProgramKind::TrueType,
            },
            bytes: Arc::from(minimal_truetype_font()),
        }
    }

    fn test_cff_program() -> FontProgram {
        FontProgram {
            key: FontProgramKey {
                reference: Reference::new(ObjectId::new(
                    ObjectNumber::new(8).expect("object number"),
                    GenerationNumber::new(0),
                )),
                kind: FontProgramKind::Cff,
            },
            bytes: Arc::from(minimal_cff_font()),
        }
    }

    fn minimal_cff_font() -> Vec<u8> {
        let header = vec![1, 0, 4, 1];
        let name_index = cff_index(&[b"A".as_slice()]);
        let string_index = cff_index(&[]);
        let global_subr_index = cff_index(&[]);
        let glyph0 = [14];
        let glyph1 = [
            type2_number(0),
            type2_number(0),
            21,
            type2_number(100),
            type2_number(0),
            type2_number(0),
            type2_number(100),
            type2_number(-100),
            type2_number(0),
            type2_number(0),
            type2_number(-100),
            5,
            14,
        ];
        let charstrings_index = cff_index(&[glyph0.as_slice(), glyph1.as_slice()]);
        let charstrings_offset =
            header.len() + name_index.len() + 7 + string_index.len() + global_subr_index.len();
        let top_dict = [dict_number(charstrings_offset as i32), 17];
        let top_index = cff_index(&[top_dict.as_slice()]);

        let mut cff = Vec::new();
        cff.extend_from_slice(&header);
        cff.extend_from_slice(&name_index);
        cff.extend_from_slice(&top_index);
        cff.extend_from_slice(&string_index);
        cff.extend_from_slice(&global_subr_index);
        cff.extend_from_slice(&charstrings_index);
        cff
    }

    fn cff_index(objects: &[&[u8]]) -> Vec<u8> {
        let mut index = Vec::new();
        index.extend_from_slice(&(objects.len() as u16).to_be_bytes());
        if objects.is_empty() {
            return index;
        }
        index.push(1);
        let mut offset = 1u8;
        index.push(offset);
        for object in objects {
            offset = offset
                .checked_add(u8::try_from(object.len()).expect("small CFF object"))
                .expect("small CFF index");
            index.push(offset);
        }
        for object in objects {
            index.extend_from_slice(object);
        }
        index
    }

    fn dict_number(value: i32) -> u8 {
        u8::try_from(value + 139).expect("small DICT number")
    }

    fn type2_number(value: i16) -> u8 {
        u8::try_from(i32::from(value) + 139).expect("small Type 2 number")
    }

    fn minimal_truetype_font() -> Vec<u8> {
        let mut head = vec![0; 54];
        head[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        head[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        head[12..16].copy_from_slice(&0x5f0f_3cf5u32.to_be_bytes());
        head[18..20].copy_from_slice(&1000u16.to_be_bytes());
        head[40..42].copy_from_slice(&100i16.to_be_bytes());
        head[42..44].copy_from_slice(&100i16.to_be_bytes());
        head[46..48].copy_from_slice(&8u16.to_be_bytes());
        head[50..52].copy_from_slice(&0i16.to_be_bytes());

        let mut maxp = Vec::new();
        maxp.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        maxp.extend_from_slice(&2u16.to_be_bytes());
        maxp.extend_from_slice(&4u16.to_be_bytes());
        maxp.extend_from_slice(&1u16.to_be_bytes());
        maxp.resize(32, 0);

        let mut hhea = vec![0; 36];
        hhea[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        hhea[4..6].copy_from_slice(&800i16.to_be_bytes());
        hhea[6..8].copy_from_slice(&(-200i16).to_be_bytes());
        hhea[10..12].copy_from_slice(&600u16.to_be_bytes());
        hhea[18..20].copy_from_slice(&100i16.to_be_bytes());
        hhea[20..22].copy_from_slice(&1i16.to_be_bytes());
        hhea[34..36].copy_from_slice(&2u16.to_be_bytes());

        let mut hmtx = Vec::new();
        hmtx.extend_from_slice(&0u16.to_be_bytes());
        hmtx.extend_from_slice(&0i16.to_be_bytes());
        hmtx.extend_from_slice(&600u16.to_be_bytes());
        hmtx.extend_from_slice(&0i16.to_be_bytes());

        let glyph = simple_rectangle_glyph();
        let mut loca = Vec::new();
        loca.extend_from_slice(&0u16.to_be_bytes());
        loca.extend_from_slice(&0u16.to_be_bytes());
        loca.extend_from_slice(&((glyph.len() / 2) as u16).to_be_bytes());

        build_truetype_font(&[
            (*b"head", head),
            (*b"maxp", maxp),
            (*b"hhea", hhea),
            (*b"hmtx", hmtx),
            (*b"loca", loca),
            (*b"glyf", glyph),
        ])
    }

    fn simple_rectangle_glyph() -> Vec<u8> {
        let mut glyph = Vec::new();
        glyph.extend_from_slice(&1i16.to_be_bytes());
        glyph.extend_from_slice(&0i16.to_be_bytes());
        glyph.extend_from_slice(&0i16.to_be_bytes());
        glyph.extend_from_slice(&100i16.to_be_bytes());
        glyph.extend_from_slice(&100i16.to_be_bytes());
        glyph.extend_from_slice(&3u16.to_be_bytes());
        glyph.extend_from_slice(&0u16.to_be_bytes());
        glyph.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        for delta in [0i16, 100, 0, -100] {
            glyph.extend_from_slice(&delta.to_be_bytes());
        }
        for delta in [0i16, 0, 100, 0] {
            glyph.extend_from_slice(&delta.to_be_bytes());
        }
        glyph
    }

    fn build_truetype_font(tables: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
        let table_count = tables.len();
        let mut offset = align4(12 + table_count * 16);
        let mut records = Vec::new();
        for (tag, data) in tables {
            records.push((*tag, offset, data.len()));
            offset = align4(offset + data.len());
        }

        let mut font = Vec::with_capacity(offset);
        font.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        font.extend_from_slice(&(table_count as u16).to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes());
        for (tag, table_offset, length) in &records {
            font.extend_from_slice(tag);
            font.extend_from_slice(&0u32.to_be_bytes());
            font.extend_from_slice(&(*table_offset as u32).to_be_bytes());
            font.extend_from_slice(&(*length as u32).to_be_bytes());
        }
        while font.len() < align4(12 + table_count * 16) {
            font.push(0);
        }
        for ((_, data), (_, table_offset, _)) in tables.iter().zip(records.iter()) {
            while font.len() < *table_offset {
                font.push(0);
            }
            font.extend_from_slice(data);
            while font.len() % 4 != 0 {
                font.push(0);
            }
        }
        font
    }

    fn align4(value: usize) -> usize {
        (value + 3) & !3
    }

    fn indirect_object_bytes(number: u32, body: &[u8]) -> Vec<u8> {
        let mut object = Vec::new();
        object.extend_from_slice(number.to_string().as_bytes());
        object.extend_from_slice(b" 0 obj\n");
        object.extend_from_slice(body);
        object.extend_from_slice(b"\nendobj\n");
        object
    }

    fn stream_object_bytes(number: u32, dictionary: &[u8], stream: &[u8]) -> Vec<u8> {
        let mut object = Vec::new();
        object.extend_from_slice(number.to_string().as_bytes());
        object.extend_from_slice(b" 0 obj\n");
        object.extend_from_slice(dictionary);
        object.extend_from_slice(b"\nstream\n");
        object.extend_from_slice(stream);
        object.extend_from_slice(b"\nendstream\nendobj\n");
        object
    }

    fn build_classic_pdf_from_objects(objects: &[Vec<u8>]) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for object in objects {
            offsets.push(pdf.len());
            pdf.extend_from_slice(object);
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(b"xref\n0 ");
        pdf.extend_from_slice((objects.len() + 1).to_string().as_bytes());
        pdf.extend_from_slice(b"\n0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(b"trailer\n<< /Size ");
        pdf.extend_from_slice((objects.len() + 1).to_string().as_bytes());
        pdf.extend_from_slice(b" /Root 5 0 R >>\nstartxref\n");
        pdf.extend_from_slice(xref_offset.to_string().as_bytes());
        pdf.extend_from_slice(b"\n%%EOF\n");
        pdf
    }

    fn test_font_resources() -> FontResources {
        FontResources::new(vec![FontDescriptor::new(
            b"F1".to_vec(),
            Some(b"Helvetica".to_vec()),
        )])
    }
}
