//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

use std::fmt;

use pdfrust_content::{ContentErrorKind, ContentResult, ContentToken, OperatorName};
use pdfrust_syntax::{ByteOffset, PdfNumber, PdfPrimitive};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "render";

/// Default maximum graphics-state stack depth.
pub const DEFAULT_GRAPHICS_STATE_STACK_LIMIT: usize = 64;

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

/// Current graphics state subset needed by early renderer milestones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphicsState {
    /// Current transformation matrix.
    pub ctm: Matrix,
    /// Current line width.
    pub line_width: f64,
    /// Current fill color.
    pub fill_gray: DeviceGray,
    /// Current stroke color.
    pub stroke_gray: DeviceGray,
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
            clip_path_pending: false,
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

struct GraphicsStateInterpreter {
    current: GraphicsState,
    stack: Vec<GraphicsState>,
    max_stack_depth: usize,
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
        self.current.fill_gray =
            DeviceGray(number_operand(offset, b"g", operands, 0)?.clamp(0.0, 1.0));
        Ok(())
    }

    fn set_stroke_gray(
        &mut self,
        offset: ByteOffset,
        operands: &[PdfPrimitive<'_>],
    ) -> GraphicsResult<()> {
        expect_operand_count(offset, b"G", operands, 1)?;
        self.current.stroke_gray =
            DeviceGray(number_operand(offset, b"G", operands, 0)?.clamp(0.0, 1.0));
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfrust_content::tokenize_content;
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
}
