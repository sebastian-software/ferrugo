//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

use pdfrust_content::{ContentErrorKind, ContentResult, ContentToken, OperatorName};
use pdfrust_syntax::{ByteOffset, PdfNumber, PdfPrimitive};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "render";

/// Default maximum graphics-state stack depth.
pub const DEFAULT_GRAPHICS_STATE_STACK_LIMIT: usize = 64;

/// Default maximum path segment count for one current path.
pub const DEFAULT_PATH_SEGMENT_LIMIT: usize = 16_384;

/// Default maximum display items for one content stream.
pub const DEFAULT_DISPLAY_ITEM_LIMIT: usize = 8_192;

/// Result alias for graphics-state interpretation.
pub type GraphicsResult<T> = Result<T, GraphicsError>;

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
}

impl Default for DisplayListOptions {
    fn default() -> Self {
        Self {
            max_stack_depth: DEFAULT_GRAPHICS_STATE_STACK_LIMIT,
            max_path_segments: DEFAULT_PATH_SEGMENT_LIMIT,
            max_display_items: DEFAULT_DISPLAY_ITEM_LIMIT,
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

struct GraphicsStateInterpreter {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    max_stack_depth: usize,
}

struct DisplayListInterpreter {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    current_path: CurrentPath,
    display_list: DisplayList,
    options: DisplayListOptions,
}

impl DisplayListInterpreter {
    fn new(options: DisplayListOptions) -> Self {
        Self {
            current: GraphicsState::default(),
            stack: Vec::new(),
            current_path: CurrentPath::default(),
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

fn invalid_operand(offset: ByteOffset, operator: &'static [u8]) -> GraphicsError {
    GraphicsError::new(Some(offset), GraphicsErrorKind::InvalidOperand { operator })
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfrust_content::tokenize_content;
    use pdfrust_object::{
        load_classic_document, GenerationNumber, ObjectId, ObjectNumber, ObjectValue,
    };
    use pdfrust_syntax::PdfBytes;

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
    fn matrix_should_transform_points_deterministically() {
        let matrix = Matrix::translate(10.0, 20.0).multiply(Matrix::scale(2.0, 3.0));

        assert_eq!(matrix.transform_point(4.0, 5.0), Point { x: 18.0, y: 35.0 });
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

    fn generated_fixture_content(file_name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/generated/{file_name}"));
        let bytes = std::fs::read(path).expect("fixture should be readable");
        let document =
            load_classic_document(PdfBytes::new(&bytes)).expect("fixture should load as PDF");
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
}
