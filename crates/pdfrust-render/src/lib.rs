//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::borrow::Cow;
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

/// Default maximum pixels in one page raster buffer.
pub const DEFAULT_PAGE_RASTER_PIXELS_LIMIT: usize = 16 * 1024 * 1024;

/// Default maximum decoded bytes for one ToUnicode CMap stream.
pub const DEFAULT_CMAP_BYTES_LIMIT: usize = 1024 * 1024;

/// Default maximum entries accepted in one parsed ToUnicode CMap.
pub const DEFAULT_CMAP_ENTRIES_LIMIT: usize = 4_096;

/// Default maximum path segments accepted in one decoded glyph outline.
pub const DEFAULT_GLYPH_OUTLINE_SEGMENT_LIMIT: usize = 2_048;

/// Default maximum cached glyph outlines per outline cache.
pub const DEFAULT_GLYPH_OUTLINE_CACHE_LIMIT: usize = 4_096;

/// Default maximum operands accepted on a charstring stack.
pub const DEFAULT_CHARSTRING_STACK_LIMIT: usize = 48;

/// Default maximum nested charstring subroutine calls.
pub const DEFAULT_CHARSTRING_SUBROUTINE_DEPTH_LIMIT: usize = 10;

/// Default maximum cached fallback glyph bitmaps per rasterization pass.
pub const DEFAULT_GLYPH_BITMAP_CACHE_LIMIT: usize = 256;

const STANDARD_BASE_FONT_CELL_SCALE: f64 = 0.75;

const DEFAULT_TEXT_RASTER_SCRATCH_RETAINED_ATOMS: usize = 4_096;

/// Default maximum cached deterministic font fallback resolutions.
pub const DEFAULT_FONT_FALLBACK_CACHE_LIMIT: usize = 128;

/// Default maximum decoded bytes for one embedded font program.
pub const DEFAULT_FONT_PROGRAM_BYTES_LIMIT: usize = 16 * 1024 * 1024;

/// Default maximum decoded bytes for one image XObject.
pub const DEFAULT_IMAGE_BYTES_LIMIT: usize = 32 * 1024 * 1024;

/// Default maximum resident decoded image bytes for one page resource map.
pub const DEFAULT_TOTAL_IMAGE_BYTES_LIMIT: usize = 128 * 1024 * 1024;

/// Default maximum decoded ICC profile bytes accepted for one image color space.
pub const DEFAULT_ICC_PROFILE_BYTES_LIMIT: usize = 1024 * 1024;

/// Default maximum scratch bytes accepted for one ICC transform.
pub const DEFAULT_ICC_TRANSFORM_WORKSPACE_LIMIT: usize = 64 * 1024;

/// Default maximum cached ICC transform entries.
pub const DEFAULT_ICC_TRANSFORM_CACHE_LIMIT: usize = 32;

/// Default maximum nested soft-mask image depth.
pub const DEFAULT_SOFT_MASK_DEPTH_LIMIT: usize = 1;

/// Default maximum Form XObject recursion depth.
pub const DEFAULT_FORM_RECURSION_DEPTH_LIMIT: usize = 16;

/// Default maximum flattened path line segments for one rasterization pass.
pub const DEFAULT_FLATTENED_PATH_SEGMENT_LIMIT: usize = 65_536;

/// Default maximum pixels in one transparency group intermediate raster.
pub const DEFAULT_TRANSPARENCY_GROUP_PIXELS_LIMIT: usize = 16 * 1024 * 1024;

/// Default maximum number of repeated pattern tiles in one rasterization pass.
pub const DEFAULT_PATTERN_TILE_LIMIT: usize = 65_536;

/// Default maximum cached tiling pattern cells per rasterization pass.
pub const DEFAULT_PATTERN_CELL_CACHE_LIMIT: usize = 32;

/// Default maximum decoded bytes accepted for one mesh shading stream.
pub const DEFAULT_MESH_SHADING_BYTES_LIMIT: usize = 1024 * 1024;

/// Default maximum triangles accepted in one decoded mesh shading.
pub const DEFAULT_MESH_SHADING_TRIANGLE_LIMIT: usize = 8_192;

/// Maximum dash segments tracked in the graphics state.
pub const MAX_STROKE_DASH_SEGMENTS: usize = 8;

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
    /// Maximum pixels accepted in one transparency group intermediate raster.
    pub max_transparency_group_pixels: usize,
    /// Maximum repeated pattern tiles accepted in one rasterization pass.
    pub max_pattern_tiles: usize,
    /// Maximum cached tiling pattern cells in one rasterization pass.
    pub max_pattern_cell_cache_entries: usize,
}

impl Default for PathRasterOptions {
    fn default() -> Self {
        Self {
            supersample: 2,
            max_flattened_segments: DEFAULT_FLATTENED_PATH_SEGMENT_LIMIT,
            max_transparency_group_pixels: DEFAULT_TRANSPARENCY_GROUP_PIXELS_LIMIT,
            max_pattern_tiles: DEFAULT_PATTERN_TILE_LIMIT,
            max_pattern_cell_cache_entries: DEFAULT_PATTERN_CELL_CACHE_LIMIT,
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
    /// RGB thumbnail approximation of a PDF spot color space.
    Spot {
        /// Red channel, normalized to `0.0..=1.0`.
        r: f64,
        /// Green channel, normalized to `0.0..=1.0`.
        g: f64,
        /// Blue channel, normalized to `0.0..=1.0`.
        b: f64,
        /// Approximation metadata exposed for diagnostics and reports.
        approximation: SpotColorApproximation,
    },
}

impl DeviceColor {
    /// Black DeviceGray.
    pub const BLACK: Self = Self::Gray(DeviceGray::BLACK);

    /// Returns spot-color approximation metadata, when this color came from a
    /// `/Separation` or `/DeviceN` color space.
    #[must_use]
    pub const fn spot_approximation(self) -> Option<SpotColorApproximation> {
        match self {
            Self::Spot { approximation, .. } => Some(approximation),
            Self::Gray(_) | Self::Rgb { .. } => None,
        }
    }
}

/// PDF spot color-space family approximated into RGB for thumbnail output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotColorSpaceKind {
    /// PDF `/Separation` color space.
    Separation,
    /// PDF `/DeviceN` color space.
    DeviceN,
}

/// Diagnostic metadata for spot-color thumbnail approximations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpotColorApproximation {
    /// Spot color-space family.
    pub kind: SpotColorSpaceKind,
    /// Number of spot colorants/tint operands consumed.
    pub colorant_count: usize,
    /// Alternate color space used by the tint transform.
    pub alternate_space: AlternateColorSpace,
}

/// Supported alternate color spaces for spot-color approximation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlternateColorSpace {
    /// PDF `/DeviceGray`.
    DeviceGray,
    /// PDF `/DeviceRGB`.
    DeviceRgb,
    /// PDF `/DeviceCMYK`.
    DeviceCmyk,
}

/// Fill color-space mode tracked for pattern color setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillColorSpace {
    /// Normal device color operators.
    Device,
    /// PDF `/Pattern` color space for colored tiling patterns.
    Pattern,
    /// PDF `[/Pattern <base-space>]` color space for uncolored tiling patterns.
    UncoloredPattern(PatternBaseColorSpace),
    /// PDF `/Separation` or `/DeviceN` color-space resource.
    Spot(usize),
}

/// Stroke color-space mode tracked for spot-color setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeColorSpace {
    /// Normal device color operators.
    Device,
    /// PDF `/Separation` or `/DeviceN` color-space resource.
    Spot(usize),
}

/// Base color space used by an uncolored tiling pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternBaseColorSpace {
    /// PDF `/DeviceGray`.
    DeviceGray,
    /// PDF `/DeviceRGB`.
    DeviceRgb,
    /// PDF `/DeviceCMYK`.
    DeviceCmyk,
}

impl PatternBaseColorSpace {
    fn component_count(self) -> usize {
        match self {
            Self::DeviceGray => 1,
            Self::DeviceRgb => 3,
            Self::DeviceCmyk => 4,
        }
    }

    fn color_from_operands(
        self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<DeviceColor> {
        match self {
            Self::DeviceGray => Ok(DeviceColor::Gray(DeviceGray(
                number_from_primitive(&operands[0])
                    .ok_or_else(|| invalid_operand(offset, operator))?
                    .clamp(0.0, 1.0),
            ))),
            Self::DeviceRgb => Ok(DeviceColor::Rgb {
                r: number_from_primitive(&operands[0])
                    .ok_or_else(|| invalid_operand(offset, operator))?
                    .clamp(0.0, 1.0),
                g: number_from_primitive(&operands[1])
                    .ok_or_else(|| invalid_operand(offset, operator))?
                    .clamp(0.0, 1.0),
                b: number_from_primitive(&operands[2])
                    .ok_or_else(|| invalid_operand(offset, operator))?
                    .clamp(0.0, 1.0),
            }),
            Self::DeviceCmyk => {
                let color = alternate_color_to_rgb(
                    AlternateColorSpace::DeviceCmyk,
                    [
                        number_from_primitive(&operands[0])
                            .ok_or_else(|| invalid_operand(offset, operator))?,
                        number_from_primitive(&operands[1])
                            .ok_or_else(|| invalid_operand(offset, operator))?,
                        number_from_primitive(&operands[2])
                            .ok_or_else(|| invalid_operand(offset, operator))?,
                        number_from_primitive(&operands[3])
                            .ok_or_else(|| invalid_operand(offset, operator))?,
                    ],
                );
                Ok(DeviceColor::Rgb {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                })
            }
        }
    }
}

/// Stroke dash pattern tracked in graphics state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeDashPattern {
    /// Dash and gap lengths.
    pub segments: [f64; MAX_STROKE_DASH_SEGMENTS],
    /// Number of active dash and gap lengths.
    pub len: usize,
    /// Initial dash phase.
    pub phase: f64,
}

impl StrokeDashPattern {
    /// Solid stroke pattern with no dashes.
    #[must_use]
    pub const fn solid() -> Self {
        Self {
            segments: [0.0; MAX_STROKE_DASH_SEGMENTS],
            len: 0,
            phase: 0.0,
        }
    }

    fn is_solid(self) -> bool {
        self.active_len() == 0
    }

    fn active_len(self) -> usize {
        self.len.min(MAX_STROKE_DASH_SEGMENTS)
    }

    fn scaled(self, scale: f64) -> Self {
        let mut scaled = self;
        scaled.len = scaled.active_len();
        let mut index = 0;
        while index < scaled.len {
            scaled.segments[index] *= scale;
            index += 1;
        }
        scaled.phase *= scale;
        scaled
    }
}

impl Default for StrokeDashPattern {
    fn default() -> Self {
        Self::solid()
    }
}

/// Stroke line-cap style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    /// End strokes exactly at path endpoints.
    #[default]
    Butt,
    /// Add a semicircular cap at path endpoints.
    Round,
    /// Extend strokes by half the line width at path endpoints.
    Square,
}

impl LineCap {
    fn from_pdf(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Butt),
            1 => Some(Self::Round),
            2 => Some(Self::Square),
            _ => None,
        }
    }
}

/// Stroke line-join style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    /// Extend outer edges until the configured miter limit is reached.
    #[default]
    Miter,
    /// Add a circular join around the path vertex.
    Round,
    /// Connect outer stroke corners directly.
    Bevel,
}

impl LineJoin {
    fn from_pdf(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Miter),
            1 => Some(Self::Round),
            2 => Some(Self::Bevel),
            _ => None,
        }
    }
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
    /// Current fill color-space mode.
    pub fill_color_space: FillColorSpace,
    /// Current stroke color-space mode.
    pub stroke_color_space: StrokeColorSpace,
    /// Current fill pattern resource index, if `/Pattern` color space is active.
    pub fill_pattern: Option<usize>,
    /// Current stroke dash pattern.
    pub stroke_dash: StrokeDashPattern,
    /// Current stroke line-cap style.
    pub line_cap: LineCap,
    /// Current stroke line-join style.
    pub line_join: LineJoin,
    /// Current stroke miter limit.
    pub miter_limit: f64,
    /// Current blend mode for path painting.
    pub blend_mode: BlendMode,
    /// Current nonstroking alpha constant.
    pub fill_alpha: f64,
    /// Current stroking alpha constant.
    pub stroke_alpha: f64,
    /// Whether nonstroking overprint was requested and approximated in RGB.
    pub fill_overprint: bool,
    /// Whether stroking overprint was requested and approximated in RGB.
    pub stroke_overprint: bool,
    /// Current PDF overprint mode, validated but approximated in RGB output.
    pub overprint_mode: u8,
    /// Current graphics-state stack depth for scoping clipping paths.
    pub graphics_state_depth: usize,
    /// Current graphics-state scope id for distinguishing sibling save scopes.
    pub graphics_state_scope_id: u64,
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
            fill_color_space: FillColorSpace::Device,
            stroke_color_space: StrokeColorSpace::Device,
            fill_pattern: None,
            stroke_dash: StrokeDashPattern::solid(),
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            blend_mode: BlendMode::Normal,
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            fill_overprint: false,
            stroke_overprint: false,
            overprint_mode: 0,
            graphics_state_depth: 0,
            graphics_state_scope_id: 0,
            clip_path_pending: false,
        }
    }
}

/// Supported PDF blend modes for thumbnail rasterization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// PDF `Normal` blend mode.
    Normal,
    /// PDF `Multiply` blend mode.
    Multiply,
    /// PDF `Screen` blend mode.
    Screen,
}

/// Parsed external graphics state subset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtGraphicsState {
    /// Blend mode applied by `gs`.
    pub blend_mode: BlendMode,
    /// Nonstroking alpha constant from `/ca`.
    pub fill_alpha: f64,
    /// Stroking alpha constant from `/CA`.
    pub stroke_alpha: f64,
    /// Nonstroking overprint flag from `/op`.
    pub fill_overprint: bool,
    /// Stroking overprint flag from `/OP`.
    pub stroke_overprint: bool,
    /// Overprint mode from `/OPM`.
    pub overprint_mode: u8,
}

impl Default for ExtGraphicsState {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::Normal,
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            fill_overprint: false,
            stroke_overprint: false,
            overprint_mode: 0,
        }
    }
}

/// Decoded shading resource subset.
#[derive(Debug, Clone, PartialEq)]
pub enum Shading {
    /// Axial gradient shading.
    Axial(AxialShading),
    /// Radial gradient shading.
    Radial(RadialShading),
    /// Free-form Gouraud triangle mesh shading.
    Mesh(MeshShading),
}

/// Axial shading data used by thumbnail rasterization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxialShading {
    /// Start point in shading coordinates.
    pub start: Point,
    /// End point in shading coordinates.
    pub end: Point,
    /// Start color.
    pub start_color: DeviceColor,
    /// End color.
    pub end_color: DeviceColor,
    /// Exponent from the sampled Type 2 function.
    pub exponent: f64,
    /// Whether samples before the start point extend the start color.
    pub extend_start: bool,
    /// Whether samples after the end point extend the end color.
    pub extend_end: bool,
}

/// Radial shading data used by thumbnail rasterization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialShading {
    /// Start circle center in shading coordinates.
    pub start_center: Point,
    /// Start circle radius.
    pub start_radius: f64,
    /// End circle center in shading coordinates.
    pub end_center: Point,
    /// End circle radius.
    pub end_radius: f64,
    /// Start color.
    pub start_color: DeviceColor,
    /// End color.
    pub end_color: DeviceColor,
    /// Exponent from the sampled Type 2 function.
    pub exponent: f64,
    /// Whether samples before the start circle extend the start color.
    pub extend_start: bool,
    /// Whether samples after the end circle extend the end color.
    pub extend_end: bool,
}

/// Bounded triangle mesh shading data used by thumbnail rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshShading {
    /// Decoded Gouraud triangles.
    pub triangles: Vec<MeshTriangle>,
}

/// One decoded mesh shading triangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshTriangle {
    /// Triangle vertices in shading coordinates.
    pub vertices: [MeshVertex; 3],
}

/// One decoded mesh shading vertex.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshVertex {
    /// Vertex point in shading coordinates.
    pub point: Point,
    /// Vertex color.
    pub color: DeviceColor,
}

/// PDF tiling pattern paint mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilingPatternPaint {
    /// Pattern stream supplies its own colors.
    Colored,
    /// Caller supplies the paint color through a pattern color space.
    Uncolored,
}

/// Decoded tiling pattern resource.
#[derive(Debug, Clone, PartialEq)]
pub struct TilingPattern {
    /// Pattern resource name without the leading slash.
    pub resource_name: Vec<u8>,
    /// Whether the pattern stream is colored or caller-colored.
    pub paint: TilingPatternPaint,
    /// Pattern cell bounding box.
    pub bbox: PathBounds,
    /// Horizontal tile step.
    pub x_step: f64,
    /// Vertical tile step.
    pub y_step: f64,
    /// Decoded pattern cell path items.
    pub items: DisplayList,
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
    /// Optional colored tiling pattern used by fill painting.
    pub fill_pattern: Option<TilingPattern>,
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
    /// Returns true when this text rendering mode can paint glyph pixels.
    #[must_use]
    pub const fn paints_pixels(self) -> bool {
        !matches!(self, Self::Invisible | Self::Clip)
    }

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
        if !self.paints_pixels() {
            return None;
        }
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
    /// Native text layout handling selected for this mapped glyph.
    pub layout: TextLayoutStatus,
}

/// Native text layout handling selected for a decoded PDF glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLayoutStatus {
    /// One source glyph maps to one simple fallback-rasterized Unicode scalar.
    Simple,
    /// One source glyph maps to multiple Unicode scalars such as a ligature expansion.
    LigatureExpanded,
    /// The mapped text contains combining marks positioned by the fallback renderer.
    CombiningMarkPositioned,
    /// The PDF already positioned shaped script glyphs in the content stream.
    PreShapedScriptPreserved,
    /// The mapped text needs a layout behavior outside the native fallback subset.
    Unsupported {
        /// Typed reason for the unsupported native layout path.
        reason: TextLayoutFallbackReason,
    },
}

/// Why native text layout fell back from full OpenType shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLayoutFallbackReason {
    /// The text needs script shaping that is not implemented by the native fallback renderer.
    ComplexScriptShaping,
    /// The text needs OpenType GSUB/GPOS table handling that is not implemented yet.
    OpenTypeLayoutTables,
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
    /// Image rendering mode.
    pub kind: ImageKind,
    /// Indexed color lookup bytes for Indexed images.
    pub indexed_lookup: Option<Arc<[u8]>>,
    /// Optional 8-bit alpha mask samples matching the image dimensions.
    pub soft_mask: Option<Arc<[u8]>>,
}

/// Decoded image rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    /// Normal sampled image with its own color samples.
    Color,
    /// One-bit stencil image painted with the current fill color.
    StencilMask {
        /// Whether a set bit paints the current fill color.
        paint_one_bits: bool,
    },
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

/// One path-only transparency group captured before rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct TransparencyGroupDisplayItem {
    /// Nested display-list items captured from the group Form XObject.
    pub items: DisplayList,
    /// Group bounding box transformed into caller user space.
    pub bounds: PathBounds,
    /// Transparency group metadata.
    pub group: TransparencyGroup,
    /// Graphics state snapshot at group invocation time.
    pub state: GraphicsState,
}

/// One shading paint operation captured before rasterization.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadingDisplayItem {
    /// Decoded shading resource.
    pub shading: Shading,
    /// Graphics state snapshot at paint time.
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
    /// Optional transparency-group metadata.
    pub transparency_group: Option<TransparencyGroup>,
}

/// Transparency group metadata supported by the current renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransparencyGroup {
    /// Whether the group is isolated from the backdrop.
    pub isolated: bool,
    /// Whether later group elements knock out earlier elements.
    pub knockout: bool,
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
    /// Path-only transparency group.
    TransparencyGroup(TransparencyGroupDisplayItem),
    /// Shading paint operation.
    Shading(ShadingDisplayItem),
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

    /// Creates a display list from already ordered items.
    #[must_use]
    pub fn from_items(items: Vec<DisplayItem>) -> Self {
        Self { items }
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
                DisplayItem::TransparencyGroup(group) => Some(group.bounds),
                DisplayItem::Shading(_) => None,
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
    icc_metrics: IccTransformMetrics,
}

impl ImageResources {
    /// Creates an empty image resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            images: Vec::new(),
            non_image_names: Vec::new(),
            icc_metrics: IccTransformMetrics::EMPTY,
        }
    }

    /// Creates an image resource map from decoded images.
    #[must_use]
    pub fn new(images: Vec<ImageXObject>) -> Self {
        Self {
            images,
            non_image_names: Vec::new(),
            icc_metrics: IccTransformMetrics::EMPTY,
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
        let mut icc_cache = IccTransformCache::new(options.max_icc_transform_cache_entries);
        Self::from_xobject_dictionary_with_icc_cache(dictionary, resolver, options, &mut icc_cache)
    }

    /// Resolves image XObjects while reusing a caller-owned ICC transform cache.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when an image resource is malformed,
    /// references a missing object, uses an unsupported color space or filter,
    /// or decodes beyond the configured image byte budget.
    pub fn from_xobject_dictionary_with_icc_cache<'a, R>(
        dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
        resolver: &'a R,
        options: DisplayListOptions,
        icc_cache: &mut IccTransformCache,
    ) -> GraphicsResult<Self>
    where
        R: ImageObjectResolver<'a> + ?Sized,
    {
        icc_cache.set_limit(options.max_icc_transform_cache_entries);
        let mut images = Vec::new();
        let mut total_image_bytes = 0usize;
        let mut non_image_names = Vec::new();
        let metrics_start = icc_cache.metrics();
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
            let image = decode_image_xobject(
                *name,
                stream,
                resolver,
                ImageDecodeLimits::from_display_options(options),
                icc_cache,
            )?;
            total_image_bytes = total_image_bytes
                .checked_add(image_resident_bytes(&image))
                .ok_or_else(|| {
                    GraphicsError::new(
                        None,
                        GraphicsErrorKind::ImageResourceBytesOverflow {
                            limit: options.max_total_image_bytes,
                        },
                    )
                })?;
            if total_image_bytes > options.max_total_image_bytes {
                return Err(GraphicsError::new(
                    None,
                    GraphicsErrorKind::ImageResourceBytesOverflow {
                        limit: options.max_total_image_bytes,
                    },
                ));
            }
            images.push(image);
        }
        Ok(Self {
            images,
            non_image_names,
            icc_metrics: icc_cache.metrics().saturating_sub(metrics_start),
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

    /// Returns ICC transform cache activity caused while building this map.
    #[must_use]
    pub const fn icc_transform_metrics(&self) -> IccTransformMetrics {
        self.icc_metrics
    }
}

/// ICC transform cache counters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IccTransformMetrics {
    /// Cache entries hit while resolving ICCBased color spaces.
    pub cache_hits: usize,
    /// Validated transforms inserted into the cache.
    pub cache_misses: usize,
    /// Oldest entries evicted due to the configured limit.
    pub evictions: usize,
    /// Maximum transform workspace bytes validated during this build.
    pub max_workspace_bytes: usize,
}

impl IccTransformMetrics {
    const EMPTY: Self = Self {
        cache_hits: 0,
        cache_misses: 0,
        evictions: 0,
        max_workspace_bytes: 0,
    };

    fn saturating_sub(self, baseline: Self) -> Self {
        Self {
            cache_hits: self.cache_hits.saturating_sub(baseline.cache_hits),
            cache_misses: self.cache_misses.saturating_sub(baseline.cache_misses),
            evictions: self.evictions.saturating_sub(baseline.evictions),
            max_workspace_bytes: self.max_workspace_bytes,
        }
    }
}

/// Bounded cache of validated ICCBased image transform metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IccTransformCache {
    entries: Vec<(IccProfileIdentity, IccTransform)>,
    limit: usize,
    metrics: IccTransformMetrics,
}

impl IccTransformCache {
    /// Creates an ICC transform cache with a maximum entry count.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            limit,
            metrics: IccTransformMetrics::EMPTY,
        }
    }

    fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
        while self.entries.len() > self.limit {
            self.entries.remove(0);
            self.metrics.evictions += 1;
        }
    }

    fn get_or_insert(&mut self, transform: IccTransform) -> IccTransform {
        self.metrics.max_workspace_bytes = self
            .metrics
            .max_workspace_bytes
            .max(transform.workspace_bytes);
        if self.limit == 0 {
            self.metrics.cache_misses += 1;
            return transform;
        }
        if let Some((_, cached)) = self
            .entries
            .iter()
            .find(|(identity, _)| *identity == transform.identity)
        {
            self.metrics.cache_hits += 1;
            return *cached;
        }
        self.metrics.cache_misses += 1;
        if self.entries.len() >= self.limit {
            self.entries.remove(0);
            self.metrics.evictions += 1;
        }
        self.entries.push((transform.identity, transform));
        transform
    }

    /// Returns the number of cached ICC transforms.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when no ICC transforms are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns cumulative cache metrics.
    #[must_use]
    pub const fn metrics(&self) -> IccTransformMetrics {
        self.metrics
    }
}

impl Default for IccTransformCache {
    fn default() -> Self {
        Self::new(DEFAULT_ICC_TRANSFORM_CACHE_LIMIT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IccProfileIdentity {
    hash: u64,
    len: usize,
    components: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IccTransform {
    identity: IccProfileIdentity,
    color_space: ImageColorSpace,
    workspace_bytes: usize,
}

fn image_resident_bytes(image: &ImageXObject) -> usize {
    image.samples.len()
        + image.soft_mask.as_ref().map_or(0, |samples| samples.len())
        + image
            .indexed_lookup
            .as_ref()
            .map_or(0, |lookup| lookup.len())
}

/// External graphics state resource map.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExtGraphicsStateResources {
    states: Vec<(Vec<u8>, ExtGraphicsState)>,
}

impl ExtGraphicsStateResources {
    /// Creates an empty external graphics state resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self { states: Vec::new() }
    }

    /// Resolves external graphics states from a PDF `/ExtGState` resource dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a graphics state dictionary uses an
    /// unsupported blend mode or enabled overprint policy.
    pub fn from_extgstate_dictionary(
        dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    ) -> GraphicsResult<Self> {
        let mut states = Vec::new();
        for (name, value) in dictionary {
            let PdfPrimitive::Dictionary(state_dictionary) = value else {
                return Err(invalid_ext_graphics_state(name.as_bytes()));
            };
            states.push((
                name.as_bytes().to_vec(),
                decode_ext_graphics_state(state_dictionary)?,
            ));
        }
        Ok(Self { states })
    }

    /// Returns the external graphics state matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<ExtGraphicsState> {
        self.states.iter().find_map(|(resource, state)| {
            (resource.as_slice() == name.as_bytes()).then_some(*state)
        })
    }
}

/// Shading resource map.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ShadingResources {
    shadings: Vec<(Vec<u8>, Shading)>,
}

impl ShadingResources {
    /// Creates an empty shading resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            shadings: Vec::new(),
        }
    }

    /// Resolves shadings from a PDF `/Shading` resource dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a shading dictionary uses an unsupported
    /// shading type, color space, or sampled function.
    pub fn from_shading_dictionary(
        dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    ) -> GraphicsResult<Self> {
        let mut shadings = Vec::new();
        for (name, value) in dictionary {
            let PdfPrimitive::Dictionary(shading_dictionary) = value else {
                return Err(invalid_shading_resource(name.as_bytes()));
            };
            shadings.push((
                name.as_bytes().to_vec(),
                decode_shading(shading_dictionary)?,
            ));
        }
        Ok(Self { shadings })
    }

    /// Resolves shadings from a PDF `/Shading` resource dictionary and
    /// decodes stream-backed mesh shadings through the provided resolver.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a shading resource is malformed, missing,
    /// or outside the configured mesh shading budgets.
    pub fn from_shading_dictionary_with_resolver<'a, R>(
        dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
        resolver: &'a R,
        options: DisplayListOptions,
    ) -> GraphicsResult<Self>
    where
        R: ShadingObjectResolver<'a> + ?Sized,
    {
        let mut shadings = Vec::new();
        for (name, value) in dictionary {
            let shading = match value {
                PdfPrimitive::Dictionary(shading_dictionary) => decode_shading(shading_dictionary)?,
                _ => {
                    let reference = reference_from_primitive(value)
                        .ok_or_else(|| invalid_shading_resource(name.as_bytes()))?;
                    let object = resolver
                        .resolve_shading_object(reference)?
                        .ok_or_else(|| invalid_shading_resource(name.as_bytes()))?;
                    let ObjectValue::Stream(stream) = &object.value else {
                        return Err(invalid_shading_resource(name.as_bytes()));
                    };
                    decode_shading_stream(name.as_bytes(), stream, options)?
                }
            };
            shadings.push((name.as_bytes().to_vec(), shading));
        }
        Ok(Self { shadings })
    }

    /// Returns the shading matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&Shading> {
        self.shadings.iter().find_map(|(resource, shading)| {
            (resource.as_slice() == name.as_bytes()).then_some(shading)
        })
    }
}

/// Resolves shading stream references from a loaded PDF document.
pub trait ShadingObjectResolver<'a> {
    /// Resolves an indirect object reference.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when object-stream parsing fails.
    fn resolve_shading_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>>;
}

impl<'a> ShadingObjectResolver<'a> for ClassicDocument<'a> {
    fn resolve_shading_object(
        &'a self,
        reference: Reference,
    ) -> GraphicsResult<Option<IndirectObject<'a>>> {
        Ok(self.objects.get(reference.id).cloned())
    }
}

impl<'a> ShadingObjectResolver<'a> for ModernDocument<'a> {
    fn resolve_shading_object(
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

/// Page color-space resource map for spot-color thumbnail approximation.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ColorSpaceResources {
    color_spaces: Vec<(Vec<u8>, SpotColorSpace)>,
    pattern_color_spaces: Vec<(Vec<u8>, PatternBaseColorSpace)>,
}

impl ColorSpaceResources {
    /// Creates an empty color-space resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            color_spaces: Vec::new(),
            pattern_color_spaces: Vec::new(),
        }
    }

    /// Creates a color-space resource map from decoded spot color spaces.
    #[must_use]
    pub fn new(color_spaces: Vec<(Vec<u8>, SpotColorSpace)>) -> Self {
        Self {
            color_spaces,
            pattern_color_spaces: Vec::new(),
        }
    }

    /// Resolves spot color spaces from a PDF `/ColorSpace` resource dictionary.
    ///
    /// Unsupported non-spot color spaces are ignored so callers can pass the
    /// full page resource dictionary without pre-filtering.
    ///
    /// # Errors
    ///
    /// Returns [`GraphicsError`] when a `/Separation` or `/DeviceN` color-space
    /// resource is malformed or uses an unsupported alternate/tint transform.
    pub fn from_color_space_dictionary(
        dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    ) -> GraphicsResult<Self> {
        let mut color_spaces = Vec::new();
        let mut pattern_color_spaces = Vec::new();
        for (name, value) in dictionary {
            if let Some(base) = decode_pattern_color_space(value)? {
                pattern_color_spaces.push((name.as_bytes().to_vec(), base));
                continue;
            }
            let Some(color_space) = decode_spot_color_space(value)? else {
                continue;
            };
            color_spaces.push((name.as_bytes().to_vec(), color_space));
        }
        Ok(Self {
            color_spaces,
            pattern_color_spaces,
        })
    }

    /// Returns the resource index matching a PDF color-space name.
    #[must_use]
    pub fn index_of(&self, name: PdfName<'_>) -> Option<usize> {
        self.color_spaces
            .iter()
            .position(|(resource, _)| resource.as_slice() == name.as_bytes())
    }

    /// Returns the spot color space at a resource index.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&SpotColorSpace> {
        self.color_spaces
            .get(index)
            .map(|(_, color_space)| color_space)
    }

    fn pattern_base_for_name(&self, name: PdfName<'_>) -> Option<PatternBaseColorSpace> {
        self.pattern_color_spaces
            .iter()
            .find_map(|(resource, base)| (resource.as_slice() == name.as_bytes()).then_some(*base))
    }
}

/// Decoded `/Separation` or `/DeviceN` color-space approximation data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotColorSpace {
    kind: SpotColorSpaceKind,
    colorant_count: usize,
    alternate_space: AlternateColorSpace,
    tint_transform: Type2TintFunction,
}

impl SpotColorSpace {
    fn evaluate(self, tints: &[f64]) -> DeviceColor {
        let alternate = self.tint_transform.evaluate(tints);
        let color = alternate_color_to_rgb(self.alternate_space, alternate);
        DeviceColor::Spot {
            r: color[0],
            g: color[1],
            b: color[2],
            approximation: SpotColorApproximation {
                kind: self.kind,
                colorant_count: self.colorant_count,
                alternate_space: self.alternate_space,
            },
        }
    }
}

const MAX_TINT_FUNCTION_COMPONENTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Type2TintFunction {
    c0: [f64; MAX_TINT_FUNCTION_COMPONENTS],
    c1: [f64; MAX_TINT_FUNCTION_COMPONENTS],
    output_components: usize,
    exponent: f64,
}

impl Type2TintFunction {
    fn evaluate(self, tints: &[f64]) -> [f64; MAX_TINT_FUNCTION_COMPONENTS] {
        let tint = average_tint(tints).powf(self.exponent);
        let mut output = [0.0; MAX_TINT_FUNCTION_COMPONENTS];
        for (index, channel) in output.iter_mut().enumerate().take(self.output_components) {
            *channel = self.c0[index]
                .mul_add(1.0 - tint, self.c1[index] * tint)
                .clamp(0.0, 1.0);
        }
        output
    }
}

fn average_tint(tints: &[f64]) -> f64 {
    if tints.is_empty() {
        return 0.0;
    }
    let sum: f64 = tints.iter().map(|value| value.clamp(0.0, 1.0)).sum();
    (sum / tints.len() as f64).clamp(0.0, 1.0)
}

/// Tiling pattern resource map.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TilingPatternResources {
    patterns: Vec<TilingPattern>,
}

impl TilingPatternResources {
    /// Creates an empty tiling pattern resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Creates a tiling pattern resource map from decoded patterns.
    #[must_use]
    pub fn new(patterns: Vec<TilingPattern>) -> Self {
        Self { patterns }
    }

    /// Returns the resource index matching a PDF pattern name.
    #[must_use]
    pub fn index_of(&self, name: PdfName<'_>) -> Option<usize> {
        self.patterns
            .iter()
            .position(|pattern| pattern.resource_name.as_slice() == name.as_bytes())
    }

    /// Returns the pattern at a resource index.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&TilingPattern> {
        self.patterns.get(index)
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
    /// Maximum operands accepted on one charstring stack.
    pub max_charstring_stack: usize,
    /// Maximum nested charstring subroutine depth.
    pub max_charstring_subroutine_depth: usize,
}

impl Default for GlyphOutlineOptions {
    fn default() -> Self {
        Self {
            max_segments: DEFAULT_GLYPH_OUTLINE_SEGMENT_LIMIT,
            max_cache_entries: DEFAULT_GLYPH_OUTLINE_CACHE_LIMIT,
            max_charstring_stack: DEFAULT_CHARSTRING_STACK_LIMIT,
            max_charstring_subroutine_depth: DEFAULT_CHARSTRING_SUBROUTINE_DEPTH_LIMIT,
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

/// Text subpixel positioning policy used by the native fallback text rasterizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSubpixelPolicy {
    /// Preserve user-space glyph origins until final device-pixel coverage.
    PreserveUserSpace,
}

/// Current native text positioning policy.
pub const TEXT_SUBPIXEL_POLICY: TextSubpixelPolicy = TextSubpixelPolicy::PreserveUserSpace;

/// Deterministic built-in font face used by the native fallback text rasterizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFallbackFace {
    /// Sans-serif fallback used for Helvetica, Arial, and unknown families.
    Sans,
    /// Serif fallback used for Times-like families.
    Serif,
    /// Monospace fallback used for Courier-like families.
    Monospace,
    /// Symbol fallback bucket for symbolic base fonts.
    Symbol,
}

/// Why the fallback rasterizer selected a built-in font face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFallbackSource {
    /// Embedded program exists, but the current rasterizer still uses the
    /// deterministic built-in bitmap fallback for visible text.
    EmbeddedProgram,
    /// PDF standard/base font without an embedded program.
    StandardBase,
    /// Non-standard named font without an embedded program.
    MissingEmbeddedProgram,
    /// No usable base font metadata was present.
    Unspecified,
}

/// Resolved deterministic built-in fallback face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontFallback {
    /// Built-in face selected by the deterministic policy.
    pub face: FontFallbackFace,
    /// Reason the built-in face was selected.
    pub source: FontFallbackSource,
}

impl FontFallback {
    fn resolve(
        base_font: Option<&[u8]>,
        subtype: Option<FontSubtype>,
        has_embedded_program: bool,
    ) -> Self {
        let face = fallback_face_for_base_font(base_font);
        let source = if has_embedded_program {
            FontFallbackSource::EmbeddedProgram
        } else if base_font.is_none() {
            FontFallbackSource::Unspecified
        } else if is_standard_base_font(base_font.unwrap_or_default(), subtype) {
            FontFallbackSource::StandardBase
        } else {
            FontFallbackSource::MissingEmbeddedProgram
        };
        Self { face, source }
    }
}

/// Small fallback glyph bitmap cache keyed by face, glyph, and quantized size.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphBitmapCache {
    entries: Vec<CachedGlyphBitmap>,
    max_entries: usize,
}

impl GlyphBitmapCache {
    /// Creates a fallback glyph bitmap cache with a bounded entry count.
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    fn bitmap_for(&mut self, fallback: FontFallback, character: char, cell: f64) -> &GlyphBitmap {
        let key = GlyphBitmapKey::new(fallback, character, cell);
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            return &self.entries[index].bitmap;
        }
        let bitmap = GlyphBitmap::from_ascii(character, cell, key.paint_policy);
        if self.max_entries == 0 {
            self.entries.clear();
            self.entries.push(CachedGlyphBitmap { key, bitmap });
            return &self.entries[0].bitmap;
        }
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(CachedGlyphBitmap { key, bitmap });
        &self.entries.last().expect("entry was just inserted").bitmap
    }

    /// Returns the number of cached fallback glyph bitmaps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when no fallback glyph bitmaps are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for GlyphBitmapCache {
    fn default() -> Self {
        Self::new(DEFAULT_GLYPH_BITMAP_CACHE_LIMIT)
    }
}

#[derive(Debug)]
struct TextRasterScratch {
    atoms: Vec<TextRasterAtom>,
    max_retained_atoms: usize,
}

impl TextRasterScratch {
    fn new(max_retained_atoms: usize) -> Self {
        Self {
            atoms: Vec::new(),
            max_retained_atoms,
        }
    }

    fn prepare(&mut self, text: &TextDisplayItem, cell: f64) {
        self.reset_for(text.glyphs.len());
        for (glyph, origin) in text.glyphs.iter().zip(text.glyph_origins.iter()) {
            let mut pen_x = origin.x;
            let mut last_base_x = origin.x;
            for character in glyph.unicode.chars() {
                if is_combining_mark(character) {
                    self.atoms.push(TextRasterAtom {
                        kind: TextRasterAtomKind::CombiningMark(character),
                        x: last_base_x,
                        baseline_y: origin.y,
                    });
                    continue;
                }
                self.atoms.push(TextRasterAtom {
                    kind: TextRasterAtomKind::Glyph(character),
                    x: pen_x,
                    baseline_y: origin.y,
                });
                last_base_x = pen_x;
                pen_x += fallback_glyph_advance(cell);
            }
        }
    }

    fn reset_for(&mut self, expected_atoms: usize) {
        if self.atoms.capacity() > self.max_retained_atoms
            && expected_atoms <= self.max_retained_atoms
        {
            self.atoms = Vec::with_capacity(expected_atoms);
            return;
        }
        self.atoms.clear();
    }
}

impl Default for TextRasterScratch {
    fn default() -> Self {
        Self::new(DEFAULT_TEXT_RASTER_SCRATCH_RETAINED_ATOMS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextRasterAtom {
    kind: TextRasterAtomKind,
    x: f64,
    baseline_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextRasterAtomKind {
    Glyph(char),
    CombiningMark(char),
}

fn fallback_glyph_advance(cell: f64) -> f64 {
    cell * 6.0
}

fn fallback_text_cell(font_size: f64, fallback: FontFallback) -> f64 {
    let scale = match fallback.source {
        FontFallbackSource::StandardBase => STANDARD_BASE_FONT_CELL_SCALE,
        FontFallbackSource::EmbeddedProgram
        | FontFallbackSource::MissingEmbeddedProgram
        | FontFallbackSource::Unspecified => 1.0,
    };
    font_size * scale / 7.0
}

fn scaled_fallback_text_cell(font_size: f64, fallback: FontFallback, state: GraphicsState) -> f64 {
    fallback_text_cell(font_size, fallback) * matrix_average_scale(state.ctm)
}

fn matrix_average_scale(matrix: Matrix) -> f64 {
    let x_scale = matrix.a.hypot(matrix.b);
    let y_scale = matrix.c.hypot(matrix.d);
    ((x_scale + y_scale) / 2.0).max(f64::EPSILON)
}

fn standard_base_glyph_width(face: FontFallbackFace, unicode: &str) -> Option<f64> {
    let mut chars = unicode.chars();
    let character = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    match face {
        FontFallbackFace::Sans => standard_sans_glyph_width(character),
        FontFallbackFace::Monospace => character.is_ascii().then_some(600.0),
        FontFallbackFace::Serif => standard_serif_glyph_width(character),
        FontFallbackFace::Symbol => None,
    }
}

fn standard_sans_glyph_width(character: char) -> Option<f64> {
    let width = match character {
        ' ' | '!' | ',' | '.' | ':' | ';' => 278.0,
        '"' => 355.0,
        '#' | '$' | '0'..='9' | '?' | '_' => 556.0,
        '%' => 889.0,
        '&' | 'A' | 'B' | 'P' | 'X' => 667.0,
        '\'' | '`' | 'i' | 'j' | 'l' => 222.0,
        '(' | ')' | '-' | '[' | ']' => 333.0,
        '*' => 389.0,
        '+' | '<' | '=' | '>' | '~' => 584.0,
        '/' | '\\' | 'I' | 'f' | 't' => 278.0,
        '@' => 1015.0,
        'C' | 'D' | 'N' | 'R' | 'U' => 722.0,
        'E' | 'K' | 'S' => 667.0,
        'F' | 'T' | 'Z' => 611.0,
        'G' | 'O' | 'Q' => 778.0,
        'H' => 722.0,
        'J' | 'a' | 'b' | 'd' | 'e' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' => 556.0,
        'L' => 556.0,
        'M' => 833.0,
        'V' | 'Y' => 667.0,
        'W' => 944.0,
        '^' => 469.0,
        'c' | 'k' | 's' | 'v' | 'x' | 'y' | 'z' => 500.0,
        'm' => 833.0,
        'r' => 333.0,
        'w' => 722.0,
        '{' | '}' => 334.0,
        '|' => 260.0,
        _ => return None,
    };
    Some(width)
}

fn standard_serif_glyph_width(character: char) -> Option<f64> {
    let width = match character {
        ' ' | ',' | '.' => 250.0,
        '!' | '(' | ')' | '-' | 'I' | '[' | ']' | '`' => 333.0,
        '"' => 408.0,
        '#'
        | '$'
        | '*'
        | '0'..='9'
        | '_'
        | 'b'
        | 'd'
        | 'g'
        | 'h'
        | 'k'
        | 'n'
        | 'o'
        | 'p'
        | 'q'
        | 'u'
        | 'x' => 500.0,
        '%' => 833.0,
        '&' | 'm' => 778.0,
        '\'' => 180.0,
        '+' | '<' | '=' | '>' => 564.0,
        '/' | '\\' | ':' | ';' | 'i' | 'j' | 'l' | 't' => 278.0,
        '?' | 'a' | 'c' | 'e' | 'z' => 444.0,
        '@' => 921.0,
        'A' | 'D' | 'G' | 'H' | 'K' | 'N' | 'O' | 'Q' | 'U' | 'V' | 'X' | 'Y' | 'w' => 722.0,
        'B' | 'C' | 'R' => 667.0,
        'E' | 'L' | 'T' | 'Z' => 611.0,
        'F' | 'P' | 'S' => 556.0,
        'J' | 's' => 389.0,
        'M' => 889.0,
        'W' => 944.0,
        '^' => 469.0,
        'f' | 'r' => 333.0,
        'v' | 'y' => 500.0,
        '{' | '}' => 480.0,
        '|' => 200.0,
        '~' => 541.0,
        _ => return None,
    };
    Some(width)
}

#[derive(Debug, Clone, PartialEq)]
struct CachedGlyphBitmap {
    key: GlyphBitmapKey,
    bitmap: GlyphBitmap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphBitmapKey {
    face: FontFallbackFace,
    character: char,
    cell_microunits: i64,
    paint_policy: GlyphBitmapPaintPolicy,
}

impl GlyphBitmapKey {
    fn new(fallback: FontFallback, character: char, cell: f64) -> Self {
        Self {
            face: fallback.face,
            character,
            cell_microunits: quantize_glyph_cell(cell),
            paint_policy: GlyphBitmapPaintPolicy::from_fallback_source(fallback.source),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphBitmapPaintPolicy {
    MaskOnly,
    StandardBaseThin,
}

impl GlyphBitmapPaintPolicy {
    const fn from_fallback_source(source: FontFallbackSource) -> Self {
        match source {
            FontFallbackSource::StandardBase => Self::StandardBaseThin,
            FontFallbackSource::EmbeddedProgram
            | FontFallbackSource::MissingEmbeddedProgram
            | FontFallbackSource::Unspecified => Self::MaskOnly,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GlyphBitmap {
    rects: Vec<GlyphBitmapRect>,
}

impl GlyphBitmap {
    fn from_ascii(character: char, cell: f64, paint_policy: GlyphBitmapPaintPolicy) -> Self {
        let glyph = ascii_glyph(character);
        let mut rects = Vec::new();
        for (row, pattern) in glyph.iter().enumerate() {
            for (col, byte) in pattern.as_bytes().iter().enumerate() {
                if *byte != b'#' {
                    continue;
                }
                let left = col as f64 * cell;
                let right = left + cell;
                let top = (7 - row) as f64 * cell;
                let bottom = top - cell;
                rects.push(GlyphBitmapRect::new(left, right, top, bottom, paint_policy));
            }
        }
        Self { rects }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlyphBitmapRect {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl GlyphBitmapRect {
    fn new(
        left: f64,
        right: f64,
        top: f64,
        bottom: f64,
        paint_policy: GlyphBitmapPaintPolicy,
    ) -> Self {
        match paint_policy {
            GlyphBitmapPaintPolicy::MaskOnly => Self {
                left,
                right,
                top,
                bottom,
            },
            GlyphBitmapPaintPolicy::StandardBaseThin => {
                let width = right - left;
                let height = top - bottom;
                let inset_x = width * 0.06;
                let inset_y = height * 0.06;
                Self {
                    left: left + inset_x,
                    right: right - inset_x,
                    top: top - inset_y,
                    bottom: bottom + inset_y,
                }
            }
        }
    }
}

fn quantize_glyph_cell(cell: f64) -> i64 {
    (cell * 1_000_000.0).round() as i64
}

fn fallback_face_for_base_font(base_font: Option<&[u8]>) -> FontFallbackFace {
    let Some(name) = base_font.map(strip_subset_font_prefix) else {
        return FontFallbackFace::Sans;
    };
    if ascii_contains_ignore_case(name, b"symbol")
        || ascii_contains_ignore_case(name, b"zapfdingbats")
    {
        FontFallbackFace::Symbol
    } else if ascii_contains_ignore_case(name, b"courier")
        || ascii_contains_ignore_case(name, b"consolas")
        || ascii_contains_ignore_case(name, b"monaco")
        || ascii_contains_ignore_case(name, b"mono")
    {
        FontFallbackFace::Monospace
    } else if ascii_contains_ignore_case(name, b"times")
        || ascii_contains_ignore_case(name, b"georgia")
        || ascii_contains_ignore_case(name, b"serif")
    {
        FontFallbackFace::Serif
    } else {
        FontFallbackFace::Sans
    }
}

fn is_standard_base_font(base_font: &[u8], subtype: Option<FontSubtype>) -> bool {
    let name = strip_subset_font_prefix(base_font);
    matches!(
        subtype,
        Some(FontSubtype::Type1 | FontSubtype::TrueType | FontSubtype::Type0)
    ) && (ascii_eq_ignore_case(name, b"courier")
        || ascii_eq_ignore_case(name, b"courier-bold")
        || ascii_eq_ignore_case(name, b"courier-oblique")
        || ascii_eq_ignore_case(name, b"courier-boldoblique")
        || ascii_eq_ignore_case(name, b"helvetica")
        || ascii_eq_ignore_case(name, b"helvetica-bold")
        || ascii_eq_ignore_case(name, b"helvetica-oblique")
        || ascii_eq_ignore_case(name, b"helvetica-boldoblique")
        || ascii_eq_ignore_case(name, b"times-roman")
        || ascii_eq_ignore_case(name, b"times-bold")
        || ascii_eq_ignore_case(name, b"times-italic")
        || ascii_eq_ignore_case(name, b"times-bolditalic")
        || ascii_eq_ignore_case(name, b"symbol")
        || ascii_eq_ignore_case(name, b"zapfdingbats"))
}

fn strip_subset_font_prefix(name: &[u8]) -> &[u8] {
    if name.len() > 7 && name[6] == b'+' && name[..6].iter().all(|byte| byte.is_ascii_uppercase()) {
        &name[7..]
    } else {
        name
    }
}

fn ascii_eq_ignore_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn ascii_contains_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| ascii_eq_ignore_case(window, needle))
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
    /// malformed, or exceeds the configured segment budget.
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
        let outline = extract_glyph_outline(program, glyph_code, options)?;
        if options.max_cache_entries == 0 {
            return Ok(outline);
        }
        if self.outlines.len() >= options.max_cache_entries {
            self.outlines.remove(0);
        }
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
        FontProgramKind::Type1 => extract_type1_glyph_outline(program, glyph_code, options),
    }
}

/// Single-byte font encoding metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontEncoding {
    differences: Vec<FontEncodingDifference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FontEncodingDifference {
    code: u8,
    name: Vec<u8>,
    character: char,
}

impl FontEncoding {
    /// Creates an encoding with no differences from the ASCII-compatible base.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            differences: Vec::new(),
        }
    }

    fn with_differences(differences: Vec<FontEncodingDifference>) -> Self {
        Self { differences }
    }

    fn decode_byte(&self, byte: u8) -> Option<char> {
        self.differences
            .iter()
            .find_map(|difference| (difference.code == byte).then_some(difference.character))
            .or_else(|| byte.is_ascii().then_some(byte as char))
    }

    fn glyph_name_for_code(&self, code: u8) -> Vec<u8> {
        self.differences
            .iter()
            .find_map(|difference| (difference.code == code).then(|| difference.name.clone()))
            .unwrap_or_else(|| ascii_glyph_name(code))
    }
}

impl Default for FontEncoding {
    fn default() -> Self {
        Self::new()
    }
}

fn ascii_glyph_name(code: u8) -> Vec<u8> {
    match code {
        b' ' => b"space".to_vec(),
        b'-' => b"hyphen".to_vec(),
        b'.' => b"period".to_vec(),
        b',' => b"comma".to_vec(),
        _ if code.is_ascii_alphanumeric() => vec![code],
        _ => b".notdef".to_vec(),
    }
}

/// Parsed ToUnicode CMap entries for character-code mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToUnicodeMap {
    entries: Vec<ToUnicodeEntry>,
    code_space_ranges: Vec<CodeSpaceRange>,
    max_code_len: usize,
    identity_width: Option<usize>,
}

impl ToUnicodeMap {
    fn new(entries: Vec<ToUnicodeEntry>, code_space_ranges: Vec<CodeSpaceRange>) -> Self {
        let max_entry_len = entries
            .iter()
            .map(|entry| entry.code.len())
            .max()
            .unwrap_or(0);
        let max_range_len = code_space_ranges
            .iter()
            .map(|range| range.start.len())
            .max()
            .unwrap_or(0);
        Self {
            entries,
            code_space_ranges,
            max_code_len: max_entry_len.max(max_range_len),
            identity_width: None,
        }
    }

    fn identity(width: usize) -> Self {
        let max = vec![0xff; width];
        Self {
            entries: Vec::new(),
            code_space_ranges: vec![CodeSpaceRange {
                start: vec![0; width],
                end: max,
            }],
            max_code_len: width,
            identity_width: Some(width),
        }
    }

    fn match_code(&self, bytes: &[u8], offset: usize) -> Option<(Cow<'_, str>, usize, u32)> {
        let remaining = bytes.len().saturating_sub(offset);
        let max_width = self.max_code_len.min(remaining);
        for width in (1..=max_width).rev() {
            let code = &bytes[offset..offset + width];
            if !self.code_space_ranges.is_empty() && !self.code_space_contains(code) {
                continue;
            }
            if let Some(entry) = self.entries.iter().find(|entry| entry.code == code) {
                return Some((
                    Cow::Borrowed(entry.text.as_str()),
                    width,
                    bytes_to_u32(code),
                ));
            }
            if self.identity_width == Some(width) {
                let scalar = bytes_to_u32(code);
                let character = char::from_u32(scalar)?;
                return Some((Cow::Owned(character.to_string()), width, scalar));
            }
        }
        None
    }

    fn code_space_contains(&self, code: &[u8]) -> bool {
        self.code_space_ranges
            .iter()
            .any(|range| range.contains(code))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToUnicodeEntry {
    code: Vec<u8>,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeSpaceRange {
    start: Vec<u8>,
    end: Vec<u8>,
}

impl CodeSpaceRange {
    fn contains(&self, code: &[u8]) -> bool {
        code.len() == self.start.len()
            && bytes_to_u32(&self.start) <= bytes_to_u32(code)
            && bytes_to_u32(code) <= bytes_to_u32(&self.end)
    }
}

/// CID descendant font metadata used by composite fonts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CidFontMetrics {
    /// Descendant CID font subtype.
    pub subtype: FontSubtype,
    /// Default glyph width in glyph space units from `/DW`.
    pub default_width: Option<i32>,
}

/// Decoded Type 3 font metadata and glyph programs.
#[derive(Debug, Clone, PartialEq)]
pub struct Type3Font {
    /// Matrix mapping Type 3 glyph space into text space.
    pub font_matrix: Matrix,
    /// Optional font bounding box.
    pub font_bbox: Option<PathBounds>,
    /// Explicit widths from `/FirstChar`, `/LastChar`, and `/Widths`.
    pub widths: Vec<Type3GlyphWidth>,
    /// Decoded CharProc streams keyed by glyph name.
    pub char_procs: Vec<Type3CharProc>,
}

impl Type3Font {
    fn char_proc_for_code(&self, code: u32, encoding: &FontEncoding) -> Option<&Type3CharProc> {
        let code = u8::try_from(code).ok()?;
        let name = encoding.glyph_name_for_code(code);
        self.char_procs
            .iter()
            .find(|char_proc| char_proc.name == name)
    }

    fn width_for_code(&self, code: u32) -> Option<f64> {
        let code = u8::try_from(code).ok()?;
        self.widths
            .iter()
            .find_map(|width| (width.code == code).then_some(width.width))
    }
}

/// Type 3 glyph width entry in glyph space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Type3GlyphWidth {
    /// Source character code.
    pub code: u8,
    /// Width value from `/Widths`.
    pub width: f64,
}

/// Decoded Type 3 glyph CharProc content stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type3CharProc {
    /// Glyph name from `/CharProcs`.
    pub name: Vec<u8>,
    /// Decoded CharProc content bytes.
    pub content: Arc<[u8]>,
}

/// Text writing mode selected by font encoding or CMap metadata.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextWritingMode {
    /// Horizontal writing advances along the text X axis.
    #[default]
    Horizontal,
    /// Vertical writing advances along the negative text Y axis.
    Vertical,
}

/// Font descriptor used by text display-list construction.
#[derive(Debug, Clone, PartialEq)]
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
    /// Optional CID descendant font metrics for Type 0 composite fonts.
    pub cid_metrics: Option<CidFontMetrics>,
    /// Optional Type 3 font metadata and decoded CharProcs.
    pub type3: Option<Arc<Type3Font>>,
    /// Writing mode selected for text advance.
    pub writing_mode: TextWritingMode,
    /// Deterministic built-in fallback used by the current text rasterizer.
    pub fallback: Option<FontFallback>,
}

impl FontDescriptor {
    /// Creates a lightweight fallback font descriptor.
    #[must_use]
    pub fn new(resource_name: impl Into<Vec<u8>>, base_font: Option<impl Into<Vec<u8>>>) -> Self {
        let base_font = base_font.map(Into::into);
        let fallback = Some(FontFallback::resolve(base_font.as_deref(), None, false));
        Self {
            resource_name: resource_name.into(),
            base_font,
            subtype: None,
            descriptor_reference: None,
            program: None,
            encoding: FontEncoding::new(),
            to_unicode: None,
            cid_metrics: None,
            type3: None,
            writing_mode: TextWritingMode::Horizontal,
            fallback,
        }
    }

    fn advance_width_for_glyph(&self, glyph: &TextGlyph) -> f64 {
        if let Some(width) = self
            .type3
            .as_ref()
            .and_then(|type3| type3.width_for_code(glyph.character_code))
        {
            return width;
        }
        if let Some(metrics) = &self.cid_metrics {
            return f64::from(metrics.default_width.unwrap_or(1000));
        }
        if let Some(fallback) = self.fallback {
            if fallback.source == FontFallbackSource::StandardBase {
                if let Some(width) = standard_base_glyph_width(fallback.face, &glyph.unicode) {
                    return width;
                }
            }
        }
        500.0
    }
}

/// Lightweight font resource map.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FontResources {
    fonts: Vec<FontDescriptor>,
    fallback_cache_entries: usize,
}

impl FontResources {
    /// Creates an empty font resource map.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            fonts: Vec::new(),
            fallback_cache_entries: 0,
        }
    }

    /// Creates a font resource map from descriptors.
    #[must_use]
    pub fn new(fonts: Vec<FontDescriptor>) -> Self {
        Self {
            fonts,
            fallback_cache_entries: 0,
        }
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
        let mut fallback_cache = FontFallbackCache::new(options.max_font_fallback_cache_entries);
        let mut fonts = Vec::new();
        for (name, value) in dictionary {
            let mut font = decode_font_resource(*name, value, resolver, &mut cache, options)?;
            font.fallback = fallback_cache.resolve(
                font.base_font.as_deref(),
                font.subtype,
                font.program.is_some(),
                font.type3.is_some(),
            );
            fonts.push(font);
        }
        Ok(Self {
            fonts,
            fallback_cache_entries: fallback_cache.len(),
        })
    }

    /// Returns the font matching a PDF resource name.
    #[must_use]
    pub fn get(&self, name: PdfName<'_>) -> Option<&FontDescriptor> {
        self.fonts
            .iter()
            .find(|font| font.resource_name.as_slice() == name.as_bytes())
    }

    /// Returns cached deterministic fallback resolutions retained for this map.
    #[must_use]
    pub const fn fallback_cache_entries(&self) -> usize {
        self.fallback_cache_entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FontFallbackCache {
    entries: Vec<CachedFontFallback>,
    max_entries: usize,
}

impl FontFallbackCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    fn resolve(
        &mut self,
        base_font: Option<&[u8]>,
        subtype: Option<FontSubtype>,
        has_embedded_program: bool,
        is_type3: bool,
    ) -> Option<FontFallback> {
        if is_type3 {
            return None;
        }
        let fallback = FontFallback::resolve(base_font, subtype, has_embedded_program);
        let key = FontFallbackKey {
            face: fallback.face,
            source: fallback.source,
            subtype,
        };
        if let Some(entry) = self.entries.iter().find(|entry| entry.key == key) {
            return Some(entry.fallback);
        }
        if self.max_entries == 0 {
            return Some(fallback);
        }
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(CachedFontFallback { key, fallback });
        Some(fallback)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FontFallbackKey {
    face: FontFallbackFace,
    source: FontFallbackSource,
    subtype: Option<FontSubtype>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CachedFontFallback {
    key: FontFallbackKey,
    fallback: FontFallback,
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
    let writing_mode = font_writing_mode(dictionary);
    let to_unicode = load_to_unicode_map(dictionary, resolver, options)?;
    let cid_metrics = load_cid_font_metrics(dictionary, resolver)?;
    let type3 = if subtype == Some(FontSubtype::Type3) {
        Some(Arc::new(load_type3_font(dictionary, resolver, options)?))
    } else {
        None
    };
    Ok(FontDescriptor {
        resource_name: resource_name.as_bytes().to_vec(),
        base_font,
        subtype,
        descriptor_reference,
        program,
        encoding,
        to_unicode,
        cid_metrics,
        type3,
        writing_mode,
        fallback: None,
    })
}

fn load_type3_font<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    options: DisplayListOptions,
) -> GraphicsResult<Type3Font>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let font_matrix =
        optional_matrix(dictionary, b"FontMatrix")?.unwrap_or_else(|| Matrix::scale(0.001, 0.001));
    let font_bbox = optional_bbox(dictionary, b"FontBBox")?;
    let widths = load_type3_widths(dictionary)?;
    let char_procs = load_type3_char_procs(dictionary, resolver, options.max_font_program_bytes)?;
    Ok(Type3Font {
        font_matrix,
        font_bbox,
        widths,
        char_procs,
    })
}

fn optional_bbox(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<PathBounds>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    bounds_from_array(value).map(Some)
}

fn load_type3_widths(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<Vec<Type3GlyphWidth>> {
    let Some(PdfPrimitive::Array(width_values)) = dictionary_value(dictionary, b"Widths") else {
        return Ok(Vec::new());
    };
    let first_char = optional_u8(dictionary, b"FirstChar")?.unwrap_or(0);
    let last_char = optional_u8(dictionary, b"LastChar")?
        .unwrap_or_else(|| first_char.saturating_add(width_values.len().saturating_sub(1) as u8));
    let expected_len = usize::from(last_char.saturating_sub(first_char)) + 1;
    if expected_len != width_values.len() {
        return Err(invalid_font_resource(b"Widths"));
    }
    let mut widths = Vec::with_capacity(width_values.len());
    for (index, value) in width_values.iter().enumerate() {
        let code = first_char
            .checked_add(u8::try_from(index).map_err(|_| invalid_font_resource(b"Widths"))?)
            .ok_or_else(|| invalid_font_resource(b"Widths"))?;
        widths.push(Type3GlyphWidth {
            code,
            width: primitive_number(value).ok_or_else(|| invalid_font_resource(b"Widths"))?,
        });
    }
    Ok(widths)
}

fn optional_u8(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<u8>> {
    let Some(value) = optional_i32(dictionary, key)? else {
        return Ok(None);
    };
    u8::try_from(value)
        .map(Some)
        .map_err(|_| invalid_font_resource(key))
}

fn load_type3_char_procs<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    max_char_proc_bytes: usize,
) -> GraphicsResult<Vec<Type3CharProc>>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let Some(PdfPrimitive::Dictionary(char_procs)) = dictionary_value(dictionary, b"CharProcs")
    else {
        return Err(invalid_font_resource(b"CharProcs"));
    };
    let mut decoded = Vec::with_capacity(char_procs.len());
    for (name, value) in char_procs {
        let stream = match value {
            PdfPrimitive::Reference(_) => {
                let reference = reference_from_primitive(value)
                    .ok_or_else(|| invalid_font_resource(b"CharProcs"))?;
                let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
                    GraphicsError::new(
                        None,
                        GraphicsErrorKind::MissingFontObject {
                            name: name.as_bytes().to_vec(),
                        },
                    )
                })?;
                let ObjectValue::Stream(stream) = object.value else {
                    return Err(invalid_font_resource(b"CharProcs"));
                };
                stream
            }
            _ => return Err(invalid_font_resource(b"CharProcs")),
        };
        let content = stream
            .decode_with_options(StreamDecodeOptions {
                max_decoded_len: max_char_proc_bytes,
            })
            .map_err(|error| match error {
                pdfrust_object::ObjectError::StreamLimitExceeded { .. } => GraphicsError::new(
                    error.offset(),
                    GraphicsErrorKind::FontProgramBytesOverflow {
                        limit: max_char_proc_bytes,
                    },
                ),
                _ => GraphicsError::new(
                    error.offset(),
                    GraphicsErrorKind::ObjectModel {
                        message: error.to_string(),
                    },
                ),
            })?;
        if content.len() > max_char_proc_bytes {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::FontProgramBytesOverflow {
                    limit: max_char_proc_bytes,
                },
            ));
        }
        decoded.push(Type3CharProc {
            name: name.as_bytes().to_vec(),
            content: Arc::from(content),
        });
    }
    Ok(decoded)
}

fn load_cid_font_metrics<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
) -> GraphicsResult<Option<CidFontMetrics>>
where
    R: FontObjectResolver<'a> + ?Sized,
{
    let Some(PdfPrimitive::Array(descendants)) = dictionary_value(dictionary, b"DescendantFonts")
    else {
        return Ok(None);
    };
    let Some(descendant) = descendants.first() else {
        return Err(invalid_font_resource(b"DescendantFonts"));
    };
    match descendant {
        PdfPrimitive::Dictionary(descendant_dictionary) => {
            decode_cid_font_metrics(descendant_dictionary).map(Some)
        }
        PdfPrimitive::Reference(_) => {
            let reference = reference_from_primitive(descendant)
                .ok_or_else(|| invalid_font_resource(b"DescendantFonts"))?;
            let object = resolver.resolve_font_object(reference)?.ok_or_else(|| {
                GraphicsError::new(
                    None,
                    GraphicsErrorKind::MissingFontObject {
                        name: b"DescendantFonts".to_vec(),
                    },
                )
            })?;
            let ObjectValue::Primitive(PdfPrimitive::Dictionary(descendant_dictionary)) =
                object.value
            else {
                return Err(invalid_font_resource(b"DescendantFonts"));
            };
            decode_cid_font_metrics(&descendant_dictionary).map(Some)
        }
        _ => Err(invalid_font_resource(b"DescendantFonts")),
    }
}

fn decode_cid_font_metrics(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<CidFontMetrics> {
    let subtype =
        optional_font_subtype(dictionary).ok_or_else(|| invalid_font_resource(b"Subtype"))?;
    if !matches!(
        subtype,
        FontSubtype::CidFontType0 | FontSubtype::CidFontType2
    ) {
        return Err(invalid_font_resource(b"DescendantFonts"));
    }
    Ok(CidFontMetrics {
        subtype,
        default_width: optional_i32(dictionary, b"DW")?,
    })
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
        return Ok(identity_to_unicode_map(dictionary));
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

fn identity_to_unicode_map(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> Option<ToUnicodeMap> {
    match dictionary_value(dictionary, b"Encoding") {
        Some(PdfPrimitive::Name(name))
            if matches!(name.as_bytes(), b"Identity-H" | b"Identity-V") =>
        {
            Some(ToUnicodeMap::identity(2))
        }
        _ => None,
    }
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

fn extract_type1_glyph_outline(
    program: &FontProgram,
    glyph_code: u32,
    options: GlyphOutlineOptions,
) -> GraphicsResult<Option<GlyphOutline>> {
    let Some(charstring) = type1_charstring_for_glyph(&program.bytes, glyph_code)? else {
        return Ok(None);
    };
    interpret_type1_charstring(&charstring, glyph_code, options).map(Some)
}

fn type1_charstring_for_glyph(bytes: &[u8], glyph_code: u32) -> GraphicsResult<Option<Vec<u8>>> {
    let mut offset = 0;
    let mut seen = 0u32;
    while let Some(slash_offset) = bytes[offset..].iter().position(|byte| *byte == b'/') {
        offset += slash_offset + 1;
        let name_start = offset;
        while offset < bytes.len() && is_type1_name_byte(bytes[offset]) {
            offset += 1;
        }
        let name = &bytes[name_start..offset];
        while offset < bytes.len() && bytes[offset].is_ascii_whitespace() {
            offset += 1;
        }
        if offset >= bytes.len() || bytes[offset] != b'<' {
            continue;
        }
        let hex_start = offset + 1;
        let Some(hex_len) = bytes[hex_start..].iter().position(|byte| *byte == b'>') else {
            return Err(invalid_glyph_outline());
        };
        let hex = &bytes[hex_start..hex_start + hex_len];
        offset = hex_start + hex_len + 1;
        if name == b".notdef" {
            continue;
        }
        seen = seen.saturating_add(1);
        if seen == glyph_code {
            return decode_hex_charstring(hex).map(Some);
        }
    }
    Ok(None)
}

fn is_type1_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')
}

fn decode_hex_charstring(hex: &[u8]) -> GraphicsResult<Vec<u8>> {
    let mut decoded = Vec::with_capacity(hex.len() / 2);
    let mut high = None;
    for byte in hex
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        let nibble = charstring_hex_nibble(byte).ok_or_else(invalid_glyph_outline)?;
        if let Some(high_nibble) = high.take() {
            decoded.push((high_nibble << 4) | nibble);
        } else {
            high = Some(nibble);
        }
    }
    if high.is_some() {
        return Err(invalid_glyph_outline());
    }
    Ok(decoded)
}

fn charstring_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn interpret_type1_charstring(
    bytes: &[u8],
    glyph_code: u32,
    options: GlyphOutlineOptions,
) -> GraphicsResult<GlyphOutline> {
    let mut interpreter = Type1CharstringInterpreter::new(glyph_code, options);
    let mut offset = 0;
    while offset < bytes.len() {
        let byte = bytes[offset];
        offset += 1;
        match byte {
            0..=31 => {
                if byte == 12 {
                    let Some(escaped) = bytes.get(offset).copied() else {
                        return Err(invalid_glyph_outline());
                    };
                    offset += 1;
                    interpreter.apply_escaped_operator(escaped)?;
                } else {
                    interpreter.apply_operator(byte)?;
                }
            }
            32..=246 => interpreter.push_number(f64::from(i16::from(byte) - 139))?,
            247..=250 => {
                let Some(next) = bytes.get(offset).copied() else {
                    return Err(invalid_glyph_outline());
                };
                offset += 1;
                let value = (i16::from(byte) - 247) * 256 + i16::from(next) + 108;
                interpreter.push_number(f64::from(value))?;
            }
            251..=254 => {
                let Some(next) = bytes.get(offset).copied() else {
                    return Err(invalid_glyph_outline());
                };
                offset += 1;
                let value = -((i16::from(byte) - 251) * 256) - i16::from(next) - 108;
                interpreter.push_number(f64::from(value))?;
            }
            255 => {
                let Some(raw) = bytes.get(offset..offset + 4) else {
                    return Err(invalid_glyph_outline());
                };
                offset += 4;
                let value = i32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
                interpreter.push_number(f64::from(value) / 65_536.0)?;
            }
        }
        if interpreter.ended {
            return interpreter.finish();
        }
    }
    Err(invalid_glyph_outline())
}

struct Type1CharstringInterpreter {
    glyph_code: u32,
    options: GlyphOutlineOptions,
    stack: Vec<f64>,
    current: Point,
    started: bool,
    ended: bool,
    advance_width: f64,
    left_side_bearing: f64,
    segments: Vec<PathSegment>,
}

impl Type1CharstringInterpreter {
    fn new(glyph_code: u32, options: GlyphOutlineOptions) -> Self {
        Self {
            glyph_code,
            options,
            stack: Vec::new(),
            current: Point { x: 0.0, y: 0.0 },
            started: false,
            ended: false,
            advance_width: 500.0,
            left_side_bearing: 0.0,
            segments: Vec::new(),
        }
    }

    fn push_number(&mut self, value: f64) -> GraphicsResult<()> {
        if self.stack.len() >= self.options.max_charstring_stack {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::GlyphOutlineStackOverflow {
                    limit: self.options.max_charstring_stack,
                },
            ));
        }
        self.stack.push(value);
        Ok(())
    }

    fn apply_operator(&mut self, operator: u8) -> GraphicsResult<()> {
        match operator {
            4 => {
                let dy = self.pop_one()?;
                self.move_by(0.0, dy)
            }
            5 => self.lines_by_pairs(),
            6 => {
                let dx = self.pop_one()?;
                self.line_by(dx, 0.0)
            }
            7 => {
                let dy = self.pop_one()?;
                self.line_by(0.0, dy)
            }
            8 => self.curves_by_sixes(),
            9 => self.close_path(),
            10 => Err(GraphicsError::new(
                None,
                GraphicsErrorKind::GlyphOutlineSubroutineOverflow {
                    limit: self.options.max_charstring_subroutine_depth,
                },
            )),
            11 => Err(invalid_glyph_outline()),
            13 => {
                let (side_bearing, width) = self.pop_two()?;
                self.left_side_bearing = side_bearing;
                self.advance_width = width;
                self.current.x = side_bearing;
                self.stack.clear();
                Ok(())
            }
            14 => {
                self.ended = true;
                self.stack.clear();
                Ok(())
            }
            21 => {
                let (dx, dy) = self.pop_two()?;
                self.move_by(dx, dy)
            }
            22 => {
                let dx = self.pop_one()?;
                self.move_by(dx, 0.0)
            }
            _ => Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedGlyphOutline {
                    feature: format!("type1-charstring-operator-{operator}").into_bytes(),
                },
            )),
        }
    }

    fn apply_escaped_operator(&mut self, operator: u8) -> GraphicsResult<()> {
        match operator {
            7 => {
                let (side_bearing_x, side_bearing_y, width_x, _width_y) = self.pop_four()?;
                self.left_side_bearing = side_bearing_x;
                self.advance_width = width_x;
                self.current = Point {
                    x: side_bearing_x,
                    y: side_bearing_y,
                };
                self.stack.clear();
                Ok(())
            }
            10 => Err(GraphicsError::new(
                None,
                GraphicsErrorKind::GlyphOutlineSubroutineOverflow {
                    limit: self.options.max_charstring_subroutine_depth,
                },
            )),
            12 => {
                let (left, right) = self.pop_two()?;
                if right == 0.0 {
                    return Err(invalid_glyph_outline());
                }
                self.push_number(left / right)
            }
            _ => Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedGlyphOutline {
                    feature: format!("type1-charstring-escaped-operator-{operator}").into_bytes(),
                },
            )),
        }
    }

    fn move_by(&mut self, dx: f64, dy: f64) -> GraphicsResult<()> {
        self.current = Point {
            x: self.current.x + dx,
            y: self.current.y + dy,
        };
        self.push_segment(PathSegment::MoveTo(self.current))?;
        self.started = true;
        self.stack.clear();
        Ok(())
    }

    fn line_by(&mut self, dx: f64, dy: f64) -> GraphicsResult<()> {
        self.ensure_started()?;
        self.current = Point {
            x: self.current.x + dx,
            y: self.current.y + dy,
        };
        self.push_segment(PathSegment::LineTo(self.current))?;
        self.stack.clear();
        Ok(())
    }

    fn lines_by_pairs(&mut self) -> GraphicsResult<()> {
        if self.stack.is_empty() || self.stack.len() % 2 != 0 {
            return Err(invalid_glyph_outline());
        }
        let values = std::mem::take(&mut self.stack);
        for pair in values.chunks_exact(2) {
            self.line_by(pair[0], pair[1])?;
        }
        Ok(())
    }

    fn curves_by_sixes(&mut self) -> GraphicsResult<()> {
        if self.stack.is_empty() || self.stack.len() % 6 != 0 {
            return Err(invalid_glyph_outline());
        }
        self.ensure_started()?;
        let values = std::mem::take(&mut self.stack);
        for curve in values.chunks_exact(6) {
            let c1 = Point {
                x: self.current.x + curve[0],
                y: self.current.y + curve[1],
            };
            let c2 = Point {
                x: c1.x + curve[2],
                y: c1.y + curve[3],
            };
            self.current = Point {
                x: c2.x + curve[4],
                y: c2.y + curve[5],
            };
            self.push_segment(PathSegment::CubicTo {
                c1,
                c2,
                to: self.current,
            })?;
        }
        Ok(())
    }

    fn close_path(&mut self) -> GraphicsResult<()> {
        self.ensure_started()?;
        self.push_segment(PathSegment::Close)?;
        self.stack.clear();
        Ok(())
    }

    fn finish(self) -> GraphicsResult<GlyphOutline> {
        Ok(GlyphOutline {
            glyph_code: self.glyph_code,
            advance_width: self.advance_width,
            left_side_bearing: self.left_side_bearing,
            segments: self.segments,
        })
    }

    fn ensure_started(&mut self) -> GraphicsResult<()> {
        if !self.started {
            self.push_segment(PathSegment::MoveTo(self.current))?;
            self.started = true;
        }
        Ok(())
    }

    fn push_segment(&mut self, segment: PathSegment) -> GraphicsResult<()> {
        if self.segments.len() >= self.options.max_segments {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::GlyphOutlineSegmentOverflow {
                    limit: self.options.max_segments,
                },
            ));
        }
        self.segments.push(segment);
        Ok(())
    }

    fn pop_one(&mut self) -> GraphicsResult<f64> {
        let [value] = self.pop_array()?;
        Ok(value)
    }

    fn pop_two(&mut self) -> GraphicsResult<(f64, f64)> {
        let [a, b] = self.pop_array()?;
        Ok((a, b))
    }

    fn pop_four(&mut self) -> GraphicsResult<(f64, f64, f64, f64)> {
        let [a, b, c, d] = self.pop_array()?;
        Ok((a, b, c, d))
    }

    fn pop_array<const N: usize>(&mut self) -> GraphicsResult<[f64; N]> {
        if self.stack.len() != N {
            return Err(invalid_glyph_outline());
        }
        let values: [f64; N] = self
            .stack
            .drain(..)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| invalid_glyph_outline())?;
        Ok(values)
    }
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
        Self::new_with_options(geometry, max_edge, PageTransformOptions::default())
    }

    /// Builds a page-to-raster transform with explicit page-raster budgets.
    ///
    /// # Errors
    ///
    /// Returns [`RasterError`] for invalid page boxes, zero `max_edge`,
    /// overflowing output dimensions, or page-raster budget exhaustion.
    pub fn new_with_options(
        geometry: PageGeometry,
        max_edge: u32,
        options: PageTransformOptions,
    ) -> RasterResult<Self> {
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
        ensure_page_raster_pixel_budget(dimensions, options.max_page_pixels)?;
        let matrix = page_to_pixel_matrix(source_box, geometry.rotation, scale, dimensions);
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

/// Page transform and raster allocation budget configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTransformOptions {
    /// Maximum pixels accepted in one page raster buffer.
    pub max_page_pixels: usize,
}

impl Default for PageTransformOptions {
    fn default() -> Self {
        Self {
            max_page_pixels: DEFAULT_PAGE_RASTER_PIXELS_LIMIT,
        }
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
    /// Maximum resident decoded image bytes accepted for one page resource map.
    pub max_total_image_bytes: usize,
    /// Maximum decoded ICC profile bytes accepted for one image color space.
    pub max_icc_profile_bytes: usize,
    /// Maximum scratch bytes accepted for one ICC transform.
    pub max_icc_transform_workspace_bytes: usize,
    /// Maximum cached ICC transform entries.
    pub max_icc_transform_cache_entries: usize,
    /// Maximum nested soft-mask image depth.
    pub max_soft_mask_depth: usize,
    /// Maximum allowed Form XObject recursion depth.
    pub max_form_recursion_depth: usize,
    /// Maximum cached deterministic font fallback resolutions.
    pub max_font_fallback_cache_entries: usize,
    /// Maximum decoded bytes accepted for one mesh shading stream.
    pub max_mesh_shading_bytes: usize,
    /// Maximum triangles accepted in one decoded mesh shading.
    pub max_mesh_shading_triangles: usize,
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
            max_total_image_bytes: DEFAULT_TOTAL_IMAGE_BYTES_LIMIT,
            max_icc_profile_bytes: DEFAULT_ICC_PROFILE_BYTES_LIMIT,
            max_icc_transform_workspace_bytes: DEFAULT_ICC_TRANSFORM_WORKSPACE_LIMIT,
            max_icc_transform_cache_entries: DEFAULT_ICC_TRANSFORM_CACHE_LIMIT,
            max_soft_mask_depth: DEFAULT_SOFT_MASK_DEPTH_LIMIT,
            max_form_recursion_depth: DEFAULT_FORM_RECURSION_DEPTH_LIMIT,
            max_font_fallback_cache_entries: DEFAULT_FONT_FALLBACK_CACHE_LIMIT,
            max_mesh_shading_bytes: DEFAULT_MESH_SHADING_BYTES_LIMIT,
            max_mesh_shading_triangles: DEFAULT_MESH_SHADING_TRIANGLE_LIMIT,
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

/// Builds a path display list with external graphics state resources.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, path or display-list
/// limits are exceeded, a named external graphics state is missing, or
/// supported operators receive malformed operands.
pub fn build_path_display_list_with_ext_graphics_states<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ext_graphics_states: &ExtGraphicsStateResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let shadings = ShadingResources::empty();
    let patterns = TilingPatternResources::empty();
    let color_spaces = ColorSpaceResources::empty();
    build_path_display_list_with_graphics_resources(
        tokens,
        ext_graphics_states,
        &shadings,
        &patterns,
        &color_spaces,
        options,
    )
}

/// Builds a path display list with external graphics and shading resources.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, path or display-list
/// limits are exceeded, a named resource is missing, or supported operators
/// receive malformed operands.
pub fn build_path_display_list_with_graphics_resources<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    ext_graphics_states: &ExtGraphicsStateResources,
    shadings: &ShadingResources,
    patterns: &TilingPatternResources,
    color_spaces: &ColorSpaceResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let mut interpreter = DisplayListInterpreter::new_with_graphics_resources(
        options,
        ext_graphics_states,
        shadings,
        patterns,
        color_spaces,
    );
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
    let ext_graphics_states = ExtGraphicsStateResources::empty();
    build_form_display_list_with_ext_graphics_states(tokens, forms, &ext_graphics_states, options)
}

/// Builds display-list items from Form XObject invocations with graphics resources.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, a named form or external
/// graphics state is missing, form recursion exceeds the configured limit, or
/// display limits are exceeded.
pub fn build_form_display_list_with_ext_graphics_states<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    forms: &FormResources,
    ext_graphics_states: &ExtGraphicsStateResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let shadings = ShadingResources::empty();
    let patterns = TilingPatternResources::empty();
    let color_spaces = ColorSpaceResources::empty();
    build_form_display_list_with_graphics_resources(
        tokens,
        forms,
        ext_graphics_states,
        &shadings,
        &patterns,
        &color_spaces,
        options,
    )
}

/// Builds display-list items from Form XObject invocations with graphics resources.
///
/// # Errors
///
/// Returns [`GraphicsError`] when tokenization fails, a named resource is
/// missing, form recursion exceeds the configured limit, or display limits are
/// exceeded.
pub fn build_form_display_list_with_graphics_resources<'a>(
    tokens: impl IntoIterator<Item = ContentResult<ContentToken<'a>>>,
    forms: &FormResources,
    ext_graphics_states: &ExtGraphicsStateResources,
    shadings: &ShadingResources,
    patterns: &TilingPatternResources,
    color_spaces: &ColorSpaceResources,
    options: DisplayListOptions,
) -> GraphicsResult<DisplayList> {
    let resources = GraphicsResourceContext {
        ext_graphics_states: Some(ext_graphics_states),
        shadings: Some(shadings),
        patterns: Some(patterns),
        color_spaces: Some(color_spaces),
    };
    let mut interpreter = DisplayListInterpreter::new_with_forms(
        GraphicsState::default(),
        options,
        forms,
        resources,
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
    let mut active_clips = Vec::new();
    let mut pattern_cache = PatternCellCache::new(options.max_pattern_cell_cache_entries);
    for item in display_list.items() {
        match item {
            DisplayItem::Path(path) => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    path.state.graphics_state_depth,
                    path.state.graphics_state_scope_id,
                );
                rasterize_path_item(
                    path,
                    device,
                    PathRasterContext {
                        transform,
                        options,
                        clips: &active_clips,
                    },
                    &mut pattern_cache,
                )?;
            }
            DisplayItem::ClipPlaceholder {
                segments,
                rule,
                state,
            } => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    state.graphics_state_depth,
                    state.graphics_state_scope_id,
                );
                active_clips.push(ActiveClip {
                    path: flatten_path_segments(
                        segments,
                        transform.matrix,
                        options.max_flattened_segments,
                    )?,
                    rule: *rule,
                    graphics_state_depth: state.graphics_state_depth,
                    graphics_state_scope_id: state.graphics_state_scope_id,
                });
            }
            DisplayItem::Shading(shading) => rasterize_shading_item(shading, device, transform)?,
            DisplayItem::TransparencyGroup(group) => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    group.state.graphics_state_depth,
                    group.state.graphics_state_scope_id,
                );
                rasterize_transparency_group(group, device, transform, options, &active_clips)?;
            }
            DisplayItem::Text(_) | DisplayItem::Image(_) => {}
        }
    }
    Ok(())
}

/// Rasterizes all supported display-list items into an existing RGBA raster device.
///
/// This preserves the item order in the display list, so mixed path, image, and
/// text content is composited in content-stream paint order.
///
/// # Errors
///
/// Returns [`RasterError`] when path, image, or text rasterization fails.
pub fn rasterize_display_list_into(
    display_list: &DisplayList,
    device: &mut RasterDevice,
    transform: PageTransform,
    options: PathRasterOptions,
) -> RasterResult<()> {
    if options.supersample == 0 {
        return Err(RasterError::new(RasterErrorKind::InvalidSupersampling));
    }
    let mut active_clips = Vec::new();
    let mut glyph_cache = GlyphBitmapCache::default();
    let mut pattern_cache = PatternCellCache::new(options.max_pattern_cell_cache_entries);
    let mut text_scratch = TextRasterScratch::default();
    for item in display_list.items() {
        match item {
            DisplayItem::Path(path) => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    path.state.graphics_state_depth,
                    path.state.graphics_state_scope_id,
                );
                rasterize_path_item(
                    path,
                    device,
                    PathRasterContext {
                        transform,
                        options,
                        clips: &active_clips,
                    },
                    &mut pattern_cache,
                )?;
            }
            DisplayItem::ClipPlaceholder {
                segments,
                rule,
                state,
            } => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    state.graphics_state_depth,
                    state.graphics_state_scope_id,
                );
                active_clips.push(ActiveClip {
                    path: flatten_path_segments(
                        segments,
                        transform.matrix,
                        options.max_flattened_segments,
                    )?,
                    rule: *rule,
                    graphics_state_depth: state.graphics_state_depth,
                    graphics_state_scope_id: state.graphics_state_scope_id,
                });
            }
            DisplayItem::Shading(shading) => rasterize_shading_item(shading, device, transform)?,
            DisplayItem::TransparencyGroup(group) => {
                truncate_clips_to_scope(
                    &mut active_clips,
                    group.state.graphics_state_depth,
                    group.state.graphics_state_scope_id,
                );
                rasterize_transparency_group(group, device, transform, options, &active_clips)?;
            }
            DisplayItem::Image(image) => draw_image(device, image, transform)?,
            DisplayItem::Text(text) => {
                draw_text_run(
                    device,
                    text,
                    transform,
                    options,
                    &mut glyph_cache,
                    &mut text_scratch,
                )?;
            }
        }
    }
    Ok(())
}

fn rasterize_shading_item(
    item: &ShadingDisplayItem,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    match &item.shading {
        Shading::Axial(shading) => rasterize_axial_shading(*shading, item.state, device, transform),
        Shading::Radial(shading) => {
            rasterize_radial_shading(*shading, item.state, device, transform)
        }
        Shading::Mesh(shading) => rasterize_mesh_shading(shading, item.state, device, transform),
    }
}

fn rasterize_axial_shading(
    shading: AxialShading,
    state: GraphicsState,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    let start = state.ctm.transform_point(shading.start.x, shading.start.y);
    let start = transform.matrix.transform_point(start.x, start.y);
    let end = state.ctm.transform_point(shading.end.x, shading.end.y);
    let end = transform.matrix.transform_point(end.x, end.y);
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx.mul_add(dx, dy * dy);
    if length_squared <= f64::EPSILON {
        return Ok(());
    }
    let dimensions = device.dimensions();
    for y in 0..dimensions.height {
        for x in 0..dimensions.width {
            let sample_x = f64::from(x) + 0.5;
            let sample_y = f64::from(y) + 0.5;
            let mut t = ((sample_x - start.x) * dx + (sample_y - start.y) * dy) / length_squared;
            if t < 0.0 {
                if !shading.extend_start {
                    continue;
                }
                t = 0.0;
            } else if t > 1.0 {
                if !shading.extend_end {
                    continue;
                }
                t = 1.0;
            }
            let source = sample_axial_color(shading, t);
            blend_pixel(device, x, y, source, state.blend_mode, 1.0)?;
        }
    }
    Ok(())
}

fn rasterize_radial_shading(
    shading: RadialShading,
    state: GraphicsState,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    let start_center = state
        .ctm
        .transform_point(shading.start_center.x, shading.start_center.y);
    let start_center = transform
        .matrix
        .transform_point(start_center.x, start_center.y);
    let end_center = state
        .ctm
        .transform_point(shading.end_center.x, shading.end_center.y);
    let end_center = transform.matrix.transform_point(end_center.x, end_center.y);
    let start_radius = shading.start_radius * transform.scale;
    let end_radius = shading.end_radius * transform.scale;
    let radius_delta = end_radius - start_radius;
    if radius_delta.abs() <= f64::EPSILON {
        return Ok(());
    }
    let dimensions = device.dimensions();
    for y in 0..dimensions.height {
        for x in 0..dimensions.width {
            let sample_x = f64::from(x) + 0.5;
            let sample_y = f64::from(y) + 0.5;
            let dx = sample_x - end_center.x;
            let dy = sample_y - end_center.y;
            let distance = dx.hypot(dy);
            let center_distance =
                (start_center.x - end_center.x).hypot(start_center.y - end_center.y);
            let mut t = (distance + center_distance - start_radius) / radius_delta;
            if t < 0.0 {
                if !shading.extend_start {
                    continue;
                }
                t = 0.0;
            } else if t > 1.0 {
                if !shading.extend_end {
                    continue;
                }
                t = 1.0;
            }
            let source = sample_radial_color(shading, t);
            blend_pixel(device, x, y, source, state.blend_mode, 1.0)?;
        }
    }
    Ok(())
}

fn sample_axial_color(shading: AxialShading, t: f64) -> Rgba {
    let ratio = t.clamp(0.0, 1.0).powf(shading.exponent);
    let start = device_color_to_rgba(shading.start_color);
    let end = device_color_to_rgba(shading.end_color);
    Rgba {
        r: interpolate_channel(start.r, end.r, ratio),
        g: interpolate_channel(start.g, end.g, ratio),
        b: interpolate_channel(start.b, end.b, ratio),
        a: 255,
    }
}

fn sample_radial_color(shading: RadialShading, t: f64) -> Rgba {
    let ratio = t.clamp(0.0, 1.0).powf(shading.exponent);
    let start = device_color_to_rgba(shading.start_color);
    let end = device_color_to_rgba(shading.end_color);
    Rgba {
        r: interpolate_channel(start.r, end.r, ratio),
        g: interpolate_channel(start.g, end.g, ratio),
        b: interpolate_channel(start.b, end.b, ratio),
        a: 255,
    }
}

fn rasterize_mesh_shading(
    shading: &MeshShading,
    state: GraphicsState,
    device: &mut RasterDevice,
    transform: PageTransform,
) -> RasterResult<()> {
    for triangle in &shading.triangles {
        let vertices = triangle.vertices.map(|vertex| {
            let user_point = state.ctm.transform_point(vertex.point.x, vertex.point.y);
            MeshVertex {
                point: transform.matrix.transform_point(user_point.x, user_point.y),
                color: vertex.color,
            }
        });
        rasterize_mesh_triangle(vertices, state, device)?;
    }
    Ok(())
}

fn rasterize_mesh_triangle(
    vertices: [MeshVertex; 3],
    state: GraphicsState,
    device: &mut RasterDevice,
) -> RasterResult<()> {
    let bounds = PathBounds {
        min_x: vertices
            .iter()
            .map(|vertex| vertex.point.x)
            .fold(f64::INFINITY, f64::min),
        min_y: vertices
            .iter()
            .map(|vertex| vertex.point.y)
            .fold(f64::INFINITY, f64::min),
        max_x: vertices
            .iter()
            .map(|vertex| vertex.point.x)
            .fold(f64::NEG_INFINITY, f64::max),
        max_y: vertices
            .iter()
            .map(|vertex| vertex.point.y)
            .fold(f64::NEG_INFINITY, f64::max),
    };
    let Some(bounds) = device_pixel_bounds(bounds, device.dimensions(), 0.0) else {
        return Ok(());
    };
    let a = vertices[0].point;
    let b = vertices[1].point;
    let c = vertices[2].point;
    let denominator = (b.y - c.y).mul_add(a.x - c.x, (c.x - b.x) * (a.y - c.y));
    if denominator.abs() <= f64::EPSILON {
        return Ok(());
    }
    let colors = vertices.map(|vertex| device_color_to_rgba(vertex.color));
    for y in bounds.min_y..bounds.max_y {
        for x in bounds.min_x..bounds.max_x {
            let point = Point {
                x: f64::from(x) + 0.5,
                y: f64::from(y) + 0.5,
            };
            let w0 = ((b.y - c.y) * (point.x - c.x) + (c.x - b.x) * (point.y - c.y)) / denominator;
            let w1 = ((c.y - a.y) * (point.x - c.x) + (a.x - c.x) * (point.y - c.y)) / denominator;
            let w2 = 1.0 - w0 - w1;
            if w0 <= f64::EPSILON || w1 < -f64::EPSILON || w2 < -f64::EPSILON {
                continue;
            }
            blend_pixel(
                device,
                x,
                y,
                interpolate_triangle_color(colors, [w0, w1, w2]),
                state.blend_mode,
                state.fill_alpha,
            )?;
        }
    }
    Ok(())
}

fn interpolate_triangle_color(colors: [Rgba; 3], weights: [f64; 3]) -> Rgba {
    Rgba {
        r: interpolate_triangle_channel([colors[0].r, colors[1].r, colors[2].r], weights),
        g: interpolate_triangle_channel([colors[0].g, colors[1].g, colors[2].g], weights),
        b: interpolate_triangle_channel([colors[0].b, colors[1].b, colors[2].b], weights),
        a: 255,
    }
}

fn interpolate_triangle_channel(channels: [u8; 3], weights: [f64; 3]) -> u8 {
    f64::from(channels[0])
        .mul_add(
            weights[0],
            f64::from(channels[1]).mul_add(weights[1], f64::from(channels[2]) * weights[2]),
        )
        .round()
        .clamp(0.0, 255.0) as u8
}

fn interpolate_channel(start: u8, end: u8, ratio: f64) -> u8 {
    f64::from(start)
        .mul_add(1.0 - ratio, f64::from(end) * ratio)
        .round() as u8
}

fn rasterize_path_item(
    path: &PathDisplayItem,
    device: &mut RasterDevice,
    context: PathRasterContext<'_>,
    pattern_cache: &mut PatternCellCache,
) -> RasterResult<()> {
    let flattened = flatten_path_segments(
        &path.segments,
        context.transform.matrix,
        context.options.max_flattened_segments,
    )?;
    match path.paint {
        PaintMode::Fill { rule } => {
            if let Some(pattern) = &path.fill_pattern {
                fill_path_with_tiling_pattern(
                    device,
                    &flattened,
                    rule,
                    pattern,
                    path.state,
                    context,
                    pattern_cache,
                )?;
            } else {
                fill_path(
                    device,
                    &flattened,
                    rule,
                    path.state.fill_color,
                    path.state.blend_mode,
                    path.state.fill_alpha,
                    context,
                )?;
            }
        }
        PaintMode::Stroke => {
            stroke_path(
                device,
                &flattened,
                StrokeRasterState {
                    line_width: device_stroke_width(path.state, context.transform),
                    ctm_scale: matrix_average_scale(path.state.ctm),
                    color: path.state.stroke_color,
                    blend_mode: path.state.blend_mode,
                    alpha: path.state.stroke_alpha,
                    dash_pattern: path.state.stroke_dash,
                    dash_scale: device_stroke_scale(path.state, context.transform),
                    line_cap: path.state.line_cap,
                    line_join: path.state.line_join,
                    miter_limit: path.state.miter_limit,
                },
                context,
            )?;
        }
        PaintMode::FillStroke { rule } => {
            if let Some(pattern) = &path.fill_pattern {
                fill_path_with_tiling_pattern(
                    device,
                    &flattened,
                    rule,
                    pattern,
                    path.state,
                    context,
                    pattern_cache,
                )?;
            } else {
                fill_path(
                    device,
                    &flattened,
                    rule,
                    path.state.fill_color,
                    path.state.blend_mode,
                    path.state.fill_alpha,
                    context,
                )?;
            }
            stroke_path(
                device,
                &flattened,
                StrokeRasterState {
                    line_width: device_stroke_width(path.state, context.transform),
                    ctm_scale: matrix_average_scale(path.state.ctm),
                    color: path.state.stroke_color,
                    blend_mode: path.state.blend_mode,
                    alpha: path.state.stroke_alpha,
                    dash_pattern: path.state.stroke_dash,
                    dash_scale: device_stroke_scale(path.state, context.transform),
                    line_cap: path.state.line_cap,
                    line_join: path.state.line_join,
                    miter_limit: path.state.miter_limit,
                },
                context,
            )?;
        }
    }
    Ok(())
}

fn device_stroke_width(state: GraphicsState, transform: PageTransform) -> f64 {
    state.line_width * device_stroke_scale(state, transform)
}

fn device_stroke_scale(state: GraphicsState, transform: PageTransform) -> f64 {
    matrix_average_scale(state.ctm) * transform.scale
}

fn rasterize_transparency_group(
    group: &TransparencyGroupDisplayItem,
    device: &mut RasterDevice,
    transform: PageTransform,
    options: PathRasterOptions,
    clips: &[ActiveClip],
) -> RasterResult<()> {
    let Some(bounds) = transparency_group_device_bounds(group.bounds, transform) else {
        return Ok(());
    };
    let pixels = (bounds.width as usize)
        .checked_mul(bounds.height as usize)
        .ok_or_else(|| RasterError::new(RasterErrorKind::BufferOverflow))?;
    if pixels > options.max_transparency_group_pixels {
        return Err(RasterError::new(
            RasterErrorKind::TransparencyGroupPixelsOverflow {
                limit: options.max_transparency_group_pixels,
            },
        ));
    }
    let mut group_matrix = transform.matrix;
    group_matrix.e -= f64::from(bounds.min_x);
    group_matrix.f -= f64::from(bounds.min_y);
    let group_dimensions = RasterDimensions::new(bounds.width, bounds.height)?;
    let group_transform = PageTransform {
        source_box: group.bounds,
        rotation: transform.rotation,
        scale: transform.scale,
        dimensions: group_dimensions,
        matrix: group_matrix,
    };
    let mut group_device = RasterDevice::new(
        bounds.width,
        bounds.height,
        Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    )?;
    rasterize_display_list_into(&group.items, &mut group_device, group_transform, options)?;
    for y in 0..bounds.height {
        for x in 0..bounds.width {
            let source = group_device.pixel(x, y)?;
            if source.a == 0 {
                continue;
            }
            let device_x = bounds.min_x + x;
            let device_y = bounds.min_y + y;
            let clip_coverage = clip_coverage_for_pixel(device_x, device_y, clips, options);
            if clip_coverage <= f64::EPSILON {
                continue;
            }
            blend_pixel(
                device,
                device_x,
                device_y,
                source,
                group.state.blend_mode,
                group.state.fill_alpha * clip_coverage,
            )?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceBounds {
    min_x: u32,
    min_y: u32,
    width: u32,
    height: u32,
}

fn transparency_group_device_bounds(
    bounds: PathBounds,
    transform: PageTransform,
) -> Option<DeviceBounds> {
    let bounds = transform_bounds(bounds, transform.matrix);
    let dimensions = transform.dimensions;
    let min_x = bounds.min_x.floor().max(0.0) as u32;
    let min_y = bounds.min_y.floor().max(0.0) as u32;
    let max_x = bounds.max_x.ceil().min(f64::from(dimensions.width)) as u32;
    let max_y = bounds.max_y.ceil().min(f64::from(dimensions.height)) as u32;
    if min_x >= max_x || min_y >= max_y {
        return None;
    }
    Some(DeviceBounds {
        min_x,
        min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
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
    let mut glyph_cache = GlyphBitmapCache::default();
    let mut text_scratch = TextRasterScratch::default();
    for item in display_list.items() {
        let DisplayItem::Text(text) = item else {
            continue;
        };
        draw_text_run(
            device,
            text,
            transform,
            PathRasterOptions::default(),
            &mut glyph_cache,
            &mut text_scratch,
        )?;
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
    resources: GraphicsResourceContext<'r>,
    forms: Option<FormInterpreterContext<'r>>,
    next_graphics_state_scope_id: u64,
}

#[derive(Debug, Default, Clone, Copy)]
struct GraphicsResourceContext<'r> {
    ext_graphics_states: Option<&'r ExtGraphicsStateResources>,
    shadings: Option<&'r ShadingResources>,
    patterns: Option<&'r TilingPatternResources>,
    color_spaces: Option<&'r ColorSpaceResources>,
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
            resources: GraphicsResourceContext::default(),
            forms: None,
            next_graphics_state_scope_id: 1,
        }
    }

    fn new_with_graphics_resources(
        options: DisplayListOptions,
        ext_graphics_states: &'r ExtGraphicsStateResources,
        shadings: &'r ShadingResources,
        patterns: &'r TilingPatternResources,
        color_spaces: &'r ColorSpaceResources,
    ) -> Self {
        let resources = GraphicsResourceContext {
            ext_graphics_states: Some(ext_graphics_states),
            shadings: Some(shadings),
            patterns: Some(patterns),
            color_spaces: Some(color_spaces),
        };
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            current_path: CurrentPath::default(),
            display_list: DisplayList::new(),
            options,
            resources,
            forms: None,
            next_graphics_state_scope_id: 1,
        }
    }

    fn new_with_forms(
        current: GraphicsState,
        options: DisplayListOptions,
        forms: &'r FormResources,
        resources: GraphicsResourceContext<'r>,
        scope: FormResourceScope<'r>,
        recursion_depth: usize,
    ) -> Self {
        Self {
            current,
            stack: Vec::new(),
            current_path: CurrentPath::default(),
            display_list: DisplayList::new(),
            options,
            resources,
            forms: Some(FormInterpreterContext {
                resources: forms,
                scope,
                recursion_depth,
            }),
            next_graphics_state_scope_id: 1,
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
            b"J" => self.set_line_cap(offset, operands),
            b"j" => self.set_line_join(offset, operands),
            b"M" => self.set_miter_limit(offset, operands),
            b"d" => self.set_stroke_dash(offset, operands),
            b"g" => self.set_fill_gray(offset, operands),
            b"G" => self.set_stroke_gray(offset, operands),
            b"rg" => self.set_fill_rgb(offset, operands),
            b"RG" => self.set_stroke_rgb(offset, operands),
            b"cs" => self.set_fill_color_space(offset, operands),
            b"CS" => self.set_stroke_color_space(offset, operands),
            b"sc" => self.set_fill_color(offset, b"sc", operands),
            b"scn" => self.set_fill_color(offset, b"scn", operands),
            b"SC" => self.set_stroke_color(offset, b"SC", operands),
            b"SCN" => self.set_stroke_color(offset, b"SCN", operands),
            b"gs" => self.set_ext_graphics_state(offset, operands),
            b"sh" => self.paint_shading(offset, operands),
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
            self.resources,
            FormResourceScope::for_form(form),
            context.recursion_depth + 1,
        );
        if form.transparency_group.is_none() {
            nested.push_form_bbox_clip(form, offset)?;
        }
        nested.interpret(tokenize_content(PdfBytes::new(&form.content)))?;
        if let Some(group) = form.transparency_group {
            let bounds = transform_bounds(form.bbox, nested_state.ctm);
            return self.display_list.push(
                DisplayItem::TransparencyGroup(TransparencyGroupDisplayItem {
                    items: nested.display_list,
                    bounds,
                    group,
                    state: self.current,
                }),
                self.options.max_display_items,
                offset,
            );
        }
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
        self.current.graphics_state_depth += 1;
        self.current.graphics_state_scope_id = self.next_graphics_state_scope_id;
        self.next_graphics_state_scope_id += 1;
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

    fn set_stroke_dash(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.stroke_dash = dash_pattern_operand(offset, operands)?;
        Ok(())
    }

    fn set_line_cap(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.line_cap = line_cap_operand(offset, operands)?;
        Ok(())
    }

    fn set_line_join(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.line_join = line_join_operand(offset, operands)?;
        Ok(())
    }

    fn set_miter_limit(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.miter_limit = miter_limit_operand(offset, operands)?;
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
        self.current.fill_color_space = FillColorSpace::Device;
        self.current.fill_pattern = None;
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
        self.current.stroke_color_space = StrokeColorSpace::Device;
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
        self.current.fill_color_space = FillColorSpace::Device;
        self.current.fill_pattern = None;
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
        self.current.stroke_color_space = StrokeColorSpace::Device;
        Ok(())
    }

    fn set_fill_color_space(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"cs", operands, 1)?;
        let name = name_operand(offset, b"cs", operands, 0)?;
        if name.as_bytes() == b"Pattern" {
            self.current.fill_color_space = FillColorSpace::Pattern;
            return Ok(());
        }
        if let Some(base) = self
            .resources
            .color_spaces
            .and_then(|color_spaces| color_spaces.pattern_base_for_name(name))
        {
            self.current.fill_color_space = FillColorSpace::UncoloredPattern(base);
            self.current.fill_pattern = None;
            return Ok(());
        }
        if let Some(index) = self
            .resources
            .color_spaces
            .and_then(|color_spaces| color_spaces.index_of(name))
        {
            self.current.fill_color_space = FillColorSpace::Spot(index);
            self.current.fill_pattern = None;
            return Ok(());
        }
        self.current.fill_color_space = FillColorSpace::Device;
        self.current.fill_pattern = None;
        Ok(())
    }

    fn set_stroke_color_space(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"CS", operands, 1)?;
        let name = name_operand(offset, b"CS", operands, 0)?;
        if let Some(index) = self
            .resources
            .color_spaces
            .and_then(|color_spaces| color_spaces.index_of(name))
        {
            self.current.stroke_color_space = StrokeColorSpace::Spot(index);
            return Ok(());
        }
        self.current.stroke_color_space = StrokeColorSpace::Device;
        Ok(())
    }

    fn set_fill_color(
        &mut self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        match self.current.fill_color_space {
            FillColorSpace::Device => Ok(()),
            FillColorSpace::Pattern => self.set_fill_pattern(offset, operator, operands),
            FillColorSpace::UncoloredPattern(base) => {
                self.set_uncolored_fill_pattern(offset, operator, operands, base)
            }
            FillColorSpace::Spot(index) => {
                let color = self.spot_color(offset, operator, operands, index)?;
                self.current.fill_color = color;
                self.current.fill_pattern = None;
                Ok(())
            }
        }
    }

    fn set_stroke_color(
        &mut self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        let StrokeColorSpace::Spot(index) = self.current.stroke_color_space else {
            return Ok(());
        };
        self.current.stroke_color = self.spot_color(offset, operator, operands, index)?;
        Ok(())
    }

    fn set_fill_pattern(
        &mut self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, operator, operands, 1)?;
        let name = name_operand(offset, operator, operands, 0)?;
        let pattern = self
            .resources
            .patterns
            .and_then(|patterns| patterns.index_of(name))
            .ok_or_else(|| {
                GraphicsError::new(
                    Some(offset),
                    GraphicsErrorKind::MissingPattern {
                        name: name.as_bytes().to_vec(),
                    },
                )
            })?;
        self.current.fill_pattern = Some(pattern);
        Ok(())
    }

    fn set_uncolored_fill_pattern(
        &mut self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
        base: PatternBaseColorSpace,
    ) -> GraphicsResult<()> {
        let expected = base.component_count() + 1;
        expect_operand_count(offset, operator, operands, expected)?;
        let name = match operands.last() {
            Some(PdfPrimitive::Name(name)) => *name,
            _ => return Err(invalid_operand(offset, operator)),
        };
        let pattern = self
            .resources
            .patterns
            .and_then(|patterns| patterns.index_of(name))
            .ok_or_else(|| {
                GraphicsError::new(
                    Some(offset),
                    GraphicsErrorKind::MissingPattern {
                        name: name.as_bytes().to_vec(),
                    },
                )
            })?;
        self.current.fill_color =
            base.color_from_operands(offset, operator, &operands[..base.component_count()])?;
        self.current.fill_pattern = Some(pattern);
        Ok(())
    }

    fn spot_color(
        &self,
        offset: ByteOffset,
        operator: &'static [u8],
        operands: &[PdfPrimitive<'_>],
        color_space_index: usize,
    ) -> GraphicsResult<DeviceColor> {
        let color_space = self
            .resources
            .color_spaces
            .and_then(|color_spaces| color_spaces.get_index(color_space_index))
            .ok_or_else(|| invalid_color_space_resource(operator))?;
        if operands.len() != color_space.colorant_count {
            return Err(GraphicsError::new(
                Some(offset),
                GraphicsErrorKind::OperandCount {
                    operator,
                    expected: color_space.colorant_count,
                    actual: operands.len(),
                },
            ));
        }
        let mut tints = [0.0; MAX_TINT_FUNCTION_COMPONENTS];
        for (target, operand) in tints.iter_mut().zip(operands.iter()) {
            *target = number_from_primitive(operand)
                .ok_or_else(|| invalid_operand(offset, operator))?
                .clamp(0.0, 1.0);
        }
        Ok(color_space.evaluate(&tints[..operands.len()]))
    }

    fn set_ext_graphics_state(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"gs", operands, 1)?;
        let name = name_operand(offset, b"gs", operands, 0)?;
        let state = self
            .resources
            .ext_graphics_states
            .and_then(|states| states.get(name))
            .ok_or_else(|| {
                GraphicsError::new(
                    Some(offset),
                    GraphicsErrorKind::MissingExtGraphicsState {
                        name: name.as_bytes().to_vec(),
                    },
                )
            })?;
        self.current.blend_mode = state.blend_mode;
        self.current.fill_alpha = state.fill_alpha;
        self.current.stroke_alpha = state.stroke_alpha;
        self.current.fill_overprint = state.fill_overprint;
        self.current.stroke_overprint = state.stroke_overprint;
        self.current.overprint_mode = state.overprint_mode;
        Ok(())
    }

    fn paint_shading(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"sh", operands, 1)?;
        let name = name_operand(offset, b"sh", operands, 0)?;
        let shading = self
            .resources
            .shadings
            .and_then(|shadings| shadings.get(name))
            .ok_or_else(|| {
                GraphicsError::new(
                    Some(offset),
                    GraphicsErrorKind::MissingShading {
                        name: name.as_bytes().to_vec(),
                    },
                )
            })?;
        self.display_list.push(
            DisplayItem::Shading(ShadingDisplayItem {
                shading: shading.clone(),
                state: self.current,
            }),
            self.options.max_display_items,
            offset,
        )
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
        let fill_pattern = self.fill_pattern_for_paint(paint);
        self.display_list.push(
            DisplayItem::Path(PathDisplayItem {
                segments,
                paint,
                state: self.current,
                fill_pattern,
            }),
            self.options.max_display_items,
            offset,
        )
    }

    fn fill_pattern_for_paint(&self, paint: PaintMode) -> Option<TilingPattern> {
        if matches!(paint, PaintMode::Stroke) {
            return None;
        }
        let pattern_index = self.current.fill_pattern?;
        self.resources
            .patterns
            .and_then(|patterns| patterns.get_index(pattern_index))
            .cloned()
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
    joins: Vec<StrokeJoin>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineSegment {
    from: Point,
    to: Point,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StrokeJoin {
    previous: LineSegment,
    next: LineSegment,
    point: Point,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveClip {
    path: FlattenedPath,
    rule: FillRule,
    graphics_state_depth: usize,
    graphics_state_scope_id: u64,
}

#[derive(Debug, Clone, Copy)]
struct PathRasterContext<'a> {
    transform: PageTransform,
    options: PathRasterOptions,
    clips: &'a [ActiveClip],
}

#[derive(Debug, Clone, Copy)]
struct StrokeRasterState {
    line_width: f64,
    ctm_scale: f64,
    color: DeviceColor,
    blend_mode: BlendMode,
    alpha: f64,
    dash_pattern: StrokeDashPattern,
    dash_scale: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
}

fn truncate_clips_to_scope(
    active_clips: &mut Vec<ActiveClip>,
    graphics_state_depth: usize,
    graphics_state_scope_id: u64,
) {
    while active_clips.last().is_some_and(|clip| {
        clip.graphics_state_depth > graphics_state_depth
            || (clip.graphics_state_depth == graphics_state_depth
                && clip.graphics_state_scope_id != graphics_state_scope_id)
    }) {
        active_clips.pop();
    }
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
        let translation = match self.current_writing_mode() {
            TextWritingMode::Horizontal => Matrix::translate(advance, 0.0),
            TextWritingMode::Vertical => Matrix::translate(0.0, -advance),
        };
        self.text.text_matrix = self.text.text_matrix.multiply(translation);
    }

    fn current_writing_mode(&self) -> TextWritingMode {
        self.text
            .font
            .as_ref()
            .map_or(TextWritingMode::Horizontal, |font| font.writing_mode)
    }

    fn glyph_advance(&self, glyph: &TextGlyph) -> f64 {
        let word_spacing = if glyph.unicode == " " {
            self.text.word_spacing
        } else {
            0.0
        };
        let font_width = self
            .text
            .font
            .as_ref()
            .map_or(500.0, |font| font.advance_width_for_glyph(glyph));
        (self.text.font_size * font_width / 1000.0 + self.text.character_spacing + word_spacing)
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
            b"g" => self.set_fill_gray(offset, operands),
            b"rg" => self.set_fill_rgb(offset, operands),
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

    fn set_fill_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"g", operands, 1)?;
        let gray = DeviceGray(number_operand(offset, b"g", operands, 0)?.clamp(0.0, 1.0));
        self.current.fill_gray = gray;
        self.current.fill_color = DeviceColor::Gray(gray);
        self.current.fill_color_space = FillColorSpace::Device;
        self.current.fill_pattern = None;
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
        self.current.fill_color_space = FillColorSpace::Device;
        self.current.fill_pattern = None;
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
            b"J" => self.set_line_cap(offset, operands),
            b"j" => self.set_line_join(offset, operands),
            b"M" => self.set_miter_limit(offset, operands),
            b"d" => self.set_stroke_dash(offset, operands),
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

    fn set_stroke_dash(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.stroke_dash = dash_pattern_operand(offset, operands)?;
        Ok(())
    }

    fn set_line_cap(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.line_cap = line_cap_operand(offset, operands)?;
        Ok(())
    }

    fn set_line_join(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.line_join = line_join_operand(offset, operands)?;
        Ok(())
    }

    fn set_miter_limit(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        self.current.miter_limit = miter_limit_operand(offset, operands)?;
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

fn dash_pattern_operand(
    offset: ByteOffset,
    operands: &[PdfPrimitive<'_>],
) -> GraphicsResult<StrokeDashPattern> {
    expect_operand_count(offset, b"d", operands, 2)?;
    let Some(PdfPrimitive::Array(values)) = operands.first() else {
        return Err(invalid_operand(offset, b"d"));
    };
    let phase = number_operand(offset, b"d", operands, 1)?;
    if phase < 0.0 {
        return Err(invalid_operand(offset, b"d"));
    }
    if values.is_empty() {
        return Ok(StrokeDashPattern::solid());
    }
    if values.len() > MAX_STROKE_DASH_SEGMENTS {
        return Err(GraphicsError::new(
            Some(offset),
            GraphicsErrorKind::UnsupportedDashPattern {
                limit: MAX_STROKE_DASH_SEGMENTS,
            },
        ));
    }

    let mut pattern = StrokeDashPattern {
        phase,
        ..StrokeDashPattern::solid()
    };
    let mut total = 0.0;
    for (index, value) in values.iter().enumerate() {
        let segment = match value {
            PdfPrimitive::Number(PdfNumber::Integer(value)) if *value >= 0 => *value as f64,
            PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() && *value >= 0.0 => {
                *value
            }
            _ => return Err(invalid_operand(offset, b"d")),
        };
        pattern.segments[index] = segment;
        total += segment;
    }
    if total <= f64::EPSILON {
        return Err(invalid_operand(offset, b"d"));
    }
    pattern.len = values.len();
    Ok(pattern)
}

fn line_cap_operand(offset: ByteOffset, operands: &[PdfPrimitive<'_>]) -> GraphicsResult<LineCap> {
    expect_operand_count(offset, b"J", operands, 1)?;
    let Some(PdfPrimitive::Number(PdfNumber::Integer(value))) = operands.first() else {
        return Err(invalid_operand(offset, b"J"));
    };
    LineCap::from_pdf(*value).ok_or_else(|| invalid_operand(offset, b"J"))
}

fn line_join_operand(
    offset: ByteOffset,
    operands: &[PdfPrimitive<'_>],
) -> GraphicsResult<LineJoin> {
    expect_operand_count(offset, b"j", operands, 1)?;
    let Some(PdfPrimitive::Number(PdfNumber::Integer(value))) = operands.first() else {
        return Err(invalid_operand(offset, b"j"));
    };
    LineJoin::from_pdf(*value).ok_or_else(|| invalid_operand(offset, b"j"))
}

fn miter_limit_operand(offset: ByteOffset, operands: &[PdfPrimitive<'_>]) -> GraphicsResult<f64> {
    expect_operand_count(offset, b"M", operands, 1)?;
    let limit = number_operand(offset, b"M", operands, 0)?;
    if limit < 1.0 {
        return Err(invalid_operand(offset, b"M"));
    }
    Ok(limit)
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
        text.push_str(&mapped);
        glyphs.push(TextGlyph {
            character_code,
            unicode: mapped.to_string(),
            layout: classify_text_layout(&mapped),
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
        let unicode = character.to_string();
        glyphs.push(TextGlyph {
            character_code: u32::from(*byte),
            unicode,
            layout: classify_text_layout_char(character),
        });
    }
    Ok(DecodedTextRun { text, glyphs })
}

fn classify_text_layout(mapped: &str) -> TextLayoutStatus {
    let mut chars = mapped.chars();
    let Some(first) = chars.next() else {
        return TextLayoutStatus::Simple;
    };
    if mapped.chars().any(is_combining_mark) {
        return TextLayoutStatus::CombiningMarkPositioned;
    }
    if chars.next().is_some() || is_unicode_ligature(first) {
        return TextLayoutStatus::LigatureExpanded;
    }
    classify_text_layout_char(first)
}

fn classify_text_layout_char(character: char) -> TextLayoutStatus {
    if is_pre_shaped_script_character(character) {
        TextLayoutStatus::PreShapedScriptPreserved
    } else if character.is_ascii() || matches!(character, '\u{00a0}'..='\u{024f}') {
        TextLayoutStatus::Simple
    } else {
        TextLayoutStatus::Unsupported {
            reason: TextLayoutFallbackReason::ComplexScriptShaping,
        }
    }
}

fn is_unicode_ligature(character: char) -> bool {
    matches!(character, '\u{fb00}'..='\u{fb06}')
}

fn is_combining_mark(character: char) -> bool {
    matches!(
        character,
        '\u{0300}'..='\u{036f}'
            | '\u{1ab0}'..='\u{1aff}'
            | '\u{1dc0}'..='\u{1dff}'
            | '\u{20d0}'..='\u{20ff}'
            | '\u{fe20}'..='\u{fe2f}'
    )
}

fn is_pre_shaped_script_character(character: char) -> bool {
    matches!(
        character,
        '\u{0590}'..='\u{08ff}' | '\u{fb1d}'..='\u{fdff}' | '\u{fe70}'..='\u{feff}'
    )
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
    let transparency_group = form_transparency_group(stream.dictionary())?;
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
        transparency_group,
    })
}

fn decode_ext_graphics_state(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<ExtGraphicsState> {
    ext_graphics_state_soft_mask(dictionary)?;
    ext_graphics_state_black_point_compensation(dictionary)?;
    Ok(ExtGraphicsState {
        blend_mode: ext_graphics_state_blend_mode(dictionary)?,
        fill_alpha: ext_graphics_state_alpha(dictionary, b"ca")?,
        stroke_alpha: ext_graphics_state_alpha(dictionary, b"CA")?,
        fill_overprint: ext_graphics_state_bool(dictionary, b"op")?.unwrap_or(false),
        stroke_overprint: ext_graphics_state_bool(dictionary, b"OP")?.unwrap_or(false),
        overprint_mode: ext_graphics_state_overprint_mode(dictionary)?,
    })
}

fn ext_graphics_state_alpha(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<f64> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(1.0);
    };
    let alpha = match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => *value as f64,
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => *value,
        _ => return Err(invalid_ext_graphics_state(key)),
    };
    if (0.0..=1.0).contains(&alpha) {
        Ok(alpha)
    } else {
        Err(invalid_ext_graphics_state(key))
    }
}

fn ext_graphics_state_blend_mode(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<BlendMode> {
    let Some(value) = dictionary_value(dictionary, b"BM") else {
        return Ok(BlendMode::Normal);
    };
    match value {
        PdfPrimitive::Name(name) => blend_mode_from_name(name.as_bytes()),
        PdfPrimitive::Array(values) => {
            for value in values {
                if let PdfPrimitive::Name(name) = value {
                    if let Ok(blend_mode) = blend_mode_from_name(name.as_bytes()) {
                        return Ok(blend_mode);
                    }
                }
            }
            Err(GraphicsError::new(
                None,
                GraphicsErrorKind::UnsupportedBlendMode {
                    mode: b"BM".to_vec(),
                },
            ))
        }
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedBlendMode {
                mode: b"malformed".to_vec(),
            },
        )),
    }
}

fn blend_mode_from_name(name: &[u8]) -> GraphicsResult<BlendMode> {
    match name {
        b"Normal" | b"Compatible" => Ok(BlendMode::Normal),
        b"Multiply" => Ok(BlendMode::Multiply),
        b"Screen" => Ok(BlendMode::Screen),
        _ => Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedBlendMode {
                mode: name.to_vec(),
            },
        )),
    }
}

fn ext_graphics_state_soft_mask(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<()> {
    let Some(value) = dictionary_value(dictionary, b"SMask") else {
        return Ok(());
    };
    if matches!(value, PdfPrimitive::Name(name) if name.as_bytes() == b"None") {
        return Ok(());
    }
    Err(GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedSoftMask {
            feature: b"SMask".to_vec(),
        },
    ))
}

fn ext_graphics_state_black_point_compensation(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<()> {
    let Some(value) = dictionary_value(dictionary, b"UseBlackPtComp") else {
        return Ok(());
    };
    let PdfPrimitive::Boolean(enabled) = value else {
        return Err(invalid_ext_graphics_state(b"UseBlackPtComp"));
    };
    if *enabled {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedColorManagement {
                feature: b"UseBlackPtComp".to_vec(),
            },
        ));
    }
    Ok(())
}

fn ext_graphics_state_bool(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<bool>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    let PdfPrimitive::Boolean(value) = value else {
        return Err(invalid_ext_graphics_state(key));
    };
    Ok(Some(*value))
}

fn ext_graphics_state_overprint_mode(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<u8> {
    let Some(value) = dictionary_value(dictionary, b"OPM") else {
        return Ok(0);
    };
    let PdfPrimitive::Number(PdfNumber::Integer(value)) = value else {
        return Err(invalid_ext_graphics_state(b"OPM"));
    };
    match *value {
        0 => Ok(0),
        1 => Ok(1),
        _ => Err(invalid_ext_graphics_state(b"OPM")),
    }
}

fn decode_shading(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> GraphicsResult<Shading> {
    let Some(PdfPrimitive::Number(PdfNumber::Integer(shading_type))) =
        dictionary_value(dictionary, b"ShadingType")
    else {
        return Err(unsupported_shading(b"ShadingType"));
    };
    if !matches!(shading_type, 2 | 3) {
        return Err(unsupported_shading(b"ShadingType"));
    }
    let color_space = shading_color_space(dictionary)?;
    let (start_color, end_color, exponent) = decode_type2_function(dictionary, color_space)?;
    let (extend_start, extend_end) = optional_bool_pair(dictionary, b"Extend")?;
    match shading_type {
        2 => {
            let coords = required_number_array::<4>(dictionary, b"Coords")?;
            Ok(Shading::Axial(AxialShading {
                start: Point {
                    x: coords[0],
                    y: coords[1],
                },
                end: Point {
                    x: coords[2],
                    y: coords[3],
                },
                start_color,
                end_color,
                exponent,
                extend_start,
                extend_end,
            }))
        }
        3 => {
            let coords = required_number_array::<6>(dictionary, b"Coords")?;
            if coords[2] < 0.0 || coords[5] < 0.0 {
                return Err(invalid_shading_resource(b"Coords"));
            }
            Ok(Shading::Radial(RadialShading {
                start_center: Point {
                    x: coords[0],
                    y: coords[1],
                },
                start_radius: coords[2],
                end_center: Point {
                    x: coords[3],
                    y: coords[4],
                },
                end_radius: coords[5],
                start_color,
                end_color,
                exponent,
                extend_start,
                extend_end,
            }))
        }
        _ => Err(unsupported_shading(b"ShadingType")),
    }
}

fn decode_shading_stream(
    resource_name: &[u8],
    stream: &StreamObject<'_>,
    options: DisplayListOptions,
) -> GraphicsResult<Shading> {
    let Some(PdfPrimitive::Number(PdfNumber::Integer(shading_type))) =
        dictionary_value(stream.dictionary(), b"ShadingType")
    else {
        return Err(invalid_shading_resource(resource_name));
    };
    if *shading_type != 4 {
        return decode_shading(stream.dictionary());
    }
    let decoded = stream
        .decode_with_options(StreamDecodeOptions {
            max_decoded_len: options.max_mesh_shading_bytes,
        })
        .map_err(|error| match error {
            pdfrust_object::ObjectError::StreamLimitExceeded { .. } => GraphicsError::new(
                None,
                GraphicsErrorKind::ShadingBytesOverflow {
                    limit: options.max_mesh_shading_bytes,
                },
            ),
            _ => GraphicsError::new(
                None,
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            ),
        })?;
    decode_type4_mesh_shading(stream.dictionary(), &decoded, options)
}

fn decode_type4_mesh_shading(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    data: &[u8],
    options: DisplayListOptions,
) -> GraphicsResult<Shading> {
    let color_space = shading_color_space(dictionary)?;
    let color_components = shading_component_count(color_space);
    let bits_per_coordinate = required_shading_u8(dictionary, b"BitsPerCoordinate")?;
    let bits_per_component = required_shading_u8(dictionary, b"BitsPerComponent")?;
    let bits_per_flag = required_shading_u8(dictionary, b"BitsPerFlag")?;
    if bits_per_coordinate != 8 {
        return Err(unsupported_shading(b"BitsPerCoordinate"));
    }
    if bits_per_component != 8 {
        return Err(unsupported_shading(b"BitsPerComponent"));
    }
    if !matches!(bits_per_flag, 2 | 8) {
        return Err(unsupported_shading(b"BitsPerFlag"));
    }
    let decode = required_mesh_decode_ranges(dictionary, color_components)?;
    let record_bits = usize::from(bits_per_flag)
        + 2 * usize::from(bits_per_coordinate)
        + color_components * usize::from(bits_per_component);
    let mut reader = MeshBitReader::new(data);
    let mut records = Vec::new();
    while reader.remaining_bits() >= record_bits {
        let flag = reader
            .read_bits(bits_per_flag)
            .ok_or_else(|| invalid_shading_resource(b"BitsPerFlag"))? as u8;
        let point = Point {
            x: decode_mesh_component(
                reader
                    .read_bits(bits_per_coordinate)
                    .ok_or_else(|| invalid_shading_resource(b"BitsPerCoordinate"))?,
                decode[0],
                decode[1],
                bits_per_coordinate,
            ),
            y: decode_mesh_component(
                reader
                    .read_bits(bits_per_coordinate)
                    .ok_or_else(|| invalid_shading_resource(b"BitsPerCoordinate"))?,
                decode[2],
                decode[3],
                bits_per_coordinate,
            ),
        };
        let color = decode_mesh_color(&mut reader, color_space, bits_per_component, &decode[4..])?;
        records.push((flag, MeshVertex { point, color }));
    }
    if records.len() < 3 {
        return Err(invalid_shading_resource(b"Data"));
    }
    let mut triangles = Vec::new();
    let mut index = 0usize;
    while index < records.len() {
        if records[index].0 != 0 {
            return Err(unsupported_shading(b"MeshFlag"));
        }
        let Some((_, first)) = records.get(index).copied() else {
            break;
        };
        let Some((_, second)) = records.get(index + 1).copied() else {
            return Err(invalid_shading_resource(b"Data"));
        };
        let Some((_, third)) = records.get(index + 2).copied() else {
            return Err(invalid_shading_resource(b"Data"));
        };
        if triangles.len() >= options.max_mesh_shading_triangles {
            return Err(GraphicsError::new(
                None,
                GraphicsErrorKind::ShadingTriangleOverflow {
                    limit: options.max_mesh_shading_triangles,
                },
            ));
        }
        triangles.push(MeshTriangle {
            vertices: [first, second, third],
        });
        index += 3;
    }
    Ok(Shading::Mesh(MeshShading { triangles }))
}

fn required_shading_u8(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> GraphicsResult<u8> {
    let Some(PdfPrimitive::Number(PdfNumber::Integer(value))) = dictionary_value(dictionary, key)
    else {
        return Err(invalid_shading_resource(key));
    };
    u8::try_from(*value).map_err(|_| invalid_shading_resource(key))
}

fn required_mesh_decode_ranges(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    color_components: usize,
) -> GraphicsResult<Vec<f64>> {
    let Some(PdfPrimitive::Array(values)) = dictionary_value(dictionary, b"Decode") else {
        return Err(invalid_shading_resource(b"Decode"));
    };
    let expected = 4 + color_components * 2;
    if values.len() != expected {
        return Err(invalid_shading_resource(b"Decode"));
    }
    values
        .iter()
        .map(|value| {
            number_from_primitive(value).ok_or_else(|| invalid_shading_resource(b"Decode"))
        })
        .collect()
}

fn shading_component_count(color_space: ShadingColorSpace) -> usize {
    match color_space {
        ShadingColorSpace::DeviceGray => 1,
        ShadingColorSpace::DeviceRgb => 3,
    }
}

fn decode_mesh_color(
    reader: &mut MeshBitReader<'_>,
    color_space: ShadingColorSpace,
    bits_per_component: u8,
    decode: &[f64],
) -> GraphicsResult<DeviceColor> {
    match color_space {
        ShadingColorSpace::DeviceGray => {
            let value = reader
                .read_bits(bits_per_component)
                .ok_or_else(|| invalid_shading_resource(b"BitsPerComponent"))?;
            Ok(DeviceColor::Gray(DeviceGray(
                decode_mesh_component(value, decode[0], decode[1], bits_per_component)
                    .clamp(0.0, 1.0),
            )))
        }
        ShadingColorSpace::DeviceRgb => {
            let r = reader
                .read_bits(bits_per_component)
                .ok_or_else(|| invalid_shading_resource(b"BitsPerComponent"))?;
            let g = reader
                .read_bits(bits_per_component)
                .ok_or_else(|| invalid_shading_resource(b"BitsPerComponent"))?;
            let b = reader
                .read_bits(bits_per_component)
                .ok_or_else(|| invalid_shading_resource(b"BitsPerComponent"))?;
            Ok(DeviceColor::Rgb {
                r: decode_mesh_component(r, decode[0], decode[1], bits_per_component)
                    .clamp(0.0, 1.0),
                g: decode_mesh_component(g, decode[2], decode[3], bits_per_component)
                    .clamp(0.0, 1.0),
                b: decode_mesh_component(b, decode[4], decode[5], bits_per_component)
                    .clamp(0.0, 1.0),
            })
        }
    }
}

fn decode_mesh_component(raw: u32, min: f64, max: f64, bits: u8) -> f64 {
    let max_raw = (1u32 << bits) - 1;
    min + (max - min) * f64::from(raw) / f64::from(max_raw)
}

#[derive(Debug, Clone, Copy)]
struct MeshBitReader<'a> {
    data: &'a [u8],
    bit_offset: usize,
}

impl<'a> MeshBitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bit_offset: 0,
        }
    }

    fn remaining_bits(&self) -> usize {
        self.data
            .len()
            .saturating_mul(8)
            .saturating_sub(self.bit_offset)
    }

    fn read_bits(&mut self, bits: u8) -> Option<u32> {
        if bits == 0 || bits > 24 || self.remaining_bits() < usize::from(bits) {
            return None;
        }
        let mut value = 0u32;
        for _ in 0..bits {
            let byte = self.data[self.bit_offset / 8];
            let bit = (byte >> (7 - (self.bit_offset % 8))) & 1;
            value = (value << 1) | u32::from(bit);
            self.bit_offset += 1;
        }
        Some(value)
    }
}

fn decode_spot_color_space(value: &PdfPrimitive<'_>) -> GraphicsResult<Option<SpotColorSpace>> {
    let PdfPrimitive::Array(items) = value else {
        return Ok(None);
    };
    let Some(PdfPrimitive::Name(kind)) = items.first() else {
        return Ok(None);
    };
    match kind.as_bytes() {
        b"Separation" => decode_separation_color_space(items).map(Some),
        b"DeviceN" => decode_devicen_color_space(items).map(Some),
        _ => Ok(None),
    }
}

fn decode_pattern_color_space(
    value: &PdfPrimitive<'_>,
) -> GraphicsResult<Option<PatternBaseColorSpace>> {
    let PdfPrimitive::Array(items) = value else {
        return Ok(None);
    };
    let [PdfPrimitive::Name(kind), base] = items.as_slice() else {
        return Ok(None);
    };
    if kind.as_bytes() != b"Pattern" {
        return Ok(None);
    }
    pattern_base_color_space(base).map(Some)
}

fn pattern_base_color_space(value: &PdfPrimitive<'_>) -> GraphicsResult<PatternBaseColorSpace> {
    match value {
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceGray" | b"G") => {
            Ok(PatternBaseColorSpace::DeviceGray)
        }
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceRGB" | b"RGB") => {
            Ok(PatternBaseColorSpace::DeviceRgb)
        }
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceCMYK" | b"CMYK") => {
            Ok(PatternBaseColorSpace::DeviceCmyk)
        }
        PdfPrimitive::Name(name) => Err(unsupported_spot_color_space(name.as_bytes())),
        _ => Err(invalid_color_space_resource(b"Pattern")),
    }
}

fn decode_separation_color_space(items: &[PdfPrimitive<'_>]) -> GraphicsResult<SpotColorSpace> {
    let [_, PdfPrimitive::Name(_colorant), alternate, function] = items else {
        return Err(invalid_color_space_resource(b"Separation"));
    };
    let alternate_space = alternate_color_space(alternate)?;
    Ok(SpotColorSpace {
        kind: SpotColorSpaceKind::Separation,
        colorant_count: 1,
        alternate_space,
        tint_transform: decode_type2_tint_function(function, alternate_space)?,
    })
}

fn decode_devicen_color_space(items: &[PdfPrimitive<'_>]) -> GraphicsResult<SpotColorSpace> {
    let [_, PdfPrimitive::Array(colorants), alternate, function, ..] = items else {
        return Err(invalid_color_space_resource(b"DeviceN"));
    };
    if colorants.is_empty() || colorants.len() > MAX_TINT_FUNCTION_COMPONENTS {
        return Err(invalid_color_space_resource(b"DeviceN"));
    }
    if !colorants
        .iter()
        .all(|colorant| matches!(colorant, PdfPrimitive::Name(_)))
    {
        return Err(invalid_color_space_resource(b"DeviceN"));
    }
    let alternate_space = alternate_color_space(alternate)?;
    Ok(SpotColorSpace {
        kind: SpotColorSpaceKind::DeviceN,
        colorant_count: colorants.len(),
        alternate_space,
        tint_transform: decode_type2_tint_function(function, alternate_space)?,
    })
}

fn alternate_color_space(value: &PdfPrimitive<'_>) -> GraphicsResult<AlternateColorSpace> {
    match value {
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceGray" | b"G") => {
            Ok(AlternateColorSpace::DeviceGray)
        }
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceRGB" | b"RGB") => {
            Ok(AlternateColorSpace::DeviceRgb)
        }
        PdfPrimitive::Name(name) if matches!(name.as_bytes(), b"DeviceCMYK" | b"CMYK") => {
            Ok(AlternateColorSpace::DeviceCmyk)
        }
        PdfPrimitive::Name(name) => Err(unsupported_spot_color_space(name.as_bytes())),
        _ => Err(invalid_color_space_resource(b"Alternate")),
    }
}

fn decode_type2_tint_function(
    value: &PdfPrimitive<'_>,
    alternate_space: AlternateColorSpace,
) -> GraphicsResult<Type2TintFunction> {
    let PdfPrimitive::Dictionary(function) = value else {
        return Err(invalid_color_space_resource(b"Function"));
    };
    let Some(PdfPrimitive::Number(PdfNumber::Integer(2))) =
        dictionary_value(function, b"FunctionType")
    else {
        return Err(unsupported_spot_color_space(b"FunctionType"));
    };
    let exponent = required_color_space_number(function, b"N")?;
    if !exponent.is_finite() || exponent <= 0.0 {
        return Err(invalid_color_space_resource(b"N"));
    }
    let output_components = alternate_space_component_count(alternate_space);
    Ok(Type2TintFunction {
        c0: optional_color_component_array(function, b"C0", output_components, 0.0)?,
        c1: optional_color_component_array(function, b"C1", output_components, 1.0)?,
        output_components,
        exponent,
    })
}

fn alternate_space_component_count(alternate_space: AlternateColorSpace) -> usize {
    match alternate_space {
        AlternateColorSpace::DeviceGray => 1,
        AlternateColorSpace::DeviceRgb => 3,
        AlternateColorSpace::DeviceCmyk => 4,
    }
}

fn optional_color_component_array(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    expected: usize,
    default: f64,
) -> GraphicsResult<[f64; MAX_TINT_FUNCTION_COMPONENTS]> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok([default; MAX_TINT_FUNCTION_COMPONENTS]);
    };
    let PdfPrimitive::Array(values) = value else {
        return Err(invalid_color_space_resource(key));
    };
    if values.len() != expected {
        return Err(invalid_color_space_resource(key));
    }
    let mut components = [default; MAX_TINT_FUNCTION_COMPONENTS];
    for (target, value) in components.iter_mut().zip(values.iter()).take(expected) {
        *target = number_from_primitive(value)
            .ok_or_else(|| invalid_color_space_resource(key))?
            .clamp(0.0, 1.0);
    }
    Ok(components)
}

fn required_color_space_number(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> GraphicsResult<f64> {
    dictionary_value(dictionary, key)
        .and_then(number_from_primitive)
        .ok_or_else(|| invalid_color_space_resource(key))
}

fn alternate_color_to_rgb(
    alternate_space: AlternateColorSpace,
    components: [f64; MAX_TINT_FUNCTION_COMPONENTS],
) -> [f64; 3] {
    match alternate_space {
        AlternateColorSpace::DeviceGray => [components[0], components[0], components[0]],
        AlternateColorSpace::DeviceRgb => [components[0], components[1], components[2]],
        AlternateColorSpace::DeviceCmyk => {
            let c = components[0].clamp(0.0, 1.0);
            let m = components[1].clamp(0.0, 1.0);
            let y = components[2].clamp(0.0, 1.0);
            let k = components[3].clamp(0.0, 1.0);
            [
                (1.0 - c) * (1.0 - k),
                (1.0 - m) * (1.0 - k),
                (1.0 - y) * (1.0 - k),
            ]
        }
    }
}

/// Decodes a colored tiling pattern stream into a reusable pattern resource.
///
/// # Errors
///
/// Returns [`GraphicsError`] when required pattern metadata is malformed or
/// when the pattern content stream exceeds display-list limits.
pub fn decode_tiling_pattern(
    resource_name: Vec<u8>,
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    content: &[u8],
    options: DisplayListOptions,
) -> GraphicsResult<TilingPattern> {
    let Some(PdfPrimitive::Number(PdfNumber::Integer(1))) =
        dictionary_value(dictionary, b"PatternType")
    else {
        return Err(invalid_pattern_resource(&resource_name));
    };
    let paint = match dictionary_value(dictionary, b"PaintType") {
        Some(PdfPrimitive::Number(PdfNumber::Integer(1))) => TilingPatternPaint::Colored,
        Some(PdfPrimitive::Number(PdfNumber::Integer(2))) => TilingPatternPaint::Uncolored,
        _ => return Err(unsupported_pattern(b"PaintType")),
    };
    match dictionary_value(dictionary, b"TilingType") {
        Some(PdfPrimitive::Number(PdfNumber::Integer(1..=3))) => {}
        _ => return Err(unsupported_pattern(b"TilingType")),
    }
    let bbox =
        required_bbox(dictionary, b"BBox").map_err(|_| invalid_pattern_resource(&resource_name))?;
    let x_step = required_pattern_number(dictionary, b"XStep", &resource_name)?;
    let y_step = required_pattern_number(dictionary, b"YStep", &resource_name)?;
    if !x_step.is_finite() || !y_step.is_finite() || x_step <= 0.0 || y_step <= 0.0 {
        return Err(invalid_pattern_resource(&resource_name));
    }
    let items = build_path_display_list(tokenize_content(PdfBytes::new(content)), options)?;
    Ok(TilingPattern {
        resource_name,
        paint,
        bbox,
        x_step,
        y_step,
        items,
    })
}

fn required_pattern_number(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    resource_name: &[u8],
) -> GraphicsResult<f64> {
    dictionary_value(dictionary, key)
        .and_then(number_from_primitive)
        .ok_or_else(|| invalid_pattern_resource(resource_name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShadingColorSpace {
    DeviceGray,
    DeviceRgb,
}

fn shading_color_space(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<ShadingColorSpace> {
    match dictionary_value(dictionary, b"ColorSpace") {
        Some(PdfPrimitive::Name(name)) if matches!(name.as_bytes(), b"DeviceGray" | b"G") => {
            Ok(ShadingColorSpace::DeviceGray)
        }
        Some(PdfPrimitive::Name(name)) if matches!(name.as_bytes(), b"DeviceRGB" | b"RGB") => {
            Ok(ShadingColorSpace::DeviceRgb)
        }
        Some(PdfPrimitive::Name(name)) => Err(unsupported_shading(name.as_bytes())),
        _ => Err(invalid_shading_resource(b"ColorSpace")),
    }
}

fn decode_type2_function(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    color_space: ShadingColorSpace,
) -> GraphicsResult<(DeviceColor, DeviceColor, f64)> {
    let Some(PdfPrimitive::Dictionary(function)) = dictionary_value(dictionary, b"Function") else {
        return Err(invalid_shading_resource(b"Function"));
    };
    let Some(PdfPrimitive::Number(PdfNumber::Integer(2))) =
        dictionary_value(function, b"FunctionType")
    else {
        return Err(unsupported_shading(b"FunctionType"));
    };
    let exponent = required_number(function, b"N")?;
    if !exponent.is_finite() || exponent <= 0.0 {
        return Err(invalid_shading_resource(b"N"));
    }
    let start_color = function_color(function, b"C0", color_space)?;
    let end_color = function_color(function, b"C1", color_space)?;
    Ok((start_color, end_color, exponent))
}

fn function_color(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
    color_space: ShadingColorSpace,
) -> GraphicsResult<DeviceColor> {
    match color_space {
        ShadingColorSpace::DeviceGray => {
            let components = required_number_array::<1>(dictionary, key)?;
            Ok(DeviceColor::Gray(DeviceGray(components[0].clamp(0.0, 1.0))))
        }
        ShadingColorSpace::DeviceRgb => {
            let components = required_number_array::<3>(dictionary, key)?;
            Ok(DeviceColor::Rgb {
                r: components[0].clamp(0.0, 1.0),
                g: components[1].clamp(0.0, 1.0),
                b: components[2].clamp(0.0, 1.0),
            })
        }
    }
}

fn required_number(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> GraphicsResult<f64> {
    dictionary_value(dictionary, key)
        .and_then(number_from_primitive)
        .ok_or_else(|| invalid_shading_resource(key))
}

fn required_number_array<const N: usize>(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> GraphicsResult<[f64; N]> {
    let Some(PdfPrimitive::Array(values)) = dictionary_value(dictionary, key) else {
        return Err(invalid_shading_resource(key));
    };
    if values.len() != N {
        return Err(invalid_shading_resource(key));
    }
    let mut numbers = [0.0; N];
    for (target, value) in numbers.iter_mut().zip(values) {
        *target = number_from_primitive(value).ok_or_else(|| invalid_shading_resource(key))?;
    }
    Ok(numbers)
}

fn optional_bool_pair(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &'static [u8],
) -> GraphicsResult<(bool, bool)> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok((false, false));
    };
    let PdfPrimitive::Array(values) = value else {
        return Err(invalid_shading_resource(key));
    };
    let [PdfPrimitive::Boolean(first), PdfPrimitive::Boolean(second)] = values.as_slice() else {
        return Err(invalid_shading_resource(key));
    };
    Ok((*first, *second))
}

fn number_from_primitive(value: &PdfPrimitive<'_>) -> Option<f64> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Some(*value as f64),
        PdfPrimitive::Number(PdfNumber::Real(value)) if value.is_finite() => Some(*value),
        _ => None,
    }
}

fn form_transparency_group(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<Option<TransparencyGroup>> {
    let Some(value) = dictionary_value(dictionary, b"Group") else {
        return Ok(None);
    };
    let PdfPrimitive::Dictionary(group) = value else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource {
                name: b"Group".to_vec(),
            },
        ));
    };
    if !dictionary_name_is(group, b"S", b"Transparency") {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::UnsupportedTransparencyGroup {
                feature: b"S".to_vec(),
            },
        ));
    }
    Ok(Some(TransparencyGroup {
        isolated: optional_bool(group, b"I")?.unwrap_or(false),
        knockout: optional_bool(group, b"K")?.unwrap_or(false),
    }))
}

fn optional_bool(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<bool>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    let PdfPrimitive::Boolean(value) = value else {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidFormResource { name: key.to_vec() },
        ));
    };
    Ok(Some(*value))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageDecodeLimits {
    max_image_bytes: usize,
    max_icc_profile_bytes: usize,
    max_icc_transform_workspace_bytes: usize,
    max_soft_mask_depth: usize,
}

impl ImageDecodeLimits {
    const fn from_display_options(options: DisplayListOptions) -> Self {
        Self {
            max_image_bytes: options.max_image_bytes,
            max_icc_profile_bytes: options.max_icc_profile_bytes,
            max_icc_transform_workspace_bytes: options.max_icc_transform_workspace_bytes,
            max_soft_mask_depth: options.max_soft_mask_depth,
        }
    }
}

fn decode_image_xobject<'a, R>(
    resource_name: PdfName<'_>,
    stream: &StreamObject<'a>,
    resolver: &'a R,
    limits: ImageDecodeLimits,
    icc_cache: &mut IccTransformCache,
) -> GraphicsResult<ImageXObject>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    decode_image_xobject_at_depth(resource_name, stream, resolver, limits, 0, icc_cache)
}

fn decode_image_xobject_at_depth<'a, R>(
    resource_name: PdfName<'_>,
    stream: &StreamObject<'a>,
    resolver: &'a R,
    limits: ImageDecodeLimits,
    soft_mask_depth: usize,
    icc_cache: &mut IccTransformCache,
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
    let image_mask = image_mask_flag(stream.dictionary())?;
    let bits_per_component = if image_mask {
        optional_image_u8(stream.dictionary(), b"BitsPerComponent")?
            .or(optional_image_u8(stream.dictionary(), b"BPC")?)
            .unwrap_or(1)
    } else {
        required_u8(stream.dictionary(), b"BitsPerComponent")
            .or_else(|_| required_u8(stream.dictionary(), b"BPC"))
            .map_err(|_| invalid_image_resource(resource_name.as_bytes()))?
    };
    if image_mask && bits_per_component != 1 {
        return Err(invalid_image_resource(resource_name.as_bytes()));
    }
    if !image_mask && bits_per_component != 8 {
        return Err(invalid_image_resource(resource_name.as_bytes()));
    }
    let color_space = if image_mask {
        ImageColorSpaceInfo::new(ImageColorSpace::DeviceGray)
    } else {
        image_color_space_with_icc(
            stream.dictionary(),
            resolver,
            limits.max_icc_profile_bytes,
            limits.max_icc_transform_workspace_bytes,
            icc_cache,
        )?
    };
    let expected_len = if image_mask {
        expected_image_mask_len(width, height)?
    } else {
        expected_image_len(width, height, color_space.kind)?
    };
    enforce_image_byte_budget(expected_len, limits.max_image_bytes)?;
    let image_filter = image_filter(stream.dictionary())?;
    let decoded = decode_image_samples(
        stream,
        image_filter,
        width,
        height,
        color_space.kind,
        bits_per_component,
        limits.max_image_bytes,
    )?;
    enforce_image_byte_budget(decoded.len(), limits.max_image_bytes)?;
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
    let (kind, soft_mask) = if image_mask {
        (
            ImageKind::StencilMask {
                paint_one_bits: image_mask_paints_one_bits(stream.dictionary())?,
            },
            None,
        )
    } else {
        apply_image_decode(
            &mut decoded,
            color_space.kind,
            image_decode_ranges(stream.dictionary())?,
        )?;
        (
            ImageKind::Color,
            soft_mask_samples(
                stream.dictionary(),
                resolver,
                width,
                height,
                limits,
                soft_mask_depth,
                icc_cache,
            )?,
        )
    };
    Ok(ImageXObject {
        resource_name: resource_name.as_bytes().to_vec(),
        width,
        height,
        bits_per_component,
        color_space: color_space.kind,
        samples: Arc::from(decoded),
        kind,
        indexed_lookup: color_space.indexed_lookup,
        soft_mask,
    })
}

fn image_mask_flag(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> GraphicsResult<bool> {
    let Some(value) =
        dictionary_value(dictionary, b"ImageMask").or_else(|| dictionary_value(dictionary, b"IM"))
    else {
        return Ok(false);
    };
    let PdfPrimitive::Boolean(value) = value else {
        return Err(invalid_image_resource(b"ImageMask"));
    };
    Ok(*value)
}

fn optional_image_u8(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<u8>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    let number = primitive_number(value).ok_or_else(|| invalid_image_resource(key))?;
    if number.fract() != 0.0 {
        return Err(invalid_image_resource(key));
    }
    u8::try_from(number as i64)
        .map(Some)
        .map_err(|_| invalid_image_resource(key))
}

fn image_mask_paints_one_bits(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> GraphicsResult<bool> {
    let Some(decode) = image_decode_ranges(dictionary)? else {
        return Ok(false);
    };
    if decode.len != 1 {
        return Err(invalid_image_resource(b"Decode"));
    }
    match decode.ranges[0] {
        (0.0, 1.0) => Ok(false),
        (1.0, 0.0) => Ok(true),
        _ => Err(invalid_image_resource(b"Decode")),
    }
}

fn soft_mask_samples<'a, R>(
    dictionary: &[(PdfName<'a>, PdfPrimitive<'a>)],
    resolver: &'a R,
    width: u32,
    height: u32,
    limits: ImageDecodeLimits,
    soft_mask_depth: usize,
    icc_cache: &mut IccTransformCache,
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
    if soft_mask_depth >= limits.max_soft_mask_depth {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::SoftMaskDepthOverflow {
                limit: limits.max_soft_mask_depth,
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
        limits,
        soft_mask_depth + 1,
        icc_cache,
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
    let expected_len = expected_image_len(width, height, color_space.kind)?;
    enforce_image_byte_budget(expected_len, max_image_bytes)?;
    enforce_image_byte_budget(data.len(), max_image_bytes)?;
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
        kind: ImageKind::Color,
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
            apply_png_predictor(decoded, predictor)
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

fn apply_png_predictor(mut samples: Vec<u8>, predictor: ImagePredictor) -> GraphicsResult<Vec<u8>> {
    if samples.len() != predictor.encoded_len() {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::InvalidImageDataLength {
                expected: predictor.encoded_len(),
                actual: samples.len(),
            },
        ));
    }
    for row_index in 0..predictor.row_count {
        apply_png_predictor_row(&mut samples, predictor, row_index)?;
    }
    samples.truncate(predictor.decoded_len());
    Ok(samples)
}

fn predictor_row_start(
    samples: &[u8],
    predictor: ImagePredictor,
    row_index: usize,
) -> GraphicsResult<(PngFilter, usize)> {
    match predictor.kind {
        PngPredictorKind::Fixed(filter) => {
            let start = row_index * predictor.row_len;
            Ok((filter, start))
        }
        PngPredictorKind::Adaptive => {
            let encoded_row_len = predictor.row_len + 1;
            let start = row_index * encoded_row_len;
            let filter = match samples[start] {
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
            Ok((filter, start + 1))
        }
    }
}

fn apply_png_predictor_row(
    samples: &mut [u8],
    predictor: ImagePredictor,
    row_index: usize,
) -> GraphicsResult<()> {
    let output_start = row_index * predictor.row_len;
    let (filter, input_start) = predictor_row_start(samples, predictor, row_index)?;
    for index in 0..predictor.row_len {
        let sample = samples[input_start + index];
        let left = if index >= predictor.bytes_per_pixel {
            samples[output_start + index - predictor.bytes_per_pixel]
        } else {
            0
        };
        let up = if output_start >= predictor.row_len {
            samples[output_start - predictor.row_len + index]
        } else {
            0
        };
        let up_left = if output_start >= predictor.row_len && index >= predictor.bytes_per_pixel {
            samples[output_start - predictor.row_len + index - predictor.bytes_per_pixel]
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
        samples[output_start + index] = sample.wrapping_add(predicted);
    }
    Ok(())
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

fn expected_image_mask_len(width: u32, height: u32) -> GraphicsResult<usize> {
    let row_bytes = (width as usize).checked_add(7).map(|bits| bits / 8);
    row_bytes
        .and_then(|row_bytes| row_bytes.checked_mul(height as usize))
        .ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::ImageBytesOverflow { limit: usize::MAX },
            )
        })
}

fn enforce_image_byte_budget(byte_len: usize, max_image_bytes: usize) -> GraphicsResult<()> {
    if byte_len > max_image_bytes {
        Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageBytesOverflow {
                limit: max_image_bytes,
            },
        ))
    } else {
        Ok(())
    }
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
    flattened.joins = stroke_joins_from_subpaths(&flattened.subpaths, limit)?;
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

fn stroke_joins_from_subpaths(
    subpaths: &[Vec<Point>],
    limit: usize,
) -> RasterResult<Vec<StrokeJoin>> {
    let mut joins = Vec::new();
    for subpath in subpaths {
        for points in subpath.windows(3) {
            if joins.len() >= limit {
                return Err(RasterError::new(RasterErrorKind::PathComplexityOverflow {
                    limit,
                }));
            }
            joins.push(StrokeJoin {
                previous: LineSegment {
                    from: points[0],
                    to: points[1],
                },
                next: LineSegment {
                    from: points[1],
                    to: points[2],
                },
                point: points[1],
            });
        }
    }
    Ok(joins)
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
    blend_mode: BlendMode,
    alpha: f64,
    context: PathRasterContext<'_>,
) -> RasterResult<()> {
    let source = device_color_to_rgba(color);
    let samples = u32::from(context.options.supersample);
    let sample_count = samples * samples;
    let dimensions = device.dimensions();
    let Some(bounds) =
        flattened_bounds(path).and_then(|bounds| device_pixel_bounds(bounds, dimensions, 0.0))
    else {
        return Ok(());
    };
    for y in bounds.min_y..bounds.max_y {
        for x in bounds.min_x..bounds.max_x {
            let mut covered = 0;
            for sample_y in 0..samples {
                for sample_x in 0..samples {
                    let point = sample_point(x, y, sample_x, sample_y, samples);
                    if point_in_active_clips(point, context.clips)
                        && point_in_path(point, path, rule)
                    {
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
                    blend_mode,
                    alpha * f64::from(covered) / f64::from(sample_count),
                )?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct PatternSample {
    path: FlattenedPath,
    rule: FillRule,
    color: Rgba,
}

#[derive(Debug, Default, Clone)]
struct PatternCellCache {
    entries: Vec<CachedPatternCell>,
    uncached_samples: Vec<PatternSample>,
    max_entries: usize,
}

impl PatternCellCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            uncached_samples: Vec::new(),
            max_entries,
        }
    }

    fn samples_for(
        &mut self,
        pattern: &TilingPattern,
        fill_color: DeviceColor,
        transform: PageTransform,
        options: PathRasterOptions,
    ) -> RasterResult<&[PatternSample]> {
        let key = PatternCellCacheKey::new(pattern, fill_color, transform.scale);
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            return Ok(self.entries[index].samples.as_slice());
        }
        let samples = build_pattern_samples(pattern, fill_color, options)?;
        if self.max_entries == 0 {
            self.entries.clear();
            self.uncached_samples = samples;
            return Ok(self.uncached_samples.as_slice());
        }
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(CachedPatternCell { key, samples });
        Ok(self
            .entries
            .last()
            .expect("pattern cache entry was just inserted")
            .samples
            .as_slice())
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone)]
struct CachedPatternCell {
    key: PatternCellCacheKey,
    samples: Vec<PatternSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatternCellCacheKey {
    resource_name: Vec<u8>,
    paint: TilingPatternPaint,
    fill_color: Rgba,
    transform_scale_microunits: i64,
}

impl PatternCellCacheKey {
    fn new(pattern: &TilingPattern, fill_color: DeviceColor, transform_scale: f64) -> Self {
        let fill_color = match pattern.paint {
            TilingPatternPaint::Colored => Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            TilingPatternPaint::Uncolored => device_color_to_rgba(fill_color),
        };
        Self {
            resource_name: pattern.resource_name.clone(),
            paint: pattern.paint,
            fill_color,
            transform_scale_microunits: quantize_pattern_scale(transform_scale),
        }
    }
}

fn quantize_pattern_scale(scale: f64) -> i64 {
    (scale * 1_000_000.0).round() as i64
}

fn fill_path_with_tiling_pattern(
    device: &mut RasterDevice,
    path: &FlattenedPath,
    rule: FillRule,
    pattern: &TilingPattern,
    state: GraphicsState,
    context: PathRasterContext<'_>,
    pattern_cache: &mut PatternCellCache,
) -> RasterResult<()> {
    validate_pattern_tile_budget(path, pattern, context.options)?;
    let pattern_samples = pattern_cache.samples_for(
        pattern,
        state.fill_color,
        context.transform,
        context.options,
    )?;
    if pattern_samples.is_empty() {
        return Ok(());
    }
    let inverse = context
        .transform
        .matrix
        .inverse()
        .ok_or_else(|| RasterError::new(RasterErrorKind::InvalidPageBox))?;
    let samples = u32::from(context.options.supersample);
    let sample_count = samples * samples;
    let dimensions = device.dimensions();
    let Some(bounds) =
        flattened_bounds(path).and_then(|bounds| device_pixel_bounds(bounds, dimensions, 0.0))
    else {
        return Ok(());
    };
    for y in bounds.min_y..bounds.max_y {
        for x in bounds.min_x..bounds.max_x {
            let mut covered = 0;
            for sample_y in 0..samples {
                for sample_x in 0..samples {
                    let point = sample_point(x, y, sample_x, sample_y, samples);
                    if point_in_active_clips(point, context.clips)
                        && point_in_path(point, path, rule)
                    {
                        covered += 1;
                    }
                }
            }
            if covered == 0 {
                continue;
            }
            let user = inverse.transform_point(f64::from(x) + 0.5, f64::from(y) + 0.5);
            let Some(source) = pattern_color_at(pattern, pattern_samples, user) else {
                continue;
            };
            blend_pixel(
                device,
                x,
                y,
                source,
                state.blend_mode,
                state.fill_alpha * f64::from(covered) / f64::from(sample_count),
            )?;
        }
    }
    Ok(())
}

fn validate_pattern_tile_budget(
    path: &FlattenedPath,
    pattern: &TilingPattern,
    options: PathRasterOptions,
) -> RasterResult<()> {
    let bounds =
        flattened_bounds(path).ok_or_else(|| RasterError::new(RasterErrorKind::InvalidPageBox))?;
    let columns = ((bounds.max_x - bounds.min_x) / pattern.x_step.abs())
        .ceil()
        .max(1.0) as usize;
    let rows = ((bounds.max_y - bounds.min_y) / pattern.y_step.abs())
        .ceil()
        .max(1.0) as usize;
    let tiles = columns
        .checked_mul(rows)
        .ok_or_else(|| RasterError::new(RasterErrorKind::BufferOverflow))?;
    if tiles > options.max_pattern_tiles {
        return Err(RasterError::new(RasterErrorKind::PatternTileOverflow {
            limit: options.max_pattern_tiles,
        }));
    }
    Ok(())
}

fn flattened_bounds(path: &FlattenedPath) -> Option<PathBounds> {
    let mut points = path.subpaths.iter().flat_map(|subpath| subpath.iter());
    let first = *points.next()?;
    let mut bounds = PathBounds {
        min_x: first.x,
        min_y: first.y,
        max_x: first.x,
        max_y: first.y,
    };
    for point in points {
        bounds.min_x = bounds.min_x.min(point.x);
        bounds.min_y = bounds.min_y.min(point.y);
        bounds.max_x = bounds.max_x.max(point.x);
        bounds.max_y = bounds.max_y.max(point.y);
    }
    Some(bounds)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelBounds {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

fn device_pixel_bounds(
    bounds: PathBounds,
    dimensions: RasterDimensions,
    padding: f64,
) -> Option<PixelBounds> {
    let min_x = (bounds.min_x - padding)
        .floor()
        .clamp(0.0, f64::from(dimensions.width)) as u32;
    let min_y = (bounds.min_y - padding)
        .floor()
        .clamp(0.0, f64::from(dimensions.height)) as u32;
    let max_x = (bounds.max_x + padding)
        .ceil()
        .min(f64::from(dimensions.width)) as u32;
    let max_y = (bounds.max_y + padding)
        .ceil()
        .min(f64::from(dimensions.height)) as u32;
    (min_x < max_x && min_y < max_y).then_some(PixelBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

fn stroke_pixel_bounds(
    lines: &[LineSegment],
    joins: &[StrokeJoin],
    radius: f64,
    dimensions: RasterDimensions,
) -> Option<PixelBounds> {
    let mut bounds = None;
    for line in lines {
        bounds = Some(include_point(bounds, line.from));
        bounds = Some(include_point(bounds, line.to));
    }
    for join in joins {
        bounds = Some(include_point(bounds, join.point));
    }
    bounds.and_then(|bounds| device_pixel_bounds(bounds, dimensions, radius.ceil() + 1.0))
}

fn build_pattern_samples(
    pattern: &TilingPattern,
    fill_color: DeviceColor,
    options: PathRasterOptions,
) -> RasterResult<Vec<PatternSample>> {
    let mut samples = Vec::new();
    for item in pattern.items.items() {
        let DisplayItem::Path(path) = item else {
            continue;
        };
        let rule = match path.paint {
            PaintMode::Fill { rule } | PaintMode::FillStroke { rule } => rule,
            PaintMode::Stroke => continue,
        };
        samples.push(PatternSample {
            path: flatten_path_segments(
                &path.segments,
                Matrix::IDENTITY,
                options.max_flattened_segments,
            )?,
            rule,
            color: device_color_to_rgba(match pattern.paint {
                TilingPatternPaint::Colored => path.state.fill_color,
                TilingPatternPaint::Uncolored => fill_color,
            }),
        });
    }
    Ok(samples)
}

fn pattern_color_at(
    pattern: &TilingPattern,
    samples: &[PatternSample],
    user_point: Point,
) -> Option<Rgba> {
    let point = Point {
        x: wrap_pattern_coordinate(user_point.x, pattern.bbox.min_x, pattern.x_step),
        y: wrap_pattern_coordinate(user_point.y, pattern.bbox.min_y, pattern.y_step),
    };
    samples
        .iter()
        .rev()
        .find_map(|sample| point_in_path(point, &sample.path, sample.rule).then_some(sample.color))
}

fn wrap_pattern_coordinate(value: f64, origin: f64, step: f64) -> f64 {
    (value - origin).rem_euclid(step) + origin
}

fn stroke_path(
    device: &mut RasterDevice,
    path: &FlattenedPath,
    state: StrokeRasterState,
    context: PathRasterContext<'_>,
) -> RasterResult<()> {
    let source = device_color_to_rgba(state.color);
    let radius = stroke_radius_for_device_line_width(state.line_width);
    let dashed_lines;
    let (base_lines, base_joins): (&[LineSegment], &[StrokeJoin]) = if state.dash_pattern.is_solid()
    {
        (path.lines.as_slice(), path.joins.as_slice())
    } else {
        dashed_lines = dashed_subpath_line_segments(
            &path.subpaths,
            state.dash_pattern.scaled(state.dash_scale),
            context.options.max_flattened_segments,
        )?;
        (dashed_lines.as_slice(), &[])
    };
    let snapped_lines;
    let snapped_joins;
    let (stroke_lines, joins): (&[LineSegment], &[StrokeJoin]) =
        if should_snap_axis_aligned_hairline(state.line_width, state.ctm_scale) {
            let snap_mode = hairline_snap_mode(state.line_width, state.ctm_scale);
            snapped_lines = base_lines
                .iter()
                .copied()
                .map(|line| snap_axis_aligned_hairline_line(line, snap_mode))
                .collect::<Vec<_>>();
            snapped_joins = base_joins
                .iter()
                .copied()
                .map(|join| snap_axis_aligned_hairline_join(join, snap_mode))
                .collect::<Vec<_>>();
            (snapped_lines.as_slice(), snapped_joins.as_slice())
        } else {
            (base_lines, base_joins)
        };
    let samples = u32::from(context.options.supersample);
    let sample_count = samples * samples;
    let dimensions = device.dimensions();
    let Some(bounds) = stroke_pixel_bounds(stroke_lines, joins, radius, dimensions) else {
        return Ok(());
    };
    for y in bounds.min_y..bounds.max_y {
        for x in bounds.min_x..bounds.max_x {
            let mut covered = 0;
            for sample_y in 0..samples {
                for sample_x in 0..samples {
                    let point = sample_point(x, y, sample_x, sample_y, samples);
                    if point_in_active_clips(point, context.clips)
                        && (point_in_stroke(point, stroke_lines, radius, state.line_cap)
                            || point_in_join(
                                point,
                                joins,
                                radius,
                                state.line_join,
                                state.miter_limit,
                            ))
                    {
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
                    state.blend_mode,
                    state.alpha * f64::from(covered) / f64::from(sample_count),
                )?;
            }
        }
    }
    Ok(())
}

fn stroke_radius_for_device_line_width(line_width: f64) -> f64 {
    if line_width <= 1.0 {
        0.5
    } else {
        line_width / 2.0
    }
}

fn should_snap_axis_aligned_hairline(line_width: f64, ctm_scale: f64) -> bool {
    // Keep these bands narrow: wider 0.7-0.8px signature/legal linework
    // loses visible coverage if it is snapped away from its authored sample row.
    (0.25..=0.45).contains(&line_width)
        || (0.55..=0.65).contains(&line_width)
        || (ctm_scale > 1.0 + f64::EPSILON && (0.15..0.25).contains(&line_width))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HairlineSnapMode {
    NearestPixelCenter,
    RoundedDeviceCoordinate,
}

fn hairline_snap_mode(line_width: f64, ctm_scale: f64) -> HairlineSnapMode {
    if ctm_scale > 1.0 + f64::EPSILON && (0.15..0.25).contains(&line_width) {
        HairlineSnapMode::RoundedDeviceCoordinate
    } else {
        HairlineSnapMode::NearestPixelCenter
    }
}

fn snap_axis_aligned_hairline_line(line: LineSegment, mode: HairlineSnapMode) -> LineSegment {
    if (line.from.x - line.to.x).abs() <= f64::EPSILON {
        let x = snap_hairline_coordinate(line.from.x, mode);
        LineSegment {
            from: Point { x, ..line.from },
            to: Point { x, ..line.to },
        }
    } else if (line.from.y - line.to.y).abs() <= f64::EPSILON {
        let y = snap_hairline_coordinate(line.from.y, mode);
        LineSegment {
            from: Point { y, ..line.from },
            to: Point { y, ..line.to },
        }
    } else {
        line
    }
}

fn snap_axis_aligned_hairline_join(join: StrokeJoin, mode: HairlineSnapMode) -> StrokeJoin {
    let previous = snap_axis_aligned_hairline_line(join.previous, mode);
    let next = snap_axis_aligned_hairline_line(join.next, mode);
    StrokeJoin {
        previous,
        next,
        point: Point {
            x: snapped_join_coordinate(
                join.point.x,
                previous,
                next,
                |line| line.from.x,
                |line| line.to.x,
            ),
            y: snapped_join_coordinate(
                join.point.y,
                previous,
                next,
                |line| line.from.y,
                |line| line.to.y,
            ),
        },
    }
}

fn snapped_join_coordinate(
    original: f64,
    previous: LineSegment,
    next: LineSegment,
    from: impl Fn(LineSegment) -> f64,
    to: impl Fn(LineSegment) -> f64,
) -> f64 {
    for value in [from(previous), to(previous), from(next), to(next)] {
        if (value - original).abs() > f64::EPSILON && is_pixel_center(value) {
            return value;
        }
    }
    original
}

fn snap_hairline_coordinate(value: f64, mode: HairlineSnapMode) -> f64 {
    // Device coordinates can land just below an integer after CTM/page scaling;
    // keep those near-ties on the forward pixel center instead of the previous one.
    match mode {
        HairlineSnapMode::NearestPixelCenter => (value - 0.5 + 1e-9).round() + 0.5,
        HairlineSnapMode::RoundedDeviceCoordinate => (value + 1e-9).round() + 0.5,
    }
}

fn is_pixel_center(value: f64) -> bool {
    ((value - 0.5).fract()).abs() <= f64::EPSILON
}

fn dashed_subpath_line_segments(
    subpaths: &[Vec<Point>],
    pattern: StrokeDashPattern,
    limit: usize,
) -> RasterResult<Vec<LineSegment>> {
    let mut output = Vec::new();
    for subpath in subpaths {
        append_dashed_line_segments(
            subpath.windows(2).map(|points| LineSegment {
                from: points[0],
                to: points[1],
            }),
            pattern,
            limit,
            &mut output,
        )?;
    }
    Ok(output)
}

fn append_dashed_line_segments(
    lines: impl Iterator<Item = LineSegment>,
    pattern: StrokeDashPattern,
    limit: usize,
    output: &mut Vec<LineSegment>,
) -> RasterResult<()> {
    let len = pattern.active_len();
    let total: f64 = pattern.segments[..len].iter().sum();
    if total <= f64::EPSILON {
        return Ok(());
    }

    let mut distance = pattern.phase.rem_euclid(total);
    let mut pattern_index = 0;
    while distance >= pattern.segments[pattern_index] && pattern.segments[pattern_index] > 0.0 {
        distance -= pattern.segments[pattern_index];
        pattern_index = (pattern_index + 1) % len;
    }
    let mut draw = pattern_index % 2 == 0;
    let mut remaining_in_pattern = pattern.segments[pattern_index] - distance;

    for line in lines {
        let dx = line.to.x - line.from.x;
        let dy = line.to.y - line.from.y;
        let length = dx.hypot(dy);
        if length <= f64::EPSILON {
            continue;
        }
        let mut consumed = 0.0;
        while consumed < length {
            if remaining_in_pattern <= f64::EPSILON {
                pattern_index = (pattern_index + 1) % len;
                draw = pattern_index % 2 == 0;
                remaining_in_pattern = pattern.segments[pattern_index];
                continue;
            }
            let step = remaining_in_pattern.min(length - consumed);
            if draw && step > f64::EPSILON {
                if output.len() >= limit {
                    return Err(RasterError::new(RasterErrorKind::PathComplexityOverflow {
                        limit,
                    }));
                }
                let start = consumed / length;
                let end = (consumed + step) / length;
                output.push(LineSegment {
                    from: Point {
                        x: dx.mul_add(start, line.from.x),
                        y: dy.mul_add(start, line.from.y),
                    },
                    to: Point {
                        x: dx.mul_add(end, line.from.x),
                        y: dy.mul_add(end, line.from.y),
                    },
                });
            }
            consumed += step;
            remaining_in_pattern -= step;
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

fn clip_coverage_for_pixel(
    x: u32,
    y: u32,
    clips: &[ActiveClip],
    options: PathRasterOptions,
) -> f64 {
    if clips.is_empty() {
        return 1.0;
    }
    let samples = u32::from(options.supersample);
    let sample_count = samples * samples;
    let mut covered = 0;
    for sample_y in 0..samples {
        for sample_x in 0..samples {
            let point = sample_point(x, y, sample_x, sample_y, samples);
            if point_in_active_clips(point, clips) {
                covered += 1;
            }
        }
    }
    f64::from(covered) / f64::from(sample_count)
}

fn point_in_active_clips(point: Point, clips: &[ActiveClip]) -> bool {
    clips
        .iter()
        .all(|clip| point_in_path(point, &clip.path, clip.rule))
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

fn point_in_stroke(point: Point, lines: &[LineSegment], radius: f64, line_cap: LineCap) -> bool {
    let radius_squared = radius * radius;
    lines.iter().any(|line| match line_cap {
        LineCap::Butt => distance_to_line_body_squared(point, *line)
            .is_some_and(|distance| distance <= radius_squared),
        LineCap::Round => distance_to_line_segment_squared(point, *line) <= radius_squared,
        LineCap::Square => {
            distance_to_line_body_squared(point, square_capped_line_segment(*line, radius))
                .is_some_and(|distance| distance <= radius_squared)
        }
    })
}

fn point_in_join(
    point: Point,
    joins: &[StrokeJoin],
    radius: f64,
    line_join: LineJoin,
    miter_limit: f64,
) -> bool {
    let radius_squared = radius * radius;
    joins.iter().any(|join| match line_join {
        LineJoin::Round => distance_squared(point, join.point) <= radius_squared,
        LineJoin::Bevel => {
            point_in_join_side(point, *join, radius, LineJoin::Bevel, miter_limit, 1.0)
                || point_in_join_side(point, *join, radius, LineJoin::Bevel, miter_limit, -1.0)
        }
        LineJoin::Miter => {
            point_in_join_side(point, *join, radius, LineJoin::Miter, miter_limit, 1.0)
                || point_in_join_side(point, *join, radius, LineJoin::Miter, miter_limit, -1.0)
        }
    })
}

fn point_in_join_side(
    point: Point,
    join: StrokeJoin,
    radius: f64,
    line_join: LineJoin,
    miter_limit: f64,
    side: f64,
) -> bool {
    let Some(previous_direction) = unit_line_direction(join.previous) else {
        return false;
    };
    let Some(next_direction) = unit_line_direction(join.next) else {
        return false;
    };
    let previous_normal = signed_left_normal(previous_direction, side);
    let next_normal = signed_left_normal(next_direction, side);
    let previous_outer = Point {
        x: previous_normal.x.mul_add(radius, join.point.x),
        y: previous_normal.y.mul_add(radius, join.point.y),
    };
    let next_outer = Point {
        x: next_normal.x.mul_add(radius, join.point.x),
        y: next_normal.y.mul_add(radius, join.point.y),
    };

    if matches!(line_join, LineJoin::Miter) {
        let miter = line_intersection(
            previous_outer,
            previous_direction,
            next_outer,
            next_direction,
        );
        if let Some(miter) = miter {
            if distance_squared(join.point, miter) <= (radius * miter_limit).powi(2) {
                return point_in_triangle(point, previous_outer, miter, next_outer);
            }
        }
    }

    point_in_triangle(point, join.point, previous_outer, next_outer)
}

fn unit_line_direction(line: LineSegment) -> Option<Point> {
    let dx = line.to.x - line.from.x;
    let dy = line.to.y - line.from.y;
    let length = dx.hypot(dy);
    (length > f64::EPSILON).then_some(Point {
        x: dx / length,
        y: dy / length,
    })
}

fn signed_left_normal(direction: Point, side: f64) -> Point {
    Point {
        x: -direction.y * side,
        y: direction.x * side,
    }
}

fn line_intersection(
    point_a: Point,
    direction_a: Point,
    point_b: Point,
    direction_b: Point,
) -> Option<Point> {
    let denominator = cross(direction_a, direction_b);
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let delta = Point {
        x: point_b.x - point_a.x,
        y: point_b.y - point_a.y,
    };
    let t = cross(delta, direction_b) / denominator;
    Some(Point {
        x: direction_a.x.mul_add(t, point_a.x),
        y: direction_a.y.mul_add(t, point_a.y),
    })
}

fn point_in_triangle(point: Point, a: Point, b: Point, c: Point) -> bool {
    let area = cross(
        Point {
            x: b.x - a.x,
            y: b.y - a.y,
        },
        Point {
            x: c.x - a.x,
            y: c.y - a.y,
        },
    );
    if area.abs() <= f64::EPSILON {
        return false;
    }
    let ab = cross(
        Point {
            x: b.x - a.x,
            y: b.y - a.y,
        },
        Point {
            x: point.x - a.x,
            y: point.y - a.y,
        },
    );
    let bc = cross(
        Point {
            x: c.x - b.x,
            y: c.y - b.y,
        },
        Point {
            x: point.x - b.x,
            y: point.y - b.y,
        },
    );
    let ca = cross(
        Point {
            x: a.x - c.x,
            y: a.y - c.y,
        },
        Point {
            x: point.x - c.x,
            y: point.y - c.y,
        },
    );
    let has_negative = ab < 0.0 || bc < 0.0 || ca < 0.0;
    let has_positive = ab > 0.0 || bc > 0.0 || ca > 0.0;
    !(has_negative && has_positive)
}

fn cross(a: Point, b: Point) -> f64 {
    a.x.mul_add(b.y, -(a.y * b.x))
}

fn distance_squared(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx.mul_add(dx, dy * dy)
}

fn distance_to_line_body_squared(point: Point, line: LineSegment) -> Option<f64> {
    let dx = line.to.x - line.from.x;
    let dy = line.to.y - line.from.y;
    let len_squared = dx.mul_add(dx, dy * dy);
    if len_squared <= f64::EPSILON {
        return None;
    }
    let t = ((point.x - line.from.x) * dx + (point.y - line.from.y) * dy) / len_squared;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    let projection = Point {
        x: line.from.x + t * dx,
        y: line.from.y + t * dy,
    };
    let px = point.x - projection.x;
    let py = point.y - projection.y;
    Some(px.mul_add(px, py * py))
}

fn square_capped_line_segment(line: LineSegment, radius: f64) -> LineSegment {
    let dx = line.to.x - line.from.x;
    let dy = line.to.y - line.from.y;
    let length = dx.hypot(dy);
    if length <= f64::EPSILON {
        return line;
    }
    let offset_x = dx / length * radius;
    let offset_y = dy / length * radius;
    LineSegment {
        from: Point {
            x: line.from.x - offset_x,
            y: line.from.y - offset_y,
        },
        to: Point {
            x: line.to.x + offset_x,
            y: line.to.y + offset_y,
        },
    }
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
            let channel = normalized_color_to_u8(value);
            Rgba {
                r: channel,
                g: channel,
                b: channel,
                a: 255,
            }
        }
        DeviceColor::Rgb { r, g, b } => Rgba {
            r: normalized_color_to_u8(r),
            g: normalized_color_to_u8(g),
            b: normalized_color_to_u8(b),
            a: 255,
        },
        DeviceColor::Spot { r, g, b, .. } => Rgba {
            r: normalized_color_to_u8(r),
            g: normalized_color_to_u8(g),
            b: normalized_color_to_u8(b),
            a: 255,
        },
    }
}

fn normalized_color_to_u8(value: f64) -> u8 {
    let scaled = value.clamp(0.0, 1.0) * 255.0;
    // Poppler rounds bright exact half-step DeviceColor channels down; keep
    // darker and midpoint colors on the existing round-to-nearest path.
    if scaled > 127.5 {
        (scaled + 0.5 - 1e-9).floor() as u8
    } else {
        scaled.round() as u8
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
    blend_mode: BlendMode,
    coverage: f64,
) -> RasterResult<()> {
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return Ok(());
    }
    let dest = device.pixel(x, y)?;
    let blended = blend_source_with_backdrop(source, dest, blend_mode);
    device.set_pixel(
        x,
        y,
        source_over(
            Rgba {
                a: source.a,
                ..blended
            },
            dest,
            coverage,
        ),
    )
}

fn blend_source_with_backdrop(source: Rgba, dest: Rgba, blend_mode: BlendMode) -> Rgba {
    match blend_mode {
        BlendMode::Normal => source,
        BlendMode::Multiply => Rgba {
            r: multiply_channel(source.r, dest.r),
            g: multiply_channel(source.g, dest.g),
            b: multiply_channel(source.b, dest.b),
            a: source.a,
        },
        BlendMode::Screen => Rgba {
            r: screen_channel(source.r, dest.r),
            g: screen_channel(source.g, dest.g),
            b: screen_channel(source.b, dest.b),
            a: source.a,
        },
    }
}

fn multiply_channel(source: u8, dest: u8) -> u8 {
    ((u16::from(source) * u16::from(dest) + 127) / 255) as u8
}

fn screen_channel(source: u8, dest: u8) -> u8 {
    255u8.saturating_sub(multiply_channel(
        255u8.saturating_sub(source),
        255u8.saturating_sub(dest),
    ))
}

fn source_over(source: Rgba, dest: Rgba, coverage: f64) -> Rgba {
    let source_alpha = (f64::from(source.a) / 255.0 * coverage).clamp(0.0, 1.0);
    if source_alpha <= f64::EPSILON {
        return dest;
    }
    let dest_alpha = f64::from(dest.a) / 255.0;
    let out_alpha = source_alpha.mul_add(1.0, dest_alpha * (1.0 - source_alpha));
    if out_alpha <= f64::EPSILON {
        return Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
    }
    Rgba {
        r: source_over_channel(source.r, dest.r, source_alpha, dest_alpha, out_alpha),
        g: source_over_channel(source.g, dest.g, source_alpha, dest_alpha, out_alpha),
        b: source_over_channel(source.b, dest.b, source_alpha, dest_alpha, out_alpha),
        a: normalized_to_u8(out_alpha),
    }
}

fn source_over_channel(
    source: u8,
    dest: u8,
    source_alpha: f64,
    dest_alpha: f64,
    out_alpha: f64,
) -> u8 {
    // Poppler truncates source-over channel compositing after normalization.
    ((f64::from(source) * source_alpha + f64::from(dest) * dest_alpha * (1.0 - source_alpha))
        / out_alpha)
        .floor()
        .clamp(0.0, 255.0) as u8
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
    let mut sample_cache = ImageSampleCache::default();
    let axis_aligned = image_transform_is_axis_aligned(image_to_device);

    for y in min_y..max_y {
        for x in min_x..max_x {
            let coverage = if axis_aligned {
                axis_aligned_image_pixel_coverage(bounds, x, y)
            } else {
                1.0
            };
            if coverage <= f64::EPSILON {
                continue;
            }
            let sample = inverse.transform_point(f64::from(x) + 0.5, f64::from(y) + 0.5);
            if axis_aligned {
                let sample_x = sample.x.clamp(0.0, 1.0);
                let sample_y = sample.y.clamp(0.0, 1.0);
                let (sample_x, sample_y) = image_sample_coords(&image.image, sample_x, sample_y);
                let pixel =
                    sample_cache.sample(&image.image, sample_x, sample_y, image.state.fill_color);
                composite_image_pixel(device, x, y, pixel, coverage)?;
                continue;
            }
            if !(0.0..1.0).contains(&sample.x) || !(0.0..1.0).contains(&sample.y) {
                continue;
            }
            let (sample_x, sample_y) = image_sample_coords(&image.image, sample.x, sample.y);
            let pixel =
                sample_cache.sample(&image.image, sample_x, sample_y, image.state.fill_color);
            composite_image_pixel(device, x, y, pixel, coverage)?;
        }
    }
    Ok(())
}

fn image_transform_is_axis_aligned(transform: Matrix) -> bool {
    transform.b.abs() <= f64::EPSILON && transform.c.abs() <= f64::EPSILON
}

fn axis_aligned_image_pixel_coverage(bounds: PathBounds, x: u32, y: u32) -> f64 {
    let min_x = f64::from(x);
    let min_y = f64::from(y);
    let x_coverage = overlap_1d(min_x, min_x + 1.0, bounds.min_x, bounds.max_x);
    let y_coverage = overlap_1d(min_y, min_y + 1.0, bounds.min_y, bounds.max_y);
    x_coverage * y_coverage
}

fn overlap_1d(pixel_min: f64, pixel_max: f64, min: f64, max: f64) -> f64 {
    (pixel_max.min(max) - pixel_min.max(min)).clamp(0.0, 1.0)
}

fn transformed_image_bounds(transform: Matrix) -> PathBounds {
    transform_bounds(
        PathBounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        },
        transform,
    )
}

fn transform_bounds(bounds: PathBounds, transform: Matrix) -> PathBounds {
    let p0 = transform.transform_point(bounds.min_x, bounds.min_y);
    let p1 = transform.transform_point(bounds.max_x, bounds.min_y);
    let p2 = transform.transform_point(bounds.max_x, bounds.max_y);
    let p3 = transform.transform_point(bounds.min_x, bounds.max_y);
    let bounds = include_point(None, p0);
    let bounds = include_point(Some(bounds), p1);
    let bounds = include_point(Some(bounds), p2);
    include_point(Some(bounds), p3)
}

#[derive(Debug, Default)]
struct ImageSampleCache {
    last: Option<CachedImageSample>,
}

#[derive(Debug, Clone, Copy)]
struct CachedImageSample {
    x: u32,
    y: u32,
    color: Rgba,
}

impl ImageSampleCache {
    fn sample(
        &mut self,
        image: &ImageXObject,
        sample_x: u32,
        sample_y: u32,
        stencil_color: DeviceColor,
    ) -> Rgba {
        if let Some(last) = self.last {
            if last.x == sample_x && last.y == sample_y {
                return last.color;
            }
        }
        let color = sample_image_at(image, sample_x, sample_y, stencil_color);
        self.last = Some(CachedImageSample {
            x: sample_x,
            y: sample_y,
            color,
        });
        color
    }
}

fn image_sample_coords(image: &ImageXObject, x: f64, y: f64) -> (u32, u32) {
    let sample_x = ((x * f64::from(image.width)).floor() as u32).min(image.width - 1);
    let sample_y = (((1.0 - y) * f64::from(image.height)).floor() as u32).min(image.height - 1);
    (sample_x, sample_y)
}

fn sample_image_at(
    image: &ImageXObject,
    sample_x: u32,
    sample_y: u32,
    stencil_color: DeviceColor,
) -> Rgba {
    if let ImageKind::StencilMask { paint_one_bits } = image.kind {
        let paints = sample_image_mask_bit(image, sample_x, sample_y) == paint_one_bits;
        let mut color = device_color_to_rgba(stencil_color);
        if !paints {
            color.a = 0;
        }
        return color;
    }
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

fn sample_image_mask_bit(image: &ImageXObject, sample_x: u32, sample_y: u32) -> bool {
    let row_bytes = (image.width as usize).div_ceil(8);
    let byte_index = sample_y as usize * row_bytes + sample_x as usize / 8;
    let bit_index = 7 - (sample_x % 8);
    image
        .samples
        .get(byte_index)
        .is_some_and(|byte| (byte & (1 << bit_index)) != 0)
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
    coverage: f64,
) -> RasterResult<()> {
    if source.a == 255 && coverage >= 1.0 {
        return device.set_pixel(x, y, source);
    }
    if source.a == 0 || coverage <= f64::EPSILON {
        return Ok(());
    }
    let dest = device.pixel(x, y)?;
    device.set_pixel(x, y, source_over(source, dest, coverage))
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
    options: PathRasterOptions,
    glyph_cache: &mut GlyphBitmapCache,
    text_scratch: &mut TextRasterScratch,
) -> RasterResult<()> {
    if !text.rendering_mode.paints_pixels() {
        return Ok(());
    }
    let Some(color) = text
        .rendering_mode
        .paint_color(text.state)
        .map(device_color_to_rgba)
    else {
        return Ok(());
    };
    if text.font.type3.is_some() {
        return draw_type3_text_run(device, text, page_transform, options);
    }
    let fallback = text.font.fallback.unwrap_or(FontFallback {
        face: FontFallbackFace::Sans,
        source: FontFallbackSource::Unspecified,
    });
    let cell = scaled_fallback_text_cell(text.font_size, fallback, text.state);
    text_scratch.prepare(text, cell);
    for atom in &text_scratch.atoms {
        let TextRasterAtomKind::Glyph(character) = atom.kind else {
            draw_combining_mark(device, page_transform, atom.x, atom.baseline_y, cell, color)?;
            continue;
        };
        if character == ' ' || character == '\u{00a0}' {
            continue;
        }
        let bitmap = glyph_cache.bitmap_for(fallback, character, cell);
        draw_ascii_glyph(
            device,
            page_transform,
            bitmap,
            atom.x,
            atom.baseline_y,
            color,
        )?;
    }
    Ok(())
}

fn draw_type3_text_run(
    device: &mut RasterDevice,
    text: &TextDisplayItem,
    page_transform: PageTransform,
    options: PathRasterOptions,
) -> RasterResult<()> {
    let type3 = text.font.type3.as_ref().ok_or_else(|| {
        RasterError::new(RasterErrorKind::Type3Glyph {
            message: "missing Type3 font metadata".to_string(),
        })
    })?;
    let base = text.state.ctm.multiply(text.text_matrix);
    let mut pattern_cache = PatternCellCache::new(options.max_pattern_cell_cache_entries);
    for (glyph, origin) in text.glyphs.iter().zip(text.glyph_origins.iter()) {
        let Some(char_proc) = type3.char_proc_for_code(glyph.character_code, &text.font.encoding)
        else {
            continue;
        };
        let mut glyph_state = text.state;
        glyph_state.ctm = Matrix {
            e: origin.x,
            f: origin.y,
            ..base
        }
        .multiply(Matrix::scale(text.font_size, text.font_size))
        .multiply(type3.font_matrix);
        let mut interpreter = DisplayListInterpreter::new(DisplayListOptions::default());
        interpreter.current = glyph_state;
        interpreter
            .interpret(tokenize_content(PdfBytes::new(&char_proc.content)))
            .map_err(raster_type3_error)?;
        for item in interpreter.display_list.items() {
            let DisplayItem::Path(path) = item else {
                continue;
            };
            rasterize_path_item(
                path,
                device,
                PathRasterContext {
                    transform: page_transform,
                    options,
                    clips: &[],
                },
                &mut pattern_cache,
            )?;
        }
    }
    Ok(())
}

fn raster_type3_error(error: GraphicsError) -> RasterError {
    RasterError::new(RasterErrorKind::Type3Glyph {
        message: error.to_string(),
    })
}

fn draw_ascii_glyph(
    device: &mut RasterDevice,
    page_transform: PageTransform,
    bitmap: &GlyphBitmap,
    x: f64,
    baseline_y: f64,
    color: Rgba,
) -> RasterResult<()> {
    for rect in &bitmap.rects {
        fill_device_rect(
            device,
            page_transform
                .matrix
                .transform_point(x + rect.left, baseline_y + rect.top),
            page_transform
                .matrix
                .transform_point(x + rect.right, baseline_y + rect.bottom),
            color,
        )?;
    }
    Ok(())
}

fn draw_combining_mark(
    device: &mut RasterDevice,
    page_transform: PageTransform,
    x: f64,
    baseline_y: f64,
    cell: f64,
    color: Rgba,
) -> RasterResult<()> {
    let mark_left = x + cell;
    let mark_right = x + cell * 4.0;
    let mark_top = baseline_y + cell * 8.0;
    let mark_bottom = baseline_y + cell * 7.0;
    fill_device_rect(
        device,
        page_transform.matrix.transform_point(mark_left, mark_top),
        page_transform
            .matrix
            .transform_point(mark_right, mark_bottom),
        color,
    )
}

fn fill_device_rect(
    device: &mut RasterDevice,
    p0: Point,
    p1: Point,
    color: Rgba,
) -> RasterResult<()> {
    let dimensions = device.dimensions();
    let left = p0.x.min(p1.x);
    let right = p0.x.max(p1.x);
    let top = p0.y.min(p1.y);
    let bottom = p0.y.max(p1.y);
    let min_x = left.floor().max(0.0) as u32;
    let max_x = right.ceil().min(f64::from(dimensions.width)) as u32;
    let min_y = top.floor().max(0.0) as u32;
    let max_y = bottom.ceil().min(f64::from(dimensions.height)) as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let coverage = rect_pixel_coverage(left, right, top, bottom, x, y);
            if coverage >= 1.0 - f64::EPSILON && color.a == 255 {
                device.set_pixel(x, y, color)?;
            } else if coverage > f64::EPSILON {
                let dest = device.pixel(x, y)?;
                device.set_pixel(x, y, source_over(color, dest, coverage))?;
            }
        }
    }
    Ok(())
}

fn rect_pixel_coverage(left: f64, right: f64, top: f64, bottom: f64, x: u32, y: u32) -> f64 {
    let pixel_left = f64::from(x);
    let pixel_right = pixel_left + 1.0;
    let pixel_top = f64::from(y);
    let pixel_bottom = pixel_top + 1.0;
    let overlap_x = right.min(pixel_right) - left.max(pixel_left);
    let overlap_y = bottom.min(pixel_bottom) - top.max(pixel_top);
    (overlap_x.max(0.0) * overlap_y.max(0.0)).clamp(0.0, 1.0)
}

fn ascii_glyph(character: char) -> [&'static str; 7] {
    if character.is_ascii_lowercase() {
        ascii_lowercase_glyph(character)
    } else {
        ascii_uppercase_glyph(character.to_ascii_uppercase())
    }
}

fn ascii_uppercase_glyph(character: char) -> [&'static str; 7] {
    match character {
        ' ' => [
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ],
        '!' => [
            "  #  ", "  #  ", "  #  ", "  #  ", "     ", "  #  ", "  #  ",
        ],
        '"' => [
            "# #  ", "# #  ", "# #  ", "     ", "     ", "     ", "     ",
        ],
        '#' => [
            " # # ", " # # ", "#####", " # # ", "#####", " # # ", " # # ",
        ],
        '$' => [
            " ### ", "# #  ", "#    ", " ### ", "   # ", "#  # ", "###  ",
        ],
        '%' => [
            "##  #", "## # ", "  #  ", " #   ", "# ## ", "#  ##", "     ",
        ],
        '&' => [
            " ##  ", "#  # ", "# #  ", " #   ", "# # #", "#  # ", " ## #",
        ],
        '\'' => [
            "  #  ", "  #  ", " #   ", "     ", "     ", "     ", "     ",
        ],
        '(' => [
            "  ## ", " #   ", "#    ", "#    ", "#    ", " #   ", "  ## ",
        ],
        ')' => [
            " ##  ", "   # ", "    #", "    #", "    #", "   # ", " ##  ",
        ],
        '*' => [
            "     ", "# # #", " ### ", "#####", " ### ", "# # #", "     ",
        ],
        '+' => [
            "     ", "  #  ", "  #  ", "#####", "  #  ", "  #  ", "     ",
        ],
        ',' => [
            "     ", "     ", "     ", "     ", "     ", " ##  ", " #   ",
        ],
        '-' => [
            "     ", "     ", "     ", "#####", "     ", "     ", "     ",
        ],
        '.' => [
            "     ", "     ", "     ", "     ", "     ", " ##  ", " ##  ",
        ],
        '/' => [
            "    #", "   # ", "   # ", "  #  ", " #   ", " #   ", "#    ",
        ],
        ':' => [
            "     ", " ##  ", " ##  ", "     ", " ##  ", " ##  ", "     ",
        ],
        ';' => [
            "     ", " ##  ", " ##  ", "     ", " ##  ", " ##  ", " #   ",
        ],
        '<' => [
            "   # ", "  #  ", " #   ", "#    ", " #   ", "  #  ", "   # ",
        ],
        '=' => [
            "     ", "     ", "#####", "     ", "#####", "     ", "     ",
        ],
        '>' => [
            " #   ", "  #  ", "   # ", "    #", "   # ", "  #  ", " #   ",
        ],
        '?' => [
            "#####", "#   #", "   # ", "  #  ", "     ", "  #  ", "  #  ",
        ],
        '@' => [
            " ### ", "#   #", "# ###", "# # #", "# ###", "#    ", " ### ",
        ],
        'A' => [
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'B' => [
            "#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### ",
        ],
        'C' => [
            " ####", "#    ", "#    ", "#    ", "#    ", "#    ", " ####",
        ],
        'D' => [
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ],
        'E' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####",
        ],
        'F' => [
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#    ",
        ],
        'G' => [
            " ####", "#    ", "#    ", "#  ##", "#   #", "#   #", " ####",
        ],
        'H' => [
            "#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'I' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####",
        ],
        'J' => [
            "#####", "    #", "    #", "    #", "#   #", "#   #", " ### ",
        ],
        'K' => [
            "#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #",
        ],
        'L' => [
            "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
        ],
        'M' => [
            "#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #",
        ],
        'N' => [
            "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #",
        ],
        'O' => [
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'P' => [
            "#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    ",
        ],
        'Q' => [
            " ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #",
        ],
        'R' => [
            "#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #",
        ],
        'S' => [
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ],
        'T' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'U' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'V' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", " # # ", "  #  ",
        ],
        'W' => [
            "#   #", "#   #", "#   #", "# # #", "# # #", "## ##", "#   #",
        ],
        'X' => [
            "#   #", "#   #", " # # ", "  #  ", " # # ", "#   #", "#   #",
        ],
        'Y' => [
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'Z' => [
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
        '[' => [
            " ### ", " #   ", " #   ", " #   ", " #   ", " #   ", " ### ",
        ],
        '\\' => [
            "#    ", " #   ", " #   ", "  #  ", "   # ", "   # ", "    #",
        ],
        ']' => [
            " ### ", "   # ", "   # ", "   # ", "   # ", "   # ", " ### ",
        ],
        '^' => [
            "  #  ", " # # ", "#   #", "     ", "     ", "     ", "     ",
        ],
        '_' => [
            "     ", "     ", "     ", "     ", "     ", "     ", "#####",
        ],
        '`' => [
            " #   ", "  #  ", "   # ", "     ", "     ", "     ", "     ",
        ],
        '{' => [
            "   ##", "  #  ", "  #  ", "##   ", "  #  ", "  #  ", "   ##",
        ],
        '|' => [
            "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        '}' => [
            "##   ", "  #  ", "  #  ", "   ##", "  #  ", "  #  ", "##   ",
        ],
        '~' => [
            "     ", "     ", " ## #", "# ## ", "     ", "     ", "     ",
        ],
        _ => [
            "#####", "#   #", "   # ", "  #  ", "     ", "  #  ", "  #  ",
        ],
    }
}

fn ascii_lowercase_glyph(character: char) -> [&'static str; 7] {
    match character {
        'a' => [
            "     ", "     ", " ### ", "    #", " ####", "#   #", " ####",
        ],
        'b' => [
            "#    ", "#    ", "#### ", "#   #", "#   #", "#   #", "#### ",
        ],
        'c' => [
            "     ", "     ", " ####", "#    ", "#    ", "#    ", " ####",
        ],
        'd' => [
            "    #", "    #", " ####", "#   #", "#   #", "#   #", " ####",
        ],
        'e' => [
            "     ", "     ", " ### ", "#   #", "#####", "#    ", " ### ",
        ],
        'f' => [
            "  ## ", " #   ", " #   ", "###  ", " #   ", " #   ", " #   ",
        ],
        'g' => [
            "     ", "     ", " ####", "#   #", "#   #", " ####", "    #",
        ],
        'h' => [
            "#    ", "#    ", "#### ", "#   #", "#   #", "#   #", "#   #",
        ],
        'i' => [
            "  #  ", "     ", " ##  ", "  #  ", "  #  ", "  #  ", " ### ",
        ],
        'j' => [
            "   # ", "     ", "  ## ", "   # ", "   # ", "#  # ", " ##  ",
        ],
        'k' => [
            "#    ", "#    ", "#  # ", "# #  ", "##   ", "# #  ", "#  # ",
        ],
        'l' => [
            " ##  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### ",
        ],
        'm' => [
            "     ", "     ", "## # ", "# # #", "# # #", "# # #", "# # #",
        ],
        'n' => [
            "     ", "     ", "#### ", "#   #", "#   #", "#   #", "#   #",
        ],
        'o' => [
            "     ", "     ", " ### ", "#   #", "#   #", "#   #", " ### ",
        ],
        'p' => [
            "     ", "     ", "#### ", "#   #", "#   #", "#### ", "#    ",
        ],
        'q' => [
            "     ", "     ", " ####", "#   #", "#   #", " ####", "    #",
        ],
        'r' => [
            "     ", "     ", "# ## ", "##   ", "#    ", "#    ", "#    ",
        ],
        's' => [
            "     ", "     ", " ####", "#    ", " ### ", "    #", "#### ",
        ],
        't' => [
            " #   ", " #   ", "###  ", " #   ", " #   ", " #   ", "  ## ",
        ],
        'u' => [
            "     ", "     ", "#   #", "#   #", "#   #", "#   #", " ####",
        ],
        'v' => [
            "     ", "     ", "#   #", "#   #", "#   #", " # # ", "  #  ",
        ],
        'w' => [
            "     ", "     ", "#   #", "# # #", "# # #", "# # #", " # # ",
        ],
        'x' => [
            "     ", "     ", "#   #", " # # ", "  #  ", " # # ", "#   #",
        ],
        'y' => [
            "     ", "     ", "#   #", "#   #", "#   #", " ####", "    #",
        ],
        'z' => [
            "     ", "     ", "#####", "   # ", "  #  ", " #   ", "#####",
        ],
        _ => ascii_uppercase_glyph(character.to_ascii_uppercase()),
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
        PdfPrimitive::Name(name) if is_flate_image_filter(name.as_bytes()) => {
            Ok(ImageFilter::StreamDecoded)
        }
        PdfPrimitive::Name(name) if is_dct_image_filter(name.as_bytes()) => {
            Ok(ImageFilter::DctDecode)
        }
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
                    if is_flate_image_filter(name.as_bytes()) {
                        return Ok(ImageFilter::StreamDecoded);
                    }
                    if is_dct_image_filter(name.as_bytes()) {
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

fn is_flate_image_filter(filter: &[u8]) -> bool {
    matches!(filter, b"FlateDecode" | b"Fl")
}

fn is_dct_image_filter(filter: &[u8]) -> bool {
    matches!(filter, b"DCTDecode" | b"DCT")
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

fn image_color_space_with_icc<'a, R>(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    resolver: &'a R,
    max_icc_profile_bytes: usize,
    max_icc_transform_workspace_bytes: usize,
    icc_cache: &mut IccTransformCache,
) -> GraphicsResult<ImageColorSpaceInfo>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
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
        PdfPrimitive::Array(values) => array_color_space_with_icc(
            values,
            resolver,
            max_icc_profile_bytes,
            max_icc_transform_workspace_bytes,
            icc_cache,
        ),
        _ => image_color_space(dictionary),
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

fn array_color_space_with_icc<'a, R>(
    values: &[PdfPrimitive<'_>],
    resolver: &'a R,
    max_icc_profile_bytes: usize,
    max_icc_transform_workspace_bytes: usize,
    icc_cache: &mut IccTransformCache,
) -> GraphicsResult<ImageColorSpaceInfo>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    let Some(PdfPrimitive::Name(kind)) = values.first() else {
        return Err(unsupported_color_space(b"array"));
    };
    match kind.as_bytes() {
        b"Indexed" | b"I" => indexed_color_space(values),
        b"CalRGB" => Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceRgb)),
        b"CalGray" => Ok(ImageColorSpaceInfo::new(ImageColorSpace::DeviceGray)),
        b"ICCBased" => icc_based_color_space(
            values,
            resolver,
            max_icc_profile_bytes,
            max_icc_transform_workspace_bytes,
            icc_cache,
        ),
        other => Err(unsupported_color_space(other)),
    }
}

fn icc_based_color_space<'a, R>(
    values: &[PdfPrimitive<'_>],
    resolver: &'a R,
    max_icc_profile_bytes: usize,
    max_icc_transform_workspace_bytes: usize,
    icc_cache: &mut IccTransformCache,
) -> GraphicsResult<ImageColorSpaceInfo>
where
    R: ImageObjectResolver<'a> + ?Sized,
{
    if values.len() != 2 {
        return Err(unsupported_color_space(b"ICCBased"));
    }
    let reference =
        reference_from_primitive(&values[1]).ok_or_else(|| unsupported_color_space(b"ICCBased"))?;
    let object = resolver.resolve_image_object(reference)?.ok_or_else(|| {
        GraphicsError::new(
            None,
            GraphicsErrorKind::MissingImageObject {
                name: b"ICCBased".to_vec(),
            },
        )
    })?;
    let ObjectValue::Stream(stream) = &object.value else {
        return Err(unsupported_color_space(b"ICCBased"));
    };
    let transform = decode_icc_transform(
        stream,
        max_icc_profile_bytes,
        max_icc_transform_workspace_bytes,
    )?;
    Ok(ImageColorSpaceInfo::new(
        icc_cache.get_or_insert(transform).color_space,
    ))
}

fn decode_icc_transform(
    stream: &StreamObject<'_>,
    max_icc_profile_bytes: usize,
    max_icc_transform_workspace_bytes: usize,
) -> GraphicsResult<IccTransform> {
    let components = required_u32(stream.dictionary(), b"N")
        .ok()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid_image_resource(b"ICCBased"))?;
    let color_space = match components {
        1 => ImageColorSpace::DeviceGray,
        3 => ImageColorSpace::DeviceRgb,
        4 => ImageColorSpace::DeviceCmyk,
        _ => return Err(unsupported_color_space(b"ICCBased-N")),
    };
    let profile = stream
        .decode_with_options(StreamDecodeOptions {
            max_decoded_len: max_icc_profile_bytes,
        })
        .map_err(|error| match error {
            pdfrust_object::ObjectError::StreamLimitExceeded { .. } => GraphicsError::new(
                None,
                GraphicsErrorKind::ImageResourceBytesOverflow {
                    limit: max_icc_profile_bytes,
                },
            ),
            _ => GraphicsError::new(
                None,
                GraphicsErrorKind::ObjectModel {
                    message: error.to_string(),
                },
            ),
        })?;
    let workspace_bytes = components
        .checked_mul(256)
        .and_then(|value| value.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| {
            GraphicsError::new(
                None,
                GraphicsErrorKind::ImageResourceBytesOverflow {
                    limit: max_icc_transform_workspace_bytes,
                },
            )
        })?;
    if workspace_bytes > max_icc_transform_workspace_bytes {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::ImageResourceBytesOverflow {
                limit: max_icc_transform_workspace_bytes,
            },
        ));
    }
    Ok(IccTransform {
        identity: IccProfileIdentity {
            hash: stable_icc_profile_hash(&profile),
            len: profile.len(),
            components,
        },
        color_space,
        workspace_bytes,
    })
}

fn stable_icc_profile_hash(profile: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in profile {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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

fn optional_i32(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> GraphicsResult<Option<i32>> {
    let Some(value) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => Ok(Some(
            i32::try_from(*value).map_err(|_| invalid_font_resource(key))?,
        )),
        _ => Err(invalid_font_resource(key)),
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
                b"WinAnsiEncoding"
                    | b"MacRomanEncoding"
                    | b"MacExpertEncoding"
                    | b"Identity-H"
                    | b"Identity-V"
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

fn font_writing_mode(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> TextWritingMode {
    match dictionary_value(dictionary, b"Encoding") {
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"Identity-V" => {
            TextWritingMode::Vertical
        }
        _ => TextWritingMode::Horizontal,
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
                differences.push(FontEncodingDifference {
                    code: current_code,
                    name: name.as_bytes().to_vec(),
                    character,
                });
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
    let mut code_space_ranges = Vec::new();
    let mut mode = CMapSection::None;
    for raw_line in bytes.split(|byte| matches!(byte, b'\n' | b'\r')) {
        let line = trim_cmap_comment(raw_line);
        if line.is_empty() {
            continue;
        }
        if contains_word(line, b"begincodespacerange") {
            mode = CMapSection::CodeSpaceRange;
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
        if contains_word(line, b"endcodespacerange")
            || contains_word(line, b"endbfchar")
            || contains_word(line, b"endbfrange")
        {
            mode = CMapSection::None;
            continue;
        }
        match mode {
            CMapSection::None => {
                if contains_word(line, b"usecmap") && !is_identity_usecmap(line) {
                    return Err(GraphicsError::new(
                        None,
                        GraphicsErrorKind::UnsupportedCMap {
                            feature: b"usecmap".to_vec(),
                        },
                    ));
                }
            }
            CMapSection::CodeSpaceRange => {
                let hex_values = collect_hex_strings(line)?;
                for pair in hex_values.chunks_exact(2) {
                    push_cmap_code_space_range(
                        &mut code_space_ranges,
                        &pair[0],
                        &pair[1],
                        entry_limit,
                    )?;
                }
                if hex_values.len() % 2 != 0 {
                    return Err(invalid_cmap());
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
    Ok(ToUnicodeMap::new(entries, code_space_ranges))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CMapSection {
    None,
    CodeSpaceRange,
    BfChar,
    BfRange,
}

fn push_cmap_code_space_range(
    ranges: &mut Vec<CodeSpaceRange>,
    start: &[u8],
    end: &[u8],
    entry_limit: usize,
) -> GraphicsResult<()> {
    if ranges.len() >= entry_limit {
        return Err(GraphicsError::new(
            None,
            GraphicsErrorKind::CMapEntriesOverflow { limit: entry_limit },
        ));
    }
    if start.is_empty() || start.len() != end.len() || bytes_to_u32(end) < bytes_to_u32(start) {
        return Err(invalid_cmap());
    }
    ranges.push(CodeSpaceRange {
        start: start.to_vec(),
        end: end.to_vec(),
    });
    Ok(())
}

fn is_identity_usecmap(line: &[u8]) -> bool {
    contains_word(line, b"Identity-H") || contains_word(line, b"Identity-V")
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

fn ensure_page_raster_pixel_budget(
    dimensions: RasterDimensions,
    max_page_pixels: usize,
) -> RasterResult<()> {
    let pixels = (dimensions.width as usize)
        .checked_mul(dimensions.height as usize)
        .ok_or_else(|| RasterError::new(RasterErrorKind::BufferOverflow))?;
    if pixels > max_page_pixels {
        return Err(RasterError::new(
            RasterErrorKind::PageRasterPixelsOverflow {
                limit: max_page_pixels,
            },
        ));
    }
    Ok(())
}

fn page_to_pixel_matrix(
    bounds: PathBounds,
    rotation: PageRotation,
    scale: f64,
    dimensions: RasterDimensions,
) -> Matrix {
    match rotation {
        PageRotation::Deg0 => Matrix::new(
            scale,
            0.0,
            0.0,
            -scale,
            -bounds.min_x * scale,
            (f64::from(dimensions.height) + bounds.min_y * scale).max(bounds.max_y * scale),
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

fn invalid_ext_graphics_state(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidExtGraphicsState {
            name: name.to_vec(),
        },
    )
}

fn invalid_shading_resource(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidShadingResource {
            name: name.to_vec(),
        },
    )
}

fn unsupported_shading(feature: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedShading {
            feature: feature.to_vec(),
        },
    )
}

fn invalid_color_space_resource(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidColorSpaceResource {
            name: name.to_vec(),
        },
    )
}

fn unsupported_spot_color_space(feature: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedColorSpace {
            feature: feature.to_vec(),
        },
    )
}

fn invalid_pattern_resource(name: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::InvalidPatternResource {
            name: name.to_vec(),
        },
    )
}

fn unsupported_pattern(feature: &[u8]) -> GraphicsError {
    GraphicsError::new(
        None,
        GraphicsErrorKind::UnsupportedPattern {
            feature: feature.to_vec(),
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
    /// Page raster pixel count exceeded the configured limit.
    PageRasterPixelsOverflow {
        /// Configured page-raster pixel limit.
        limit: usize,
    },
    /// Flattened path complexity exceeded the configured limit.
    PathComplexityOverflow {
        /// Configured flattened line segment limit.
        limit: usize,
    },
    /// Transparency group intermediate raster exceeds the configured limit.
    TransparencyGroupPixelsOverflow {
        /// Configured transparency group pixel limit.
        limit: usize,
    },
    /// Tiling pattern expansion exceeds the configured repeat limit.
    PatternTileOverflow {
        /// Configured pattern tile limit.
        limit: usize,
    },
    /// Image transform could not be inverted for sampling.
    SingularImageTransform,
    /// Type 3 glyph CharProc rendering failed.
    Type3Glyph {
        /// Underlying deterministic renderer error.
        message: String,
    },
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
            Self::PageRasterPixelsOverflow { limit } => {
                write!(f, "page raster exceeds pixel limit {limit}")
            }
            Self::PathComplexityOverflow { limit } => {
                write!(f, "flattened path exceeds segment limit {limit}")
            }
            Self::TransparencyGroupPixelsOverflow { limit } => {
                write!(f, "transparency group exceeds pixel limit {limit}")
            }
            Self::PatternTileOverflow { limit } => {
                write!(f, "tiling pattern exceeds tile limit {limit}")
            }
            Self::SingularImageTransform => f.write_str("image transform is singular"),
            Self::Type3Glyph { message } => write!(f, "Type3 glyph render error: {message}"),
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
    /// Charstring operand stack exceeds the configured limit.
    GlyphOutlineStackOverflow {
        /// Configured charstring stack limit.
        limit: usize,
    },
    /// Charstring subroutine recursion would exceed the configured limit.
    GlyphOutlineSubroutineOverflow {
        /// Configured charstring subroutine recursion limit.
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
    /// Decoded image resources exceed the configured page resource limit.
    ImageResourceBytesOverflow {
        /// Configured decoded image resource byte limit.
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
    /// Transparency group metadata uses a feature outside the current support.
    UnsupportedTransparencyGroup {
        /// Unsupported transparency-group feature name.
        feature: Vec<u8>,
    },
    /// External graphics state blend mode is unsupported.
    UnsupportedBlendMode {
        /// Unsupported blend mode name.
        mode: Vec<u8>,
    },
    /// External graphics state overprint policy is unsupported.
    UnsupportedOverprint {
        /// Unsupported overprint feature name.
        feature: Vec<u8>,
    },
    /// Color-management feature is unsupported.
    UnsupportedColorManagement {
        /// Unsupported color-management feature name.
        feature: Vec<u8>,
    },
    /// Stroke dash pattern exceeds the fixed graphics-state representation.
    UnsupportedDashPattern {
        /// Configured dash segment limit.
        limit: usize,
    },
    /// External graphics state resource name was not present in the resource map.
    MissingExtGraphicsState {
        /// Missing external graphics state resource name.
        name: Vec<u8>,
    },
    /// External graphics state resource is malformed.
    InvalidExtGraphicsState {
        /// Invalid external graphics state resource name.
        name: Vec<u8>,
    },
    /// Shading resource name was not present in the resource map.
    MissingShading {
        /// Missing shading resource name.
        name: Vec<u8>,
    },
    /// Shading resource dictionary is malformed.
    InvalidShadingResource {
        /// Invalid shading resource or field name.
        name: Vec<u8>,
    },
    /// Shading resource uses a feature outside the current support.
    UnsupportedShading {
        /// Unsupported shading feature.
        feature: Vec<u8>,
    },
    /// Decoded shading stream data exceeds the configured byte limit.
    ShadingBytesOverflow {
        /// Configured decoded shading byte limit.
        limit: usize,
    },
    /// Decoded mesh shading exceeds the configured triangle limit.
    ShadingTriangleOverflow {
        /// Configured decoded mesh triangle limit.
        limit: usize,
    },
    /// Color-space resource dictionary is malformed.
    InvalidColorSpaceResource {
        /// Invalid color-space resource or field name.
        name: Vec<u8>,
    },
    /// Color-space resource uses a feature outside the current support.
    UnsupportedColorSpace {
        /// Unsupported color-space feature.
        feature: Vec<u8>,
    },
    /// Pattern resource name was not present in the resource map.
    MissingPattern {
        /// Missing pattern resource name.
        name: Vec<u8>,
    },
    /// Pattern resource dictionary is malformed.
    InvalidPatternResource {
        /// Invalid pattern resource or field name.
        name: Vec<u8>,
    },
    /// Pattern resource uses a feature outside the current support.
    UnsupportedPattern {
        /// Unsupported pattern feature.
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
            Self::GlyphOutlineStackOverflow { limit } => {
                write!(f, "glyph outline charstring stack exceeds limit {limit}")
            }
            Self::GlyphOutlineSubroutineOverflow { limit } => {
                write!(
                    f,
                    "glyph outline charstring subroutine exceeds limit {limit}"
                )
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
            Self::ImageResourceBytesOverflow { limit } => {
                write!(f, "decoded image resources exceed byte limit {limit}")
            }
            Self::SoftMaskDepthOverflow { limit } => {
                write!(f, "soft-mask recursion exceeds depth limit {limit}")
            }
            Self::UnsupportedSoftMask { feature } => write!(
                f,
                "unsupported soft-mask feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::UnsupportedTransparencyGroup { feature } => write!(
                f,
                "unsupported transparency-group feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::UnsupportedBlendMode { mode } => {
                write!(
                    f,
                    "unsupported blend mode {}",
                    String::from_utf8_lossy(mode)
                )
            }
            Self::UnsupportedOverprint { feature } => write!(
                f,
                "unsupported overprint feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::UnsupportedColorManagement { feature } => write!(
                f,
                "unsupported color-management feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::UnsupportedDashPattern { limit } => {
                write!(f, "stroke dash pattern exceeds segment limit {limit}")
            }
            Self::MissingExtGraphicsState { name } => write!(
                f,
                "missing external graphics state resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidExtGraphicsState { name } => write!(
                f,
                "invalid external graphics state resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::MissingShading { name } => write!(
                f,
                "missing shading resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidShadingResource { name } => write!(
                f,
                "invalid shading resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::UnsupportedShading { feature } => write!(
                f,
                "unsupported shading feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::ShadingBytesOverflow { limit } => {
                write!(f, "decoded shading stream exceeds byte limit {limit}")
            }
            Self::ShadingTriangleOverflow { limit } => {
                write!(f, "mesh shading exceeds triangle limit {limit}")
            }
            Self::InvalidColorSpaceResource { name } => write!(
                f,
                "invalid color-space resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::UnsupportedColorSpace { feature } => write!(
                f,
                "unsupported color-space feature {}",
                String::from_utf8_lossy(feature)
            ),
            Self::MissingPattern { name } => write!(
                f,
                "missing pattern resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidPatternResource { name } => write!(
                f,
                "invalid pattern resource {}",
                String::from_utf8_lossy(name)
            ),
            Self::UnsupportedPattern { feature } => write!(
                f,
                "unsupported pattern feature {}",
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
    fn page_transform_should_extend_translation_only_for_rounded_up_height() {
        let rounded_up = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 360.0,
                    max_y: 240.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            160,
        )
        .expect("valid rounded-up page transform");
        let rounded_down = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 380.0,
                    max_y: 260.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            160,
        )
        .expect("valid rounded-down page transform");

        assert_eq!(rounded_up.dimensions.height, 107);
        assert_eq!(rounded_up.matrix.transform_point(0.0, 0.0).y, 107.0);
        assert_eq!(rounded_down.dimensions.height, 109);
        assert!((rounded_down.matrix.transform_point(0.0, 0.0).y - 109.473_684).abs() < 0.000_001);
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
    fn page_transform_should_enforce_page_raster_pixel_budget() {
        let error = PageTransform::new_with_options(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 100.0,
                    max_y: 100.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            100,
            PageTransformOptions {
                max_page_pixels: 99,
            },
        )
        .expect_err("page raster budget should fail before allocation");

        assert_eq!(
            error.kind(),
            &RasterErrorKind::PageRasterPixelsOverflow { limit: 99 }
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
                r: 229,
                g: 51,
                b: 26,
                a: 255,
            }
        );
    }

    #[test]
    fn path_rasterizer_should_draw_generated_vector_stress_fixture() {
        let decoded = generated_fixture_content("vector-stress.pdf");
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            DisplayListOptions::default(),
        )
        .expect("vector stress fixture display list");
        assert!(list.len() > 40);
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 160.0,
                    max_y: 120.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            160,
        )
        .expect("vector stress fixture transform");
        let raster = rasterize_paths(&list, transform, Rgba::WHITE, PathRasterOptions::default())
            .expect("vector stress fixture should rasterize");

        assert_eq!(raster.dimensions().width, 160);
        assert_eq!(raster.dimensions().height, 120);
        assert!(
            raster
                .pixels()
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count()
                > 8_000
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
    fn path_rasterizer_should_enforce_vector_stress_segment_budget() {
        let decoded = generated_fixture_content("vector-stress.pdf");
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(&decoded)),
            DisplayListOptions::default(),
        )
        .expect("vector stress fixture display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 160.0,
                    max_y: 120.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            160,
        )
        .expect("vector stress fixture transform");
        let error = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                max_flattened_segments: 12,
                ..PathRasterOptions::default()
            },
        )
        .expect_err("stress curve should exceed configured segment limit");

        assert_eq!(
            error.kind(),
            &RasterErrorKind::PathComplexityOverflow { limit: 12 }
        );
    }

    #[test]
    fn device_pixel_bounds_should_clip_to_raster_dimensions() {
        let dimensions = RasterDimensions::new(20, 10).expect("valid dimensions");
        let bounds = device_pixel_bounds(
            PathBounds {
                min_x: -3.2,
                min_y: 1.25,
                max_x: 22.8,
                max_y: 4.75,
            },
            dimensions,
            0.5,
        )
        .expect("bounds should overlap raster");

        assert_eq!(
            bounds,
            PixelBounds {
                min_x: 0,
                min_y: 0,
                max_x: 20,
                max_y: 6,
            }
        );
    }

    #[test]
    fn stroke_pixel_bounds_should_include_radius_padding() {
        let dimensions = RasterDimensions::new(30, 20).expect("valid dimensions");
        let bounds = stroke_pixel_bounds(
            &[LineSegment {
                from: Point { x: 10.5, y: 8.0 },
                to: Point { x: 15.0, y: 12.25 },
            }],
            &[],
            2.25,
            dimensions,
        )
        .expect("stroke bounds should overlap raster");

        assert_eq!(
            bounds,
            PixelBounds {
                min_x: 6,
                min_y: 4,
                max_x: 19,
                max_y: 17,
            }
        );
    }

    #[test]
    fn device_pixel_bounds_should_skip_paths_outside_raster() {
        let dimensions = RasterDimensions::new(20, 10).expect("valid dimensions");

        assert_eq!(
            device_pixel_bounds(
                PathBounds {
                    min_x: 25.0,
                    min_y: 0.0,
                    max_x: 30.0,
                    max_y: 5.0,
                },
                dimensions,
                0.0,
            ),
            None
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
    fn image_rasterizer_should_draw_generated_soft_mask_fixture() {
        let document = generated_fixture_document("soft-mask-image.pdf");
        let resources =
            image_resources_from_document(&document).expect("generated soft-mask image resource");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("generated soft-mask image display list");
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
        .expect("soft-mask image fixture transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("soft-mask image should rasterize");

        assert_eq!(
            device.pixel(44, 44).expect("transparent sample"),
            Rgba::WHITE
        );
        assert_eq!(
            device.pixel(76, 44).expect("half-alpha sample"),
            Rgba {
                r: 127,
                g: 255,
                b: 127,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(44, 76).expect("opaque sample"),
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
    fn graphics_state_should_track_stroke_dash_pattern() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"[4 2] 1 d")),
            GraphicsStateOptions::default(),
        )
        .expect("valid dash pattern");

        assert_eq!(state.stroke_dash.len, 2);
        assert_eq!(state.stroke_dash.segments[0], 4.0);
        assert_eq!(state.stroke_dash.segments[1], 2.0);
        assert_eq!(state.stroke_dash.phase, 1.0);
    }

    #[test]
    fn graphics_state_should_track_line_cap() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"2 J")),
            GraphicsStateOptions::default(),
        )
        .expect("valid line-cap operator");

        assert_eq!(state.line_cap, LineCap::Square);
    }

    #[test]
    fn graphics_state_should_track_line_join_and_miter_limit() {
        let state = interpret_graphics_state(
            tokenize_content(PdfBytes::new(b"1 j 3 M")),
            GraphicsStateOptions::default(),
        )
        .expect("valid line-join operators");

        assert_eq!(state.line_join, LineJoin::Round);
        assert_eq!(state.miter_limit, 3.0);
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
    fn display_list_should_capture_stroke_dash_pattern() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"[4 4] 0 d 2 w 0 5 m 20 5 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid dashed stroke stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.state.stroke_dash.len, 2);
        assert_eq!(path.state.stroke_dash.segments[0], 4.0);
        assert_eq!(path.state.stroke_dash.segments[1], 4.0);
    }

    #[test]
    fn display_list_should_capture_line_cap() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"1 J 2 w 0 5 m 20 5 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid line-cap stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.state.line_cap, LineCap::Round);
    }

    #[test]
    fn display_list_should_capture_line_join_and_miter_limit() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"2 j 4 M 2 w 0 5 m 10 5 l 10 10 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid line-join stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.state.line_join, LineJoin::Bevel);
        assert_eq!(path.state.miter_limit, 4.0);
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
    fn ext_graphics_state_resources_should_parse_supported_blend_modes() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"BM"),
                PdfPrimitive::Name(PdfName::new(b"Multiply")),
            )]),
        )];
        let resources = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect("supported blend mode");

        assert_eq!(
            resources.get(PdfName::new(b"GS1")),
            Some(ExtGraphicsState {
                blend_mode: BlendMode::Multiply,
                fill_alpha: 1.0,
                stroke_alpha: 1.0,
                fill_overprint: false,
                stroke_overprint: false,
                overprint_mode: 0,
            })
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_parse_alpha_constants() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"ca"),
                    PdfPrimitive::Number(PdfNumber::Real(0.5)),
                ),
                (
                    PdfName::new(b"CA"),
                    PdfPrimitive::Number(PdfNumber::Real(0.25)),
                ),
            ]),
        )];
        let resources =
            ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary).expect("alpha state");

        assert_eq!(
            resources.get(PdfName::new(b"GS1")),
            Some(ExtGraphicsState {
                blend_mode: BlendMode::Normal,
                fill_alpha: 0.5,
                stroke_alpha: 0.25,
                fill_overprint: false,
                stroke_overprint: false,
                overprint_mode: 0,
            })
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_accept_none_soft_mask() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"SMask"),
                PdfPrimitive::Name(PdfName::new(b"None")),
            )]),
        )];
        let resources = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect("soft-mask none state");

        assert_eq!(
            resources.get(PdfName::new(b"GS1")),
            Some(ExtGraphicsState::default())
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_reject_luminosity_soft_mask() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"SMask"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"S"),
                        PdfPrimitive::Name(PdfName::new(b"Luminosity")),
                    ),
                    (
                        PdfName::new(b"G"),
                        PdfPrimitive::Name(PdfName::new(b"FormMask")),
                    ),
                ]),
            )]),
        )];
        let error = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect_err("luminosity soft mask should be typed unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedSoftMask {
                feature: b"SMask".to_vec(),
            }
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_reject_invalid_alpha_constants() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"ca"),
                PdfPrimitive::Number(PdfNumber::Real(1.2)),
            )]),
        )];
        let error = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect_err("invalid alpha should fail explicitly");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::InvalidExtGraphicsState {
                name: b"ca".to_vec(),
            }
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_reject_unsupported_blend_mode() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"BM"),
                PdfPrimitive::Name(PdfName::new(b"Overlay")),
            )]),
        )];
        let error = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect_err("unsupported blend mode should fail explicitly");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedBlendMode {
                mode: b"Overlay".to_vec(),
            }
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_reject_pdf20_black_point_compensation() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"UseBlackPtComp"),
                PdfPrimitive::Boolean(true),
            )]),
        )];
        let error = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect_err("black point compensation should be typed unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedColorManagement {
                feature: b"UseBlackPtComp".to_vec(),
            }
        );
    }

    #[test]
    fn ext_graphics_state_resources_should_parse_overprint_approximation_flags() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![
                (PdfName::new(b"OP"), PdfPrimitive::Boolean(true)),
                (PdfName::new(b"op"), PdfPrimitive::Boolean(true)),
                (
                    PdfName::new(b"OPM"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
            ]),
        )];
        let resources = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect("overprint should be approximated in RGB thumbnails");

        assert_eq!(
            resources.get(PdfName::new(b"GS1")),
            Some(ExtGraphicsState {
                blend_mode: BlendMode::Normal,
                fill_alpha: 1.0,
                stroke_alpha: 1.0,
                fill_overprint: true,
                stroke_overprint: true,
                overprint_mode: 1,
            })
        );
    }

    #[test]
    fn display_list_should_apply_external_graphics_state_blend_mode() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"BM"),
                PdfPrimitive::Name(PdfName::new(b"Screen")),
            )]),
        )];
        let resources = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect("supported blend mode");
        let list = build_path_display_list_with_ext_graphics_states(
            tokenize_content(PdfBytes::new(b"/GS1 gs 0.9 0.2 0.1 rg 70 55 80 50 re f")),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid graphics state stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.state.blend_mode, BlendMode::Screen);
    }

    #[test]
    fn display_list_should_apply_external_graphics_state_alpha_constants() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"ca"),
                    PdfPrimitive::Number(PdfNumber::Real(0.5)),
                ),
                (
                    PdfName::new(b"CA"),
                    PdfPrimitive::Number(PdfNumber::Real(0.25)),
                ),
            ]),
        )];
        let resources =
            ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary).expect("alpha state");
        let list = build_path_display_list_with_ext_graphics_states(
            tokenize_content(PdfBytes::new(b"/GS1 gs 1 0 0 rg 0 0 10 10 re f")),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid alpha graphics state stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(path.state.fill_alpha, 0.5);
        assert_eq!(path.state.stroke_alpha, 0.25);
    }

    #[test]
    fn display_list_should_expose_overprint_approximation_flags() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![
                (PdfName::new(b"OP"), PdfPrimitive::Boolean(true)),
                (PdfName::new(b"op"), PdfPrimitive::Boolean(true)),
                (
                    PdfName::new(b"OPM"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
            ]),
        )];
        let resources = ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary)
            .expect("overprint state");
        let list = build_path_display_list_with_ext_graphics_states(
            tokenize_content(PdfBytes::new(b"/GS1 gs 1 0 0 rg 0 0 10 10 re f")),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid overprint graphics state stream");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert!(path.state.fill_overprint);
        assert!(path.state.stroke_overprint);
        assert_eq!(path.state.overprint_mode, 1);
    }

    #[test]
    fn rasterize_paths_should_composite_ext_graphics_state_alpha() {
        let dictionary = vec![(
            PdfName::new(b"GS1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"ca"),
                PdfPrimitive::Number(PdfNumber::Real(0.5)),
            )]),
        )];
        let resources =
            ExtGraphicsStateResources::from_extgstate_dictionary(&dictionary).expect("alpha state");
        let list = build_path_display_list_with_ext_graphics_states(
            tokenize_content(PdfBytes::new(b"/GS1 gs 1 0 0 rg 0 0 10 10 re f")),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid alpha graphics state stream");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 10.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            10,
        )
        .expect("alpha transform");
        let device = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("alpha path should rasterize");

        assert_eq!(
            device.pixel(5, 5).expect("covered alpha pixel"),
            Rgba {
                r: 255,
                g: 127,
                b: 127,
                a: 255,
            }
        );
    }

    #[test]
    fn blend_source_with_backdrop_should_apply_supported_blend_modes() {
        let source = Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        let backdrop = Rgba {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        };

        assert_eq!(
            blend_source_with_backdrop(source, backdrop, BlendMode::Normal),
            source
        );
        assert_eq!(
            blend_source_with_backdrop(source, backdrop, BlendMode::Multiply),
            Rgba {
                r: 128,
                g: 0,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            blend_source_with_backdrop(source, backdrop, BlendMode::Screen),
            Rgba {
                r: 255,
                g: 128,
                b: 128,
                a: 255,
            }
        );
    }

    #[test]
    fn color_quantization_should_round_bright_half_values_down() {
        assert_eq!(normalized_color_to_u8(0.1), 26);
        assert_eq!(normalized_color_to_u8(0.5), 128);
        assert_eq!(normalized_color_to_u8(0.9), 229);
        assert_eq!(normalized_to_u8(0.5), 128);
    }

    #[test]
    fn source_over_should_preserve_intermediate_alpha() {
        let source = Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        let transparent = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        assert_eq!(
            source_over(source, transparent, 0.5),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            }
        );

        let gray = Rgba {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        };
        assert_eq!(
            source_over(source, gray, 0.5),
            Rgba {
                r: 191,
                g: 64,
                b: 64,
                a: 255,
            }
        );
    }

    #[test]
    fn fill_device_rect_should_antialias_subpixel_edges() {
        let mut device = RasterDevice::new(3, 3, Rgba::WHITE).expect("valid device");

        fill_device_rect(
            &mut device,
            Point { x: 0.25, y: 0.25 },
            Point { x: 1.75, y: 1.75 },
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        )
        .expect("subpixel text rect should rasterize");

        assert_eq!(
            device.pixel(0, 0).expect("covered edge pixel"),
            Rgba {
                r: 111,
                g: 111,
                b: 111,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(1, 1).expect("covered edge pixel"),
            Rgba {
                r: 111,
                g: 111,
                b: 111,
                a: 255,
            }
        );
        assert_eq!(device.pixel(2, 2).expect("uncovered pixel"), Rgba::WHITE);
    }

    #[test]
    fn shading_resources_should_parse_axial_device_rgb_shading() {
        let dictionary = vec![(
            PdfName::new(b"Sh1"),
            PdfPrimitive::Dictionary(test_axial_shading_dictionary()),
        )];
        let resources =
            ShadingResources::from_shading_dictionary(&dictionary).expect("axial shading");

        let Some(Shading::Axial(shading)) = resources.get(PdfName::new(b"Sh1")) else {
            panic!("expected axial shading");
        };
        assert_eq!(shading.start, Point { x: 0.0, y: 0.0 });
        assert_eq!(shading.end, Point { x: 100.0, y: 0.0 });
        assert_eq!(shading.exponent, 1.0);
        assert!(shading.extend_start);
        assert!(shading.extend_end);
    }

    #[test]
    fn shading_resources_should_parse_radial_device_rgb_shading() {
        let dictionary = vec![(
            PdfName::new(b"Sh1"),
            PdfPrimitive::Dictionary(test_radial_shading_dictionary()),
        )];
        let resources =
            ShadingResources::from_shading_dictionary(&dictionary).expect("radial shading");

        let Some(Shading::Radial(shading)) = resources.get(PdfName::new(b"Sh1")) else {
            panic!("expected radial shading");
        };
        assert_eq!(shading.start_center, Point { x: 60.0, y: 60.0 });
        assert_eq!(shading.start_radius, 0.0);
        assert_eq!(shading.end_center, Point { x: 60.0, y: 60.0 });
        assert_eq!(shading.end_radius, 60.0);
    }

    #[test]
    fn shading_resources_should_decode_generated_type4_mesh_stream() {
        let document = generated_fixture_document("type4-mesh-shading.pdf");
        let object = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(2).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("mesh shading stream");
        let ObjectValue::Stream(stream) = &object.value else {
            panic!("expected shading stream");
        };

        let shading =
            decode_shading_stream(b"Sh1", stream, DisplayListOptions::default()).expect("mesh");

        let Shading::Mesh(mesh) = shading else {
            panic!("expected mesh shading");
        };
        assert_eq!(mesh.triangles.len(), 1);
        assert_eq!(
            mesh.triangles[0].vertices[0].color,
            DeviceColor::Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }
        );
    }

    #[test]
    fn shading_resources_should_enforce_mesh_triangle_budget() {
        let document = generated_fixture_document("type4-mesh-shading.pdf");
        let object = document
            .objects
            .get(ObjectId::new(
                ObjectNumber::new(2).expect("object number"),
                GenerationNumber::new(0),
            ))
            .expect("mesh shading stream");
        let ObjectValue::Stream(stream) = &object.value else {
            panic!("expected shading stream");
        };
        let error = decode_shading_stream(
            b"Sh1",
            stream,
            DisplayListOptions {
                max_mesh_shading_triangles: 0,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("triangle budget should reject mesh");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ShadingTriangleOverflow { limit: 0 }
        );
    }

    #[test]
    fn shading_resources_should_reject_unsupported_shading_type() {
        let dictionary = vec![(
            PdfName::new(b"Sh1"),
            PdfPrimitive::Dictionary(vec![(
                PdfName::new(b"ShadingType"),
                PdfPrimitive::Number(PdfNumber::Integer(4)),
            )]),
        )];
        let error = ShadingResources::from_shading_dictionary(&dictionary)
            .expect_err("mesh shading should be unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedShading {
                feature: b"ShadingType".to_vec(),
            }
        );
    }

    #[test]
    fn display_list_should_capture_shading_operator() {
        let dictionary = vec![(
            PdfName::new(b"Sh1"),
            PdfPrimitive::Dictionary(test_axial_shading_dictionary()),
        )];
        let shadings =
            ShadingResources::from_shading_dictionary(&dictionary).expect("axial shading");
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let patterns = TilingPatternResources::empty();
        let color_spaces = ColorSpaceResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/Sh1 sh")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid shading stream");

        let DisplayItem::Shading(item) = &list.items()[0] else {
            panic!("expected shading display item");
        };
        assert_eq!(item.state.blend_mode, BlendMode::Normal);
    }

    #[test]
    fn axial_shading_sampling_should_interpolate_colors() {
        let shading = AxialShading {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 100.0, y: 0.0 },
            start_color: DeviceColor::Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            end_color: DeviceColor::Rgb {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            },
            exponent: 1.0,
            extend_start: true,
            extend_end: true,
        };

        assert_eq!(
            sample_axial_color(shading, 0.5),
            Rgba {
                r: 128,
                g: 0,
                b: 128,
                a: 255,
            }
        );
    }

    #[test]
    fn radial_shading_sampling_should_interpolate_colors() {
        let shading = RadialShading {
            start_center: Point { x: 60.0, y: 60.0 },
            start_radius: 0.0,
            end_center: Point { x: 60.0, y: 60.0 },
            end_radius: 60.0,
            start_color: DeviceColor::Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            end_color: DeviceColor::Rgb {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            },
            exponent: 1.0,
            extend_start: true,
            extend_end: true,
        };

        assert_eq!(
            sample_radial_color(shading, 0.5),
            Rgba {
                r: 128,
                g: 128,
                b: 255,
                a: 255,
            }
        );
    }

    #[test]
    fn color_space_resources_should_parse_separation_tint_transform() {
        let dictionary = vec![(PdfName::new(b"CS1"), test_separation_color_space())];
        let resources = ColorSpaceResources::from_color_space_dictionary(&dictionary)
            .expect("separation color space");
        let index = resources
            .index_of(PdfName::new(b"CS1"))
            .expect("color space index");
        let color = resources
            .get_index(index)
            .expect("color space")
            .evaluate(&[1.0]);

        assert_eq!(
            color.spot_approximation(),
            Some(SpotColorApproximation {
                kind: SpotColorSpaceKind::Separation,
                colorant_count: 1,
                alternate_space: AlternateColorSpace::DeviceCmyk,
            })
        );
        assert_eq!(
            device_color_to_rgba(color),
            Rgba {
                r: 255,
                g: 89,
                b: 0,
                a: 255,
            }
        );
    }

    #[test]
    fn color_space_resources_should_reject_unsupported_tint_function() {
        let dictionary = vec![(
            PdfName::new(b"CS1"),
            PdfPrimitive::Array(vec![
                PdfPrimitive::Name(PdfName::new(b"Separation")),
                PdfPrimitive::Name(PdfName::new(b"Spot")),
                PdfPrimitive::Name(PdfName::new(b"DeviceRGB")),
                PdfPrimitive::Dictionary(vec![(
                    PdfName::new(b"FunctionType"),
                    PdfPrimitive::Number(PdfNumber::Integer(4)),
                )]),
            ]),
        )];
        let error = ColorSpaceResources::from_color_space_dictionary(&dictionary)
            .expect_err("Type 4 tint transform should be unsupported");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::UnsupportedColorSpace {
                feature: b"FunctionType".to_vec(),
            }
        );
    }

    #[test]
    fn display_list_should_apply_separation_fill_color() {
        let color_spaces = ColorSpaceResources::from_color_space_dictionary(&[(
            PdfName::new(b"CS1"),
            test_separation_color_space(),
        )])
        .expect("separation color space");
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let shadings = ShadingResources::empty();
        let patterns = TilingPatternResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/CS1 cs 1 scn 0 0 20 10 re f")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid separation fill");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(
            path.state.fill_color.spot_approximation(),
            Some(SpotColorApproximation {
                kind: SpotColorSpaceKind::Separation,
                colorant_count: 1,
                alternate_space: AlternateColorSpace::DeviceCmyk,
            })
        );
    }

    #[test]
    fn display_list_should_apply_devicen_stroke_color() {
        let color_spaces = ColorSpaceResources::from_color_space_dictionary(&[(
            PdfName::new(b"CS2"),
            test_devicen_color_space(),
        )])
        .expect("DeviceN color space");
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let shadings = ShadingResources::empty();
        let patterns = TilingPatternResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/CS2 CS 0.5 1 SCN 0 0 m 20 0 l S")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid DeviceN stroke");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert_eq!(
            path.state.stroke_color.spot_approximation(),
            Some(SpotColorApproximation {
                kind: SpotColorSpaceKind::DeviceN,
                colorant_count: 2,
                alternate_space: AlternateColorSpace::DeviceRgb,
            })
        );
    }

    #[test]
    fn tiling_pattern_should_decode_colored_pattern_stream() {
        let pattern = test_tiling_pattern();

        assert_eq!(pattern.resource_name, b"P1");
        assert_eq!(pattern.paint, TilingPatternPaint::Colored);
        assert_eq!(pattern.x_step, 10.0);
        assert_eq!(pattern.y_step, 10.0);
        assert_eq!(pattern.items.len(), 2);
    }

    #[test]
    fn display_list_should_capture_tiling_pattern_fill() {
        let patterns = TilingPatternResources::new(vec![test_tiling_pattern()]);
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let shadings = ShadingResources::empty();
        let color_spaces = ColorSpaceResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/Pattern cs /P1 scn 0 0 20 10 re f")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid tiling pattern fill");

        let DisplayItem::Path(path) = &list.items()[0] else {
            panic!("expected path display item");
        };
        assert!(path.fill_pattern.is_some());
    }

    #[test]
    fn rasterize_paths_should_repeat_tiling_pattern_fill() {
        let patterns = TilingPatternResources::new(vec![test_tiling_pattern()]);
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let shadings = ShadingResources::empty();
        let color_spaces = ColorSpaceResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/Pattern cs /P1 scn 0 0 20 10 re f")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid tiling pattern fill");
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
        .expect("pattern transform");
        let raster = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("pattern should rasterize");

        assert_eq!(
            raster.pixel(2, 5).expect("left first tile"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            raster.pixel(7, 5).expect("right first tile"),
            Rgba {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }
        );
        assert_eq!(
            raster.pixel(12, 5).expect("left second tile"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
        assert_eq!(
            raster.pixel(17, 5).expect("right second tile"),
            Rgba {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }
        );
    }

    #[test]
    fn rasterize_paths_should_apply_uncolored_tiling_pattern_fill_color() {
        let pattern = decode_tiling_pattern(
            b"P2".to_vec(),
            &[
                (
                    PdfName::new(b"PatternType"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
                (
                    PdfName::new(b"PaintType"),
                    PdfPrimitive::Number(PdfNumber::Integer(2)),
                ),
                (
                    PdfName::new(b"TilingType"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
                (
                    PdfName::new(b"BBox"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(10)),
                        PdfPrimitive::Number(PdfNumber::Integer(10)),
                    ]),
                ),
                (
                    PdfName::new(b"XStep"),
                    PdfPrimitive::Number(PdfNumber::Integer(10)),
                ),
                (
                    PdfName::new(b"YStep"),
                    PdfPrimitive::Number(PdfNumber::Integer(10)),
                ),
            ],
            b"0 0 10 10 re f",
            DisplayListOptions::default(),
        )
        .expect("valid uncolored tiling pattern");
        let patterns = TilingPatternResources::new(vec![pattern]);
        let color_spaces = ColorSpaceResources::from_color_space_dictionary(&[(
            PdfName::new(b"CS1"),
            PdfPrimitive::Array(vec![
                PdfPrimitive::Name(PdfName::new(b"Pattern")),
                PdfPrimitive::Name(PdfName::new(b"DeviceRGB")),
            ]),
        )])
        .expect("valid pattern color space");
        let ext_graphics_states = ExtGraphicsStateResources::empty();
        let shadings = ShadingResources::empty();
        let list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(b"/CS1 cs 0.2 0.7 0.3 /P2 scn 0 0 20 10 re f")),
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            DisplayListOptions::default(),
        )
        .expect("valid uncolored tiling pattern fill");
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
        .expect("pattern transform");
        let raster = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("pattern should rasterize");

        assert_eq!(
            raster.pixel(5, 5).expect("first tile"),
            Rgba {
                r: 51,
                g: 178,
                b: 77,
                a: 255,
            }
        );
        assert_eq!(
            raster.pixel(15, 5).expect("second tile"),
            Rgba {
                r: 51,
                g: 178,
                b: 77,
                a: 255,
            }
        );
    }

    #[test]
    fn pattern_cell_cache_should_evict_by_entry_budget() {
        let first = test_tiling_pattern();
        let mut second = test_tiling_pattern();
        second.resource_name = b"P2".to_vec();
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
        .expect("pattern transform");
        let mut cache = PatternCellCache::new(1);

        cache
            .samples_for(
                &first,
                DeviceColor::BLACK,
                transform,
                PathRasterOptions::default(),
            )
            .expect("first pattern samples");
        cache
            .samples_for(
                &second,
                DeviceColor::BLACK,
                transform,
                PathRasterOptions::default(),
            )
            .expect("second pattern samples");

        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn pattern_cell_cache_should_keep_no_entries_when_disabled() {
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
        .expect("pattern transform");
        let mut cache = PatternCellCache::new(0);

        cache
            .samples_for(
                &test_tiling_pattern(),
                DeviceColor::BLACK,
                transform,
                PathRasterOptions::default(),
            )
            .expect("uncached pattern samples");

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn rasterize_paths_should_apply_stroke_dash_pattern() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"[4 4] 0 d 2 w 0 5 m 20 5 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid dashed stroke stream");
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
        .expect("dash transform");
        let raster = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("dashed stroke should rasterize");

        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(1, 4).expect("first dash"), black);
        assert_eq!(raster.pixel(5, 4).expect("first gap"), Rgba::WHITE);
        assert_eq!(raster.pixel(9, 4).expect("second dash"), black);
        assert_eq!(raster.pixel(13, 4).expect("second gap"), Rgba::WHITE);
    }

    #[test]
    fn rasterize_paths_should_restart_dash_phase_for_each_subpath() {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(b"[4 4] 0 d 2 w 0 3 m 4 3 l 0 7 m 4 7 l S")),
            DisplayListOptions::default(),
        )
        .expect("valid multi-subpath dashed stroke stream");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 10.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            10,
        )
        .expect("dash transform");
        let raster = rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("dashed stroke should rasterize");

        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(1, 6).expect("second subpath dash"), black);
    }

    #[test]
    fn rasterize_paths_should_apply_stroke_line_caps() {
        let butt = rasterize_line_cap_stream(b"0 J 4 w 5 5 m 15 5 l S");
        let round = rasterize_line_cap_stream(b"1 J 4 w 5 5 m 15 5 l S");
        let square = rasterize_line_cap_stream(b"2 J 4 w 5 5 m 15 5 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(butt.pixel(3, 4).expect("butt before start"), Rgba::WHITE);
        assert_eq!(round.pixel(3, 4).expect("round before start"), black);
        assert_eq!(square.pixel(3, 4).expect("square before start"), black);
    }

    #[test]
    fn rasterize_paths_should_keep_zero_width_hairline_visible() {
        let raster = rasterize_stroke_width_stream(b"0 w 0 5 m 20 5 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(10, 4).expect("zero-width hairline"), black);
    }

    #[test]
    fn rasterize_paths_should_promote_subpixel_strokes_to_hairline() {
        let raster = rasterize_stroke_width_stream(b"0.1 w 0 5 m 20 5 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(10, 4).expect("subpixel hairline"), black);
    }

    #[test]
    fn device_stroke_width_should_include_graphics_ctm_scale() {
        let state = GraphicsState {
            ctm: Matrix::scale(2.0, 2.0),
            line_width: 0.35,
            ..GraphicsState::default()
        };
        let transform = PageTransform {
            source_box: PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 30.0,
                max_y: 30.0,
            },
            rotation: PageRotation::Deg0,
            scale: 1.0 / 3.0,
            dimensions: RasterDimensions::new(10, 10).expect("valid raster dimensions"),
            matrix: Matrix::scale(1.0 / 3.0, -1.0 / 3.0),
        };

        assert!((device_stroke_width(state, transform) - 0.2333333333333333).abs() < 0.000001);
    }

    #[test]
    fn hairline_snap_policy_should_limit_ultrathin_band_to_scaled_ctm() {
        assert!(!should_snap_axis_aligned_hairline(0.177, 1.0));
        assert!(should_snap_axis_aligned_hairline(0.233, 2.0));
        assert_eq!(
            hairline_snap_mode(0.233, 2.0),
            HairlineSnapMode::RoundedDeviceCoordinate
        );
        assert_eq!(
            hairline_snap_mode(0.355, 1.0),
            HairlineSnapMode::NearestPixelCenter
        );
    }

    #[test]
    fn rasterize_paths_should_snap_axis_aligned_hairlines_to_pixel_centers() {
        let raster = rasterize_stroke_width_stream(b"0.6 w 0 5 m 20 5 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(
            raster.pixel(10, 4).expect("before snapped hairline"),
            Rgba::WHITE
        );
        assert_eq!(raster.pixel(10, 5).expect("snapped hairline"), black);
    }

    #[test]
    fn rasterize_paths_should_snap_ultrathin_axis_aligned_hairlines_to_pixel_centers() {
        let raster = rasterize_stroke_width_stream(b"0.3 w 0 5 m 20 5 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(
            raster
                .pixel(10, 4)
                .expect("before snapped ultrathin hairline"),
            Rgba::WHITE
        );
        assert_eq!(
            raster.pixel(10, 5).expect("snapped ultrathin hairline"),
            black
        );
    }

    #[test]
    fn hairline_snap_should_stabilize_near_integer_coordinates() {
        assert_eq!(
            snap_hairline_coordinate(31.9999999999, HairlineSnapMode::NearestPixelCenter),
            32.5
        );
        assert_eq!(
            snap_hairline_coordinate(32.0, HairlineSnapMode::NearestPixelCenter),
            32.5
        );
        assert_eq!(
            snap_hairline_coordinate(32.25, HairlineSnapMode::NearestPixelCenter),
            32.5
        );
        assert_eq!(
            snap_hairline_coordinate(30.6666666667, HairlineSnapMode::NearestPixelCenter),
            30.5
        );
        assert_eq!(
            snap_hairline_coordinate(30.6666666667, HairlineSnapMode::RoundedDeviceCoordinate),
            31.5
        );
    }

    #[test]
    fn rasterize_paths_should_apply_round_line_join() {
        let bevel = rasterize_line_join_stream(b"2 j 6 w 5 5 m 10 5 l 10 10 l S");
        let round = rasterize_line_join_stream(b"1 j 6 w 5 5 m 10 5 l 10 10 l S");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(
            bevel.pixel(12, 11).expect("bevel outside corner"),
            Rgba::WHITE
        );
        assert_eq!(round.pixel(12, 11).expect("round outside corner"), black);
    }

    #[test]
    fn rasterize_paths_should_apply_nonzero_clip() {
        let raster = rasterize_clip_stream(b"4 4 8 8 re W n 0 0 0 rg 0 0 20 20 re f");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(6, 10).expect("inside clip"), black);
        assert_eq!(raster.pixel(2, 10).expect("left of clip"), Rgba::WHITE);
        assert_eq!(raster.pixel(6, 4).expect("above clip"), Rgba::WHITE);
    }

    #[test]
    fn rasterize_paths_should_restore_clip_with_graphics_state() {
        let raster = rasterize_clip_stream(
            b"q 4 4 4 12 re W n 0 0 0 rg 0 0 20 20 re f Q \
              q 12 4 4 12 re W n 1 0 0 rg 0 0 20 20 re f Q",
        );
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        let red = Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(6, 10).expect("inside first clip"), black);
        assert_eq!(raster.pixel(14, 10).expect("inside second clip"), red);
        assert_eq!(
            raster.pixel(10, 10).expect("between clip scopes"),
            Rgba::WHITE
        );
    }

    #[test]
    fn rasterize_paths_should_apply_even_odd_clip() {
        let raster = rasterize_clip_stream(b"2 2 16 16 re 6 6 8 8 re W* n 0 0 0 rg 0 0 20 20 re f");
        let black = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };

        assert_eq!(raster.pixel(4, 4).expect("inside outer clip"), black);
        assert_eq!(
            raster.pixel(10, 10).expect("inside even-odd hole"),
            Rgba::WHITE
        );
        assert_eq!(raster.pixel(1, 1).expect("outside clip"), Rgba::WHITE);
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
                layout: TextLayoutStatus::Simple,
            }]
        );
    }

    #[test]
    fn text_display_list_should_classify_ligature_tounicode_sequence() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 beginbfchar\n<01> <00660069>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("ligature text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "fi");
        assert_eq!(text.glyphs[0].layout, TextLayoutStatus::LigatureExpanded);
    }

    #[test]
    fn text_display_list_should_classify_combining_mark_sequence() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 beginbfchar\n<01> <00650301>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("combining mark text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "e\u{301}");
        assert_eq!(
            text.glyphs[0].layout,
            TextLayoutStatus::CombiningMarkPositioned
        );
    }

    #[test]
    fn text_display_list_should_expose_unsupported_complex_shaping_reason() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+CjkFixture /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+CjkFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 >>] /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 beginbfchar\n<01> <65e5>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("unsupported shaping text should still decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(
            text.glyphs[0].layout,
            TextLayoutStatus::Unsupported {
                reason: TextLayoutFallbackReason::ComplexScriptShaping
            }
        );
    }

    #[test]
    fn text_display_list_should_classify_emoji_as_unsupported_layout_boundary() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEE+EmojiBoundary /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 beginbfchar\n<01> <D83DDE00>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("emoji text should decode before layout classification");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "\u{1f600}");
        assert_eq!(
            text.glyphs[0].layout,
            TextLayoutStatus::Unsupported {
                reason: TextLayoutFallbackReason::ComplexScriptShaping
            }
        );
    }

    #[test]
    fn text_display_list_should_decode_type0_cid_font_with_descendant_metrics() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 10 Tf <00010002> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+CIDFixture /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+CIDFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 600 >>] /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n2 beginbfchar\n<0001> <0041>\n<0002> <005a>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let font = resources
            .get(PdfName::new(b"F1"))
            .expect("Type0 font should resolve");
        assert_eq!(font.subtype, Some(FontSubtype::Type0));
        assert_eq!(
            font.cid_metrics,
            Some(CidFontMetrics {
                subtype: FontSubtype::CidFontType2,
                default_width: Some(600),
            })
        );

        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("Type0 CID text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "AZ");
        assert_eq!(
            text.glyphs,
            vec![
                TextGlyph {
                    character_code: 1,
                    unicode: "A".to_string(),
                    layout: TextLayoutStatus::Simple,
                },
                TextGlyph {
                    character_code: 2,
                    unicode: "Z".to_string(),
                    layout: TextLayoutStatus::Simple,
                },
            ]
        );
        assert_eq!(text.glyph_origins[0], Point { x: 0.0, y: 0.0 });
        assert_eq!(text.glyph_origins[1], Point { x: 6.0, y: 0.0 });
    }

    #[test]
    fn text_display_list_should_advance_identity_v_text_vertically() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 10 Tf <00010002> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+VerticalFixture /Encoding /Identity-V /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+VerticalFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 >>] /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n2 beginbfchar\n<0001> <65e5>\n<0002> <672c>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let font = resources
            .get(PdfName::new(b"F1"))
            .expect("vertical Type0 font should resolve");
        assert_eq!(font.writing_mode, TextWritingMode::Vertical);

        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("vertical Type0 CID text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "日本");
        assert_eq!(text.glyph_origins[0], Point { x: 0.0, y: 0.0 });
        assert_eq!(text.glyph_origins[1], Point { x: 0.0, y: -10.0 });
    }

    #[test]
    fn text_display_list_should_decode_identity_h_without_tounicode() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 10 Tf <65e5672c> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+IdentityFixture /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+IdentityFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 >>] >>",
            b"",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("Identity-H text should decode through identity CMap fallback");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "日本");
        assert_eq!(text.glyphs[0].character_code, 0x65e5);
    }

    #[test]
    fn text_display_list_should_respect_tounicode_codespace_ranges() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 10 Tf <65e5672c> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+CodeSpaceFixture /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+CodeSpaceFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 >>] /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n1 begincodespacerange\n<0000> <ffff>\nendcodespacerange\n2 beginbfchar\n<65e5> <65e5>\n<672c> <672c>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("codespace CMap should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "日本");
    }

    #[test]
    fn text_display_list_should_allow_identity_usecmap_base() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 10 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+UseCMapFixture /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /ABCDEE+UseCMapFixture /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 >>] /ToUnicode 6 0 R >>",
            b"/CIDInit /ProcSet findresource begin\n/Identity-H usecmap\n1 beginbfchar\n<01> <0055>\nendbfchar\nend",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid font resources");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("Identity-H usecmap base should be accepted");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.text, "U");
    }

    #[test]
    fn font_resources_should_reject_malformed_codespace_range() {
        let document = load_tounicode_text_pdf(
            b"BT /F1 12 Tf <01> Tj ET",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>",
            b"1 begincodespacerange\n<00> <ffff>\nendcodespacerange",
        );
        let error = font_resources_from_document(&document, &[("F1", 4)])
            .expect_err("malformed codespace range should fail");

        assert_eq!(error.kind(), &GraphicsErrorKind::InvalidCMap);
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
    fn font_resources_should_load_type3_charprocs_and_widths() {
        let document = load_type3_text_pdf(
            b"BT /F1 12 Tf (A) Tj ET",
            b"0 0 500 700 re f",
            b"<< /Type /Font /Subtype /Type3 /FontBBox [0 0 500 700] /FontMatrix [0.001 0 0 0.001 0 0] /FirstChar 65 /LastChar 65 /Widths [500] /Encoding << /Differences [65 /A] >> /CharProcs << /A 6 0 R >> >>",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid Type3 font");
        let font = resources.get(PdfName::new(b"F1")).expect("font resource");
        let type3 = font.type3.as_ref().expect("Type3 metadata");

        assert_eq!(font.subtype, Some(FontSubtype::Type3));
        assert_eq!(type3.font_matrix, Matrix::scale(0.001, 0.001));
        assert_eq!(
            type3.font_bbox,
            Some(PathBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 500.0,
                max_y: 700.0,
            })
        );
        assert_eq!(
            type3.widths,
            vec![Type3GlyphWidth {
                code: 65,
                width: 500.0,
            }]
        );
        assert_eq!(type3.char_procs[0].name, b"A");
        assert_eq!(&*type3.char_procs[0].content, b"0 0 500 700 re f");
    }

    #[test]
    fn text_display_list_should_advance_type3_text_with_widths() {
        let document = load_type3_text_pdf(
            b"BT /F1 10 Tf (AA) Tj ET",
            b"0 0 700 700 re f",
            b"<< /Type /Font /Subtype /Type3 /FontMatrix [0.001 0 0 0.001 0 0] /FirstChar 65 /LastChar 65 /Widths [700] /Encoding << /Differences [65 /A] >> /CharProcs << /A 6 0 R >> >>",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid Type3 font");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("Type3 text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(text.glyph_origins[0], Point { x: 0.0, y: 0.0 });
        assert_eq!(text.glyph_origins[1], Point { x: 7.0, y: 0.0 });
    }

    #[test]
    fn rasterize_text_should_render_type3_charproc_paths() {
        let document = load_type3_text_pdf(
            b"BT /F1 20 Tf 10 10 Td (A) Tj ET",
            b"0 0 500 700 re f",
            b"<< /Type /Font /Subtype /Type3 /FontMatrix [0.001 0 0 0.001 0 0] /FirstChar 65 /LastChar 65 /Widths [500] /Encoding << /Differences [65 /A] >> /CharProcs << /A 6 0 R >> >>",
        );
        let resources =
            font_resources_from_document(&document, &[("F1", 4)]).expect("valid Type3 font");
        let content = content_stream_from_document(&document);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("Type3 text should decode");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 40.0,
                    max_y: 40.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            40,
        )
        .expect("page transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_text(&list, &mut device, transform).expect("Type3 text should rasterize");

        let non_white_pixels = device
            .pixels()
            .chunks_exact(PixelFormat::Rgba8.bytes_per_pixel())
            .filter(|pixel| *pixel != [255, 255, 255, 255])
            .count();
        assert!(non_white_pixels > 0);
    }

    #[test]
    fn rasterize_text_should_skip_invisible_ocr_layer_pixels() {
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(
                b"BT /F1 18 Tf 3 Tr 10 20 Td (Hidden OCR) Tj ET",
            )),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("invisible text should decode");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 120.0,
                    max_y: 60.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            120,
        )
        .expect("page transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_text(&list, &mut device, transform).expect("invisible text should be skipped");

        assert!(device
            .pixels()
            .chunks_exact(PixelFormat::Rgba8.bytes_per_pixel())
            .all(|pixel| pixel == [255, 255, 255, 255]));
        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected invisible text item");
        };
        assert!(!text.rendering_mode.paints_pixels());
    }

    #[test]
    fn font_resources_should_enforce_type3_charproc_byte_budget() {
        let document = load_type3_text_pdf(
            b"BT /F1 12 Tf (A) Tj ET",
            b"0 0 500 700 re f",
            b"<< /Type /Font /Subtype /Type3 /FirstChar 65 /LastChar 65 /Widths [500] /Encoding << /Differences [65 /A] >> /CharProcs << /A 6 0 R >> >>",
        );
        let error = font_resources_from_document_with_options(
            &document,
            &[("F1", 4)],
            DisplayListOptions {
                max_font_program_bytes: 3,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("CharProc should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::FontProgramBytesOverflow { limit: 3 }
        );
    }

    #[test]
    fn glyph_bitmap_cache_should_reuse_same_character_and_size() {
        let mut cache = GlyphBitmapCache::new(8);
        let fallback = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::MissingEmbeddedProgram,
        };
        let first = cache.bitmap_for(fallback, 'A', 2.0).clone();
        let second = cache.bitmap_for(fallback, 'A', 2.0).clone();

        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn glyph_bitmap_cache_should_include_size_in_key() {
        let mut cache = GlyphBitmapCache::new(8);
        let fallback = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::MissingEmbeddedProgram,
        };
        let small = cache.bitmap_for(fallback, 'A', 1.0).clone();
        let large = cache.bitmap_for(fallback, 'A', 2.0).clone();

        assert_ne!(small, large);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn glyph_bitmap_cache_should_evict_oldest_entry_at_limit() {
        let mut cache = GlyphBitmapCache::new(1);
        let fallback = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::MissingEmbeddedProgram,
        };
        cache.bitmap_for(fallback, 'A', 1.0);
        cache.bitmap_for(fallback, 'B', 1.0);

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.entries[0].key.character, 'B');
    }

    #[test]
    fn glyph_bitmap_cache_should_include_fallback_face_in_key() {
        let mut cache = GlyphBitmapCache::new(8);
        cache.bitmap_for(
            FontFallback {
                face: FontFallbackFace::Sans,
                source: FontFallbackSource::MissingEmbeddedProgram,
            },
            'A',
            1.0,
        );
        cache.bitmap_for(
            FontFallback {
                face: FontFallbackFace::Serif,
                source: FontFallbackSource::MissingEmbeddedProgram,
            },
            'A',
            1.0,
        );

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn glyph_bitmap_cache_should_include_standard_base_paint_policy_in_key() {
        let mut cache = GlyphBitmapCache::new(8);
        let missing = cache
            .bitmap_for(
                FontFallback {
                    face: FontFallbackFace::Sans,
                    source: FontFallbackSource::MissingEmbeddedProgram,
                },
                'A',
                2.0,
            )
            .clone();
        let standard = cache
            .bitmap_for(
                FontFallback {
                    face: FontFallbackFace::Sans,
                    source: FontFallbackSource::StandardBase,
                },
                'A',
                2.0,
            )
            .clone();

        let missing_area = glyph_bitmap_area(&missing);
        let standard_area = glyph_bitmap_area(&standard);
        assert!(
            standard_area < missing_area,
            "standard base fallback should paint a lighter mask"
        );
        assert!(
            standard_area > missing_area * 0.70,
            "standard base fallback should stay close enough to real base-font weight"
        );
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn fallback_space_glyph_should_not_paint_pixels() {
        let mut cache = GlyphBitmapCache::new(8);
        let bitmap = cache
            .bitmap_for(
                FontFallback {
                    face: FontFallbackFace::Sans,
                    source: FontFallbackSource::StandardBase,
                },
                ' ',
                2.0,
            )
            .clone();

        assert_eq!(glyph_bitmap_area(&bitmap), 0.0);
    }

    #[test]
    fn fallback_printable_punctuation_should_not_use_unknown_glyph() {
        let unknown = GlyphBitmap::from_ascii('\u{7f}', 2.0, GlyphBitmapPaintPolicy::MaskOnly);

        for character in [
            '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';',
            '<', '=', '>', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
        ] {
            let bitmap = GlyphBitmap::from_ascii(character, 2.0, GlyphBitmapPaintPolicy::MaskOnly);
            assert_ne!(
                bitmap, unknown,
                "punctuation glyph {character:?} should have a dedicated fallback bitmap"
            );
            assert!(
                glyph_bitmap_area(&bitmap) > 0.0,
                "punctuation glyph {character:?} should paint visible pixels"
            );
        }
    }

    #[test]
    fn standard_base_fallback_should_use_shorter_cap_height_cell() {
        let standard = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::StandardBase,
        };
        let missing = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::MissingEmbeddedProgram,
        };

        assert!((fallback_text_cell(14.0, standard) - 1.5).abs() < f64::EPSILON);
        assert!((fallback_text_cell(14.0, missing) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn standard_base_fallback_should_scale_cell_with_graphics_ctm() {
        let standard = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::StandardBase,
        };
        let state = GraphicsState {
            ctm: Matrix::scale(2.0, 2.0),
            ..GraphicsState::default()
        };

        assert!((scaled_fallback_text_cell(14.0, standard, state) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn standard_base_fallback_should_use_helvetica_advance_widths() {
        let mut font = FontDescriptor::new("F1", Some("Helvetica"));
        font.subtype = Some(FontSubtype::Type1);
        font.fallback = Some(FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::StandardBase,
        });

        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('i'),
                unicode: "i".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            222.0
        );
        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('W'),
                unicode: "W".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            944.0
        );
        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('w'),
                unicode: "w".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            722.0
        );
        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('^'),
                unicode: "^".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            469.0
        );
    }

    #[test]
    fn standard_base_fallback_should_use_times_advance_widths() {
        let mut font = FontDescriptor::new("F1", Some("Times-Roman"));
        font.subtype = Some(FontSubtype::Type1);
        font.fallback = Some(FontFallback {
            face: FontFallbackFace::Serif,
            source: FontFallbackSource::StandardBase,
        });

        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('i'),
                unicode: "i".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            278.0
        );
        assert_eq!(
            font.advance_width_for_glyph(&TextGlyph {
                character_code: u32::from('W'),
                unicode: "W".to_string(),
                layout: TextLayoutStatus::Simple,
            }),
            944.0
        );
    }

    #[test]
    fn glyph_bitmap_cache_should_use_lowercase_x_height_masks() {
        let mut cache = GlyphBitmapCache::new(8);
        let fallback = FontFallback {
            face: FontFallbackFace::Sans,
            source: FontFallbackSource::StandardBase,
        };
        let uppercase = cache.bitmap_for(fallback, 'A', 2.0).clone();
        let lowercase = cache.bitmap_for(fallback, 'a', 2.0).clone();

        assert!(
            glyph_bitmap_area(&lowercase) < glyph_bitmap_area(&uppercase),
            "lowercase fallback glyphs should paint x-height masks"
        );
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn text_raster_scratch_should_expand_ligature_mapping_without_losing_capacity() {
        let mut scratch = TextRasterScratch::default();
        let first = fallback_text_item(
            vec![TextGlyph {
                character_code: 1,
                unicode: "fi".to_string(),
                layout: TextLayoutStatus::LigatureExpanded,
            }],
            vec![Point { x: 10.0, y: 20.0 }],
        );

        scratch.prepare(&first, 2.0);
        let capacity = scratch.atoms.capacity();

        assert_eq!(
            scratch.atoms,
            vec![
                TextRasterAtom {
                    kind: TextRasterAtomKind::Glyph('f'),
                    x: 10.0,
                    baseline_y: 20.0,
                },
                TextRasterAtom {
                    kind: TextRasterAtomKind::Glyph('i'),
                    x: 22.0,
                    baseline_y: 20.0,
                },
            ]
        );

        let second = fallback_text_item(
            vec![TextGlyph {
                character_code: 2,
                unicode: "A".to_string(),
                layout: TextLayoutStatus::Simple,
            }],
            vec![Point { x: 0.0, y: 0.0 }],
        );
        scratch.prepare(&second, 2.0);

        assert_eq!(scratch.atoms.capacity(), capacity);
    }

    #[test]
    fn text_raster_scratch_should_release_oversized_capacity_before_small_run() {
        let mut scratch = TextRasterScratch::new(1);
        let large = fallback_text_item(
            vec![TextGlyph {
                character_code: 1,
                unicode: "wide".to_string(),
                layout: TextLayoutStatus::LigatureExpanded,
            }],
            vec![Point { x: 10.0, y: 20.0 }],
        );

        scratch.prepare(&large, 2.0);
        let large_capacity = scratch.atoms.capacity();
        assert!(large_capacity > 1);

        let small = fallback_text_item(
            vec![TextGlyph {
                character_code: 2,
                unicode: "A".to_string(),
                layout: TextLayoutStatus::Simple,
            }],
            vec![Point { x: 0.0, y: 0.0 }],
        );
        scratch.prepare(&small, 2.0);

        assert!(scratch.atoms.capacity() < large_capacity);
        assert_eq!(
            scratch.atoms,
            vec![TextRasterAtom {
                kind: TextRasterAtomKind::Glyph('A'),
                x: 0.0,
                baseline_y: 0.0,
            }]
        );
    }

    #[test]
    fn text_raster_scratch_should_position_combining_marks_on_previous_base() {
        let mut scratch = TextRasterScratch::default();
        let text = fallback_text_item(
            vec![TextGlyph {
                character_code: 1,
                unicode: "e\u{301}".to_string(),
                layout: TextLayoutStatus::CombiningMarkPositioned,
            }],
            vec![Point { x: 15.0, y: 25.0 }],
        );

        scratch.prepare(&text, 2.0);

        assert_eq!(
            scratch.atoms,
            vec![
                TextRasterAtom {
                    kind: TextRasterAtomKind::Glyph('e'),
                    x: 15.0,
                    baseline_y: 25.0,
                },
                TextRasterAtom {
                    kind: TextRasterAtomKind::CombiningMark('\u{301}'),
                    x: 15.0,
                    baseline_y: 25.0,
                },
            ]
        );
    }

    #[test]
    fn text_display_list_should_preserve_subpixel_glyph_origins() {
        assert_eq!(TEXT_SUBPIXEL_POLICY, TextSubpixelPolicy::PreserveUserSpace);
        let list = build_text_display_list(
            tokenize_content(PdfBytes::new(b"BT /F1 10 Tf 10.25 20.75 Td (AA) Tj ET")),
            &test_font_resources(),
            DisplayListOptions::default(),
        )
        .expect("fractional text should decode");

        let DisplayItem::Text(text) = &list.items()[0] else {
            panic!("expected text display item");
        };
        assert_eq!(
            text.glyph_origins,
            vec![Point { x: 10.25, y: 20.75 }, Point { x: 15.25, y: 20.75 },]
        );
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
        assert_eq!(
            font.fallback,
            Some(FontFallback {
                face: FontFallbackFace::Sans,
                source: FontFallbackSource::StandardBase,
            })
        );
    }

    #[test]
    fn font_resources_should_resolve_missing_embedded_font_deterministically() {
        let document = generated_fixture_document("text-page.pdf");
        let resources = FontResources::from_font_dictionary(
            &[(
                PdfName::new(b"F1"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"Subtype"),
                        PdfPrimitive::Name(PdfName::new(b"TrueType")),
                    ),
                    (
                        PdfName::new(b"BaseFont"),
                        PdfPrimitive::Name(PdfName::new(b"ABCDEE+InvoiceSerif")),
                    ),
                    (
                        PdfName::new(b"FontDescriptor"),
                        PdfPrimitive::Dictionary(vec![(
                            PdfName::new(b"FontName"),
                            PdfPrimitive::Name(PdfName::new(b"ABCDEE+InvoiceSerif")),
                        )]),
                    ),
                ]),
            )],
            &document,
            DisplayListOptions::default(),
        )
        .expect("missing embedded program should use deterministic fallback");
        let font = resources.get(PdfName::new(b"F1")).expect("font resource");

        assert_eq!(
            font.fallback,
            Some(FontFallback {
                face: FontFallbackFace::Serif,
                source: FontFallbackSource::MissingEmbeddedProgram,
            })
        );
    }

    #[test]
    fn font_resources_should_bound_fallback_resolution_cache() {
        let document = generated_fixture_document("text-page.pdf");
        let resources = FontResources::from_font_dictionary(
            &[
                (
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
                ),
                (
                    PdfName::new(b"F2"),
                    PdfPrimitive::Dictionary(vec![
                        (
                            PdfName::new(b"Subtype"),
                            PdfPrimitive::Name(PdfName::new(b"TrueType")),
                        ),
                        (
                            PdfName::new(b"BaseFont"),
                            PdfPrimitive::Name(PdfName::new(b"InvoiceSerif")),
                        ),
                    ]),
                ),
                (
                    PdfName::new(b"F3"),
                    PdfPrimitive::Dictionary(vec![
                        (
                            PdfName::new(b"Subtype"),
                            PdfPrimitive::Name(PdfName::new(b"TrueType")),
                        ),
                        (
                            PdfName::new(b"BaseFont"),
                            PdfPrimitive::Name(PdfName::new(b"InvoiceMono")),
                        ),
                    ]),
                ),
            ],
            &document,
            DisplayListOptions {
                max_font_fallback_cache_entries: 2,
                ..DisplayListOptions::default()
            },
        )
        .expect("fallback cache should evict without failing");

        assert_eq!(resources.fallback_cache_entries(), 2);
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
    fn glyph_outline_cache_should_evict_oldest_entry_at_limit() {
        let program = test_truetype_program();
        let mut cache = GlyphOutlineCache::default();
        let options = GlyphOutlineOptions {
            max_cache_entries: 1,
            ..GlyphOutlineOptions::default()
        };

        cache
            .outline_for(&program, 0, options)
            .expect("first lookup");
        cache
            .outline_for(&program, 1, options)
            .expect("second lookup");

        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn glyph_outline_cache_should_allow_uncached_lookup() {
        let program = test_truetype_program();
        let mut cache = GlyphOutlineCache::default();
        let options = GlyphOutlineOptions {
            max_cache_entries: 0,
            ..GlyphOutlineOptions::default()
        };

        let outline = cache
            .outline_for(&program, 1, options)
            .expect("uncached lookup")
            .expect("glyph should exist");

        assert_eq!(outline.glyph_code, 1);
        assert!(cache.is_empty());
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
    fn glyph_outline_should_extract_simple_type1_charstring() {
        let program = test_type1_program(&[
            139, 239, 13, 139, 139, 21, 239, 139, 5, 139, 239, 5, 39, 139, 5, 139, 39, 5, 9, 14,
        ]);
        let outline = extract_glyph_outline(&program, 1, GlyphOutlineOptions::default())
            .expect("Type1 outline extraction should succeed")
            .expect("glyph should exist");

        assert_eq!(outline.glyph_code, 1);
        assert_eq!(outline.advance_width, 100.0);
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
    fn glyph_outline_should_reject_malformed_type1_hex_charstring() {
        let program = FontProgram {
            key: FontProgramKey {
                reference: Reference::new(ObjectId::new(
                    ObjectNumber::new(7).expect("object number"),
                    GenerationNumber::new(0),
                )),
                kind: FontProgramKind::Type1,
            },
            bytes: Arc::from(b"/CharStrings 1 dict dup begin /A <0> def end".as_slice()),
        };
        let error = extract_glyph_outline(&program, 1, GlyphOutlineOptions::default())
            .expect_err("odd Type1 hex charstring should be malformed");

        assert_eq!(error.kind(), &GraphicsErrorKind::InvalidGlyphOutline);
    }

    #[test]
    fn glyph_outline_should_enforce_type1_charstring_stack_limit() {
        let program = test_type1_program(&[139, 139, 139, 14]);
        let error = extract_glyph_outline(
            &program,
            1,
            GlyphOutlineOptions {
                max_charstring_stack: 2,
                ..GlyphOutlineOptions::default()
            },
        )
        .expect_err("Type1 charstring stack should exceed configured limit");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::GlyphOutlineStackOverflow { limit: 2 }
        );
    }

    #[test]
    fn glyph_outline_should_report_type1_subroutine_limit() {
        let program = test_type1_program(&[139, 10]);
        let error = extract_glyph_outline(
            &program,
            1,
            GlyphOutlineOptions {
                max_charstring_subroutine_depth: 0,
                ..GlyphOutlineOptions::default()
            },
        )
        .expect_err("Type1 subroutine should be rejected by bounded interpreter");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::GlyphOutlineSubroutineOverflow { limit: 0 }
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
    fn image_resources_should_decode_flate_alias_xobject() {
        let document = load_image_xobject_pdf(
            b"q 64 0 0 64 28 28 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /Fl /Length 16 >>",
            &[120, 156, 251, 207, 192, 192, 240, 31, 132, 129, 0, 0, 29, 238, 5, 251],
        );
        let resources = image_resources_from_document(&document).expect("valid image resources");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.samples.len(), 12);
    }

    #[test]
    fn image_resources_should_decode_one_bit_image_mask() {
        let document = load_image_xobject_pdf(
            b"q 2 0 0 1 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ImageMask true /Length 1 >>",
            &[0b1000_0000],
        );
        let resources = image_resources_from_document(&document).expect("image mask resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image mask");

        assert_eq!(image.bits_per_component, 1);
        assert_eq!(
            image.kind,
            ImageKind::StencilMask {
                paint_one_bits: false,
            }
        );
        assert_eq!(image.samples.as_ref(), &[0b1000_0000]);
    }

    #[test]
    fn image_resources_should_decode_inverted_image_mask() {
        let document = load_image_xobject_pdf(
            b"q 2 0 0 1 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ImageMask true /Decode [1 0] /Length 1 >>",
            &[0b1000_0000],
        );
        let resources =
            image_resources_from_document(&document).expect("inverted image mask resource");
        let image = resources.get(PdfName::new(b"Im1")).expect("image mask");

        assert_eq!(
            image.kind,
            ImageKind::StencilMask {
                paint_one_bits: true,
            }
        );
    }

    #[test]
    fn image_resources_should_reject_unsupported_image_mask_decode() {
        let document = load_image_xobject_pdf(
            b"q 2 0 0 1 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ImageMask true /Decode [0 0] /Length 1 >>",
            &[0b1000_0000],
        );

        assert!(image_resources_from_document(&document).is_err());
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
    fn image_resources_should_enforce_declared_image_byte_budget() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 100000 /Height 100000 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 0 >>",
            &[],
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_image_bytes: 4,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("declared image sample size should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageBytesOverflow { limit: 4 }
        );
    }

    #[test]
    fn image_resources_should_enforce_total_image_byte_budget() {
        let document = load_two_image_xobject_pdf();
        let xobjects = vec![
            (
                PdfName::new(b"Im1"),
                PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
            ),
            (
                PdfName::new(b"Im2"),
                PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(5, 0)),
            ),
        ];
        let error = ImageResources::from_xobject_dictionary(
            &xobjects,
            &document,
            DisplayListOptions {
                max_image_bytes: 8,
                max_total_image_bytes: 8,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("combined image samples should exceed configured page image budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageResourceBytesOverflow { limit: 8 }
        );
    }

    #[test]
    fn image_resources_should_enforce_image_mask_byte_budget() {
        let document = load_image_xobject_pdf(
            b"q 16 0 0 1 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 16 /Height 1 /ImageMask true /Length 2 >>",
            &[0xff, 0x00],
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_image_bytes: 1,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("image mask samples should exceed configured budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageBytesOverflow { limit: 1 }
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
                kind: ImageKind::Color,
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
    fn rasterize_images_should_antialias_axis_aligned_image_edges() {
        let image = ImageDisplayItem {
            image: ImageXObject {
                resource_name: b"Im1".to_vec(),
                width: 1,
                height: 1,
                bits_per_component: 8,
                color_space: ImageColorSpace::DeviceRgb,
                samples: Arc::from([255, 0, 0].as_slice()),
                kind: ImageKind::Color,
                indexed_lookup: None,
                soft_mask: None,
            },
            transform: Matrix::translate(0.25, 0.0),
            bounds: unit_square_bounds(Matrix::translate(0.25, 0.0)),
            state: GraphicsState::default(),
        };
        let mut device = RasterDevice::new(2, 1, Rgba::WHITE).expect("raster device");
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
        .expect("subpixel image draw");

        assert_eq!(
            device.pixel(0, 0).expect("left edge pixel"),
            Rgba {
                r: 255,
                g: 63,
                b: 63,
                a: 255,
            }
        );
        assert_eq!(
            device.pixel(1, 0).expect("right edge pixel"),
            Rgba {
                r: 255,
                g: 191,
                b: 191,
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
    fn image_resources_should_decode_icc_based_rgb_color_space() {
        let document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
            b"<< /N 3 /Length 22 >>",
            b"pdfrust synthetic srgb",
        );
        let resources =
            image_resources_from_document(&document).expect("ICCBased RGB should decode");
        let image = resources.get(PdfName::new(b"Im1")).expect("image resource");

        assert_eq!(image.color_space, ImageColorSpace::DeviceRgb);
        assert_eq!(&*image.samples, &[10, 20, 30]);
        assert_eq!(
            resources.icc_transform_metrics(),
            IccTransformMetrics {
                cache_hits: 0,
                cache_misses: 1,
                evictions: 0,
                max_workspace_bytes: 3 * 256 * std::mem::size_of::<f32>(),
            }
        );
    }

    #[test]
    fn image_resources_should_reuse_icc_transform_cache_across_builds() {
        let document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
            b"<< /N 3 /Length 22 >>",
            b"pdfrust synthetic srgb",
        );
        let xobjects = vec![(
            PdfName::new(b"Im1"),
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
        )];
        let mut cache = IccTransformCache::new(4);
        let first = ImageResources::from_xobject_dictionary_with_icc_cache(
            &xobjects,
            &document,
            DisplayListOptions::default(),
            &mut cache,
        )
        .expect("first ICCBased build should decode");
        let second = ImageResources::from_xobject_dictionary_with_icc_cache(
            &xobjects,
            &document,
            DisplayListOptions::default(),
            &mut cache,
        )
        .expect("second ICCBased build should reuse cache");

        assert_eq!(first.icc_transform_metrics().cache_misses, 1);
        assert_eq!(second.icc_transform_metrics().cache_hits, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn image_resources_should_evict_icc_transform_cache_by_entry_budget() {
        let first_document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
            b"<< /N 3 /Length 24 >>",
            b"pdfrust synthetic srgb A",
        );
        let second_document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[40, 50, 60],
            b"<< /N 3 /Length 24 >>",
            b"pdfrust synthetic srgb B",
        );
        let xobjects = vec![(
            PdfName::new(b"Im1"),
            PdfPrimitive::Reference(pdfrust_syntax::PdfReference::new(4, 0)),
        )];
        let mut cache = IccTransformCache::new(1);
        let options = DisplayListOptions {
            max_icc_transform_cache_entries: 1,
            ..DisplayListOptions::default()
        };

        ImageResources::from_xobject_dictionary_with_icc_cache(
            &xobjects,
            &first_document,
            options,
            &mut cache,
        )
        .expect("first ICCBased build should decode");
        let second = ImageResources::from_xobject_dictionary_with_icc_cache(
            &xobjects,
            &second_document,
            options,
            &mut cache,
        )
        .expect("second ICCBased build should decode and evict");
        let third = ImageResources::from_xobject_dictionary_with_icc_cache(
            &xobjects,
            &first_document,
            options,
            &mut cache,
        )
        .expect("first ICCBased profile should decode again after eviction");

        assert_eq!(cache.len(), 1);
        assert_eq!(second.icc_transform_metrics().evictions, 1);
        assert_eq!(third.icc_transform_metrics().cache_misses, 1);
        assert_eq!(third.icc_transform_metrics().evictions, 1);
    }

    #[test]
    fn image_resources_should_enforce_icc_profile_byte_budget() {
        let document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
            b"<< /N 3 /Length 22 >>",
            b"pdfrust synthetic srgb",
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_icc_profile_bytes: 4,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("ICC profile should exceed configured byte budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageResourceBytesOverflow { limit: 4 }
        );
    }

    #[test]
    fn image_resources_should_enforce_icc_transform_workspace_budget() {
        let document = load_image_xobject_pdf_with_icc_profile(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length 3 >>",
            &[10, 20, 30],
            b"<< /N 3 /Length 22 >>",
            b"pdfrust synthetic srgb",
        );
        let error = image_resources_from_document_with_options(
            &document,
            DisplayListOptions {
                max_icc_transform_workspace_bytes: 128,
                ..DisplayListOptions::default()
            },
        )
        .expect_err("ICC transform should exceed configured workspace budget");

        assert_eq!(
            error.kind(),
            &GraphicsErrorKind::ImageResourceBytesOverflow { limit: 128 }
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
    fn image_sample_cache_should_reuse_last_converted_sample() {
        let image = ImageXObject {
            resource_name: b"Im1".to_vec(),
            width: 1,
            height: 1,
            bits_per_component: 8,
            color_space: ImageColorSpace::DeviceCmyk,
            samples: Arc::from([0, 255, 255, 0].as_slice()),
            kind: ImageKind::Color,
            indexed_lookup: None,
            soft_mask: Some(Arc::from([128].as_slice())),
        };
        let mut cache = ImageSampleCache::default();

        let first = cache.sample(&image, 0, 0, DeviceColor::BLACK);
        let second = cache.sample(&image, 0, 0, DeviceColor::BLACK);

        assert_eq!(
            first,
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            }
        );
        assert_eq!(second, first);
        let cached = cache.last.expect("cached sample");
        assert_eq!(cached.x, 0);
        assert_eq!(cached.y, 0);
        assert_eq!(cached.color, first);
    }

    #[test]
    fn image_rasterizer_should_apply_image_mask_with_fill_color() {
        let document = load_image_xobject_pdf(
            b"q 2 0 0 1 0 0 cm 1 0 0 rg /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ImageMask true /Length 1 >>",
            &[0b1000_0000],
        );
        let resources = image_resources_from_document(&document).expect("image mask resource");
        let content = content_stream_from_document(&document);
        let list = build_image_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("image mask display list");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 2.0,
                    max_y: 1.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            2,
        )
        .expect("image mask transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_images(&list, &mut device, transform).expect("image mask should rasterize");

        assert_eq!(
            device.pixel(0, 0).expect("transparent mask pixel"),
            Rgba::WHITE
        );
        assert_eq!(
            device.pixel(1, 0).expect("painted mask pixel"),
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
    fn image_resources_should_route_dct_alias_to_jpeg_decoder() {
        let document = load_image_xobject_pdf(
            b"q 10 0 0 10 0 0 cm /Im1 Do Q",
            b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCT /Length 3 >>",
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
        for filter in [
            b"CCITTFaxDecode".as_slice(),
            b"CCF",
            b"JPXDecode",
            b"JBIG2Decode",
        ] {
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
    fn form_display_list_should_capture_transparency_group_item() {
        let document = load_form_xobject_pdf(
            b"q 1 0 0 1 10 10 cm /Fm1 Do Q",
            b"1 0 0 rg 0 0 20 20 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Group << /S /Transparency /I true /K false >> /Length 23 >>",
            None,
        );
        let resources =
            form_resources_from_document(&document, &[("Fm1", 4)]).expect("valid form resources");
        let form = resources.get(PdfName::new(b"Fm1")).expect("form resource");

        assert_eq!(
            form.transparency_group,
            Some(TransparencyGroup {
                isolated: true,
                knockout: false,
            })
        );

        let content = content_stream_from_document(&document);
        let list = build_form_display_list(
            tokenize_content(PdfBytes::new(&content)),
            &resources,
            DisplayListOptions::default(),
        )
        .expect("valid transparency group form invocation");

        assert_eq!(list.len(), 1);
        let DisplayItem::TransparencyGroup(group) = &list.items()[0] else {
            panic!("expected transparency group item");
        };
        assert_eq!(
            group.bounds,
            PathBounds {
                min_x: 10.0,
                min_y: 10.0,
                max_x: 30.0,
                max_y: 30.0,
            }
        );
        assert_eq!(group.items.len(), 1);
    }

    #[test]
    fn form_transparency_group_should_rasterize_through_bounded_intermediate() {
        let document = load_form_xobject_pdf(
            b"q 1 0 0 1 10 10 cm /Fm1 Do Q",
            b"1 0 0 rg 0 0 20 20 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Group << /S /Transparency /I true >> /Length 23 >>",
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
        .expect("valid transparency group form invocation");
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
        .expect("group transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");

        rasterize_paths_into(&list, &mut device, transform, PathRasterOptions::default())
            .expect("transparency group should rasterize");

        assert_eq!(
            device.pixel(20, 100).expect("group fill pixel"),
            Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        );
    }

    #[test]
    fn form_transparency_group_should_enforce_intermediate_pixel_budget() {
        let document = load_form_xobject_pdf(
            b"q 1 0 0 1 10 10 cm /Fm1 Do Q",
            b"1 0 0 rg 0 0 20 20 re f",
            b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Group << /S /Transparency >> /Length 23 >>",
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
        .expect("valid transparency group form invocation");
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
        .expect("group transform");
        let mut device = transform.create_device(Rgba::WHITE).expect("raster device");
        let error = rasterize_paths_into(
            &list,
            &mut device,
            transform,
            PathRasterOptions {
                max_transparency_group_pixels: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect_err("group should exceed intermediate pixel budget");

        assert_eq!(
            error.kind(),
            &RasterErrorKind::TransparencyGroupPixelsOverflow { limit: 1 }
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

    fn rasterize_line_cap_stream(content: &[u8]) -> RasterDevice {
        rasterize_stroke_width_stream(content)
    }

    fn rasterize_stroke_width_stream(content: &[u8]) -> RasterDevice {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(content)),
            DisplayListOptions::default(),
        )
        .expect("valid stroke-width stream");
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
        .expect("line-cap transform");

        rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("stroke-width stream should rasterize")
    }

    fn rasterize_clip_stream(content: &[u8]) -> RasterDevice {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(content)),
            DisplayListOptions::default(),
        )
        .expect("valid clipped stream");
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
        .expect("clip transform");

        rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("clipped paths should rasterize")
    }

    fn rasterize_line_join_stream(content: &[u8]) -> RasterDevice {
        let list = build_path_display_list(
            tokenize_content(PdfBytes::new(content)),
            DisplayListOptions::default(),
        )
        .expect("valid line-join stream");
        let transform = PageTransform::new(
            PageGeometry {
                media_box: PathBounds {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 20.0,
                    max_y: 15.0,
                },
                crop_box: None,
                rotation: PageRotation::Deg0,
            },
            20,
        )
        .expect("line-join transform");

        rasterize_paths(
            &list,
            transform,
            Rgba::WHITE,
            PathRasterOptions {
                supersample: 1,
                ..PathRasterOptions::default()
            },
        )
        .expect("line-join stroke should rasterize")
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

    fn test_axial_shading_dictionary() -> Vec<(PdfName<'static>, PdfPrimitive<'static>)> {
        vec![
            (
                PdfName::new(b"ShadingType"),
                PdfPrimitive::Number(PdfNumber::Integer(2)),
            ),
            (
                PdfName::new(b"ColorSpace"),
                PdfPrimitive::Name(PdfName::new(b"DeviceRGB")),
            ),
            (
                PdfName::new(b"Coords"),
                PdfPrimitive::Array(vec![
                    PdfPrimitive::Number(PdfNumber::Integer(0)),
                    PdfPrimitive::Number(PdfNumber::Integer(0)),
                    PdfPrimitive::Number(PdfNumber::Integer(100)),
                    PdfPrimitive::Number(PdfNumber::Integer(0)),
                ]),
            ),
            (
                PdfName::new(b"Function"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"FunctionType"),
                        PdfPrimitive::Number(PdfNumber::Integer(2)),
                    ),
                    (
                        PdfName::new(b"C0"),
                        PdfPrimitive::Array(vec![
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                        ]),
                    ),
                    (
                        PdfName::new(b"C1"),
                        PdfPrimitive::Array(vec![
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                        ]),
                    ),
                    (
                        PdfName::new(b"N"),
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                    ),
                ]),
            ),
            (
                PdfName::new(b"Extend"),
                PdfPrimitive::Array(vec![
                    PdfPrimitive::Boolean(true),
                    PdfPrimitive::Boolean(true),
                ]),
            ),
        ]
    }

    fn test_radial_shading_dictionary() -> Vec<(PdfName<'static>, PdfPrimitive<'static>)> {
        vec![
            (
                PdfName::new(b"ShadingType"),
                PdfPrimitive::Number(PdfNumber::Integer(3)),
            ),
            (
                PdfName::new(b"ColorSpace"),
                PdfPrimitive::Name(PdfName::new(b"DeviceRGB")),
            ),
            (
                PdfName::new(b"Coords"),
                PdfPrimitive::Array(vec![
                    PdfPrimitive::Number(PdfNumber::Integer(60)),
                    PdfPrimitive::Number(PdfNumber::Integer(60)),
                    PdfPrimitive::Number(PdfNumber::Integer(0)),
                    PdfPrimitive::Number(PdfNumber::Integer(60)),
                    PdfPrimitive::Number(PdfNumber::Integer(60)),
                    PdfPrimitive::Number(PdfNumber::Integer(60)),
                ]),
            ),
            (
                PdfName::new(b"Function"),
                PdfPrimitive::Dictionary(vec![
                    (
                        PdfName::new(b"FunctionType"),
                        PdfPrimitive::Number(PdfNumber::Integer(2)),
                    ),
                    (
                        PdfName::new(b"C0"),
                        PdfPrimitive::Array(vec![
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                        ]),
                    ),
                    (
                        PdfName::new(b"C1"),
                        PdfPrimitive::Array(vec![
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                            PdfPrimitive::Number(PdfNumber::Integer(0)),
                            PdfPrimitive::Number(PdfNumber::Integer(1)),
                        ]),
                    ),
                    (
                        PdfName::new(b"N"),
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                    ),
                ]),
            ),
            (
                PdfName::new(b"Extend"),
                PdfPrimitive::Array(vec![
                    PdfPrimitive::Boolean(true),
                    PdfPrimitive::Boolean(true),
                ]),
            ),
        ]
    }

    fn test_separation_color_space() -> PdfPrimitive<'static> {
        PdfPrimitive::Array(vec![
            PdfPrimitive::Name(PdfName::new(b"Separation")),
            PdfPrimitive::Name(PdfName::new(b"SpotOrange")),
            PdfPrimitive::Name(PdfName::new(b"DeviceCMYK")),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"FunctionType"),
                    PdfPrimitive::Number(PdfNumber::Integer(2)),
                ),
                (
                    PdfName::new(b"C0"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                    ]),
                ),
                (
                    PdfName::new(b"C1"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Real(0.65)),
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                    ]),
                ),
                (
                    PdfName::new(b"N"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
            ]),
        ])
    }

    fn test_devicen_color_space() -> PdfPrimitive<'static> {
        PdfPrimitive::Array(vec![
            PdfPrimitive::Name(PdfName::new(b"DeviceN")),
            PdfPrimitive::Array(vec![
                PdfPrimitive::Name(PdfName::new(b"SpotOrange")),
                PdfPrimitive::Name(PdfName::new(b"SpotBlue")),
            ]),
            PdfPrimitive::Name(PdfName::new(b"DeviceRGB")),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"FunctionType"),
                    PdfPrimitive::Number(PdfNumber::Integer(2)),
                ),
                (
                    PdfName::new(b"C0"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                        PdfPrimitive::Number(PdfNumber::Integer(1)),
                    ]),
                ),
                (
                    PdfName::new(b"C1"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Real(0.2)),
                        PdfPrimitive::Number(PdfNumber::Real(0.4)),
                        PdfPrimitive::Number(PdfNumber::Real(0.9)),
                    ]),
                ),
                (
                    PdfName::new(b"N"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
            ]),
        ])
    }

    fn test_tiling_pattern() -> TilingPattern {
        decode_tiling_pattern(
            b"P1".to_vec(),
            &[
                (
                    PdfName::new(b"PatternType"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
                (
                    PdfName::new(b"PaintType"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
                (
                    PdfName::new(b"TilingType"),
                    PdfPrimitive::Number(PdfNumber::Integer(1)),
                ),
                (
                    PdfName::new(b"BBox"),
                    PdfPrimitive::Array(vec![
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(0)),
                        PdfPrimitive::Number(PdfNumber::Integer(10)),
                        PdfPrimitive::Number(PdfNumber::Integer(10)),
                    ]),
                ),
                (
                    PdfName::new(b"XStep"),
                    PdfPrimitive::Number(PdfNumber::Integer(10)),
                ),
                (
                    PdfName::new(b"YStep"),
                    PdfPrimitive::Number(PdfNumber::Integer(10)),
                ),
            ],
            b"1 0 0 rg 0 0 5 10 re f 0 0 1 rg 5 0 5 10 re f",
            DisplayListOptions::default(),
        )
        .expect("valid tiling pattern")
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

    fn load_image_xobject_pdf_with_icc_profile(
        content_stream: &[u8],
        image_dictionary: &[u8],
        image_stream: &[u8],
        profile_dictionary: &[u8],
        profile_stream: &[u8],
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
            stream_object_bytes(6, profile_dictionary, profile_stream),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked))
            .expect("ICCBased image XObject PDF should load")
    }

    fn load_two_image_xobject_pdf() -> ClassicDocument<'static> {
        let content_stream = b"q 2 0 0 2 0 0 cm /Im1 Do Q q 2 0 0 2 10 0 cm /Im2 Do Q";
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let image_dictionary =
            b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 6 >>";
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /XObject << /Im1 4 0 R /Im2 5 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            stream_object_bytes(4, image_dictionary, &[255, 0, 0, 0, 255, 0]),
            stream_object_bytes(5, image_dictionary, &[0, 0, 255, 255, 255, 0]),
            indirect_object_bytes(6, b"<< /Type /Catalog /Pages 3 0 R >>"),
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

    fn load_type3_text_pdf(
        content_stream: &[u8],
        char_proc_stream: &[u8],
        font_dictionary: &[u8],
    ) -> ClassicDocument<'static> {
        let content_dictionary = format!("<< /Length {} >>", content_stream.len());
        let char_proc_dictionary = format!("<< /Length {} >>", char_proc_stream.len());
        let objects = vec![
            stream_object_bytes(1, content_dictionary.as_bytes(), content_stream),
            indirect_object_bytes(
                2,
                b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] /Resources << /Font << /F1 4 0 R >> >> /Contents 1 0 R >>",
            ),
            indirect_object_bytes(3, b"<< /Type /Pages /Kids [2 0 R] /Count 1 >>"),
            indirect_object_bytes(4, font_dictionary),
            indirect_object_bytes(5, b"<< /Type /Catalog /Pages 3 0 R >>"),
            stream_object_bytes(6, char_proc_dictionary.as_bytes(), char_proc_stream),
        ];
        let pdf = build_classic_pdf_from_objects(&objects);
        let leaked = Box::leak(pdf.into_boxed_slice());
        load_classic_document(PdfBytes::new(leaked)).expect("Type3 text PDF should load")
    }

    fn fallback_text_item(glyphs: Vec<TextGlyph>, glyph_origins: Vec<Point>) -> TextDisplayItem {
        TextDisplayItem {
            text: glyphs
                .iter()
                .map(|glyph| glyph.unicode.as_str())
                .collect::<String>(),
            glyphs,
            glyph_origins,
            font: FontDescriptor::new("F1", Some("Helvetica")),
            font_size: 14.0,
            origin: Point { x: 0.0, y: 0.0 },
            text_matrix: Matrix::IDENTITY,
            rendering_mode: TextRenderingMode::Fill,
            state: GraphicsState::default(),
        }
    }

    fn glyph_bitmap_area(bitmap: &GlyphBitmap) -> f64 {
        bitmap
            .rects
            .iter()
            .map(|rect| (rect.right - rect.left).max(0.0) * (rect.top - rect.bottom).max(0.0))
            .sum()
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

    fn test_type1_program(charstring: &[u8]) -> FontProgram {
        FontProgram {
            key: FontProgramKey {
                reference: Reference::new(ObjectId::new(
                    ObjectNumber::new(9).expect("object number"),
                    GenerationNumber::new(0),
                )),
                kind: FontProgramKind::Type1,
            },
            bytes: Arc::from(minimal_type1_font(charstring)),
        }
    }

    fn minimal_type1_font(charstring: &[u8]) -> Vec<u8> {
        let mut font = b"%!PS-AdobeFont-1.0: PdfrustType1 1.0\n/CharStrings 2 dict dup begin\n/.notdef <0e> def\n/A <".to_vec();
        for byte in charstring {
            font.extend_from_slice(format!("{byte:02x}").as_bytes());
        }
        font.extend_from_slice(b"> def\nend\n");
        font
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
