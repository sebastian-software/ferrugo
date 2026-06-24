//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;

use pdfrust_content::{
    tokenize_content, ContentErrorKind, ContentResult, ContentToken, InlineImage, OperatorName,
};
use pdfrust_object::{
    ClassicDocument, GenerationNumber, IndirectObject, ModernDocument, ObjectId, ObjectNumber,
    ObjectValue, Reference, StreamObject,
};
use pdfrust_syntax::{ByteOffset, PdfBytes, PdfName, PdfNumber, PdfPrimitive, PdfString};
use pdfrust_thumbnail::{PixelFormat, Rgba};

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

/// Default maximum decoded bytes for one image XObject.
pub const DEFAULT_IMAGE_BYTES_LIMIT: usize = 32 * 1024 * 1024;

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
    /// Font descriptor selected by `Tf`.
    pub font: FontDescriptor,
    /// Font size selected by `Tf`.
    pub font_size: f64,
    /// Text origin after text and graphics transforms are applied.
    pub origin: Point,
    /// Text matrix at paint time.
    pub text_matrix: Matrix,
    /// Graphics state snapshot at paint time.
    pub state: GraphicsState,
}

/// Supported image color-space metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageColorSpace {
    /// DeviceGray image samples.
    DeviceGray,
    /// DeviceRGB image samples.
    DeviceRgb,
}

impl ImageColorSpace {
    /// Returns bytes per pixel for the supported 8-bit color spaces.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::DeviceGray => 1,
            Self::DeviceRgb => 3,
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
                options.max_image_bytes,
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

/// Lightweight font descriptor used before full font loading lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDescriptor {
    /// Resource name used by content streams, without the leading slash.
    pub resource_name: Vec<u8>,
    /// Optional base font name from page resources.
    pub base_font: Option<Vec<u8>>,
}

impl FontDescriptor {
    /// Creates a lightweight font descriptor.
    #[must_use]
    pub fn new(resource_name: impl Into<Vec<u8>>, base_font: Option<impl Into<Vec<u8>>>) -> Self {
        Self {
            resource_name: resource_name.into(),
            base_font: base_font.map(Into::into),
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

    /// Returns the font matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&FontDescriptor> {
        self.fonts
            .iter()
            .find(|font| font.resource_name.as_slice() == name.as_bytes())
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
    /// Maximum decoded bytes accepted for one image XObject.
    pub max_image_bytes: usize,
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
            max_image_bytes: DEFAULT_IMAGE_BYTES_LIMIT,
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
        let text = decode_pdf_text_string(
            string_operand(offset, b"Tj", operands, 0)?,
            offset,
            self.options.max_text_run_bytes,
        )?;
        self.push_text(offset, text)
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
        let mut text = String::new();
        let mut adjustment = 0.0;
        for value in values {
            match value {
                PdfPrimitive::String(string) => {
                    let chunk =
                        decode_pdf_text_string(*string, offset, self.options.max_text_run_bytes)?;
                    if text.len() + chunk.len() > self.options.max_text_run_bytes {
                        return Err(GraphicsError::new(
                            Some(offset),
                            GraphicsErrorKind::TextRunOverflow {
                                limit: self.options.max_text_run_bytes,
                            },
                        ));
                    }
                    text.push_str(&chunk);
                }
                PdfPrimitive::Number(PdfNumber::Integer(value)) => {
                    adjustment += *value as f64;
                }
                PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => {
                    adjustment += *value;
                }
                _ => return Err(invalid_operand(offset, b"TJ")),
            }
        }
        self.push_text(offset, text)?;
        self.advance_text(-adjustment / 1000.0 * self.text.font_size);
        Ok(())
    }

    fn push_text(&mut self, offset: ByteOffset, text: String) -> GraphicsResult<()> {
        let byte_len = text.len();
        let font =
            self.text.font.clone().ok_or_else(|| {
                GraphicsError::new(Some(offset), GraphicsErrorKind::FontNotSelected)
            })?;
        let origin_matrix = self.current.ctm.multiply(self.text.text_matrix);
        let origin = origin_matrix.transform_point(0.0, 0.0);
        self.display_list.push(
            DisplayItem::Text(TextDisplayItem {
                text,
                font,
                font_size: self.text.font_size,
                origin,
                text_matrix: self.text.text_matrix,
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )?;
        self.advance_text(byte_len as f64 * self.text.font_size * 0.5);
        Ok(())
    }

    fn advance_text(&mut self, advance: f64) {
        self.text.text_matrix = self
            .text
            .text_matrix
            .multiply(Matrix::translate(advance, 0.0));
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

#[derive(Debug, Default, Clone, PartialEq)]
struct TextState {
    in_text_object: bool,
    text_matrix: Matrix,
    line_matrix: Matrix,
    font: Option<FontDescriptor>,
    font_size: f64,
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
    offset: ByteOffset,
    limit: usize,
) -> GraphicsResult<String> {
    let PdfString::Literal(bytes) = string else {
        return Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::UnsupportedTextEncoding,
        ));
    };
    if bytes.len() > limit {
        return Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::TextRunOverflow { limit },
        ));
    }
    if !bytes.is_ascii() {
        return Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::UnsupportedTextEncoding,
        ));
    }
    String::from_utf8(bytes.to_vec())
        .map_err(|_| GraphicsError::new(Some(offset), GraphicsErrorKind::UnsupportedTextEncoding))
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

fn decode_image_xobject(
    resource_name: PdfName<'_>,
    stream: &StreamObject<'_>,
    max_image_bytes: usize,
) -> GraphicsResult<ImageXObject> {
    require_supported_image_filter(stream.dictionary())?;
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
    let decoded = stream.decode().map_err(|error| {
        GraphicsError::new(
            error.offset(),
            GraphicsErrorKind::ObjectModel {
                message: error.to_string(),
            },
        )
    })?;
    if decoded.len() > max_image_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        ));
    }
    let expected_len = expected_image_len(width, height, color_space)?;
    if decoded.len() != expected_len {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: expected_len,
                actual: decoded.len(),
            },
        ));
    }
    Ok(ImageXObject {
        resource_name: resource_name.as_bytes().to_vec(),
        width,
        height,
        bits_per_component,
        color_space,
        samples: Arc::from(decoded),
    })
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
    let expected_len = expected_image_len(width, height, color_space)?;
    if data.len() != expected_len {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: expected_len,
                actual: data.len(),
            },
        ));
    }
    Ok(ImageXObject {
        resource_name: b"inline-image".to_vec(),
        width,
        height,
        bits_per_component,
        color_space,
        samples: Arc::from(data),
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
            device.set_pixel(x, y, pixel)?;
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
    match image.color_space {
        ImageColorSpace::DeviceGray => {
            let channel = image.samples[index];
            Rgba {
                r: channel,
                g: channel,
                b: channel,
                a: 255,
            }
        }
        ImageColorSpace::DeviceRgb => Rgba {
            r: image.samples[index],
            g: image.samples[index + 1],
            b: image.samples[index + 2],
            a: 255,
        },
    }
}

fn draw_text_run(
    device: &mut RasterDevice,
    text: &TextDisplayItem,
    page_transform: PageTransform,
) -> RasterResult<()> {
    let color = device_color_to_rgba(text.state.fill_color);
    let cell = text.font_size / 7.0;
    let glyph_advance = cell * 6.0;
    let mut cursor_x = text.origin.x;
    for character in text.text.chars() {
        if character == ' ' {
            cursor_x += glyph_advance;
            continue;
        }
        draw_ascii_glyph(
            device,
            page_transform,
            character,
            cursor_x,
            text.origin.y,
            cell,
            color,
        )?;
        cursor_x += glyph_advance;
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

fn require_supported_image_filter(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<()> {
    let Some(filter) =
        dictionary_value(dictionary, b"Filter").or_else(|| dictionary_value(dictionary, b"F"))
    else {
        return Ok(());
    };
    match filter {
        PdfPrimitive::Name(name) if name.as_bytes() == b"FlateDecode" => Ok(()),
        PdfPrimitive::Name(name) if name.as_bytes() == b"DCTDecode" => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedImageFilter {
                filter: name.as_bytes().to_vec(),
            },
        )),
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
                        return Ok(());
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

fn image_color_space(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<ImageColorSpace> {
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
            Ok(ImageColorSpace::DeviceRgb)
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"RGB" => Ok(ImageColorSpace::DeviceRgb),
        PdfPrimitive::Name(name) if name.as_bytes() == b"DeviceGray" => {
            Ok(ImageColorSpace::DeviceGray)
        }
        PdfPrimitive::Name(name) if name.as_bytes() == b"G" => Ok(ImageColorSpace::DeviceGray),
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
    /// Text string uses an encoding outside the current ASCII stub policy.
    UnsupportedTextEncoding,
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
            Self::UnsupportedTextEncoding => f.write_str("unsupported text encoding"),
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

        assert_eq!(list.len(), 2);
        let DisplayItem::Text(first) = &list.items()[0] else {
            panic!("expected first text item");
        };
        let DisplayItem::Text(second) = &list.items()[1] else {
            panic!("expected second text item");
        };
        assert_eq!(first.text, "AB");
        assert_eq!(second.text, "C");
        assert!((second.origin.x - 18.8).abs() < 0.001);
        assert_eq!(second.origin.y, 20.0);
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
    fn text_display_list_should_report_unsupported_hex_text() {
        let error = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 12 Tf <4869> Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect_err("hex strings are unsupported in this milestone");

        assert_eq!(error.kind(), &GraphicsErrorKind::UnsupportedTextEncoding);
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
    fn image_resources_should_report_unsupported_dct_filter() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length 3 >>",
            &[0, 0, 0],
        );
        let error = image_resources_from_document(&document).expect_err("DCT is not decoded yet");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageFilter {
                filter: b"DCTDecode".to_vec(),
            }
        );
    }

    #[test]
    fn image_resources_should_report_unsupported_color_space() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceCMYK /BitsPerComponent 8 /Length 4 >>",
            &[0, 0, 0, 0],
        );
        let error =
            image_resources_from_document(&document).expect_err("CMYK is not supported yet");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedImageColorSpace {
                color_space: b"DeviceCMYK".to_vec(),
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
        let xobjects = vec![(
            PdfName::new(b"Im1"),
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
        )];
        ImageResources::from_xobject_dictionary(&xobjects, document, DisplayListOptions::default())
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
