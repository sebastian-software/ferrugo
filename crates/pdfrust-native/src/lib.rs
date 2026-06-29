//! Rust-native backend adapter for the thumbnail facade.

#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use pdfrust_content::{tokenize_content, ContentToken};
use pdfrust_object::{
    load_classic_document, load_linearized_first_page_document, load_modern_document,
    ClassicDocument, DocumentLoadMetrics, GenerationNumber, ObjectError, ObjectId, ObjectNumber,
    ObjectValue, PageBox, PageMetadata as ObjectPageMetadata, PageTree, Reference,
    StreamDecodeOptions,
};
use pdfrust_render::{
    build_form_display_list_with_graphics_resources, build_image_display_list,
    build_path_display_list_with_graphics_resources, build_text_display_list,
    decode_tiling_pattern, rasterize_display_list_into, rasterize_images, rasterize_paths_into,
    rasterize_text, ColorSpaceResources, DisplayItem, DisplayList, DisplayListOptions,
    ExtGraphicsStateResources, FontResources, FormResources, GraphicsError, GraphicsErrorKind,
    ImageResources, PageGeometry, PageRotation, PageTransform, PageTransformOptions, PathBounds,
    PathRasterOptions, RasterError, RasterErrorKind, ShadingResources, TilingPatternResources,
};
use pdfrust_syntax::{PdfBytes, PdfName, PdfNumber, PdfPrimitive, PdfReference, PdfString};
use pdfrust_thumbnail::{
    unsupported_feature_buckets as buckets, AccessibilityMetadata, AnnotationMode,
    ArchivalMetadata, DocumentInfo, DocumentMetadata, DocumentMetadataBackend, DocumentStructure,
    OptionalContentBaseState, OptionalContentMetadata, OutlineMetadata, PageLabel,
    PageLabelsMetadata, PageMetadata as ThumbnailPageMetadata, PageSize, PageText, PdfSource,
    PositionedGlyph, TextExtractionBackend, TextExtractionOptions, TextPoint, TextRun, Thumbnail,
    ThumbnailBackend, ThumbnailError, ThumbnailOptions,
};

#[cfg(test)]
use pdfrust_thumbnail::FormAppearanceMode;

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "native-backend";

const BUCKET_GRAPHICS_OPTIONAL_CONTENT: &str = buckets::GRAPHICS_OPTIONAL_CONTENT;
const BUCKET_GRAPHICS_COLOR_MANAGEMENT: &str = buckets::GRAPHICS_COLOR_MANAGEMENT;
const BUCKET_GRAPHICS_PATTERN_SHADING: &str = buckets::GRAPHICS_PATTERN_SHADING;
const BUCKET_GRAPHICS_STROKE_CLIP: &str = buckets::GRAPHICS_STROKE_CLIP;
const BUCKET_GRAPHICS_TRANSPARENCY: &str = buckets::GRAPHICS_TRANSPARENCY;
const BUCKET_ANNOTATION_APPEARANCE: &str = buckets::ANNOTATION_APPEARANCE;
const BUCKET_IMAGE_COLOR_SPACE: &str = buckets::IMAGE_COLOR_SPACE;
const BUCKET_IMAGE_FILTER: &str = buckets::IMAGE_FILTER;
const BUCKET_FORM_XFA_DYNAMIC: &str = buckets::FORM_XFA_DYNAMIC;
const BUCKET_FORM_APPEARANCE_MUTATION: &str = buckets::FORM_APPEARANCE_MUTATION;
const BUCKET_RENDERER_FORM_XOBJECT: &str = buckets::RENDERER_FORM_XOBJECT_COMPOSITION;
const BUCKET_RENDERER_MEMORY_BUDGET: &str = buckets::RENDERER_MEMORY_BUDGET;
const BUCKET_RENDERER_UNSUPPORTED: &str = buckets::NATIVE_UNSUPPORTED;
const BUCKET_TEXT_CMAP_TOUNICODE: &str = buckets::TEXT_CMAP_TOUNICODE;
const BUCKET_TEXT_FONT_PROGRAM: &str = buckets::TEXT_FONT_PROGRAM;
const BUCKET_TEXT_GLYPH_OUTLINE: &str = buckets::TEXT_GLYPH_OUTLINE;
const ANNOTATION_OPAQUE_GRAPHICS_STATE: &[u8] = b"AnnotOpaque";
const ANNOTATION_UNDERLINE_GRAPHICS_STATE: &[u8] = b"AnnotUnderline";
const MAX_ANNOTATION_FALLBACK_QUADS: usize = 32;
const MAX_METADATA_OUTLINE_ITEMS: usize = 256;
const MAX_METADATA_PAGE_LABELS: usize = 4096;
const MAX_METADATA_SIGNATURE_FIELDS: usize = 4096;
const MAX_METADATA_ATTACHMENT_ANNOTATIONS: usize = 4096;
const MAX_METADATA_STRUCTURE_ITEMS: usize = 4096;
const MAX_METADATA_XMP_BYTES: usize = 64 * 1024;
const DEFAULT_SPOOL_BYTES_LIMIT: usize = 0;

/// Rust-native thumbnail backend.
///
/// The backend owns the render budget profile used by the native renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeBackend {
    limits: NativeRenderLimits,
}

/// Native operator coverage scan options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorCoverageOptions {
    /// Zero-based page index to scan.
    pub page_index: u32,
    /// Include annotation appearance and synthesized fallback appearance
    /// streams in addition to page content.
    pub include_annotations: bool,
}

impl Default for OperatorCoverageOptions {
    fn default() -> Self {
        Self {
            page_index: 0,
            include_annotations: true,
        }
    }
}

/// Native support classification for a PDF content-stream operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorSupportStatus {
    /// Native rendering currently implements the operator for common cases.
    Implemented,
    /// Native rendering implements a bounded subset or policy-dependent subset.
    Partial,
    /// Native rendering does not implement the operator semantics.
    Unsupported,
    /// Native rendering intentionally ignores the operator because it is
    /// non-visual or only carries metadata for current thumbnail output.
    Ignored,
}

impl OperatorSupportStatus {
    /// Stable JSON/report string for the status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::Partial => "partial",
            Self::Unsupported => "unsupported",
            Self::Ignored => "ignored",
        }
    }
}

/// One operator row in a native coverage scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorCoverageEntry {
    /// PDF operator name. Inline images are reported as `BI`.
    pub operator: String,
    /// Number of occurrences in scanned streams.
    pub count: usize,
    /// Native support status for this operator.
    pub status: OperatorSupportStatus,
    /// Suggested typed fallback bucket for unsupported or partial behavior.
    pub fallback_bucket: Option<&'static str>,
}

/// Native operator coverage scan result for one document page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorCoverageReport {
    /// Scanned zero-based page index.
    pub page_index: u32,
    /// Number of decoded content streams scanned.
    pub streams_scanned: usize,
    /// Number of content-stream operators, including inline image markers.
    pub total_operators: usize,
    /// Number of inline image objects encountered.
    pub inline_images: usize,
    /// Sorted operator coverage rows.
    pub operators: Vec<OperatorCoverageEntry>,
}

impl NativeBackend {
    /// Creates a Rust-native backend using the default desktop-oriented render
    /// budgets.
    #[must_use]
    pub fn new() -> Self {
        Self::with_render_limits(NativeRenderLimits::default())
    }

    /// Creates a Rust-native backend using constrained low-memory render
    /// budgets for embedded, serverless, and batch-thumbnail workloads.
    #[must_use]
    pub const fn low_memory() -> Self {
        Self::with_render_limits(NativeRenderLimits::low_memory())
    }

    /// Creates a Rust-native backend using explicit render budgets.
    #[must_use]
    pub const fn with_render_limits(limits: NativeRenderLimits) -> Self {
        Self { limits }
    }

    /// Returns the active native render budget snapshot.
    #[must_use]
    pub const fn render_limits(&self) -> NativeRenderLimits {
        self.limits
    }

    /// Returns the stable backend name.
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        "rust-native"
    }

    /// Returns the current default memory and cache budget snapshot.
    #[must_use]
    pub const fn memory_diagnostics(&self) -> NativeMemoryDiagnostics {
        self.limits.memory_diagnostics()
    }

    /// Renders page zero through the preview boundary and reports whether the
    /// bounded linearized first-page loader was usable.
    ///
    /// # Errors
    ///
    /// Returns [`ThumbnailError`] when the source cannot be read or page zero
    /// cannot be rendered.
    pub fn render_first_page_preview(
        &self,
        source: PdfSource<'_>,
        options: &ThumbnailOptions,
    ) -> Result<FirstPagePreview, ThumbnailError> {
        let bytes = load_source(source)?;
        let mut preview_options = *options;
        preview_options.page_index = 0;
        let input = PdfBytes::new(bytes.as_ref());
        let (document, page_tree) = load_render_document(input, preview_options.page_index)?;
        let load_mode = FirstPagePreviewLoadMode::from_load_metrics(document.load_metrics);
        let memory = FirstPagePreviewMemory::from_load_metrics(document.load_metrics);
        let thumbnail =
            render_loaded_document(&document, &page_tree, &preview_options, self.limits)?;
        Ok(FirstPagePreview {
            thumbnail,
            load_mode,
            memory,
        })
    }

    /// Renders requested preview pages while preserving page-level outcomes.
    ///
    /// This is the backend-owned partial preview boundary for callers that need
    /// cancellation and partial results without falling back to PDFium.
    ///
    /// # Errors
    ///
    /// Returns [`ThumbnailError`] when the source cannot be read, the scheduler
    /// configuration is invalid, or the memory budget cannot schedule one page.
    pub fn render_preview_pages_partial(
        &self,
        source: PdfSource<'_>,
        page_indices: &[u32],
        options: &ThumbnailOptions,
        parallel_options: ParallelRenderOptions,
        cancellation: &RenderCancellation,
    ) -> Result<ParallelRenderPartialResult, ThumbnailError> {
        render_pages_parallel_partial_with_limits(
            source,
            page_indices,
            options,
            parallel_options,
            cancellation,
            self.limits,
        )
    }
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Loader path used for a first-page preview render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstPagePreviewLoadMode {
    /// The document's linearization metadata was sufficient for bounded
    /// first-page loading.
    LinearizedFirstPage,
    /// Preview rendering used the normal full-document loader.
    FullDocument,
}

impl FirstPagePreviewLoadMode {
    /// Stable report string for the load mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LinearizedFirstPage => "linearized-first-page",
            Self::FullDocument => "full-document",
        }
    }

    const fn from_load_metrics(metrics: DocumentLoadMetrics) -> Self {
        if metrics.first_page_only {
            Self::LinearizedFirstPage
        } else {
            Self::FullDocument
        }
    }
}

/// Result from the explicit first-page preview boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPagePreview {
    /// Rendered page-zero thumbnail.
    pub thumbnail: Thumbnail,
    /// Loader mode selected for the preview.
    pub load_mode: FirstPagePreviewLoadMode,
    /// Parser/object retention metrics for the selected preview loader.
    pub memory: FirstPagePreviewMemory,
}

/// Memory-relevant loader metrics for first-page preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstPagePreviewMemory {
    /// Total input byte length.
    pub input_bytes: usize,
    /// Number of indirect objects parsed into the document table.
    pub loaded_objects: usize,
    /// Sum of parsed indirect-object byte spans held by the document table.
    pub loaded_object_bytes: usize,
    /// Declared first-page section end for linearized inputs.
    pub first_page_section_bytes: Option<usize>,
    /// Whether the loader avoided parsing objects past the first-page section.
    pub first_page_only: bool,
}

impl FirstPagePreviewMemory {
    const fn from_load_metrics(metrics: DocumentLoadMetrics) -> Self {
        Self {
            input_bytes: metrics.input_bytes,
            loaded_objects: metrics.loaded_objects,
            loaded_object_bytes: metrics.loaded_object_bytes,
            first_page_section_bytes: metrics.first_page_end,
            first_page_only: metrics.first_page_only,
        }
    }
}

/// Rust-native renderer memory and cache budget profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRenderLimits {
    /// Maximum pixels accepted in one page raster buffer.
    pub max_page_pixels: usize,
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
    /// Maximum decoded bytes accepted for one embedded font program.
    pub max_font_program_bytes: usize,
    /// Maximum decoded bytes accepted for one ToUnicode CMap stream.
    pub max_cmap_bytes: usize,
    /// Maximum bytes accepted in one decoded text run.
    pub max_text_run_bytes: usize,
    /// Maximum display items accepted in one display list.
    pub max_display_items: usize,
    /// Maximum cached deterministic font fallback resolutions.
    pub max_font_fallback_cache_entries: usize,
    /// Maximum pixels accepted in one transparency group intermediate raster.
    pub max_transparency_group_pixels: usize,
    /// Maximum flattened path line segments accepted in one rasterization pass.
    pub max_flattened_segments: usize,
    /// Maximum repeated pattern tiles accepted in one rasterization pass.
    pub max_pattern_tiles: usize,
    /// Maximum cached tiling pattern cells in one rasterization pass.
    pub max_pattern_cell_cache_entries: usize,
    /// Whether temporary spooling is enabled for sensitive intermediates.
    pub spooling_enabled: bool,
    /// Maximum bytes allowed for temporary spooling.
    pub max_spool_bytes: usize,
}

impl NativeRenderLimits {
    /// Returns the default desktop-oriented Rust-native renderer budgets.
    #[must_use]
    pub fn default_profile() -> Self {
        Self::default()
    }

    /// Returns constrained budgets intended for low-memory thumbnail workloads.
    #[must_use]
    pub const fn low_memory() -> Self {
        Self {
            max_page_pixels: 384 * 384,
            max_image_bytes: 12 * 1024 * 1024,
            max_total_image_bytes: 24 * 1024 * 1024,
            max_icc_profile_bytes: 256 * 1024,
            max_icc_transform_workspace_bytes: 32 * 1024,
            max_icc_transform_cache_entries: 8,
            max_font_program_bytes: 4 * 1024 * 1024,
            max_cmap_bytes: 256 * 1024,
            max_text_run_bytes: 16 * 1024,
            max_display_items: 2_048,
            max_font_fallback_cache_entries: 32,
            max_transparency_group_pixels: 512 * 512,
            max_flattened_segments: 16_384,
            max_pattern_tiles: 16_384,
            max_pattern_cell_cache_entries: 8,
            spooling_enabled: false,
            max_spool_bytes: DEFAULT_SPOOL_BYTES_LIMIT,
        }
    }

    const fn memory_diagnostics(self) -> NativeMemoryDiagnostics {
        NativeMemoryDiagnostics {
            max_page_pixels: self.max_page_pixels,
            max_image_bytes: self.max_image_bytes,
            max_total_image_bytes: self.max_total_image_bytes,
            max_icc_profile_bytes: self.max_icc_profile_bytes,
            max_icc_transform_workspace_bytes: self.max_icc_transform_workspace_bytes,
            max_icc_transform_cache_entries: self.max_icc_transform_cache_entries,
            max_font_program_bytes: self.max_font_program_bytes,
            max_cmap_bytes: self.max_cmap_bytes,
            max_text_run_bytes: self.max_text_run_bytes,
            max_display_items: self.max_display_items,
            max_font_fallback_cache_entries: self.max_font_fallback_cache_entries,
            max_transparency_group_pixels: self.max_transparency_group_pixels,
            max_flattened_segments: self.max_flattened_segments,
            max_pattern_tiles: self.max_pattern_tiles,
            max_pattern_cell_cache_entries: self.max_pattern_cell_cache_entries,
            spooling_enabled: self.spooling_enabled,
            max_spool_bytes: self.max_spool_bytes,
        }
    }

    fn display_options(self) -> DisplayListOptions {
        DisplayListOptions {
            max_display_items: self.max_display_items,
            max_text_run_bytes: self.max_text_run_bytes,
            max_cmap_bytes: self.max_cmap_bytes,
            max_font_program_bytes: self.max_font_program_bytes,
            max_image_bytes: self.max_image_bytes,
            max_total_image_bytes: self.max_total_image_bytes,
            max_icc_profile_bytes: self.max_icc_profile_bytes,
            max_icc_transform_workspace_bytes: self.max_icc_transform_workspace_bytes,
            max_icc_transform_cache_entries: self.max_icc_transform_cache_entries,
            max_font_fallback_cache_entries: self.max_font_fallback_cache_entries,
            ..DisplayListOptions::default()
        }
    }

    const fn page_transform_options(self) -> PageTransformOptions {
        PageTransformOptions {
            max_page_pixels: self.max_page_pixels,
        }
    }

    fn path_raster_options(self) -> PathRasterOptions {
        PathRasterOptions {
            max_flattened_segments: self.max_flattened_segments,
            max_transparency_group_pixels: self.max_transparency_group_pixels,
            max_pattern_tiles: self.max_pattern_tiles,
            max_pattern_cell_cache_entries: self.max_pattern_cell_cache_entries,
            ..PathRasterOptions::default()
        }
    }
}

impl Default for NativeRenderLimits {
    fn default() -> Self {
        let display = DisplayListOptions::default();
        let page = PageTransformOptions::default();
        let path = PathRasterOptions::default();
        Self {
            max_page_pixels: page.max_page_pixels,
            max_image_bytes: display.max_image_bytes,
            max_total_image_bytes: display.max_total_image_bytes,
            max_icc_profile_bytes: display.max_icc_profile_bytes,
            max_icc_transform_workspace_bytes: display.max_icc_transform_workspace_bytes,
            max_icc_transform_cache_entries: display.max_icc_transform_cache_entries,
            max_font_program_bytes: display.max_font_program_bytes,
            max_cmap_bytes: display.max_cmap_bytes,
            max_text_run_bytes: display.max_text_run_bytes,
            max_display_items: display.max_display_items,
            max_font_fallback_cache_entries: display.max_font_fallback_cache_entries,
            max_transparency_group_pixels: path.max_transparency_group_pixels,
            max_flattened_segments: path.max_flattened_segments,
            max_pattern_tiles: path.max_pattern_tiles,
            max_pattern_cell_cache_entries: path.max_pattern_cell_cache_entries,
            spooling_enabled: false,
            max_spool_bytes: DEFAULT_SPOOL_BYTES_LIMIT,
        }
    }
}

/// Default Rust-native renderer memory and cache budget diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMemoryDiagnostics {
    /// Maximum pixels accepted in one page raster buffer.
    pub max_page_pixels: usize,
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
    /// Maximum decoded bytes accepted for one embedded font program.
    pub max_font_program_bytes: usize,
    /// Maximum decoded bytes accepted for one ToUnicode CMap stream.
    pub max_cmap_bytes: usize,
    /// Maximum bytes accepted in one decoded text run.
    pub max_text_run_bytes: usize,
    /// Maximum display items accepted in one display list.
    pub max_display_items: usize,
    /// Maximum cached deterministic font fallback resolutions.
    pub max_font_fallback_cache_entries: usize,
    /// Maximum pixels accepted in one transparency group intermediate raster.
    pub max_transparency_group_pixels: usize,
    /// Maximum flattened path line segments accepted in one rasterization pass.
    pub max_flattened_segments: usize,
    /// Maximum repeated pattern tiles accepted in one rasterization pass.
    pub max_pattern_tiles: usize,
    /// Maximum cached tiling pattern cells in one rasterization pass.
    pub max_pattern_cell_cache_entries: usize,
    /// Whether temporary spooling is enabled for sensitive intermediates.
    pub spooling_enabled: bool,
    /// Maximum bytes allowed for temporary spooling.
    pub max_spool_bytes: usize,
}

/// Native page artifact cache policy.
///
/// The current renderer keeps reusable state scoped to a single render pass.
/// Callers that need longer-lived page artifacts should key them with
/// [`NativePageCacheKey`] and keep ownership outside the backend until the
/// renderer grows a document-session cache with explicit lifetime boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePageCachePolicy {
    /// Every render owns its decoded resources and pass-local caches.
    IsolatedRender,
}

impl NativePageCachePolicy {
    /// Returns the stable policy identifier used in benchmark reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IsolatedRender => "isolated-render",
        }
    }

    /// Returns whether the policy permits writing document-derived artifacts to
    /// disk without an explicit caller-managed opt-in.
    #[must_use]
    pub const fn permits_disk_persistence(self) -> bool {
        match self {
            Self::IsolatedRender => false,
        }
    }
}

/// Versioned key shape for caller-owned reusable native page artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativePageCacheKey {
    /// Caller-provided document identity, usually a content hash or a
    /// tenant-scoped document version id.
    pub document_identity: u64,
    /// Zero-based page index.
    pub page_index: u32,
    /// Maximum output edge in pixels.
    pub max_edge: u32,
    /// Background color encoded as RGBA bytes.
    pub background: [u8; 4],
    /// Native backend version that produced the artifact.
    pub renderer_version: &'static str,
    /// Native memory/profile identifier.
    pub native_profile: &'static str,
    /// Annotation visibility mode identifier.
    pub annotation_mode: &'static str,
    /// AcroForm appearance-state mode identifier.
    pub form_appearance_mode: &'static str,
}

impl NativePageCacheKey {
    /// Builds a key from the render options that influence page raster output.
    #[must_use]
    pub fn from_options(
        document_identity: u64,
        options: &ThumbnailOptions,
        native_profile: &'static str,
    ) -> Self {
        Self {
            document_identity,
            page_index: options.page_index,
            max_edge: options.max_edge,
            background: [
                options.background.r,
                options.background.g,
                options.background.b,
                options.background.a,
            ],
            renderer_version: env!("CARGO_PKG_VERSION"),
            native_profile,
            annotation_mode: options.annotation_mode.as_str(),
            form_appearance_mode: options.form_appearance_mode.as_str(),
        }
    }
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

/// Cooperative cancellation flag for multi-page native rendering.
#[derive(Debug, Default)]
pub struct RenderCancellation {
    cancelled: AtomicBool,
}

impl RenderCancellation {
    /// Creates a non-cancelled token.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
        }
    }

    /// Requests cancellation of future page scheduling.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns true when cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
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

/// Per-page multi-page render outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelPageResult {
    /// Requested zero-based page index.
    pub page_index: u32,
    /// Page thumbnail or stable page-level render error.
    pub result: Result<Thumbnail, ThumbnailError>,
}

/// Partial multi-page render result preserving page-level outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelRenderPartialResult {
    /// Page results in the same order as requested page indices that were
    /// scheduled before cancellation.
    pub pages: Vec<ParallelPageResult>,
    /// Effective worker count after applying worker and memory budgets.
    pub workers: usize,
    /// True when cancellation stopped scheduling before all requests ran.
    pub cancelled: bool,
}

impl Default for NativeMemoryDiagnostics {
    fn default() -> Self {
        NativeRenderLimits::default().memory_diagnostics()
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
    let cancellation = RenderCancellation::new();
    let partial = render_pages_parallel_partial(
        source,
        page_indices,
        options,
        parallel_options,
        &cancellation,
    )?;
    let workers = partial.workers;
    let mut pages = Vec::with_capacity(partial.pages.len());
    for page in partial.pages {
        pages.push(page.result?);
    }

    Ok(ParallelRenderResult { pages, workers })
}

/// Renders multiple pages while preserving page-level success and error status.
///
/// Cancellation is cooperative and checked before each worker batch is
/// scheduled. Already-started page jobs are allowed to finish and keep their
/// page-level status.
///
/// # Errors
///
/// Returns [`ThumbnailError`] when the input cannot be loaded, the scheduler
/// configuration is invalid, or a memory budget prevents even one page from
/// being scheduled.
pub fn render_pages_parallel_partial(
    source: PdfSource<'_>,
    page_indices: &[u32],
    options: &ThumbnailOptions,
    parallel_options: ParallelRenderOptions,
    cancellation: &RenderCancellation,
) -> Result<ParallelRenderPartialResult, ThumbnailError> {
    render_pages_parallel_partial_with_limits(
        source,
        page_indices,
        options,
        parallel_options,
        cancellation,
        NativeRenderLimits::default(),
    )
}

fn render_pages_parallel_partial_with_limits(
    source: PdfSource<'_>,
    page_indices: &[u32],
    options: &ThumbnailOptions,
    parallel_options: ParallelRenderOptions,
    cancellation: &RenderCancellation,
    limits: NativeRenderLimits,
) -> Result<ParallelRenderPartialResult, ThumbnailError> {
    reject_form_appearance_mutation(options)?;
    let worker_count = effective_worker_count(options, parallel_options)?;
    if cancellation.is_cancelled() {
        return Ok(ParallelRenderPartialResult {
            pages: Vec::new(),
            workers: worker_count,
            cancelled: true,
        });
    }
    let source_bytes = load_source(source)?;
    let bytes = source_bytes.as_ref();
    let mut pages = Vec::with_capacity(page_indices.len());
    let mut cancelled = false;
    if page_indices.is_empty() {
        return Ok(ParallelRenderPartialResult {
            pages,
            workers: worker_count,
            cancelled,
        });
    }
    let input = PdfBytes::new(bytes);
    let (document, page_tree) = load_render_document_for_page_set(input, page_indices)?;

    for chunk in page_indices.chunks(worker_count) {
        if cancellation.is_cancelled() {
            cancelled = true;
            break;
        }
        let batch = thread::scope(|scope| {
            let document_ref = &document;
            let page_tree_ref = &page_tree;
            let handles = chunk
                .iter()
                .copied()
                .map(|page_index| {
                    scope.spawn(move || {
                        let mut page_options = *options;
                        page_options.page_index = page_index;
                        render_loaded_document(document_ref, page_tree_ref, &page_options, limits)
                    })
                })
                .collect::<Vec<_>>();

            handles
                .into_iter()
                .zip(chunk.iter().copied())
                .map(|(handle, page_index)| {
                    let result = handle
                        .join()
                        .map_err(|_| ThumbnailError::internal("parallel render worker panicked"))?;
                    Ok(ParallelPageResult { page_index, result })
                })
                .collect::<Result<Vec<_>, ThumbnailError>>()
        })?;
        pages.extend(batch);
    }

    Ok(ParallelRenderPartialResult {
        pages,
        workers: worker_count,
        cancelled,
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
        reject_form_appearance_mutation(options)?;
        let bytes = load_source(source)?;
        render_bytes(&bytes, options, self.limits)
    }
}

fn reject_form_appearance_mutation(options: &ThumbnailOptions) -> Result<(), ThumbnailError> {
    if options.form_appearance_mode.requires_mutation() {
        return Err(unsupported_feature(BUCKET_FORM_APPEARANCE_MUTATION));
    }
    Ok(())
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

impl TextExtractionBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        Self::backend_name(self)
    }

    fn extract_text(
        &self,
        source: PdfSource<'_>,
        options: &TextExtractionOptions,
    ) -> Result<PageText, ThumbnailError> {
        let bytes = load_source(source)?;
        extract_text_bytes(&bytes, options, self.limits)
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

fn extract_text_bytes(
    bytes: &[u8],
    options: &TextExtractionOptions,
    limits: NativeRenderLimits,
) -> Result<PageText, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    let (document, page_tree) = load_render_document(input, options.page_index)?;
    enforce_xfa_render_policy(&document)?;
    let page = page_tree
        .pages()
        .get(options.page_index as usize)
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_UNSUPPORTED))?;
    let content = page_content_stream(&document, page)?;
    let optional_content = page_optional_content_properties(&document, page)?;
    let optional_content_state = document_optional_content_state(&document)?;
    let content = filter_optional_content(&content, &optional_content, &optional_content_state)?;
    let font_resources = page_font_resources(&document, page, limits.display_options())?;
    let text_list = build_text_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &font_resources,
        limits.display_options(),
    )
    .map_err(map_graphics_error)?;

    let mut runs = Vec::new();
    let mut glyph_count = 0usize;
    let mut truncated = false;

    for item in text_list.items() {
        let DisplayItem::Text(text) = item else {
            continue;
        };
        if runs.len() >= options.max_runs {
            truncated = true;
            break;
        }

        let visible = text.rendering_mode.paints_pixels();
        let remaining_glyphs = options.max_glyphs.saturating_sub(glyph_count);
        let glyphs = text
            .glyphs
            .iter()
            .zip(text.glyph_origins.iter())
            .take(remaining_glyphs)
            .map(|(glyph, origin)| PositionedGlyph {
                text: glyph.unicode.clone(),
                origin: TextPoint {
                    x: origin.x,
                    y: origin.y,
                },
                visible,
            })
            .collect::<Vec<_>>();

        if glyphs.len() < text.glyphs.len() {
            truncated = true;
        }
        glyph_count += glyphs.len();
        runs.push(TextRun {
            text: text.text.clone(),
            glyphs,
            origin: TextPoint {
                x: text.origin.x,
                y: text.origin.y,
            },
            font_size: text.font_size,
            visible,
        });

        if truncated {
            break;
        }
    }

    Ok(PageText {
        page_index: options.page_index,
        runs,
        truncated,
    })
}

fn inspect_bytes(bytes: &[u8]) -> Result<DocumentMetadata, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    match load_modern_document(input).and_then(|document| document.page_tree()) {
        Ok(page_tree) => metadata_from_page_tree(&page_tree),
        Err(ObjectError::Encrypted) => Err(ThumbnailError::Encrypted),
        Err(_) => {
            let document = load_classic_document(input).map_err(map_object_error)?;
            metadata_from_classic_document(&document)
        }
    }
}

fn render_bytes(
    bytes: &[u8],
    options: &ThumbnailOptions,
    limits: NativeRenderLimits,
) -> Result<Thumbnail, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    let (document, page_tree) = load_render_document(input, options.page_index)?;
    render_loaded_document(&document, &page_tree, options, limits)
}

fn render_loaded_document(
    document: &ClassicDocument<'_>,
    page_tree: &PageTree,
    options: &ThumbnailOptions,
    limits: NativeRenderLimits,
) -> Result<Thumbnail, ThumbnailError> {
    enforce_xfa_render_policy(document)?;
    let page = page_tree
        .pages()
        .get(options.page_index as usize)
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_UNSUPPORTED))?;
    let content = page_content_stream(document, page)?;
    let optional_content = page_optional_content_properties(document, page)?;
    let optional_content_state = document_optional_content_state(document)?;
    let content = filter_optional_content(&content, &optional_content, &optional_content_state)?;
    let xobject_invocations = xobject_invocation_names(&content)?;
    let display_options = limits.display_options();
    let path_options = limits.path_raster_options();
    let ext_graphics_states = page_ext_graphics_state_resources(document, page)?;
    let shadings = page_shading_resources(document, page, display_options)?;
    let patterns = page_tiling_pattern_resources(document, page, display_options)?;
    let color_spaces = page_color_space_resources(document, page)?;
    let display_list = build_path_display_list_with_graphics_resources(
        tokenize_content(PdfBytes::new(&content)),
        &ext_graphics_states,
        &shadings,
        &patterns,
        &color_spaces,
        display_options,
    )
    .map_err(map_graphics_error)?;
    let transform = PageTransform::new_with_options(
        page_geometry(*page),
        options.max_edge,
        limits.page_transform_options(),
    )
    .map_err(map_raster_error)?;
    let form_resources = page_form_resources(document, page, &xobject_invocations)?;
    let form_list = build_form_display_list_with_graphics_resources(
        tokenize_content(PdfBytes::new(&content)),
        &form_resources,
        &ext_graphics_states,
        &shadings,
        &patterns,
        &color_spaces,
        display_options,
    )
    .map_err(map_graphics_error)?;
    let image_resources =
        page_image_resources(document, page, &xobject_invocations, display_options)?;
    let image_list = build_image_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &image_resources,
        display_options,
    )
    .map_err(map_graphics_error)?;
    let font_resources = page_font_resources(document, page, display_options)?;
    let text_list = build_text_display_list(
        tokenize_content(PdfBytes::new(&content)),
        &font_resources,
        display_options,
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
        rasterize_display_list_into(&ordered_list, &mut raster, transform, path_options)
            .map_err(map_raster_error)?;
    } else {
        rasterize_paths_into(&display_list, &mut raster, transform, path_options)
            .map_err(map_raster_error)?;
        rasterize_paths_into(&form_list, &mut raster, transform, path_options)
            .map_err(map_raster_error)?;
        rasterize_images(&image_list, &mut raster, transform).map_err(map_raster_error)?;
        rasterize_text(&text_list, &mut raster, transform).map_err(map_raster_error)?;
    }
    let (annotation_forms, annotation_content, annotation_fallback_content) =
        page_annotation_appearance_resources(document, page, options.annotation_mode)?;
    if !annotation_content.is_empty() {
        let annotation_list = build_form_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(&annotation_content)),
            &annotation_forms,
            &ext_graphics_states,
            &shadings,
            &patterns,
            &color_spaces,
            display_options,
        )
        .map_err(map_graphics_error)?;
        rasterize_paths_into(&annotation_list, &mut raster, transform, path_options)
            .map_err(map_raster_error)?;
    }
    if !annotation_fallback_content.is_empty() {
        let annotation_ext_graphics_states =
            annotation_fallback_ext_graphics_states().map_err(map_graphics_error)?;
        let annotation_list = build_path_display_list_with_graphics_resources(
            tokenize_content(PdfBytes::new(&annotation_fallback_content)),
            &annotation_ext_graphics_states,
            &ShadingResources::empty(),
            &TilingPatternResources::empty(),
            &ColorSpaceResources::empty(),
            display_options,
        )
        .map_err(map_graphics_error)?;
        rasterize_paths_into(&annotation_list, &mut raster, transform, path_options)
            .map_err(map_raster_error)?;
        match build_text_display_list(
            tokenize_content(PdfBytes::new(&annotation_fallback_content)),
            &font_resources,
            display_options,
        ) {
            Ok(annotation_text_list) => {
                rasterize_text(&annotation_text_list, &mut raster, transform)
                    .map_err(map_raster_error)?;
            }
            Err(error) if is_ignorable_annotation_fallback_text_error(&error) => {}
            Err(error) => return Err(map_graphics_error(error)),
        }
    }
    let dimensions = raster.dimensions();
    Thumbnail::rgba(dimensions.width, dimensions.height, raster.into_pixels())
}

/// Scans native renderer operator coverage for one document page.
///
/// The scan uses the same document loading and content decoding boundary as
/// native rendering, but it only tokenizes content streams and records operator
/// usage. It does not rasterize or expand image samples.
///
/// # Errors
///
/// Returns [`ThumbnailError`] when the document cannot be loaded, the page is
/// unavailable, or a scanned content stream is malformed.
pub fn scan_operator_coverage(
    bytes: &[u8],
    options: OperatorCoverageOptions,
) -> Result<OperatorCoverageReport, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    let (document, page_tree) = load_render_document(input, options.page_index)?;
    let page = page_tree
        .pages()
        .get(options.page_index as usize)
        .ok_or_else(|| unsupported_feature(BUCKET_RENDERER_UNSUPPORTED))?;
    let mut scanner = OperatorCoverageScanner::default();

    let content = page_content_stream(&document, page)?;
    scanner.scan_stream(&content)?;

    if options.include_annotations {
        let (_, annotation_content, annotation_fallback_content) =
            page_annotation_appearance_resources(&document, page, AnnotationMode::Screen)?;
        if !annotation_content.is_empty() {
            scanner.scan_stream(&annotation_content)?;
        }
        if !annotation_fallback_content.is_empty() {
            scanner.scan_stream(&annotation_fallback_content)?;
        }
    }

    Ok(scanner.finish(options.page_index))
}

#[derive(Default)]
struct OperatorCoverageScanner {
    streams_scanned: usize,
    total_operators: usize,
    inline_images: usize,
    operators: BTreeMap<String, OperatorCoverageAccumulator>,
}

#[derive(Debug, Clone, Copy)]
struct OperatorCoverageAccumulator {
    count: usize,
    status: OperatorSupportStatus,
    fallback_bucket: Option<&'static str>,
}

impl OperatorCoverageScanner {
    fn scan_stream(&mut self, content: &[u8]) -> Result<(), ThumbnailError> {
        self.streams_scanned += 1;
        for token in tokenize_content(PdfBytes::new(content)) {
            match token.map_err(|_| ThumbnailError::Malformed)? {
                ContentToken::Operator { name, .. } => {
                    self.record(name.as_bytes());
                }
                ContentToken::InlineImage { .. } => {
                    self.inline_images += 1;
                    self.record(b"BI");
                }
                ContentToken::Operand { .. } => {}
            }
        }
        Ok(())
    }

    fn record(&mut self, operator: &[u8]) {
        let (status, fallback_bucket) = classify_operator_support(operator);
        let entry = self
            .operators
            .entry(operator_name_string(operator))
            .or_insert(OperatorCoverageAccumulator {
                count: 0,
                status,
                fallback_bucket,
            });
        entry.count += 1;
        self.total_operators += 1;
    }

    fn finish(self, page_index: u32) -> OperatorCoverageReport {
        let operators = self
            .operators
            .into_iter()
            .map(|(operator, entry)| OperatorCoverageEntry {
                operator,
                count: entry.count,
                status: entry.status,
                fallback_bucket: entry.fallback_bucket,
            })
            .collect();
        OperatorCoverageReport {
            page_index,
            streams_scanned: self.streams_scanned,
            total_operators: self.total_operators,
            inline_images: self.inline_images,
            operators,
        }
    }
}

fn operator_name_string(operator: &[u8]) -> String {
    String::from_utf8_lossy(operator).into_owned()
}

fn classify_operator_support(operator: &[u8]) -> (OperatorSupportStatus, Option<&'static str>) {
    match operator {
        b"q" | b"Q" | b"cm" | b"w" | b"J" | b"j" | b"M" | b"d" | b"g" | b"G" | b"rg" | b"RG"
        | b"m" | b"l" | b"c" | b"h" | b"re" | b"S" | b"s" | b"f" | b"F" | b"f*" | b"B" | b"B*"
        | b"n" | b"BT" | b"ET" | b"Tf" | b"Tc" | b"Tw" | b"Tz" | b"Tr" | b"Td" | b"Tm" | b"Tj"
        | b"TJ" | b"Do" | b"BI" => (OperatorSupportStatus::Implemented, None),
        b"W" | b"W*" => (
            OperatorSupportStatus::Partial,
            Some(BUCKET_GRAPHICS_STROKE_CLIP),
        ),
        b"cs" | b"CS" | b"sc" | b"scn" | b"SC" | b"SCN" => (
            OperatorSupportStatus::Partial,
            Some(BUCKET_IMAGE_COLOR_SPACE),
        ),
        b"gs" => (
            OperatorSupportStatus::Partial,
            Some(BUCKET_GRAPHICS_TRANSPARENCY),
        ),
        b"sh" => (
            OperatorSupportStatus::Partial,
            Some(BUCKET_GRAPHICS_PATTERN_SHADING),
        ),
        b"v" | b"y" | b"b" | b"b*" => (
            OperatorSupportStatus::Unsupported,
            Some(BUCKET_GRAPHICS_STROKE_CLIP),
        ),
        b"T*" | b"TD" | b"TL" | b"Ts" | b"'" | b"\"" => (
            OperatorSupportStatus::Unsupported,
            Some(BUCKET_TEXT_FONT_PROGRAM),
        ),
        b"K" | b"k" => (
            OperatorSupportStatus::Unsupported,
            Some(BUCKET_IMAGE_COLOR_SPACE),
        ),
        b"MP" | b"DP" | b"BMC" | b"BDC" | b"EMC" | b"BX" | b"EX" => {
            (OperatorSupportStatus::Ignored, None)
        }
        _ => (
            OperatorSupportStatus::Unsupported,
            Some(BUCKET_RENDERER_UNSUPPORTED),
        ),
    }
}

fn load_render_document(
    input: PdfBytes<'_>,
    page_index: u32,
) -> Result<(ClassicDocument<'_>, PageTree), ThumbnailError> {
    if page_index == 0 {
        if let Ok(document) = load_linearized_first_page_document(input) {
            if let Ok(Some(page_tree)) = document.linearized_first_page_tree() {
                return Ok((document, page_tree));
            }
        }
    }

    let document = load_classic_document(input).map_err(map_object_error)?;
    let page_tree = document.page_tree().map_err(map_object_error)?;
    Ok((document, page_tree))
}

fn load_render_document_for_page_set<'a>(
    input: PdfBytes<'a>,
    page_indices: &[u32],
) -> Result<(ClassicDocument<'a>, PageTree), ThumbnailError> {
    if let [page_index] = page_indices {
        return load_render_document(input, *page_index);
    }

    let document = load_classic_document(input).map_err(map_object_error)?;
    let page_tree = document.page_tree().map_err(map_object_error)?;
    Ok((document, page_tree))
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
    options: DisplayListOptions,
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
    ShadingResources::from_shading_dictionary_with_resolver(shadings, document, options)
        .map_err(map_graphics_error)
}

fn page_color_space_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
) -> Result<ColorSpaceResources, ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(ColorSpaceResources::empty());
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
    let Some(PdfPrimitive::Dictionary(color_spaces)) =
        dictionary_value(resource_dictionary, b"ColorSpace")
    else {
        return Ok(ColorSpaceResources::empty());
    };
    ColorSpaceResources::from_color_space_dictionary(color_spaces).map_err(map_graphics_error)
}

fn page_tiling_pattern_resources(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
    options: DisplayListOptions,
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
                options,
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
    options: DisplayListOptions,
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
    ImageResources::from_xobject_dictionary(xobjects.as_slice(), document, options)
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
    mode: AnnotationMode,
) -> Result<(FormResources, Vec<u8>, Vec<u8>), ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(annots) = dictionary_value(dictionary, b"Annots") else {
        return Ok((FormResources::empty(), Vec::new(), Vec::new()));
    };
    let annotations = annotation_array(document, annots)?;
    let mut names = Vec::new();
    let mut references = Vec::new();
    let mut rects = Vec::new();
    let mut fallback_content = Vec::new();

    for annotation in annotations {
        let Some(dictionary) = annotation_dictionary(document, annotation)? else {
            continue;
        };
        if !annotation_visible_in_mode(dictionary, mode) {
            continue;
        }
        let Some(reference) = normal_appearance_reference(dictionary) else {
            append_annotation_fallback(&mut fallback_content, dictionary)?;
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
        return Ok((FormResources::empty(), Vec::new(), fallback_content));
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
    Ok((resources, content, fallback_content))
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

fn optional_content_metadata(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
    page_tree: &PageTree,
) -> Result<OptionalContentMetadata, ThumbnailError> {
    let mut metadata = OptionalContentMetadata::default();
    let Some(PdfPrimitive::Dictionary(properties)) = dictionary_value(catalog, b"OCProperties")
    else {
        return Ok(metadata);
    };
    metadata.has_oc_properties = true;
    metadata.group_count = optional_content_group_count(properties)?;

    if let Some(PdfPrimitive::Dictionary(default_config)) = dictionary_value(properties, b"D") {
        metadata.has_default_configuration = true;
        metadata.base_state = optional_content_base_state(default_config)?;
        metadata.default_on_count = optional_content_reference_array(default_config, b"ON")?.len();
        metadata.default_off_count =
            optional_content_reference_array(default_config, b"OFF")?.len();
        metadata.has_usage_application = dictionary_value(default_config, b"AS").is_some();
    } else if dictionary_value(properties, b"D").is_some() {
        return Err(ThumbnailError::Malformed);
    }

    for page in page_tree.pages() {
        classify_page_optional_content(document, page, &mut metadata)?;
    }
    metadata.has_unsupported_behavior = metadata.has_usage_application
        || metadata.has_unsupported_membership_policy
        || metadata.has_direct_group_dictionary;
    Ok(metadata)
}

fn optional_content_group_count(
    properties: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<usize, ThumbnailError> {
    let Some(groups) = dictionary_value(properties, b"OCGs") else {
        return Ok(0);
    };
    let PdfPrimitive::Array(groups) = groups else {
        return Err(ThumbnailError::Malformed);
    };
    for group in groups {
        match group {
            PdfPrimitive::Reference(_) | PdfPrimitive::Dictionary(_) => {}
            _ => return Err(ThumbnailError::Malformed),
        }
    }
    Ok(groups.len())
}

fn optional_content_base_state(
    default_config: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<OptionalContentBaseState, ThumbnailError> {
    match dictionary_value(default_config, b"BaseState") {
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"ON" => {
            Ok(OptionalContentBaseState::On)
        }
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"OFF" => {
            Ok(OptionalContentBaseState::Off)
        }
        Some(PdfPrimitive::Name(name)) if name.as_bytes() == b"Unchanged" => {
            Ok(OptionalContentBaseState::Unchanged)
        }
        Some(_) => Err(ThumbnailError::Malformed),
        None => Ok(OptionalContentBaseState::Unspecified),
    }
}

fn classify_page_optional_content(
    document: &ClassicDocument<'_>,
    page: &ObjectPageMetadata,
    metadata: &mut OptionalContentMetadata,
) -> Result<(), ThumbnailError> {
    let object = document
        .objects
        .get(page.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    let Some(resources) = dictionary_value(dictionary, b"Resources") else {
        return Ok(());
    };
    let resource_dictionary = resource_dictionary(document, resources)?;
    let Some(PdfPrimitive::Dictionary(properties)) =
        dictionary_value(resource_dictionary, b"Properties")
    else {
        return Ok(());
    };
    for (_, value) in properties {
        classify_optional_content_property(document, value, metadata)?;
    }
    Ok(())
}

fn classify_optional_content_property(
    document: &ClassicDocument<'_>,
    value: &PdfPrimitive<'_>,
    metadata: &mut OptionalContentMetadata,
) -> Result<(), ThumbnailError> {
    match value {
        PdfPrimitive::Reference(reference) => {
            let reference_id = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference_id.id)
                .ok_or(ThumbnailError::Malformed)?;
            let dictionary = object_dictionary(&object.value)?;
            classify_optional_content_dictionary(dictionary, metadata)
        }
        PdfPrimitive::Dictionary(dictionary) => {
            if dictionary_name_is(dictionary, b"Type", b"OCG") {
                metadata.has_direct_group_dictionary = true;
                return Ok(());
            }
            classify_optional_content_dictionary(dictionary, metadata)
        }
        _ => Err(ThumbnailError::Malformed),
    }
}

fn classify_optional_content_dictionary(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    metadata: &mut OptionalContentMetadata,
) -> Result<(), ThumbnailError> {
    if dictionary_name_is(dictionary, b"Type", b"OCG") {
        return Ok(());
    }
    if dictionary_name_is(dictionary, b"Type", b"OCMD") {
        metadata.has_unsupported_membership_policy = true;
        return Ok(());
    }
    Err(ThumbnailError::Malformed)
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

fn enforce_xfa_render_policy(document: &ClassicDocument<'_>) -> Result<(), ThumbnailError> {
    let catalog = document_catalog_dictionary(document)?;
    let Some(acroform_value) = dictionary_value(catalog, b"AcroForm") else {
        return Ok(());
    };
    let acroform = resource_dictionary(document, acroform_value)?;
    if dictionary_value(acroform, b"XFA").is_none() {
        return Ok(());
    }
    if acroform_has_static_fields(document, acroform)? {
        return Ok(());
    }
    Err(unsupported_feature(BUCKET_FORM_XFA_DYNAMIC))
}

fn acroform_has_static_fields(
    document: &ClassicDocument<'_>,
    acroform: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<bool, ThumbnailError> {
    let Some(fields) = dictionary_value(acroform, b"Fields") else {
        return Ok(false);
    };
    match fields {
        PdfPrimitive::Array(items) => Ok(!items.is_empty()),
        PdfPrimitive::Reference(reference) => {
            let reference = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            let ObjectValue::Primitive(PdfPrimitive::Array(items)) = &object.value else {
                return Err(ThumbnailError::Malformed);
            };
            Ok(!items.is_empty())
        }
        _ => Err(ThumbnailError::Malformed),
    }
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
    options: DisplayListOptions,
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
    FontResources::from_font_dictionary(fonts, document, options).map_err(map_graphics_error)
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

const ANNOTATION_FLAG_INVISIBLE: i64 = 1 << 0;
const ANNOTATION_FLAG_HIDDEN: i64 = 1 << 1;
const ANNOTATION_FLAG_PRINT: i64 = 1 << 2;
const ANNOTATION_FLAG_NO_VIEW: i64 = 1 << 5;

fn annotation_visible_in_mode(
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    mode: AnnotationMode,
) -> bool {
    let flags = annotation_flags(annotation);
    if flags & (ANNOTATION_FLAG_INVISIBLE | ANNOTATION_FLAG_HIDDEN) != 0 {
        return false;
    }
    match mode {
        AnnotationMode::Screen => flags & ANNOTATION_FLAG_NO_VIEW == 0,
        AnnotationMode::Print => flags & ANNOTATION_FLAG_PRINT != 0,
    }
}

fn annotation_flags(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> i64 {
    match dictionary_value(annotation, b"F") {
        Some(PdfPrimitive::Number(PdfNumber::Integer(flags))) => *flags,
        _ => 0,
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

fn append_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<(), ThumbnailError> {
    let Some(PdfPrimitive::Name(subtype)) = dictionary_value(annotation, b"Subtype") else {
        return Ok(());
    };
    let Some(rect) = annotation_rect(annotation).filter(valid_annotation_bounds) else {
        return Ok(());
    };
    match subtype.as_bytes() {
        b"Highlight" => append_highlight_annotation_fallback(content, annotation, rect),
        b"Underline" => append_underline_annotation_fallback(content, annotation, rect),
        b"Square" => append_square_annotation_fallback(content, annotation, rect),
        b"Circle" => append_circle_annotation_fallback(content, annotation, rect),
        b"Text" => append_text_note_annotation_fallback(content, annotation, rect),
        b"Widget" => append_widget_annotation_fallback(content, annotation, rect),
        b"FreeText" => return Err(unsupported_feature(BUCKET_ANNOTATION_APPEARANCE)),
        b"Link" => {}
        _ => {}
    }
    Ok(())
}

fn valid_annotation_bounds(bounds: &PathBounds) -> bool {
    bounds.max_x > bounds.min_x && bounds.max_y > bounds.min_y
}

fn append_highlight_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let color = annotation_color(annotation, [1.0, 1.0, 0.0]);
    append_annotation_graphics_state(content, ANNOTATION_OPAQUE_GRAPHICS_STATE, color, true);
    let quads = annotation_quad_points(annotation);
    if quads.is_empty() {
        append_fill_rect(content, rect);
    } else {
        for quad in quads {
            append_fill_quad(content, quad);
        }
    }
    content.extend_from_slice(b"Q\n");
}

fn append_underline_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let color = annotation_color(annotation, [1.0, 0.0, 0.0]);
    append_annotation_graphics_state(content, ANNOTATION_UNDERLINE_GRAPHICS_STATE, color, false);
    let quads = annotation_quad_points(annotation);
    if quads.is_empty() {
        append_stroked_line(
            content,
            rect.min_x,
            rect.min_y + 1.0,
            rect.max_x,
            rect.min_y + 1.0,
        );
    } else {
        for quad in quads {
            let min_x = quad[0].min(quad[2]).min(quad[4]).min(quad[6]);
            let max_x = quad[0].max(quad[2]).max(quad[4]).max(quad[6]);
            let min_y = quad[1].min(quad[3]).min(quad[5]).min(quad[7]);
            append_stroked_line(content, min_x, min_y + 1.0, max_x, min_y + 1.0);
        }
    }
    content.extend_from_slice(b"Q\n");
}

fn append_square_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let color = annotation_color(annotation, [0.85, 0.0, 0.0]);
    append_annotation_graphics_state(content, ANNOTATION_OPAQUE_GRAPHICS_STATE, color, false);
    content.extend_from_slice(
        format!(
            "{} {} {} {} re S Q\n",
            format_pdf_number(rect.min_x),
            format_pdf_number(rect.min_y),
            format_pdf_number(rect.max_x - rect.min_x),
            format_pdf_number(rect.max_y - rect.min_y)
        )
        .as_bytes(),
    );
}

fn append_circle_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let color = annotation_color(annotation, [0.85, 0.0, 0.0]);
    let center_x = (rect.min_x + rect.max_x) * 0.5;
    let center_y = (rect.min_y + rect.max_y) * 0.5;
    let radius_x = (rect.max_x - rect.min_x) * 0.5;
    let radius_y = (rect.max_y - rect.min_y) * 0.5;

    append_annotation_graphics_state(content, ANNOTATION_OPAQUE_GRAPHICS_STATE, color, false);
    append_ellipse_polyline(content, center_x, center_y, radius_x, radius_y);
    content.extend_from_slice(b"S Q\n");
}

fn append_text_note_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let icon = text_note_icon_bounds(rect);
    let body = text_note_body_bounds(icon);
    let color = annotation_color(annotation, [1.0, 0.85, 0.0]);
    append_annotation_graphics_state(content, ANNOTATION_OPAQUE_GRAPHICS_STATE, color, true);
    append_fill_rect(content, body);
    append_text_note_tail(content, icon);
    content.extend_from_slice(b"Q\n");
    append_text_note_icon_rules(content, body);
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [0.0, 0.0, 0.0],
        true,
    );
    append_text_note_icon_border(content, icon, body);
    content.extend_from_slice(b"Q\n");
}

fn append_widget_annotation_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let Some(PdfPrimitive::Name(field_type)) = dictionary_value(annotation, b"FT") else {
        return;
    };
    match field_type.as_bytes() {
        b"Tx" => append_text_widget_fallback(content, annotation, rect),
        b"Ch" => append_text_widget_fallback(content, annotation, rect),
        b"Btn" if widget_button_is_radio(annotation) => {
            append_radio_widget_fallback(content, annotation, rect);
        }
        b"Btn" => append_checkbox_widget_fallback(content, annotation, rect),
        _ => {}
    }
}

fn append_text_widget_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [0.92, 0.96, 1.0],
        true,
    );
    append_fill_rect(content, rect);
    content.extend_from_slice(b"Q\n");
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [0.0, 0.0, 0.0],
        false,
    );
    content.extend_from_slice(
        format!(
            "1 w {} {} {} {} re S Q\n",
            format_pdf_number(rect.min_x),
            format_pdf_number(rect.min_y),
            format_pdf_number(rect.max_x - rect.min_x),
            format_pdf_number(rect.max_y - rect.min_y)
        )
        .as_bytes(),
    );
    if let Some(value) = widget_text_value(annotation) {
        append_widget_text_value(content, rect, value);
    }
}

fn append_checkbox_widget_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [1.0, 1.0, 1.0],
        true,
    );
    append_fill_rect(content, rect);
    content.extend_from_slice(b"Q\n");
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [0.0, 0.0, 0.0],
        false,
    );
    content.extend_from_slice(
        format!(
            "1 w {} {} {} {} re S Q\n",
            format_pdf_number(rect.min_x),
            format_pdf_number(rect.min_y),
            format_pdf_number(rect.max_x - rect.min_x),
            format_pdf_number(rect.max_y - rect.min_y)
        )
        .as_bytes(),
    );
    if widget_button_is_on(annotation) {
        let inset_x = (rect.max_x - rect.min_x) * 0.3;
        let inset_y = (rect.max_y - rect.min_y) * 0.3;
        append_annotation_graphics_state(
            content,
            ANNOTATION_OPAQUE_GRAPHICS_STATE,
            [0.0, 0.0, 0.0],
            true,
        );
        append_fill_rect(
            content,
            PathBounds {
                min_x: rect.min_x + inset_x,
                min_y: rect.min_y + inset_y,
                max_x: rect.max_x - inset_x,
                max_y: rect.max_y - inset_y,
            },
        );
        content.extend_from_slice(b"Q\n");
    }
}

fn append_radio_widget_fallback(
    content: &mut Vec<u8>,
    annotation: &[(PdfName<'_>, PdfPrimitive<'_>)],
    rect: PathBounds,
) {
    let center_x = (rect.min_x + rect.max_x) * 0.5;
    let center_y = (rect.min_y + rect.max_y) * 0.5;
    let radius_x = (rect.max_x - rect.min_x) * 0.5;
    let radius_y = (rect.max_y - rect.min_y) * 0.5;
    append_annotation_graphics_state(
        content,
        ANNOTATION_OPAQUE_GRAPHICS_STATE,
        [0.0, 0.0, 0.0],
        false,
    );
    append_ellipse_polyline(content, center_x, center_y, radius_x, radius_y);
    content.extend_from_slice(b"S Q\n");
    if widget_button_is_on(annotation) {
        append_annotation_graphics_state(
            content,
            ANNOTATION_OPAQUE_GRAPHICS_STATE,
            [0.0, 0.0, 0.0],
            true,
        );
        let inset_x = (rect.max_x - rect.min_x) * 0.35;
        let inset_y = (rect.max_y - rect.min_y) * 0.35;
        append_fill_rect(
            content,
            PathBounds {
                min_x: rect.min_x + inset_x,
                min_y: rect.min_y + inset_y,
                max_x: rect.max_x - inset_x,
                max_y: rect.max_y - inset_y,
            },
        );
        content.extend_from_slice(b"Q\n");
    }
}

fn text_note_icon_bounds(rect: PathBounds) -> PathBounds {
    let size = (rect.max_x - rect.min_x)
        .min(rect.max_y - rect.min_y)
        .min(20.0);
    PathBounds {
        min_x: rect.min_x,
        min_y: rect.min_y,
        max_x: rect.min_x + size,
        max_y: rect.min_y + size,
    }
}

fn text_note_body_bounds(icon: PathBounds) -> PathBounds {
    let size = icon.max_x - icon.min_x;
    PathBounds {
        min_x: icon.min_x,
        min_y: icon.min_y + size * 0.25,
        max_x: icon.max_x,
        max_y: icon.max_y,
    }
}

fn append_text_note_tail(content: &mut Vec<u8>, icon: PathBounds) {
    let size = icon.max_x - icon.min_x;
    let top_y = icon.min_y + size * 0.25;
    content.extend_from_slice(
        format!(
            "{} {} m {} {} l {} {} l {} {} l h f ",
            format_pdf_number(icon.min_x + size * 0.25),
            format_pdf_number(top_y),
            format_pdf_number(icon.min_x + size * 0.5),
            format_pdf_number(top_y),
            format_pdf_number(icon.min_x + size * 0.45),
            format_pdf_number(icon.min_y),
            format_pdf_number(icon.min_x + size * 0.35),
            format_pdf_number(icon.min_y)
        )
        .as_bytes(),
    );
}

fn append_text_note_icon_rules(content: &mut Vec<u8>, icon: PathBounds) {
    let start_x = icon.min_x + 3.0;
    let end_x = icon.max_x - 3.0;
    for (offset, color) in [
        (3.0, [0.75, 0.75, 0.0]),
        (7.0, [0.5, 0.5, 0.0]),
        (11.0, [0.25, 0.25, 0.0]),
    ] {
        append_annotation_graphics_state(content, ANNOTATION_OPAQUE_GRAPHICS_STATE, color, false);
        content.extend_from_slice(b"1 w ");
        append_stroked_line(
            content,
            start_x,
            icon.max_y - offset,
            end_x,
            icon.max_y - offset,
        );
        content.extend_from_slice(b"Q\n");
    }
}

fn append_text_note_icon_border(content: &mut Vec<u8>, icon: PathBounds, body: PathBounds) {
    let size = icon.max_x - icon.min_x;
    append_fill_rect(
        content,
        PathBounds {
            min_x: body.min_x,
            min_y: body.max_y - 1.0,
            max_x: body.max_x,
            max_y: body.max_y,
        },
    );
    append_fill_rect(
        content,
        PathBounds {
            min_x: body.min_x,
            min_y: body.min_y,
            max_x: body.min_x + 1.0,
            max_y: body.max_y,
        },
    );
    append_fill_rect(
        content,
        PathBounds {
            min_x: body.max_x - 1.0,
            min_y: body.min_y,
            max_x: body.max_x,
            max_y: body.max_y,
        },
    );
    append_fill_rect(
        content,
        PathBounds {
            min_x: body.min_x,
            min_y: body.min_y,
            max_x: body.min_x + size * 0.25,
            max_y: body.min_y + 1.0,
        },
    );
    append_fill_rect(
        content,
        PathBounds {
            min_x: body.min_x + size * 0.5,
            min_y: body.min_y,
            max_x: body.max_x,
            max_y: body.min_y + 1.0,
        },
    );
    append_fill_rect(
        content,
        PathBounds {
            min_x: icon.min_x + size * 0.45,
            min_y: icon.min_y,
            max_x: icon.min_x + size * 0.5,
            max_y: body.min_y,
        },
    );
}

fn widget_button_is_radio(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> bool {
    let Some(PdfPrimitive::Number(PdfNumber::Integer(flags))) = dictionary_value(annotation, b"Ff")
    else {
        return false;
    };
    flags & 32_768 != 0
}

fn widget_button_is_on(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> bool {
    if let Some(PdfPrimitive::Name(active_state)) = dictionary_value(annotation, b"AS") {
        return active_state.as_bytes() != b"Off";
    }
    matches!(
        dictionary_value(annotation, b"V"),
        Some(PdfPrimitive::Name(value)) if value.as_bytes() != b"Off"
    )
}

fn widget_text_value<'a>(annotation: &'a [(PdfName<'a>, PdfPrimitive<'a>)]) -> Option<&'a [u8]> {
    match dictionary_value(annotation, b"V")? {
        PdfPrimitive::String(value) => Some(match value {
            PdfString::Literal(bytes) | PdfString::Hex(bytes) => *bytes,
        }),
        PdfPrimitive::Name(value) => Some(value.as_bytes()),
        _ => None,
    }
}

fn append_widget_text_value(content: &mut Vec<u8>, rect: PathBounds, value: &[u8]) {
    if value.is_empty() {
        return;
    }
    let font_size = ((rect.max_y - rect.min_y) * 0.45).clamp(6.0, 12.0);
    let baseline = rect.min_y + (rect.max_y - rect.min_y - font_size) * 0.5;
    content.extend_from_slice(
        format!(
            "BT /F1 {} Tf 0 0 0 rg {} {} Td ",
            format_pdf_number(font_size),
            format_pdf_number(rect.min_x + 4.0),
            format_pdf_number(baseline)
        )
        .as_bytes(),
    );
    append_pdf_literal_string(content, value);
    content.extend_from_slice(b" Tj ET\n");
}

fn append_pdf_literal_string(content: &mut Vec<u8>, value: &[u8]) {
    content.push(b'(');
    for byte in value.iter().take(64) {
        match *byte {
            b'(' | b')' | b'\\' => {
                content.push(b'\\');
                content.push(*byte);
            }
            0x20..=0x7e => content.push(*byte),
            _ => content.push(b'?'),
        }
    }
    content.push(b')');
}

fn is_ignorable_annotation_fallback_text_error(error: &GraphicsError) -> bool {
    matches!(
        error.kind(),
        GraphicsErrorKind::MissingFont { .. } | GraphicsErrorKind::FontNotSelected
    )
}

fn append_annotation_graphics_state(
    content: &mut Vec<u8>,
    state_name: &[u8],
    color: [f64; 3],
    fill: bool,
) {
    let operator = if fill { "rg" } else { "RG" };
    content.extend_from_slice(
        format!(
            "q /{} gs {} {} {} {} 1.5 w ",
            String::from_utf8_lossy(state_name),
            format_pdf_number(color[0]),
            format_pdf_number(color[1]),
            format_pdf_number(color[2]),
            operator
        )
        .as_bytes(),
    );
}

fn append_fill_rect(content: &mut Vec<u8>, rect: PathBounds) {
    content.extend_from_slice(
        format!(
            "{} {} {} {} re f ",
            format_pdf_number(rect.min_x),
            format_pdf_number(rect.min_y),
            format_pdf_number(rect.max_x - rect.min_x),
            format_pdf_number(rect.max_y - rect.min_y)
        )
        .as_bytes(),
    );
}

fn append_fill_quad(content: &mut Vec<u8>, quad: [f64; 8]) {
    content.extend_from_slice(
        format!(
            "{} {} m {} {} l {} {} l {} {} l h f ",
            format_pdf_number(quad[0]),
            format_pdf_number(quad[1]),
            format_pdf_number(quad[2]),
            format_pdf_number(quad[3]),
            format_pdf_number(quad[6]),
            format_pdf_number(quad[7]),
            format_pdf_number(quad[4]),
            format_pdf_number(quad[5])
        )
        .as_bytes(),
    );
}

fn append_stroked_line(content: &mut Vec<u8>, start_x: f64, y: f64, end_x: f64, end_y: f64) {
    content.extend_from_slice(
        format!(
            "{} {} m {} {} l S ",
            format_pdf_number(start_x),
            format_pdf_number(y),
            format_pdf_number(end_x),
            format_pdf_number(end_y)
        )
        .as_bytes(),
    );
}

fn append_ellipse_polyline(
    content: &mut Vec<u8>,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
) {
    const SEGMENTS: usize = 12;
    for segment in 0..SEGMENTS {
        let angle = std::f64::consts::TAU * segment as f64 / SEGMENTS as f64;
        let x = center_x + radius_x * angle.cos();
        let y = center_y + radius_y * angle.sin();
        let operator = if segment == 0 { "m" } else { "l" };
        content.extend_from_slice(
            format!(
                "{} {} {} ",
                format_pdf_number(x),
                format_pdf_number(y),
                operator
            )
            .as_bytes(),
        );
    }
    content.extend_from_slice(b"h ");
}

fn annotation_quad_points(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)]) -> Vec<[f64; 8]> {
    let Some(PdfPrimitive::Array(values)) = dictionary_value(annotation, b"QuadPoints") else {
        return Vec::new();
    };
    values
        .chunks_exact(8)
        .take(MAX_ANNOTATION_FALLBACK_QUADS)
        .filter_map(|chunk| {
            let mut quad = [0.0; 8];
            for (index, value) in chunk.iter().enumerate() {
                quad[index] = primitive_number(value)?;
            }
            Some(quad)
        })
        .collect()
}

fn annotation_color(annotation: &[(PdfName<'_>, PdfPrimitive<'_>)], default: [f64; 3]) -> [f64; 3] {
    let Some(PdfPrimitive::Array(values)) = dictionary_value(annotation, b"C") else {
        return default;
    };
    let [red, green, blue] = values.as_slice() else {
        return default;
    };
    [
        primitive_number(red).map_or(default[0], clamp_pdf_color),
        primitive_number(green).map_or(default[1], clamp_pdf_color),
        primitive_number(blue).map_or(default[2], clamp_pdf_color),
    ]
}

fn clamp_pdf_color(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn annotation_fallback_ext_graphics_states() -> Result<ExtGraphicsStateResources, GraphicsError> {
    ExtGraphicsStateResources::from_extgstate_dictionary(&[
        (
            PdfName::new(ANNOTATION_OPAQUE_GRAPHICS_STATE),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"ca"),
                    PdfPrimitive::Number(PdfNumber::Real(1.0)),
                ),
                (
                    PdfName::new(b"CA"),
                    PdfPrimitive::Number(PdfNumber::Real(1.0)),
                ),
            ]),
        ),
        (
            PdfName::new(ANNOTATION_UNDERLINE_GRAPHICS_STATE),
            PdfPrimitive::Dictionary(vec![
                (
                    PdfName::new(b"ca"),
                    PdfPrimitive::Number(PdfNumber::Real(1.0)),
                ),
                (
                    PdfName::new(b"CA"),
                    PdfPrimitive::Number(PdfNumber::Real(0.5)),
                ),
            ]),
        ),
    ])
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
        | GraphicsErrorKind::GlyphOutlineStackOverflow { .. }
        | GraphicsErrorKind::GlyphOutlineSubroutineOverflow { .. }
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
        GraphicsErrorKind::ImageBytesOverflow { .. }
        | GraphicsErrorKind::ImageResourceBytesOverflow { .. }
        | GraphicsErrorKind::ShadingBytesOverflow { .. }
        | GraphicsErrorKind::ShadingTriangleOverflow { .. } => {
            unsupported_feature(BUCKET_RENDERER_MEMORY_BUDGET)
        }
        GraphicsErrorKind::UnsupportedSoftMask { .. }
        | GraphicsErrorKind::UnsupportedTransparencyGroup { .. }
        | GraphicsErrorKind::UnsupportedBlendMode { .. }
        | GraphicsErrorKind::UnsupportedOverprint { .. }
        | GraphicsErrorKind::SoftMaskDepthOverflow { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_TRANSPARENCY)
        }
        GraphicsErrorKind::UnsupportedColorManagement { .. } => {
            unsupported_feature(BUCKET_GRAPHICS_COLOR_MANAGEMENT)
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

fn metadata_from_classic_document(
    document: &ClassicDocument<'_>,
) -> Result<DocumentMetadata, ThumbnailError> {
    let page_tree = document.page_tree().map_err(map_object_error)?;
    let mut metadata = metadata_from_page_tree(&page_tree)?;
    let catalog = document_catalog_dictionary(document)?;
    metadata.info = document_info(document)?;
    metadata.structure = document_structure(document, catalog, &page_tree)?;
    metadata.outlines = outline_metadata(document, catalog)?;
    metadata.page_labels = page_labels_metadata(document, catalog, page_tree.page_count())?;
    metadata.accessibility = accessibility_metadata(document, catalog)?;
    metadata.archival = archival_metadata(document, catalog)?;
    metadata.optional_content = optional_content_metadata(document, catalog, &page_tree)?;
    Ok(metadata)
}

fn document_catalog_dictionary<'a>(
    document: &'a ClassicDocument<'a>,
) -> Result<&'a [(PdfName<'a>, PdfPrimitive<'a>)], ThumbnailError> {
    let Some(PdfPrimitive::Reference(reference)) =
        dictionary_value(document.trailer.entries(), b"Root")
    else {
        return Err(ThumbnailError::Malformed);
    };
    let reference = object_reference(*reference)?;
    let object = document
        .objects
        .get(reference.id)
        .ok_or(ThumbnailError::Malformed)?;
    object_dictionary(&object.value)
}

fn document_info(document: &ClassicDocument<'_>) -> Result<DocumentInfo, ThumbnailError> {
    let Some(PdfPrimitive::Reference(reference)) =
        dictionary_value(document.trailer.entries(), b"Info")
    else {
        return Ok(DocumentInfo::default());
    };
    let reference = object_reference(*reference)?;
    let object = document
        .objects
        .get(reference.id)
        .ok_or(ThumbnailError::Malformed)?;
    let dictionary = object_dictionary(&object.value)?;
    Ok(DocumentInfo {
        title: metadata_string(dictionary, b"Title"),
        author: metadata_string(dictionary, b"Author"),
        subject: metadata_string(dictionary, b"Subject"),
        keywords: metadata_string(dictionary, b"Keywords"),
        creator: metadata_string(dictionary, b"Creator"),
        producer: metadata_string(dictionary, b"Producer"),
        creation_date: metadata_string(dictionary, b"CreationDate"),
        modification_date: metadata_string(dictionary, b"ModDate"),
    })
}

fn document_structure(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
    page_tree: &PageTree,
) -> Result<DocumentStructure, ThumbnailError> {
    let (has_signature_fields, has_signature_byte_range) =
        document_signature_structure(document, catalog)?;
    Ok(DocumentStructure {
        has_xmp_metadata: dictionary_value(catalog, b"Metadata").is_some(),
        has_mark_info: dictionary_value(catalog, b"MarkInfo").is_some(),
        has_struct_tree_root: dictionary_value(catalog, b"StructTreeRoot").is_some(),
        has_named_destinations: has_named_destinations(document, catalog)?,
        has_signature_fields,
        has_signature_byte_range,
        has_embedded_files: has_embedded_files(document, catalog)?,
        has_portfolio_collection: dictionary_value(catalog, b"Collection").is_some(),
        has_file_attachment_annotations: has_file_attachment_annotations(document, page_tree)?,
    })
}

fn archival_metadata(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<ArchivalMetadata, ThumbnailError> {
    let xmp = catalog_metadata_stream_bytes(document, catalog)?;
    Ok(ArchivalMetadata {
        pdfa_part: xmp
            .as_deref()
            .and_then(|metadata| xmp_marker_value(metadata, b"pdfaid:part")),
        pdfa_conformance: xmp
            .as_deref()
            .and_then(|metadata| xmp_marker_value(metadata, b"pdfaid:conformance")),
        has_output_intents: dictionary_value(catalog, b"OutputIntents").is_some(),
        conformance_validation_performed: false,
    })
}

fn catalog_metadata_stream_bytes(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<Option<Vec<u8>>, ThumbnailError> {
    let Some(PdfPrimitive::Reference(reference)) = dictionary_value(catalog, b"Metadata") else {
        return Ok(None);
    };
    let reference = object_reference(*reference)?;
    let object = document
        .objects
        .get(reference.id)
        .ok_or(ThumbnailError::Malformed)?;
    let ObjectValue::Stream(stream) = &object.value else {
        return Err(ThumbnailError::Malformed);
    };
    stream
        .decode_with_options(StreamDecodeOptions {
            max_decoded_len: MAX_METADATA_XMP_BYTES,
        })
        .map(Some)
        .map_err(|_| ThumbnailError::Malformed)
}

fn xmp_marker_value(metadata: &[u8], marker: &[u8]) -> Option<String> {
    xmp_element_value(metadata, marker)
        .or_else(|| xmp_attribute_value(metadata, marker, b'"'))
        .or_else(|| xmp_attribute_value(metadata, marker, b'\''))
        .and_then(|value| std::str::from_utf8(value).ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn xmp_element_value<'a>(metadata: &'a [u8], marker: &[u8]) -> Option<&'a [u8]> {
    let mut open = Vec::with_capacity(marker.len() + 2);
    open.push(b'<');
    open.extend_from_slice(marker);
    open.push(b'>');
    let start = find_subslice(metadata, &open)? + open.len();

    let mut close = Vec::with_capacity(marker.len() + 3);
    close.extend_from_slice(b"</");
    close.extend_from_slice(marker);
    close.push(b'>');
    let end = find_subslice(&metadata[start..], &close)? + start;
    Some(&metadata[start..end])
}

fn xmp_attribute_value<'a>(metadata: &'a [u8], marker: &[u8], quote: u8) -> Option<&'a [u8]> {
    let mut prefix = Vec::with_capacity(marker.len() + 2);
    prefix.extend_from_slice(marker);
    prefix.push(b'=');
    prefix.push(quote);
    let start = find_subslice(metadata, &prefix)? + prefix.len();
    let end = metadata[start..].iter().position(|byte| *byte == quote)? + start;
    Some(&metadata[start..end])
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn has_embedded_files(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<bool, ThumbnailError> {
    let Some(names_value) = dictionary_value(catalog, b"Names") else {
        return Ok(false);
    };
    let names = metadata_dictionary_from_value(document, names_value)?;
    Ok(dictionary_value(names, b"EmbeddedFiles").is_some())
}

fn accessibility_metadata(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<AccessibilityMetadata, ThumbnailError> {
    let mut metadata = AccessibilityMetadata {
        language: metadata_string(catalog, b"Lang"),
        mark_info_marked: mark_info_marked(document, catalog)?,
        ..AccessibilityMetadata::default()
    };
    let Some(struct_tree_value) = dictionary_value(catalog, b"StructTreeRoot") else {
        return Ok(metadata);
    };
    let struct_tree_root = metadata_dictionary_from_value(document, struct_tree_value)?;
    metadata.has_role_map = dictionary_value(struct_tree_root, b"RoleMap").is_some();
    let Some(kids) = dictionary_value(struct_tree_root, b"K") else {
        return Ok(metadata);
    };
    let summary = summarize_structure_tree(document, kids)?;
    metadata.structure_role_count = summary.role_count;
    metadata.has_marked_content_references = summary.has_marked_content_references;
    metadata.marked_content_reference_count = summary.marked_content_reference_count;
    metadata.page_content_reference_count = summary.page_content_reference_count;
    metadata.alt_text_count = summary.alt_text_count;
    metadata.reading_order_warning_count = summary.reading_order_warning_count;
    metadata.truncated = summary.truncated;
    Ok(metadata)
}

fn mark_info_marked(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<Option<bool>, ThumbnailError> {
    let Some(mark_info_value) = dictionary_value(catalog, b"MarkInfo") else {
        return Ok(None);
    };
    let mark_info = metadata_dictionary_from_value(document, mark_info_value)?;
    match dictionary_value(mark_info, b"Marked") {
        Some(PdfPrimitive::Boolean(value)) => Ok(Some(*value)),
        Some(_) => Err(ThumbnailError::Malformed),
        None => Ok(None),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct StructureTreeSummary {
    role_count: usize,
    has_marked_content_references: bool,
    marked_content_reference_count: usize,
    page_content_reference_count: usize,
    alt_text_count: usize,
    reading_order_warning_count: usize,
    truncated: bool,
}

#[derive(Debug, Clone, Copy)]
struct StructureStackItem<'a> {
    value: &'a PdfPrimitive<'a>,
    has_page_context: bool,
}

fn summarize_structure_tree(
    document: &ClassicDocument<'_>,
    root_kids: &PdfPrimitive<'_>,
) -> Result<StructureTreeSummary, ThumbnailError> {
    let mut summary = StructureTreeSummary::default();
    let mut stack = Vec::new();
    push_structure_value(&mut stack, root_kids, false);
    let mut visited = HashSet::new();
    let mut reached_items = 0usize;

    while let Some(item) = stack.pop() {
        if reached_items == MAX_METADATA_STRUCTURE_ITEMS {
            summary.truncated = true;
            break;
        }
        reached_items += 1;
        let value = item.value;
        match value {
            PdfPrimitive::Reference(reference) => {
                let reference = object_reference(*reference)?;
                if !visited.insert(reference.id) {
                    continue;
                }
                let object = document
                    .objects
                    .get(reference.id)
                    .ok_or(ThumbnailError::Malformed)?;
                let dictionary = object_dictionary(&object.value)?;
                summarize_structure_dictionary(
                    document,
                    dictionary,
                    item.has_page_context,
                    &mut stack,
                    &mut summary,
                )?;
            }
            PdfPrimitive::Dictionary(dictionary) => {
                summarize_structure_dictionary(
                    document,
                    dictionary,
                    item.has_page_context,
                    &mut stack,
                    &mut summary,
                )?;
            }
            PdfPrimitive::Array(values) => {
                for value in values.iter().rev() {
                    push_structure_value(&mut stack, value, item.has_page_context);
                }
            }
            PdfPrimitive::Number(_) => {
                summary.has_marked_content_references = true;
                summary.marked_content_reference_count += 1;
                if item.has_page_context {
                    summary.page_content_reference_count += 1;
                } else {
                    summary.reading_order_warning_count += 1;
                }
            }
            _ => return Err(ThumbnailError::Malformed),
        }
    }

    Ok(summary)
}

fn summarize_structure_dictionary<'a>(
    document: &ClassicDocument<'a>,
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    inherited_page_context: bool,
    stack: &mut Vec<StructureStackItem<'a>>,
    summary: &mut StructureTreeSummary,
) -> Result<(), ThumbnailError> {
    let has_page_context = match dictionary_value(dictionary, b"Pg") {
        Some(page) => {
            let _ = metadata_dictionary_from_value(document, page)?;
            true
        }
        None => inherited_page_context,
    };
    if dictionary_name_is(dictionary, b"Type", b"MCR")
        || dictionary_value(dictionary, b"MCID").is_some()
    {
        summary.has_marked_content_references = true;
        summary.marked_content_reference_count += 1;
        if has_page_context {
            summary.page_content_reference_count += 1;
        } else {
            summary.reading_order_warning_count += 1;
        }
        return Ok(());
    }
    if dictionary_value(dictionary, b"S").is_some() {
        summary.role_count += 1;
    }
    if dictionary_value(dictionary, b"Alt").is_some() {
        summary.alt_text_count += 1;
    }
    if let Some(kids) = dictionary_value(dictionary, b"K") {
        push_structure_value(stack, kids, has_page_context);
    }
    Ok(())
}

fn push_structure_value<'a>(
    stack: &mut Vec<StructureStackItem<'a>>,
    value: &'a PdfPrimitive<'a>,
    has_page_context: bool,
) {
    stack.push(StructureStackItem {
        value,
        has_page_context,
    });
}

fn has_file_attachment_annotations(
    document: &ClassicDocument<'_>,
    page_tree: &PageTree,
) -> Result<bool, ThumbnailError> {
    let mut visited = 0usize;
    for page in page_tree.pages() {
        let object = document
            .objects
            .get(page.id)
            .ok_or(ThumbnailError::Malformed)?;
        let dictionary = object_dictionary(&object.value)?;
        let Some(annots) = dictionary_value(dictionary, b"Annots") else {
            continue;
        };
        for annotation in annotation_array(document, annots)? {
            if visited >= MAX_METADATA_ATTACHMENT_ANNOTATIONS {
                return Ok(false);
            }
            visited += 1;
            let Some(annotation) = annotation_dictionary(document, annotation)? else {
                continue;
            };
            if dictionary_name_is(annotation, b"Subtype", b"FileAttachment") {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn document_signature_structure(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<(bool, bool), ThumbnailError> {
    let Some(acroform_value) = dictionary_value(catalog, b"AcroForm") else {
        return Ok((false, false));
    };
    let acroform = resource_dictionary(document, acroform_value)?;
    let Some(fields_value) = dictionary_value(acroform, b"Fields") else {
        return Ok((false, false));
    };
    let fields = metadata_array_from_value(document, fields_value)?;
    let mut has_signature_fields = false;
    let mut has_signature_byte_range = false;
    for field in fields.iter().take(MAX_METADATA_SIGNATURE_FIELDS) {
        let Some(dictionary) = annotation_dictionary(document, field)? else {
            continue;
        };
        if !dictionary_name_is(dictionary, b"FT", b"Sig") {
            continue;
        }
        has_signature_fields = true;
        if signature_value_has_byte_range(document, dictionary_value(dictionary, b"V"))? {
            has_signature_byte_range = true;
        }
        if has_signature_byte_range {
            break;
        }
    }
    Ok((has_signature_fields, has_signature_byte_range))
}

fn metadata_array_from_value<'a>(
    document: &'a ClassicDocument<'a>,
    value: &'a PdfPrimitive<'a>,
) -> Result<&'a [PdfPrimitive<'a>], ThumbnailError> {
    match value {
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

fn signature_value_has_byte_range(
    document: &ClassicDocument<'_>,
    value: Option<&PdfPrimitive<'_>>,
) -> Result<bool, ThumbnailError> {
    let Some(value) = value else {
        return Ok(false);
    };
    let dictionary = match value {
        PdfPrimitive::Dictionary(dictionary) => dictionary.as_slice(),
        PdfPrimitive::Reference(reference) => {
            let reference = object_reference(*reference)?;
            let object = document
                .objects
                .get(reference.id)
                .ok_or(ThumbnailError::Malformed)?;
            object_dictionary(&object.value)?
        }
        _ => return Err(ThumbnailError::Malformed),
    };
    Ok(dictionary_value(dictionary, b"ByteRange").is_some())
}

fn has_named_destinations(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<bool, ThumbnailError> {
    if dictionary_value(catalog, b"Dests").is_some() {
        return Ok(true);
    }
    let Some(names_value) = dictionary_value(catalog, b"Names") else {
        return Ok(false);
    };
    let names = metadata_dictionary_from_value(document, names_value)?;
    Ok(dictionary_value(names, b"Dests").is_some())
}

fn outline_metadata(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
) -> Result<OutlineMetadata, ThumbnailError> {
    let Some(outlines_value) = dictionary_value(catalog, b"Outlines") else {
        return Ok(OutlineMetadata::default());
    };
    let outlines = metadata_dictionary_from_value(document, outlines_value)?;
    let Some(first) = dictionary_reference(outlines, b"First")? else {
        return Ok(OutlineMetadata {
            has_outlines: true,
            item_count: 0,
            truncated: false,
        });
    };
    let (item_count, truncated) = count_outline_items(document, first)?;
    Ok(OutlineMetadata {
        has_outlines: true,
        item_count,
        truncated,
    })
}

fn count_outline_items(
    document: &ClassicDocument<'_>,
    first: Reference,
) -> Result<(usize, bool), ThumbnailError> {
    let mut stack = vec![first];
    let mut visited = HashSet::new();
    let mut item_count = 0;

    while let Some(reference) = stack.pop() {
        if !visited.insert(reference.id) {
            continue;
        }
        if item_count == MAX_METADATA_OUTLINE_ITEMS {
            return Ok((item_count, true));
        }
        item_count += 1;
        let object = document
            .objects
            .get(reference.id)
            .ok_or(ThumbnailError::Malformed)?;
        let dictionary = object_dictionary(&object.value)?;
        if let Some(next) = dictionary_reference(dictionary, b"Next")? {
            stack.push(next);
        }
        if let Some(child) = dictionary_reference(dictionary, b"First")? {
            stack.push(child);
        }
    }

    Ok((item_count, false))
}

fn page_labels_metadata(
    document: &ClassicDocument<'_>,
    catalog: &[(PdfName<'_>, PdfPrimitive<'_>)],
    page_count: usize,
) -> Result<PageLabelsMetadata, ThumbnailError> {
    let Some(page_labels_value) = dictionary_value(catalog, b"PageLabels") else {
        return Ok(PageLabelsMetadata::default());
    };
    let page_labels = metadata_dictionary_from_value(document, page_labels_value)?;
    let Some(PdfPrimitive::Array(nums)) = dictionary_value(page_labels, b"Nums") else {
        return Err(ThumbnailError::Malformed);
    };
    let mut ranges = Vec::new();
    for pair in nums.chunks_exact(2) {
        let Some(start_page) = primitive_usize(&pair[0]) else {
            return Err(ThumbnailError::Malformed);
        };
        let PdfPrimitive::Dictionary(dictionary) = &pair[1] else {
            return Err(ThumbnailError::Malformed);
        };
        ranges.push(PageLabelRange::from_dictionary(start_page, dictionary)?);
    }
    ranges.sort_by_key(|range| range.start_page);
    if ranges.is_empty() {
        return Ok(PageLabelsMetadata::default());
    }

    let label_count = page_count.min(MAX_METADATA_PAGE_LABELS);
    let mut labels = Vec::with_capacity(label_count);
    let mut range_index = 0;
    for page_index in 0..label_count {
        while range_index + 1 < ranges.len() && ranges[range_index + 1].start_page <= page_index {
            range_index += 1;
        }
        if ranges[range_index].start_page > page_index {
            continue;
        }
        labels.push(PageLabel {
            page_index: u32::try_from(page_index)
                .map_err(|_| ThumbnailError::internal("page label index exceeds u32"))?,
            label: ranges[range_index].label_for(page_index),
        });
    }

    Ok(PageLabelsMetadata {
        labels,
        truncated: page_count > label_count,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PageLabelRange {
    start_page: usize,
    prefix: String,
    style: Option<PageLabelStyle>,
    start_number: u32,
}

impl PageLabelRange {
    fn from_dictionary(
        start_page: usize,
        dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    ) -> Result<Self, ThumbnailError> {
        Ok(Self {
            start_page,
            prefix: metadata_string(dictionary, b"P").unwrap_or_default(),
            style: dictionary_name_bytes(dictionary, b"S").and_then(PageLabelStyle::from_name),
            start_number: dictionary_value(dictionary, b"St")
                .and_then(primitive_u32)
                .unwrap_or(1),
        })
    }

    fn label_for(&self, page_index: usize) -> String {
        let number = self.start_number + (page_index - self.start_page) as u32;
        match self.style {
            Some(style) => format!("{}{}", self.prefix, style.format(number)),
            None => self.prefix.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageLabelStyle {
    Decimal,
    UpperRoman,
    LowerRoman,
    UpperAlpha,
    LowerAlpha,
}

impl PageLabelStyle {
    fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"D" => Some(Self::Decimal),
            b"R" => Some(Self::UpperRoman),
            b"r" => Some(Self::LowerRoman),
            b"A" => Some(Self::UpperAlpha),
            b"a" => Some(Self::LowerAlpha),
            _ => None,
        }
    }

    fn format(self, number: u32) -> String {
        match self {
            Self::Decimal => number.to_string(),
            Self::UpperRoman => roman_label(number),
            Self::LowerRoman => roman_label(number).to_ascii_lowercase(),
            Self::UpperAlpha => alpha_label(number, b'A'),
            Self::LowerAlpha => alpha_label(number, b'a'),
        }
    }
}

fn roman_label(mut number: u32) -> String {
    if number == 0 {
        return String::new();
    }
    const ROMAN: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut output = String::new();
    for (value, symbol) in ROMAN {
        while number >= *value {
            output.push_str(symbol);
            number -= *value;
        }
    }
    output
}

fn alpha_label(mut number: u32, base: u8) -> String {
    if number == 0 {
        return String::new();
    }
    let mut bytes = Vec::new();
    while number > 0 {
        number -= 1;
        bytes.push(base + (number % 26) as u8);
        number /= 26;
    }
    bytes.reverse();
    String::from_utf8(bytes).unwrap_or_default()
}

fn metadata_dictionary_from_value<'a>(
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

fn dictionary_reference(
    dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)],
    key: &[u8],
) -> Result<Option<Reference>, ThumbnailError> {
    let Some(PdfPrimitive::Reference(reference)) = dictionary_value(dictionary, key) else {
        return Ok(None);
    };
    object_reference(*reference).map(Some)
}

fn metadata_string(dictionary: &[(PdfName<'_>, PdfPrimitive<'_>)], key: &[u8]) -> Option<String> {
    match dictionary_value(dictionary, key)? {
        PdfPrimitive::String(value) => Some(match value {
            PdfString::Literal(bytes) | PdfString::Hex(bytes) => {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }),
        PdfPrimitive::Name(name) => Some(String::from_utf8_lossy(name.as_bytes()).into_owned()),
        _ => None,
    }
}

fn dictionary_name_bytes<'a>(
    dictionary: &'a [(PdfName<'a>, PdfPrimitive<'a>)],
    key: &[u8],
) -> Option<&'a [u8]> {
    let Some(PdfPrimitive::Name(name)) = dictionary_value(dictionary, key) else {
        return None;
    };
    Some(name.as_bytes())
}

fn primitive_u32(value: &PdfPrimitive<'_>) -> Option<u32> {
    match value {
        PdfPrimitive::Number(PdfNumber::Integer(value)) => u32::try_from(*value).ok(),
        _ => None,
    }
}

fn primitive_usize(value: &PdfPrimitive<'_>) -> Option<usize> {
    primitive_u32(value).and_then(|value| usize::try_from(value).ok())
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
    fn native_page_cache_policy_should_be_isolated_by_default() {
        let policy = NativePageCachePolicy::IsolatedRender;

        assert_eq!(policy.as_str(), "isolated-render");
        assert!(!policy.permits_disk_persistence());
    }

    #[test]
    fn native_page_cache_key_should_include_document_options_version_and_profile() {
        let options = ThumbnailOptions {
            page_index: 2,
            max_edge: 160,
            background: pdfrust_thumbnail::Rgba {
                r: 12,
                g: 34,
                b: 56,
                a: 255,
            },
            output_format: pdfrust_thumbnail::OutputFormat::Rgba,
            timeout: pdfrust_thumbnail::DEFAULT_TIMEOUT,
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: FormAppearanceMode::DocumentState,
        };

        let first = NativePageCacheKey::from_options(0x1111, &options, "default");
        let second_document = NativePageCacheKey::from_options(0x2222, &options, "default");
        let low_memory = NativePageCacheKey::from_options(0x1111, &options, "low-memory");

        assert_ne!(first, second_document);
        assert_ne!(first, low_memory);
        assert_eq!(first.page_index, 2);
        assert_eq!(first.max_edge, 160);
        assert_eq!(first.background, [12, 34, 56, 255]);
        assert_eq!(first.annotation_mode, "screen");
        assert_eq!(first.form_appearance_mode, "document-state");
        assert_eq!(first.renderer_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn native_page_cache_key_should_isolate_long_document_navigation_pages() {
        let options = ThumbnailOptions {
            max_edge: 210,
            ..ThumbnailOptions::default()
        };
        let first_page = NativePageCacheKey::from_options(0x7171, &options, "default");
        let random_page = NativePageCacheKey::from_options(
            0x7171,
            &ThumbnailOptions {
                page_index: 11,
                ..options
            },
            "default",
        );
        let different_background = NativePageCacheKey::from_options(
            0x7171,
            &ThumbnailOptions {
                background: pdfrust_thumbnail::Rgba {
                    r: 245,
                    g: 246,
                    b: 250,
                    a: 255,
                },
                ..options
            },
            "default",
        );

        assert_ne!(first_page, random_page);
        assert_ne!(first_page, different_background);
        assert_eq!(random_page.page_index, 11);
        assert!(!NativePageCachePolicy::IsolatedRender.permits_disk_persistence());
    }

    #[test]
    fn native_page_cache_key_should_isolate_high_dpi_scale() {
        let standard = ThumbnailOptions {
            max_edge: 160,
            ..ThumbnailOptions::default()
        };
        let high_dpi = ThumbnailOptions {
            max_edge: 480,
            ..standard
        };

        let standard_key = NativePageCacheKey::from_options(0x172, &standard, "default");
        let high_dpi_key = NativePageCacheKey::from_options(0x172, &high_dpi, "default");

        assert_ne!(standard_key, high_dpi_key);
        assert_eq!(high_dpi_key.max_edge, 480);
    }

    #[test]
    fn native_backend_should_expose_memory_diagnostics() {
        let diagnostics = NativeBackend::new().memory_diagnostics();

        assert_eq!(diagnostics.max_page_pixels, 16 * 1024 * 1024);
        assert_eq!(diagnostics.max_image_bytes, 32 * 1024 * 1024);
        assert_eq!(diagnostics.max_total_image_bytes, 128 * 1024 * 1024);
        assert_eq!(diagnostics.max_icc_profile_bytes, 1024 * 1024);
        assert_eq!(diagnostics.max_icc_transform_workspace_bytes, 64 * 1024);
        assert_eq!(diagnostics.max_icc_transform_cache_entries, 32);
        assert_eq!(diagnostics.max_display_items, 8_192);
        assert_eq!(diagnostics.max_font_fallback_cache_entries, 128);
        assert_eq!(diagnostics.max_transparency_group_pixels, 16 * 1024 * 1024);
        assert_eq!(diagnostics.max_flattened_segments, 65_536);
        assert_eq!(diagnostics.max_pattern_tiles, 65_536);
        assert_eq!(diagnostics.max_pattern_cell_cache_entries, 32);
        assert!(!diagnostics.spooling_enabled);
        assert_eq!(diagnostics.max_spool_bytes, 0);
    }

    #[test]
    fn native_low_memory_profile_should_expose_tighter_memory_diagnostics() {
        let default = NativeBackend::new().memory_diagnostics();
        let low_memory = NativeBackend::low_memory().memory_diagnostics();

        assert!(low_memory.max_page_pixels < default.max_page_pixels);
        assert!(low_memory.max_image_bytes < default.max_image_bytes);
        assert!(low_memory.max_total_image_bytes < default.max_total_image_bytes);
        assert!(low_memory.max_font_program_bytes < default.max_font_program_bytes);
        assert!(low_memory.max_display_items < default.max_display_items);
        assert!(low_memory.max_transparency_group_pixels < default.max_transparency_group_pixels);
        assert!(low_memory.max_page_pixels > 0);
        assert!(low_memory.max_total_image_bytes >= low_memory.max_image_bytes);
    }

    #[test]
    fn native_low_memory_profile_should_render_common_thumbnail_fixtures() {
        let backend = NativeBackend::low_memory();

        for &(bytes, label) in &[
            (
                include_bytes!("../../../fixtures/generated/text-page.pdf").as_slice(),
                "text page",
            ),
            (
                include_bytes!("../../../fixtures/generated/business-invoice-dense.pdf").as_slice(),
                "dense business invoice",
            ),
            (
                include_bytes!("../../../fixtures/generated/mobile-cropped-photo-scan.pdf")
                    .as_slice(),
                "cropped mobile scan",
            ),
        ] {
            let thumbnail = backend
                .render(
                    PdfSource::from_bytes(bytes),
                    &ThumbnailOptions {
                        page_index: 0,
                        max_edge: 160,
                        background: pdfrust_thumbnail::Rgba::WHITE,
                        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
                        timeout: std::time::Duration::from_secs(5),
                        annotation_mode: AnnotationMode::Screen,
                        form_appearance_mode: FormAppearanceMode::DocumentState,
                    },
                )
                .unwrap_or_else(|error| panic!("{label} should render under low memory: {error}"));

            assert!(thumbnail.width <= 160);
            assert!(thumbnail.height <= 160);
            assert!(!thumbnail.bytes.is_empty());
        }
    }

    #[test]
    fn native_low_memory_budget_errors_should_remain_typed() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let limits = NativeRenderLimits {
            max_page_pixels: 1,
            ..NativeRenderLimits::low_memory()
        };
        let error = NativeBackend::with_render_limits(limits)
            .render(
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    page_index: 0,
                    max_edge: 160,
                    background: pdfrust_thumbnail::Rgba::WHITE,
                    output_format: pdfrust_thumbnail::OutputFormat::Rgba,
                    timeout: std::time::Duration::from_secs(5),
                    annotation_mode: AnnotationMode::Screen,
                    form_appearance_mode: FormAppearanceMode::DocumentState,
                },
            )
            .expect_err("tight page raster budget should fail deterministically");

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
    fn native_backend_should_render_generated_high_dpi_preview_fixtures() {
        type HighDpiFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[HighDpiFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/high-dpi-preview-fidelity.pdf")
                    as &[u8],
                480,
                360,
                480,
                "high-DPI preview fidelity",
                20_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/text-page.pdf") as &[u8],
                300,
                160,
                600,
                "high-DPI text baseline",
                1_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/vector-paths.pdf") as &[u8],
                220,
                180,
                440,
                "high-DPI vector baseline",
                500,
            ),
            (
                include_bytes!("../../../fixtures/generated/image-xobject.pdf") as &[u8],
                120,
                120,
                360,
                "high-DPI image baseline",
                1_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/transparency-alpha.pdf") as &[u8],
                120,
                120,
                360,
                "high-DPI transparency baseline",
                1_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve high-DPI visible content"
            );
        }
    }

    #[test]
    fn native_backend_should_enforce_high_dpi_raster_budget() {
        let bytes = include_bytes!("../../../fixtures/generated/high-dpi-preview-fidelity.pdf");
        let limits = NativeRenderLimits {
            max_page_pixels: 10_000,
            ..NativeRenderLimits::default()
        };
        let error = NativeBackend::with_render_limits(limits)
            .render(
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: 480,
                    ..ThumbnailOptions::default()
                },
            )
            .expect_err("high-DPI output should exceed the configured page raster budget");

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
    fn native_backend_should_render_generated_icc_rgb_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/icc-rgb-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ICCBased RGB image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [255, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [0, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [0, 0, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_icc_gray_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/icc-gray-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ICCBased Gray image fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 76, 44), [85, 85, 85, 255]);
        assert_eq!(rgba_at(&thumbnail, 44, 76), [170, 170, 170, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_icc_cmyk_image_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/icc-cmyk-image.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ICCBased CMYK image fixture should render through native backend");

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
    fn native_backend_should_render_generated_image_mask_signature_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/image-mask-signature.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ImageMask signature fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert!(
            thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| pixel[0] < 16 && pixel[1] < 48 && pixel[2] < 96)
                .count()
                > 500
        );
    }

    #[test]
    fn native_backend_should_render_generated_monochrome_image_mask_icon_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/image-mask-monochrome-icon.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated monochrome ImageMask icon fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 44, 44), [10, 115, 56, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 60), [240, 245, 250, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_compressed_image_mask_logo_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/image-mask-logo.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 150,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated compressed ImageMask logo fixture should render through native backend");

        assert_eq!(thumbnail.width, 150);
        assert_eq!(thumbnail.height, 100);
        assert!(
            thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| pixel[0] > 220 && pixel[1] > 160 && pixel[2] < 80)
                .count()
                > 1_000
        );
    }

    #[test]
    fn native_backend_should_report_generated_unsupported_ccitt_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/unsupported-ccitt-image.pdf");
        assert_unsupported_image_filter_fixture(bytes);
    }

    #[test]
    fn native_backend_should_report_generated_unsupported_jbig2_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/unsupported-jbig2-image.pdf");
        assert_unsupported_image_filter_fixture(bytes);
    }

    #[test]
    fn native_backend_should_report_generated_unsupported_jpx_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/unsupported-jpx-image.pdf");
        assert_unsupported_image_filter_fixture(bytes);
    }

    #[test]
    fn native_backend_should_freeze_typed_unsupported_boundary_buckets() {
        let fixtures: &[(&[u8], &'static str, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/unsupported-ccitt-image.pdf")
                    as &[u8],
                buckets::IMAGE_FILTER,
                "unsupported CCITT image filter",
            ),
            (
                include_bytes!("../../../fixtures/generated/optional-content-ocmd.pdf") as &[u8],
                buckets::GRAPHICS_OPTIONAL_CONTENT,
                "unsupported optional-content membership policy",
            ),
            (
                include_bytes!("../../../fixtures/generated/xfa-dynamic-no-static-appearance.pdf")
                    as &[u8],
                buckets::FORM_XFA_DYNAMIC,
                "dynamic XFA without static appearance",
            ),
            (
                include_bytes!("../../../fixtures/generated/chat-emoji-fallback-boundary.pdf")
                    as &[u8],
                buckets::TEXT_FONT_PROGRAM,
                "unsupported emoji text layout boundary",
            ),
        ];

        for &(bytes, bucket, label) in fixtures {
            let error = match ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions::default(),
            ) {
                Ok(_) => panic!("{label} should fail as unsupported"),
                Err(error) => error,
            };
            assert_eq!(
                error.class(),
                pdfrust_thumbnail::ThumbnailErrorClass::Unsupported,
                "{label}"
            );
            assert_eq!(error.unsupported_feature_bucket(), Some(bucket), "{label}");
        }

        let memory_error = NativeBackend::with_render_limits(NativeRenderLimits {
            max_page_pixels: 1,
            ..NativeRenderLimits::default()
        })
        .render(
            PdfSource::from_bytes(include_bytes!("../../../fixtures/generated/text-page.pdf")),
            &ThumbnailOptions::default(),
        )
        .expect_err("tight page raster budget should fail");
        assert_eq!(
            memory_error.unsupported_feature_bucket(),
            Some(buckets::RENDERER_MEMORY_BUDGET)
        );
    }

    #[test]
    fn native_backend_should_render_generated_pdf20_basic_office_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/pdf20-basic-office.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("PDF 2.0 basic office fixture should render through native backend");

        assert!(thumbnail.width > 0);
        assert!(thumbnail.height > 0);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_pdf20_associated_files_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/pdf20-associated-files.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("PDF 2.0 associated-files fixture should render through native backend");

        assert!(thumbnail.width > 0);
        assert!(thumbnail.height > 0);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_report_generated_pdf20_black_point_compensation_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/pdf20-black-point-compensation.pdf");
        assert_unsupported_feature_fixture(bytes, "graphics.color-management");
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
    fn native_backend_should_keep_generated_ocr_text_layer_invisible() {
        let bytes = include_bytes!("../../../fixtures/generated/ocr-invisible-text-layer.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated OCR text layer fixture should render");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 160);
        assert_eq!(rgba_at(&thumbnail, 20, 124), [209, 209, 199, 255]);
        assert_eq!(rgba_at(&thumbnail, 20, 100), [199, 199, 189, 255]);
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
    fn native_backend_should_render_generated_mobile_scan_fixtures() {
        type MobileScanFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[MobileScanFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/mobile-rotated-camera-scan.pdf")
                    as &[u8],
                320,
                240,
                320,
                "mobile rotated camera scan",
                70_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/mobile-cropped-photo-scan.pdf")
                    as &[u8],
                200,
                260,
                260,
                "mobile cropped photo scan",
                50_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/mobile-ocr-overlay-scan.pdf")
                    as &[u8],
                220,
                300,
                300,
                "mobile OCR overlay scan",
                60_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/mobile-mixed-compression-scan.pdf")
                    as &[u8],
                260,
                180,
                260,
                "mobile mixed compression scan",
                45_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve image-dominant scan content"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_image_heavy_memory_fixtures() {
        type ImageHeavyFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[ImageHeavyFixture] = &[
            (
                include_bytes!(
                    "../../../fixtures/generated/image-heavy-repeated-xobject-report.pdf"
                ) as &[u8],
                400,
                480,
                480,
                "repeated image XObject report",
                150_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/image-heavy-rotated-mask-sheet.pdf")
                    as &[u8],
                360,
                260,
                360,
                "rotated masked image sheet",
                60_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/scanner-large-image-budget.pdf")
                    as &[u8],
                320,
                440,
                440,
                "large scanner image budget",
                120_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve image-heavy page content"
            );
        }
    }

    #[test]
    fn native_low_memory_profile_should_render_generated_image_heavy_fixtures() {
        let backend = NativeBackend::low_memory();

        for &(bytes, label) in &[
            (
                include_bytes!(
                    "../../../fixtures/generated/image-heavy-repeated-xobject-report.pdf"
                )
                .as_slice(),
                "repeated image XObject report",
            ),
            (
                include_bytes!("../../../fixtures/generated/image-heavy-rotated-mask-sheet.pdf")
                    .as_slice(),
                "rotated masked image sheet",
            ),
        ] {
            let thumbnail = backend
                .render(
                    PdfSource::from_bytes(bytes),
                    &ThumbnailOptions {
                        page_index: 0,
                        max_edge: 160,
                        background: pdfrust_thumbnail::Rgba::WHITE,
                        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
                        timeout: std::time::Duration::from_secs(5),
                        annotation_mode: AnnotationMode::Screen,
                        form_appearance_mode: FormAppearanceMode::DocumentState,
                    },
                )
                .unwrap_or_else(|error| panic!("{label} should render under low memory: {error}"));

            assert!(thumbnail.width <= 160);
            assert!(thumbnail.height <= 160);
            assert!(thumbnail.bytes.len() <= 160 * 160 * 4);
            assert!(thumbnail
                .bytes
                .chunks_exact(4)
                .any(|pixel| pixel != [255, 255, 255, 255]));
        }
    }

    #[test]
    fn native_backend_should_keep_generated_mobile_ocr_layer_invisible() {
        let bytes = include_bytes!("../../../fixtures/generated/mobile-ocr-overlay-scan.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 300,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated mobile OCR overlay fixture should render");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 300);
        assert_eq!(rgba_at(&thumbnail, 42, 86), [230, 230, 230, 255]);
        assert_eq!(rgba_at(&thumbnail, 42, 114), [230, 230, 230, 255]);
    }

    #[test]
    fn native_backend_should_inspect_generated_mobile_scan_geometry() {
        let rotated = include_bytes!("../../../fixtures/generated/mobile-rotated-camera-scan.pdf");
        let cropped = include_bytes!("../../../fixtures/generated/mobile-cropped-photo-scan.pdf");

        let rotated_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(rotated))
                .expect("rotated mobile scan fixture should inspect");
        let cropped_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(cropped))
                .expect("cropped mobile scan fixture should inspect");

        assert_eq!(
            rotated_metadata.first_page_size(),
            Some(PageSize {
                width: 320.0,
                height: 240.0,
            })
        );
        assert_eq!(
            cropped_metadata.first_page_size(),
            Some(PageSize {
                width: 200.0,
                height: 260.0,
            })
        );
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
        assert_eq!(rgba_at(&thumbnail, 30, 30), [51, 178, 77, 255]);
        assert_eq!(rgba_at(&thumbnail, 88, 24), [51, 178, 77, 255]);
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
    fn native_backend_should_render_generated_knockout_transparency_group_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/transparency-knockout-group.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect(
            "generated knockout transparency group fixture should render through native backend",
        );

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 5, 5), [128, 128, 128, 255]);
        assert_eq!(rgba_at(&thumbnail, 25, 85), [191, 63, 63, 255]);
        assert_eq!(rgba_at(&thumbnail, 55, 55), [95, 31, 158, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_isolated_alpha_group_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/transparency-isolated-alpha-group.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated isolated alpha group fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 25, 85), [191, 63, 63, 255]);
        assert_eq!(rgba_at(&thumbnail, 55, 55), [95, 31, 158, 255]);
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
    fn native_backend_should_render_generated_blend_mode_array_fallback_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/blend-mode-array-fallback.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated blend mode array fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 40, 80), [0, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_report_generated_unsupported_blend_mode_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/unsupported-blend-mode.pdf");
        assert_unsupported_feature_fixture(bytes, "graphics.transparency");
    }

    #[test]
    fn native_backend_should_report_generated_extgstate_luminosity_soft_mask_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/extgstate-luminosity-soft-mask.pdf");
        assert_unsupported_feature_fixture(bytes, "graphics.transparency");
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
        assert_eq!(rgba_at(&thumbnail, 20, 100), [191, 64, 64, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 100), [64, 64, 191, 255]);
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
    fn native_backend_should_report_generated_unsupported_mesh_shading_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/mesh-shading-unsupported.pdf");
        assert_unsupported_feature_fixture(bytes, "graphics.pattern-shading");
    }

    #[test]
    fn native_backend_should_render_generated_type4_mesh_shading_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/type4-mesh-shading.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type 4 mesh shading fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        let red_corner = rgba_at(&thumbnail, 8, 112);
        let green_corner = rgba_at(&thumbnail, 104, 112);
        let blue_corner = rgba_at(&thumbnail, 8, 16);
        assert!(red_corner[0] > red_corner[1] && red_corner[0] > red_corner[2]);
        assert!(green_corner[1] > green_corner[0] && green_corner[1] > green_corner[2]);
        assert!(blue_corner[2] > blue_corner[0] && blue_corner[2] > blue_corner[1]);
    }

    #[test]
    fn native_backend_should_render_generated_separation_spot_color_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/separation-spot-color.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Separation spot-color fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 24, 36), [255, 180, 140, 255]);
        assert_eq!(rgba_at(&thumbnail, 24, 90), [255, 89, 0, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_overprint_spot_approximation_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/overprint-spot-approximation.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated overprint spot-color approximation fixture should render natively");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 24, 90), [255, 125, 71, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_devicen_spot_color_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/devicen-spot-color.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated DeviceN spot-color fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 24, 88), [128, 159, 239, 255]);
        assert_eq!(rgba_at(&thumbnail, 24, 44), [117, 152, 238, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_spot_color_visual_review_fixtures() {
        let fixtures = [
            (
                include_bytes!("../../../fixtures/generated/spot-letterhead-separation.pdf")
                    as &[u8],
                180,
                130,
                "spot letterhead separation",
                (10, 10),
            ),
            (
                include_bytes!("../../../fixtures/generated/spot-invoice-devicen-stamp.pdf")
                    as &[u8],
                190,
                140,
                "spot invoice devicen stamp",
                (132, 105),
            ),
            (
                include_bytes!("../../../fixtures/generated/spot-cmyk-tint-swatch.pdf") as &[u8],
                180,
                130,
                "spot cmyk tint swatch",
                (132, 45),
            ),
        ];

        for (bytes, expected_width, expected_height, label, sample) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} should render natively: {error}"));

            assert_eq!(thumbnail.width, expected_width);
            assert_eq!(thumbnail.height, expected_height);
            let sample = rgba_at(&thumbnail, sample.0, sample.1);
            assert!(
                sample[0] < 245 || sample[1] < 245 || sample[2] < 245,
                "{label} should keep visible spot-color review content at sample point"
            );
        }
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
    fn native_backend_should_render_generated_uncolored_tiling_pattern_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/uncolored-tiling-pattern.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated uncolored tiling-pattern fixture should render through native backend");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 6, 114), [51, 178, 77, 255]);
        assert_eq!(rgba_at(&thumbnail, 18, 114), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 30, 90), [51, 178, 77, 255]);
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
    fn native_backend_should_synthesize_highlight_annotation_without_appearance() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/highlight-annotation-without-appearance.pdf"
        );
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated highlight fallback fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 30, 52), [255, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 15, 95), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 70, 45), [255, 255, 0, 255]);
    }

    #[test]
    fn native_backend_should_synthesize_markup_annotations_without_appearance() {
        let bytes =
            include_bytes!("../../../fixtures/generated/markup-annotations-without-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated markup fallback fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 30, 38), [255, 127, 127, 255]);
        assert_eq!(rgba_at(&thumbnail, 15, 85), [0, 115, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 109, 80), [0, 140, 0, 255]);
    }

    #[test]
    fn native_backend_should_keep_link_annotation_without_appearance_invisible() {
        let bytes =
            include_bytes!("../../../fixtures/generated/link-annotation-without-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated link fallback control fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 70, 50), [255, 255, 255, 255]);
        assert_eq!(rgba_at(&thumbnail, 15, 95), [0, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_synthesize_text_note_annotation_without_appearance() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/text-note-annotation-without-appearance.pdf"
        );
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated text note fallback fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 120);
        assert_eq!(rgba_at(&thumbnail, 90, 29), [255, 255, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 80, 29), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 105, 29), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_apply_annotation_flags_for_screen_and_print_preview() {
        let bytes =
            include_bytes!("../../../fixtures/generated/annotation-print-preview-flags.pdf");
        let screen = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                annotation_mode: AnnotationMode::Screen,
                ..ThumbnailOptions::default()
            },
        )
        .expect("annotation flag fixture should render in screen mode");
        let print = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                annotation_mode: AnnotationMode::Print,
                ..ThumbnailOptions::default()
            },
        )
        .expect("annotation flag fixture should render in print mode");

        assert_eq!(screen.width, 180);
        assert_eq!(screen.height, 80);
        assert_eq!(rgba_at(&screen, 20, 60), [229, 0, 0, 255]);
        assert_eq!(rgba_at(&screen, 55, 60), [0, 51, 229, 255]);
        assert_eq!(rgba_at(&screen, 90, 60), [235, 235, 235, 255]);
        assert_eq!(rgba_at(&screen, 125, 60), [235, 235, 235, 255]);
        assert_eq!(rgba_at(&screen, 160, 60), [235, 235, 235, 255]);

        assert_eq!(rgba_at(&print, 20, 60), [229, 0, 0, 255]);
        assert_eq!(rgba_at(&print, 55, 60), [235, 235, 235, 255]);
        assert_eq!(rgba_at(&print, 90, 60), [235, 235, 235, 255]);
        assert_eq!(rgba_at(&print, 125, 60), [235, 235, 235, 255]);
        assert_eq!(rgba_at(&print, 160, 60), [140, 51, 204, 255]);
    }

    #[test]
    fn native_backend_should_report_unsupported_freetext_synthesis() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/freetext-annotation-without-appearance.pdf"
        );
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("FreeText without appearance should be a typed unsupported boundary");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_ANNOTATION_APPEARANCE)
        );
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
        assert_eq!(rgba_at(&thumbnail, 25, 86), [229, 229, 229, 255]);
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
    fn native_backend_should_render_existing_text_appearance_over_stale_value() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-text-field-stale-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("stale AcroForm text appearance fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 40, 40), [242, 209, 184, 255]);
        assert_eq!(rgba_at(&thumbnail, 95, 40), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_reject_requested_form_appearance_mutation() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-text-field-stale-appearance.pdf");
        let input = bytes.to_vec();
        let before = input.clone();
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(&input),
            &ThumbnailOptions {
                max_edge: 140,
                form_appearance_mode: FormAppearanceMode::RequestedMutation,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("form preview mutation should not render silently");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_FORM_APPEARANCE_MUTATION)
        );
        assert_eq!(input, before);
    }

    #[test]
    fn native_backend_should_render_generated_static_xfa_appearance_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/xfa-static-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated static XFA appearance fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 40, 40), [217, 235, 255, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 30, 30), 96);
        assert_eq!(rgba_at(&thumbnail, 95, 40), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_reject_generated_dynamic_xfa_without_static_appearance() {
        let bytes =
            include_bytes!("../../../fixtures/generated/xfa-dynamic-no-static-appearance.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("dynamic XFA without static appearances should not render silently");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(
            error.unsupported_feature_bucket(),
            Some(BUCKET_FORM_XFA_DYNAMIC)
        );
    }

    #[test]
    fn native_backend_should_synthesize_acroform_text_field_without_appearance() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/acroform-text-field-missing-appearance.pdf"
        );
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm text field fallback fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 60, 37), [235, 245, 255, 255]);
        assert_region_contains_low_intensity(&thumbnail, 36..86, 32..44, 220);
        assert_eq!(rgba_at(&thumbnail, 105, 40), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_synthesize_acroform_choice_without_appearance() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-choice-missing-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm choice fallback fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 70, 37), [235, 245, 255, 255]);
        assert_region_contains_low_intensity(&thumbnail, 36..92, 32..44, 220);
        assert_eq!(rgba_at(&thumbnail, 115, 40), [255, 255, 255, 255]);
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
    fn native_backend_should_use_appearance_state_over_checkbox_value() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/acroform-checkbox-stale-appearance-state.pdf"
        );
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 80,
                ..ThumbnailOptions::default()
            },
        )
        .expect("stale AcroForm checkbox appearance-state fixture should render");

        assert_eq!(thumbnail.width, 80);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 30, 40), [255, 255, 255, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 20, 30), 96);
    }

    #[test]
    fn native_backend_should_synthesize_acroform_checkbox_without_appearance() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-checkbox-missing-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 80,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm checkbox fallback fixture should render");

        assert_eq!(thumbnail.width, 80);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 30, 40), [0, 0, 0, 255]);
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
    fn native_backend_should_synthesize_acroform_radio_without_appearance() {
        let bytes =
            include_bytes!("../../../fixtures/generated/acroform-radio-missing-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 100,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated AcroForm radio fallback fixture should render");

        assert_eq!(thumbnail.width, 100);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 30, 40), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 45, 40), [255, 255, 255, 255]);
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
    fn native_backend_should_render_generated_digital_signature_appearance_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/digital-signature-appearance.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated digital signature appearance fixture should render");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 30, 35), [240, 240, 240, 255]);
        assert_low_intensity(rgba_at(&thumbnail, 20, 25), 96);
        assert_eq!(rgba_at(&thumbnail, 130, 45), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_e_signature_workflow_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/e-signature-contract-workflow.pdf")
                    as &[u8],
                360,
                260,
                "e-signature contract workflow",
                16_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/e-signature-audit-certificate.pdf")
                    as &[u8],
                420,
                300,
                "e-signature audit certificate",
                18_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/e-signature-incremental-revision.pdf")
                    as &[u8],
                300,
                180,
                "e-signature incremental revision",
                4_300,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve visible workflow content: {visible_pixels} < {min_visible_pixels}"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_tagged_visual_integrity_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/tagged-report-visual-integrity.pdf")
                    as &[u8],
                360,
                260,
                "tagged report visual integrity",
                18_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-form-visual-integrity.pdf")
                    as &[u8],
                300,
                200,
                "tagged form visual integrity",
                10_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-office-alt-text.pdf") as &[u8],
                420,
                280,
                "tagged office alt text",
                20_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-structure-heavy-report.pdf")
                    as &[u8],
                360,
                300,
                "tagged structure heavy report",
                14_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve visible page content"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_file_attachment_annotation_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/file-attachment-annotation.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated file attachment annotation fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 30, 45), [38, 38, 38, 255]);
        assert_eq!(rgba_at(&thumbnail, 65, 45), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_linearized_first_page_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-first-page.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated linearized first page fixture should render");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 24, 44), [26, 64, 115, 255]);
        assert_eq!(rgba_at(&thumbnail, 110, 44), [222, 240, 255, 255]);
    }

    #[test]
    fn native_backend_should_fallback_from_generated_malformed_linearization_hints() {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-malformed-hints.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated malformed linearization fixture should render through full fallback");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 24, 44), [26, 64, 115, 255]);
    }

    #[test]
    fn native_backend_first_page_preview_should_report_linearized_loader() {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-first-page.pdf");
        let preview = NativeBackend::new()
            .render_first_page_preview(
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: 160,
                    ..ThumbnailOptions::default()
                },
            )
            .expect("linearized fixture should render through first-page preview");

        assert_eq!(
            preview.load_mode,
            FirstPagePreviewLoadMode::LinearizedFirstPage
        );
        assert_eq!(preview.load_mode.as_str(), "linearized-first-page");
        assert!(preview.memory.first_page_only);
        assert_eq!(preview.memory.input_bytes, bytes.len());
        assert!(preview.memory.loaded_objects > 0);
        assert!(preview.memory.loaded_object_bytes > 0);
        assert!(preview.memory.first_page_section_bytes.is_some());
        let full = load_classic_document(PdfBytes::new(bytes)).expect("full classic document");
        assert!(preview.memory.loaded_objects < full.load_metrics.loaded_objects);
        assert!(preview.memory.loaded_object_bytes < full.load_metrics.loaded_object_bytes);
        assert_eq!(preview.thumbnail.width, 160);
        assert_eq!(preview.thumbnail.height, 90);
        assert_eq!(rgba_at(&preview.thumbnail, 24, 44), [26, 64, 115, 255]);
    }

    #[test]
    fn native_backend_first_page_preview_should_report_full_document_fallback() {
        let bytes = include_bytes!("../../../fixtures/generated/linearized-malformed-hints.pdf");
        let preview = NativeBackend::new()
            .render_first_page_preview(
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    page_index: 1,
                    max_edge: 160,
                    ..ThumbnailOptions::default()
                },
            )
            .expect("malformed linearization should render through first-page preview fallback");

        assert_eq!(preview.load_mode, FirstPagePreviewLoadMode::FullDocument);
        assert_eq!(preview.load_mode.as_str(), "full-document");
        assert!(!preview.memory.first_page_only);
        assert_eq!(preview.memory.input_bytes, bytes.len());
        assert!(preview.memory.loaded_objects > 0);
        assert!(preview.memory.loaded_object_bytes > 0);
        assert!(preview.memory.first_page_section_bytes.is_some());
        assert_eq!(preview.thumbnail.width, 160);
        assert_eq!(preview.thumbnail.height, 90);
        assert_eq!(rgba_at(&preview.thumbnail, 24, 44), [26, 64, 115, 255]);
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
        assert_eq!(rgba_at(&thumbnail, 70, 50), [229, 0, 0, 255]);
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
    fn native_backend_should_hide_nested_optional_content_layer_fixture() {
        let bytes =
            include_bytes!("../../../fixtures/generated/optional-content-nested-layers.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 140,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated nested optional-content fixture should render");

        assert_eq!(thumbnail.width, 140);
        assert_eq!(thumbnail.height, 90);
        assert_eq!(rgba_at(&thumbnail, 20, 65), [0, 153, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 65), [0, 51, 229, 255]);
        assert_eq!(rgba_at(&thumbnail, 100, 65), [255, 255, 255, 255]);
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
    fn native_backend_should_report_unsupported_optional_content_usage_application() {
        let bytes =
            include_bytes!("../../../fixtures/generated/optional-content-usage-application.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect_err("usage application policy should not render silently");

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
        assert_eq!(rgba_at(&thumbnail, 20, 50), [229, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 60, 50), [255, 255, 255, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_incremental_deleted_object_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/incremental-deleted-object.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated incremental deleted-object fixture should render");

        assert_eq!(thumbnail.width, 120);
        assert_eq!(thumbnail.height, 80);
        assert_eq!(rgba_at(&thumbnail, 30, 40), [0, 128, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 100, 70), [255, 255, 255, 255]);
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
        assert_eq!(rgba_at(&thumbnail, 20, 50), [0, 0, 229, 255]);
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
        assert_eq!(rgba_at(&thumbnail, 20, 50), [229, 0, 0, 255]);
    }

    #[test]
    fn native_backend_should_render_recoverable_corrupt_common_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/malformed-xref-offset-drift.pdf")
                    as &[u8],
                120,
                80,
                "xref offset drift",
                2_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/linearized-malformed-hints.pdf")
                    as &[u8],
                160,
                90,
                "malformed linearization hints",
                2_000,
            ),
            (
                include_bytes!(
                    "../../../fixtures/generated/corrupt-broken-annotation-reference.pdf"
                ) as &[u8],
                120,
                80,
                "broken annotation reference",
                2_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(thumbnail.width, expected_width, "{label} width");
            assert_eq!(thumbnail.height, expected_height, "{label} height");
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should keep visible page content"
            );
        }
    }

    #[test]
    fn native_backend_should_report_common_corruption_as_stable_malformed() {
        let fixtures: &[(&[u8], &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/corrupt-xref-object-mismatch.pdf")
                    as &[u8],
                "xref object mismatch",
            ),
            (
                include_bytes!("../../../fixtures/generated/corrupt-partial-stream.pdf") as &[u8],
                "partial stream",
            ),
            (
                include_bytes!("../../../fixtures/generated/corrupt-page-tree-missing-kids.pdf")
                    as &[u8],
                "missing page tree kids",
            ),
        ];

        for &(bytes, label) in fixtures {
            let render_error = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: 120,
                    ..ThumbnailOptions::default()
                },
            )
            .expect_err("corrupt fixture should not render");
            let metadata_error = DocumentMetadataBackend::inspect(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
            )
            .expect_err("corrupt fixture should not inspect");

            assert_eq!(render_error, ThumbnailError::Malformed, "{label} render");
            assert_eq!(
                metadata_error,
                ThumbnailError::Malformed,
                "{label} metadata"
            );
        }
    }

    #[test]
    fn native_backend_should_keep_malformed_metadata_from_aborting_render() {
        let fixtures: &[(&[u8], u32, u32, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/malformed-tagged-structure.pdf")
                    as &[u8],
                120,
                80,
                "malformed tagged structure",
            ),
            (
                include_bytes!(
                    "../../../fixtures/generated/corrupt-info-metadata-non-dictionary.pdf"
                ) as &[u8],
                120,
                80,
                "non-dictionary info metadata",
            ),
        ];

        for &(bytes, expected_width, expected_height, label) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));
            let metadata_error = DocumentMetadataBackend::inspect(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
            )
            .expect_err("corrupt metadata fixture should not inspect");

            assert_eq!(thumbnail.width, expected_width, "{label} width");
            assert_eq!(thumbnail.height, expected_height, "{label} height");
            assert_eq!(metadata_error, ThumbnailError::Malformed, "{label}");
        }
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
    fn native_backend_should_render_generated_identity_h_cjk_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/identity-h-cjk-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Identity-H CJK fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_identity_v_cjk_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/identity-v-cjk-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Identity-V CJK fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_cmap_codespace_range_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/cmap-codespace-range-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated CMap codespace fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
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
    fn native_backend_should_render_generated_ligature_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/opentype-ligature-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated ligature fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_combining_mark_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/combining-mark-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated combining mark fixture should render through native backend");

        assert_eq!(thumbnail.width, 180);
        assert_eq!(thumbnail.height, 100);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_report_generated_chat_emoji_boundary_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/chat-emoji-fallback-boundary.pdf");
        assert_unsupported_feature_fixture(bytes, buckets::TEXT_FONT_PROGRAM);
    }

    #[test]
    fn native_backend_should_render_generated_arabic_shaped_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/arabic-shaped-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 180,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Arabic shaped text fixture should render through native backend");

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
    fn native_backend_should_render_generated_missing_font_office_export_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/missing-font-office-export.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated office missing-font fixture should render through native backend");

        assert_eq!(thumbnail.width, 260);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_missing_font_invoice_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/missing-font-invoice.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated invoice missing-font fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_missing_font_browser_print_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/missing-font-browser-print.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 260,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated browser missing-font fixture should render through native backend");

        assert_eq!(thumbnail.width, 260);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_type1_fontfile_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/type1-fontfile-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 240,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type1 FontFile fixture should render through native backend");

        assert_eq!(thumbnail.width, 240);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_cff_fontfile3_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/cff-fontfile3-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 240,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated CFF FontFile3 fixture should render through native backend");

        assert_eq!(thumbnail.width, 240);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_browser_print_edge_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/browser-print-sticky-headers.pdf")
                    as &[u8],
                160,
                160,
                "sticky headers browser print",
            ),
            (
                include_bytes!("../../../fixtures/generated/browser-print-clipped-backgrounds.pdf")
                    as &[u8],
                160,
                160,
                "clipped backgrounds browser print",
            ),
            (
                include_bytes!("../../../fixtures/generated/browser-print-transformed-cards.pdf")
                    as &[u8],
                160,
                160,
                "transformed cards browser print",
            ),
            (
                include_bytes!("../../../fixtures/generated/browser-print-raster-vector-mix.pdf")
                    as &[u8],
                160,
                160,
                "raster/vector browser print",
            ),
        ];

        for &(bytes, expected_width, expected_height, label) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .expect("generated browser-print edge fixture should render through native backend");

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} height should match"
            );
            assert!(
                thumbnail
                    .bytes
                    .chunks_exact(4)
                    .any(|pixel| pixel != [255, 255, 255, 255]),
                "{label} fixture should render visible content"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_email_web_archive_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/email-client-thread.pdf") as &[u8],
                300,
                420,
                "email client thread",
                20_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/email-inline-image-link.pdf")
                    as &[u8],
                260,
                220,
                "email inline image link",
                10_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/web-archive-snapshot.pdf") as &[u8],
                320,
                360,
                "web archive snapshot",
                20_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/email-attachment-summary.pdf")
                    as &[u8],
                240,
                180,
                "email attachment summary",
                6_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} should preserve visible email/web archive structure"
            );
        }
    }

    #[test]
    fn native_backend_should_inspect_generated_email_attachment_policy_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/email-attachment-summary.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("email attachment summary fixture should inspect");

        assert!(metadata.structure.has_embedded_files);
        assert!(!metadata.structure.has_portfolio_collection);
        assert!(!metadata.structure.has_file_attachment_annotations);
    }

    #[test]
    fn native_parallel_renderer_should_sample_generated_email_thread_pages() {
        let bytes = include_bytes!("../../../fixtures/generated/email-client-thread.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("email client thread fixture should inspect");
        assert_eq!(metadata.page_count(), 3);

        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0, 2],
            &ThumbnailOptions {
                max_edge: 420,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 420 * 420 * 2,
            },
        )
        .expect("email thread sample pages should render through parallel scheduler");

        assert_eq!(result.workers, 2);
        assert_eq!(result.pages.len(), 2);
        for page in result.pages {
            assert_eq!(page.width, 300);
            assert_eq!(page.height, 420);
        }
    }

    #[test]
    fn native_backend_should_render_generated_font_subset_regression_fixtures() {
        let cases: &[(&str, &[u8], u32, u32)] = &[
            (
                "subset TrueType widths",
                include_bytes!("../../../fixtures/generated/subset-truetype-widths.pdf"),
                220,
                120,
            ),
            (
                "subset CFF ToUnicode",
                include_bytes!("../../../fixtures/generated/subset-cff-tounicode.pdf"),
                220,
                120,
            ),
            (
                "subset CID widths",
                include_bytes!("../../../fixtures/generated/subset-cid-widths.pdf"),
                220,
                120,
            ),
            (
                "subset Type3 repeated CharProcs",
                include_bytes!("../../../fixtures/generated/subset-type3-repeated-charprocs.pdf"),
                260,
                120,
            ),
            (
                "subset missing font",
                include_bytes!("../../../fixtures/generated/subset-missing-font.pdf"),
                240,
                120,
            ),
        ];

        for (name, bytes, expected_width, expected_height) in cases {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: 260,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{name} should render through native backend: {error}"));

            assert_eq!(thumbnail.width, *expected_width, "{name} width");
            assert_eq!(thumbnail.height, *expected_height, "{name} height");
            assert!(
                thumbnail
                    .bytes
                    .chunks_exact(4)
                    .any(|pixel| pixel != [255, 255, 255, 255]),
                "{name} should paint visible pixels"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_type3_vector_text_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/type3-vector-text.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type3 vector text fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_type3_symbol_font_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/type3-symbol-font.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type3 symbol fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 120);
        assert!(thumbnail
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
    }

    #[test]
    fn native_backend_should_render_generated_type3_barcode_font_fixture() {
        let bytes = include_bytes!("../../../fixtures/generated/type3-barcode-font.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 220,
                ..ThumbnailOptions::default()
            },
        )
        .expect("generated Type3 barcode fixture should render through native backend");

        assert_eq!(thumbnail.width, 220);
        assert_eq!(thumbnail.height, 160);
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
    fn native_backend_should_render_generated_office_vector_effect_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/office-vector-grouped-shapes.pdf")
                    as &[u8],
                320,
                220,
                "grouped office vector shapes",
            ),
            (
                include_bytes!("../../../fixtures/generated/office-vector-nested-clips.pdf")
                    as &[u8],
                300,
                210,
                "nested office vector clips",
            ),
            (
                include_bytes!(
                    "../../../fixtures/generated/office-vector-clipped-transparency-group.pdf"
                ) as &[u8],
                320,
                220,
                "clipped office transparency group",
            ),
            (
                include_bytes!("../../../fixtures/generated/office-vector-repeated-effects.pdf")
                    as &[u8],
                360,
                240,
                "repeated office vector effects",
            ),
        ];

        for &(bytes, expected_width, expected_height, label) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .expect("generated office vector fixture should render through native backend");

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} height should match"
            );
            assert!(
                thumbnail
                    .bytes
                    .chunks_exact(4)
                    .any(|pixel| pixel != [255, 255, 255, 255]),
                "{label} fixture should render visible content"
            );
        }
    }

    #[test]
    fn native_backend_should_clip_transparency_group_to_parent_clip() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/office-vector-clipped-transparency-group.pdf"
        );
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 160,
                ..ThumbnailOptions::default()
            },
        )
        .expect("clipped transparency group should render through native backend");

        assert_eq!(thumbnail.width, 160);
        assert_eq!(thumbnail.height, 110);
        assert_eq!(rgba_at(&thumbnail, 25, 60), [250, 250, 247, 255]);
        assert_ne!(rgba_at(&thumbnail, 60, 60), [250, 250, 247, 255]);
    }

    #[test]
    fn native_backend_should_render_generated_business_document_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/business-invoice-dense.pdf") as &[u8],
                300,
                200,
                "business invoice",
            ),
            (
                include_bytes!("../../../fixtures/generated/account-statement-ledger.pdf")
                    as &[u8],
                300,
                200,
                "account statement",
            ),
            (
                include_bytes!("../../../fixtures/generated/financial-annual-report-page.pdf")
                    as &[u8],
                420,
                300,
                "financial annual report",
            ),
            (
                include_bytes!("../../../fixtures/generated/financial-cashflow-statement.pdf")
                    as &[u8],
                340,
                260,
                "financial cashflow statement",
            ),
            (
                include_bytes!("../../../fixtures/generated/financial-chart-summary.pdf")
                    as &[u8],
                380,
                240,
                "financial chart summary",
            ),
            (
                include_bytes!("../../../fixtures/generated/thermal-receipt.pdf") as &[u8],
                160,
                260,
                "thermal receipt",
            ),
            (
                include_bytes!("../../../fixtures/generated/business-form-stamp-signature.pdf")
                    as &[u8],
                260,
                180,
                "business form",
            ),
        ];

        for &(bytes, expected_width, expected_height, label) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            assert!(
                thumbnail
                    .bytes
                    .chunks_exact(4)
                    .any(|pixel| pixel != [255, 255, 255, 255]),
                "{label} fixture should render visible content"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_legal_document_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/legal-contract-signature-blocks.pdf")
                    as &[u8],
                320,
                420,
                "legal contract signature blocks",
                3_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/legal-visible-redactions.pdf")
                    as &[u8],
                300,
                380,
                "legal visible redactions",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/legal-filing-stamp-comments.pdf")
                    as &[u8],
                320,
                400,
                "legal filing stamp comments",
                3_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/legal-scanned-attachment-packet.pdf")
                    as &[u8],
                260,
                340,
                "legal scanned attachment packet first page",
                1_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve legal document visual content"
            );
        }
    }

    #[test]
    fn native_backend_should_keep_generated_legal_redaction_rectangles_visible() {
        let bytes = include_bytes!("../../../fixtures/generated/legal-visible-redactions.pdf");
        let thumbnail = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                max_edge: 380,
                ..ThumbnailOptions::default()
            },
        )
        .expect("legal visible redaction fixture should render");

        assert_eq!(rgba_at(&thumbnail, 120, 103), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 150, 127), [0, 0, 0, 255]);
        assert_eq!(rgba_at(&thumbnail, 120, 151), [0, 0, 0, 255]);
    }

    #[test]
    fn native_parallel_renderer_should_sample_generated_legal_attachment_pages() {
        let bytes =
            include_bytes!("../../../fixtures/generated/legal-scanned-attachment-packet.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("legal scanned attachment packet should inspect");

        assert_eq!(metadata.page_count(), 2);

        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0, 1],
            &ThumbnailOptions {
                max_edge: 340,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 340 * 340 * 2,
            },
        )
        .expect("legal attachment pages should render through parallel scheduler");

        assert_eq!(result.pages.len(), 2);
        for page in result.pages {
            assert_eq!(page.width, 260);
            assert_eq!(page.height, 340);
        }
    }

    #[test]
    fn native_backend_should_render_generated_presentation_slide_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/slide-title-gradient.pdf") as &[u8],
                320,
                180,
                "title gradient slide",
            ),
            (
                include_bytes!("../../../fixtures/generated/slide-layered-image-shadow.pdf")
                    as &[u8],
                320,
                180,
                "layered image slide",
            ),
            (
                include_bytes!("../../../fixtures/generated/slide-rotated-callout.pdf") as &[u8],
                320,
                180,
                "rotated callout slide",
            ),
            (
                include_bytes!("../../../fixtures/generated/slide-speaker-notes-page.pdf")
                    as &[u8],
                240,
                320,
                "speaker notes page",
            ),
        ];

        for &(bytes, expected_width, expected_height, label) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            assert!(
                thumbnail
                    .bytes
                    .chunks_exact(4)
                    .any(|pixel| pixel != [255, 255, 255, 255]),
                "{label} fixture should render visible content"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_spreadsheet_grid_fixtures() {
        let fixtures: &[(&[u8], u32, u32, &str, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/spreadsheet-frozen-header.pdf")
                    as &[u8],
                320,
                200,
                "frozen header spreadsheet",
                5_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/spreadsheet-dense-numeric-grid.pdf")
                    as &[u8],
                320,
                220,
                "dense numeric spreadsheet",
                5_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/spreadsheet-clipped-cells.pdf")
                    as &[u8],
                260,
                180,
                "clipped cells spreadsheet",
                3_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/spreadsheet-vector-stress-grid.pdf")
                    as &[u8],
                360,
                240,
                "vector stress spreadsheet",
                6_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve dense grid/text pixels"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_technical_drawing_fixtures() {
        type TechnicalFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[TechnicalFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/technical-linework-dimensions.pdf")
                    as &[u8],
                360,
                240,
                360,
                "linework dimensions drawing",
                4_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/technical-hatch-clipping.pdf")
                    as &[u8],
                300,
                220,
                300,
                "hatch clipping drawing",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/technical-large-coordinate-plan.pdf")
                    as &[u8],
                400,
                240,
                400,
                "large coordinate drawing",
                2_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/technical-repeated-symbols.pdf")
                    as &[u8],
                320,
                220,
                320,
                "repeated symbols drawing",
                3_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/engineering-floorplan-precision.pdf")
                    as &[u8],
                420,
                300,
                420,
                "engineering floorplan precision",
                5_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/engineering-schematic-symbols.pdf")
                    as &[u8],
                360,
                240,
                360,
                "engineering schematic symbols",
                3_500,
            ),
            (
                include_bytes!("../../../fixtures/generated/engineering-large-transform-detail.pdf")
                    as &[u8],
                400,
                267,
                400,
                "engineering large transform detail",
                2_500,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve fine technical linework"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_chart_dashboard_fixtures() {
        type DashboardFixture = (&'static [u8], u32, u32, &'static str, usize);

        let fixtures: &[DashboardFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/chart-combo-legend.pdf") as &[u8],
                360,
                240,
                "combo chart with legend",
                5_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/dashboard-kpi-panels.pdf") as &[u8],
                360,
                220,
                "kpi dashboard panels",
                8_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/map-marker-clusters.pdf") as &[u8],
                360,
                240,
                "map marker clusters",
                7_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/map-raster-tile-routes.pdf") as &[u8],
                420,
                300,
                "map raster tile routes",
                30_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/map-transparent-zoning-overlay.pdf")
                    as &[u8],
                380,
                260,
                "map transparent zoning overlay",
                18_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/map-optional-layer-policy.pdf")
                    as &[u8],
                360,
                240,
                "map optional layer policy",
                8_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/dashboard-heatmap-overlay.pdf")
                    as &[u8],
                340,
                220,
                "dashboard heatmap overlay",
                20_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve markers, labels, and panels"
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_scientific_report_fixtures() {
        type ScientificFixture = (&'static [u8], u32, u32, &'static str, usize);

        let fixtures: &[ScientificFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/scientific-two-column-paper.pdf")
                    as &[u8],
                360,
                480,
                "two-column scientific paper",
                10_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/academic-publisher-first-page.pdf")
                    as &[u8],
                360,
                480,
                "academic publisher first page",
                10_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/scientific-equation-figure.pdf")
                    as &[u8],
                320,
                240,
                "equation and figure page",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/academic-equation-symbols-page.pdf")
                    as &[u8],
                340,
                260,
                "academic equation symbols page",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/reference-footnote-layout.pdf")
                    as &[u8],
                320,
                260,
                "references and footnotes layout",
                6_000,
            ),
            (
                include_bytes!(
                    "../../../fixtures/generated/layout-columns-footnotes-table-stress.pdf"
                ) as &[u8],
                400,
                520,
                "columns footnotes table stress layout",
                12_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/academic-references-appendix.pdf")
                    as &[u8],
                340,
                300,
                "academic references appendix",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/long-report-sampling.pdf") as &[u8],
                300,
                220,
                "long report first page",
                6_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve paper/report layout structure"
            );
        }
    }

    #[test]
    fn native_parallel_renderer_should_sample_generated_long_report_pages() {
        let bytes = include_bytes!("../../../fixtures/generated/long-report-sampling.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("long report sampling fixture should inspect");

        assert_eq!(metadata.page_count(), 3);

        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0, 2],
            &ThumbnailOptions {
                max_edge: 300,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 300 * 300 * 2,
            },
        )
        .expect("long report sample pages should render through parallel scheduler");

        assert_eq!(result.workers, 2);
        assert_eq!(result.pages.len(), 2);
        for page in &result.pages {
            assert_eq!(page.width, 300);
            assert_eq!(page.height, 220);
            assert!(
                page.bytes
                    .chunks_exact(4)
                    .filter(|pixel| *pixel != [255, 255, 255, 255])
                    .count()
                    >= 6_000
            );
        }
    }

    #[test]
    fn native_parallel_renderer_should_sample_generated_long_document_navigation_pages() {
        let bytes = include_bytes!("../../../fixtures/generated/long-document-navigation-deck.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("long document navigation fixture should inspect");

        assert_eq!(metadata.page_count(), 12);

        let result = render_pages_parallel(
            PdfSource::from_bytes(bytes),
            &[0, 1, 11, 5],
            &ThumbnailOptions {
                max_edge: 210,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 3,
                max_in_flight_pixels: 210 * 210 * 2,
            },
        )
        .expect("first, next, and random long-document pages should render");

        assert_eq!(result.workers, 2);
        assert_eq!(result.pages.len(), 4);
        for page in &result.pages {
            assert_eq!(page.width, 150);
            assert_eq!(page.height, 210);
            assert!(
                page.bytes
                    .chunks_exact(4)
                    .filter(|pixel| *pixel != [255, 255, 255, 255])
                    .count()
                    >= 12_000
            );
        }
    }

    #[test]
    fn native_backend_should_render_generated_longform_text_fixtures() {
        type LongformFixture = (&'static [u8], u32, u32, &'static str, usize);

        let fixtures: &[LongformFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/book-frontmatter-page-labels.pdf")
                    as &[u8],
                260,
                360,
                "book frontmatter",
                4_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/manual-illustrated-chapter.pdf")
                    as &[u8],
                320,
                260,
                "illustrated manual chapter",
                7_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/ebook-narrow-longform.pdf") as &[u8],
                180,
                300,
                "narrow ebook page",
                6_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/longform-repeated-resources.pdf")
                    as &[u8],
                240,
                320,
                "longform repeated resources",
                8_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, label, min_visible_pixels) in fixtures {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge: expected_width.max(expected_height),
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve longform text structure"
            );
        }
    }

    #[test]
    fn native_backend_should_inspect_generated_longform_book_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/book-frontmatter-page-labels.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("book frontmatter fixture should inspect");

        assert_eq!(metadata.page_count(), 5);
        assert!(metadata.outlines.has_outlines);
        assert_eq!(metadata.outlines.item_count, 3);
        assert_eq!(metadata.page_labels.labels.len(), 5);
        assert_eq!(metadata.page_labels.labels[0].label, "i");
        assert_eq!(metadata.page_labels.labels[1].label, "ii");
        assert_eq!(metadata.page_labels.labels[2].label, "Ch-1");
        assert_eq!(metadata.page_labels.labels[4].label, "Ch-3");
    }

    #[test]
    fn native_parallel_renderer_should_sample_generated_longform_pages() {
        let book = include_bytes!("../../../fixtures/generated/book-frontmatter-page-labels.pdf");
        let repeated =
            include_bytes!("../../../fixtures/generated/longform-repeated-resources.pdf");
        let options = ThumbnailOptions {
            max_edge: 320,
            ..ThumbnailOptions::default()
        };

        let book_result = render_pages_parallel(
            PdfSource::from_bytes(book),
            &[0, 2, 4],
            &options,
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 320 * 320 * 2,
            },
        )
        .expect("book frontmatter, chapter, and appendix pages should render");
        assert_eq!(book_result.workers, 2);
        assert_eq!(book_result.pages.len(), 3);

        let repeated_result = render_pages_parallel(
            PdfSource::from_bytes(repeated),
            &[0, 2],
            &options,
            ParallelRenderOptions {
                max_workers: 2,
                max_in_flight_pixels: 320 * 320 * 2,
            },
        )
        .expect("longform repeated resource sample pages should render");
        assert_eq!(repeated_result.workers, 2);
        assert_eq!(repeated_result.pages.len(), 2);
        for page in repeated_result.pages {
            assert_eq!(page.width, 240);
            assert_eq!(page.height, 320);
        }
    }

    #[test]
    fn native_memory_diagnostics_should_bound_longform_caches() {
        let diagnostics = NativeBackend::new().memory_diagnostics();

        assert!(diagnostics.max_font_fallback_cache_entries > 0);
        assert!(diagnostics.max_image_bytes > 0);
        assert!(diagnostics.max_total_image_bytes >= diagnostics.max_image_bytes);
        assert!(diagnostics.max_display_items > 0);
    }

    #[test]
    fn native_backend_should_render_generated_prepress_boundary_fixtures() {
        type PrepressFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[PrepressFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/prepress-trim-bleed-marks.pdf")
                    as &[u8],
                300,
                220,
                300,
                "prepress trim and bleed marks",
                5_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/prepress-output-intent-page-boxes.pdf")
                    as &[u8],
                300,
                220,
                300,
                "prepress output intent page boxes",
                10_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/prepress-registration-color-bars.pdf")
                    as &[u8],
                360,
                180,
                360,
                "prepress registration color bars",
                2_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/prepress-spot-overprint-boundary.pdf")
                    as &[u8],
                240,
                180,
                240,
                "prepress spot overprint boundary",
                10_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match the selected visible box"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match the selected visible box"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve print-oriented visual markers"
            );
        }
    }

    #[test]
    fn native_backend_should_inspect_generated_prepress_page_box_metadata() {
        let bytes =
            include_bytes!("../../../fixtures/generated/prepress-output-intent-page-boxes.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("prepress output-intent fixture should inspect");

        assert_eq!(metadata.page_count(), 1);
        assert_eq!(
            metadata.first_page_size(),
            Some(PageSize {
                width: 300.0,
                height: 220.0,
            })
        );
    }

    #[test]
    fn native_backend_should_render_generated_print_imposition_fixtures() {
        type PrintFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[PrintFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/print-booklet-spread.pdf") as &[u8],
                460,
                280,
                460,
                "print booklet spread",
                20_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/print-nup-imposed-sheet.pdf")
                    as &[u8],
                420,
                300,
                420,
                "print n-up imposed sheet",
                25_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(
                thumbnail.width, expected_width,
                "{label} fixture width should match the selected imposed sheet geometry"
            );
            assert_eq!(
                thumbnail.height, expected_height,
                "{label} fixture height should match the selected imposed sheet geometry"
            );
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve imposed page frames and print marks"
            );
        }
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
    fn native_parallel_partial_renderer_should_preserve_mixed_page_status() {
        let bytes = include_bytes!("../../../fixtures/generated/page-targeted-stream.pdf");
        let cancellation = RenderCancellation::new();
        let result = render_pages_parallel_partial(
            PdfSource::from_bytes(bytes),
            &[0, 1],
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 1,
                ..ParallelRenderOptions::default()
            },
            &cancellation,
        )
        .expect("partial scheduler should preserve page errors");

        assert!(!result.cancelled);
        assert_eq!(result.workers, 1);
        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.pages[0].page_index, 0);
        assert!(result.pages[0].result.is_ok());
        assert_eq!(result.pages[1].page_index, 1);
        assert_eq!(result.pages[1].result, Err(ThumbnailError::Malformed));
    }

    #[test]
    fn native_parallel_partial_renderer_should_stop_before_cancelled_work() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let cancellation = RenderCancellation::new();
        cancellation.cancel();

        let result = render_pages_parallel_partial(
            PdfSource::from_bytes(bytes),
            &[0, 1],
            &ThumbnailOptions {
                max_edge: 120,
                ..ThumbnailOptions::default()
            },
            ParallelRenderOptions {
                max_workers: 2,
                ..ParallelRenderOptions::default()
            },
            &cancellation,
        )
        .expect("pre-cancelled scheduler should return a partial result");

        assert!(result.cancelled);
        assert!(result.pages.is_empty());
        assert_eq!(result.workers, 2);
    }

    #[test]
    fn native_backend_preview_partial_should_preserve_page_results() {
        let bytes = include_bytes!("../../../fixtures/generated/page-targeted-stream.pdf");
        let cancellation = RenderCancellation::new();
        let result = NativeBackend::new()
            .render_preview_pages_partial(
                PdfSource::from_bytes(bytes),
                &[0, 1],
                &ThumbnailOptions {
                    max_edge: 120,
                    ..ThumbnailOptions::default()
                },
                ParallelRenderOptions {
                    max_workers: 1,
                    ..ParallelRenderOptions::default()
                },
                &cancellation,
            )
            .expect("preview partial scheduler should preserve page results");

        assert!(!result.cancelled);
        assert_eq!(result.workers, 1);
        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.pages[0].page_index, 0);
        assert!(result.pages[0].result.is_ok());
        assert_eq!(result.pages[1].page_index, 1);
        assert_eq!(result.pages[1].result, Err(ThumbnailError::Malformed));
    }

    #[test]
    fn native_backend_preview_partial_should_stop_before_cancelled_work() {
        let bytes = include_bytes!("../../../fixtures/generated/multi-page-report.pdf");
        let cancellation = RenderCancellation::new();
        cancellation.cancel();

        let result = NativeBackend::new()
            .render_preview_pages_partial(
                PdfSource::from_bytes(bytes),
                &[0, 1],
                &ThumbnailOptions {
                    max_edge: 120,
                    ..ThumbnailOptions::default()
                },
                ParallelRenderOptions {
                    max_workers: 2,
                    ..ParallelRenderOptions::default()
                },
                &cancellation,
            )
            .expect("pre-cancelled preview scheduler should return a partial result");

        assert!(result.cancelled);
        assert!(result.pages.is_empty());
        assert_eq!(result.workers, 2);
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
        assert_eq!(rgba_at(&thumbnail, 160, 96), [229, 51, 26, 255]);
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
    fn operator_coverage_should_classify_common_vector_fixture_operators() {
        let bytes = include_bytes!("../../../fixtures/generated/vector-paths.pdf");
        let report = scan_operator_coverage(bytes, OperatorCoverageOptions::default())
            .expect("operator coverage should scan generated vector fixture");

        assert!(report.streams_scanned >= 1);
        assert!(report.total_operators > 0);
        assert_operator_status(&report, "m", OperatorSupportStatus::Implemented);
        assert_operator_status(&report, "l", OperatorSupportStatus::Implemented);
        assert_operator_status(&report, "S", OperatorSupportStatus::Implemented);
    }

    #[test]
    fn operator_coverage_should_count_inline_images() {
        let bytes = include_bytes!("../../../fixtures/generated/inline-image.pdf");
        let report = scan_operator_coverage(bytes, OperatorCoverageOptions::default())
            .expect("operator coverage should scan inline image fixture");

        assert_eq!(report.inline_images, 1);
        assert_operator_status(&report, "BI", OperatorSupportStatus::Implemented);
    }

    #[test]
    fn operator_coverage_should_surface_unsupported_shorthand_curves() {
        let mut scanner = OperatorCoverageScanner::default();
        scanner
            .scan_stream(b"10 10 m 20 20 30 30 v")
            .expect("synthetic content should tokenize");
        let report = scanner.finish(0);

        let entry = operator_entry(&report, "v");
        assert_eq!(entry.status, OperatorSupportStatus::Unsupported);
        assert_eq!(entry.fallback_bucket, Some(BUCKET_GRAPHICS_STROKE_CLIP));
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
    fn native_backend_should_extract_visible_text_runs() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let text = TextExtractionBackend::extract_text(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &TextExtractionOptions::default(),
        )
        .expect("generated text fixture should extract text");

        assert_eq!(text.page_index, 0);
        assert!(!text.truncated);
        assert_eq!(text.runs.len(), 1);
        assert_eq!(text.runs[0].text, "pdfrust thumbnail fixture");
        assert!(text.runs[0].visible);
        assert_eq!(text.runs[0].glyphs.len(), 25);
    }

    #[test]
    fn native_backend_should_extract_invisible_ocr_text_runs() {
        let bytes = include_bytes!("../../../fixtures/generated/ocr-invisible-text-layer.pdf");
        let text = TextExtractionBackend::extract_text(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &TextExtractionOptions::default(),
        )
        .expect("generated OCR fixture should extract invisible text");

        assert_eq!(text.runs.len(), 2);
        assert_eq!(text.runs[0].text, "Invisible OCR text line one");
        assert_eq!(text.runs[1].text, "Invisible OCR text line two");
        assert!(text.runs.iter().all(|run| !run.visible));
        assert!(text
            .runs
            .iter()
            .flat_map(|run| &run.glyphs)
            .all(|glyph| !glyph.visible));
    }

    #[test]
    fn native_backend_should_bound_extracted_text_glyphs() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let text = TextExtractionBackend::extract_text(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &TextExtractionOptions {
                max_glyphs: 3,
                ..TextExtractionOptions::default()
            },
        )
        .expect("generated text fixture should extract bounded text");

        assert!(text.truncated);
        assert_eq!(text.runs.len(), 1);
        assert_eq!(text.runs[0].glyphs.len(), 3);
    }

    #[test]
    fn native_backend_should_inspect_generated_structure_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/metadata-outline-page-labels.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated structure metadata fixture should inspect");

        assert_eq!(metadata.info.title.as_deref(), Some("Metadata Fixture"));
        assert!(metadata.structure.has_xmp_metadata);
        assert!(metadata.structure.has_mark_info);
        assert!(metadata.structure.has_struct_tree_root);
        assert!(metadata.structure.has_named_destinations);
        assert_eq!(metadata.outlines.item_count, 2);
        assert_eq!(metadata.page_labels.labels[0].label, "A-1");
    }

    #[test]
    fn native_backend_should_report_optional_content_metadata() {
        let nested =
            include_bytes!("../../../fixtures/generated/optional-content-nested-layers.pdf");
        let nested_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(nested))
                .expect("nested optional-content fixture should inspect");

        assert!(nested_metadata.optional_content.has_oc_properties);
        assert_eq!(nested_metadata.optional_content.group_count, 2);
        assert!(nested_metadata.optional_content.has_default_configuration);
        assert_eq!(
            nested_metadata.optional_content.base_state,
            OptionalContentBaseState::On
        );
        assert_eq!(nested_metadata.optional_content.default_on_count, 0);
        assert_eq!(nested_metadata.optional_content.default_off_count, 1);
        assert!(!nested_metadata.optional_content.has_usage_application);
        assert!(
            !nested_metadata
                .optional_content
                .has_unsupported_membership_policy
        );
        assert!(!nested_metadata.optional_content.has_direct_group_dictionary);
        assert!(!nested_metadata.optional_content.has_unsupported_behavior);

        let usage =
            include_bytes!("../../../fixtures/generated/optional-content-usage-application.pdf");
        let usage_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(usage))
                .expect("usage application optional-content fixture should inspect");

        assert_eq!(usage_metadata.optional_content.group_count, 1);
        assert!(usage_metadata.optional_content.has_usage_application);
        assert!(
            !usage_metadata
                .optional_content
                .has_unsupported_membership_policy
        );
        assert!(usage_metadata.optional_content.has_unsupported_behavior);

        let ocmd = include_bytes!("../../../fixtures/generated/optional-content-ocmd.pdf");
        let ocmd_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(ocmd))
                .expect("OCMD optional-content fixture should inspect");

        assert_eq!(ocmd_metadata.optional_content.group_count, 1);
        assert!(!ocmd_metadata.optional_content.has_usage_application);
        assert!(
            ocmd_metadata
                .optional_content
                .has_unsupported_membership_policy
        );
        assert!(ocmd_metadata.optional_content.has_unsupported_behavior);
    }

    #[test]
    fn native_backend_should_report_generated_pdfa_archival_metadata() {
        let archive =
            include_bytes!("../../../fixtures/generated/pdfa-2b-archival-record.pdf") as &[u8];
        let packet =
            include_bytes!("../../../fixtures/generated/pdfa-3u-embedded-record.pdf") as &[u8];

        let archive_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(archive))
                .expect("PDF/A-2B archive fixture should inspect");
        assert_eq!(archive_metadata.archival.pdfa_part.as_deref(), Some("2"));
        assert_eq!(
            archive_metadata.archival.pdfa_conformance.as_deref(),
            Some("B")
        );
        assert!(archive_metadata.archival.has_output_intents);
        assert!(!archive_metadata.archival.conformance_validation_performed);

        let packet_metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(packet))
                .expect("PDF/A-3U embedded record fixture should inspect");
        assert_eq!(packet_metadata.archival.pdfa_part.as_deref(), Some("3"));
        assert_eq!(
            packet_metadata.archival.pdfa_conformance.as_deref(),
            Some("U")
        );
        assert!(packet_metadata.structure.has_embedded_files);
        assert!(!packet_metadata.archival.conformance_validation_performed);
    }

    #[test]
    fn native_backend_should_render_generated_pdfa_archival_fixtures() {
        type ArchivalFixture = (&'static [u8], u32, u32, u32, &'static str, usize);

        let fixtures: &[ArchivalFixture] = &[
            (
                include_bytes!("../../../fixtures/generated/pdfa-2b-archival-record.pdf")
                    as &[u8],
                260,
                180,
                260,
                "PDF/A-2B archive record",
                8_000,
            ),
            (
                include_bytes!("../../../fixtures/generated/pdfa-3u-embedded-record.pdf")
                    as &[u8],
                280,
                190,
                280,
                "PDF/A-3U embedded record",
                10_000,
            ),
        ];

        for &(bytes, expected_width, expected_height, max_edge, label, min_visible_pixels) in
            fixtures
        {
            let thumbnail = ThumbnailBackend::render(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
                &ThumbnailOptions {
                    max_edge,
                    ..ThumbnailOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("{label} fixture should render: {error}"));

            assert_eq!(thumbnail.width, expected_width);
            assert_eq!(thumbnail.height, expected_height);
            let visible_pixels = thumbnail
                .bytes
                .chunks_exact(4)
                .filter(|pixel| *pixel != [255, 255, 255, 255])
                .count();
            assert!(
                visible_pixels >= min_visible_pixels,
                "{label} fixture should preserve archival record layout"
            );
        }
    }

    #[test]
    fn native_backend_should_report_tagged_pdf_accessibility_signals() {
        let bytes = include_bytes!("../../../fixtures/generated/tagged-accessibility-metadata.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated tagged metadata fixture should inspect");

        assert_eq!(
            metadata.info.title.as_deref(),
            Some("Tagged Accessibility Fixture")
        );
        assert!(metadata.structure.has_mark_info);
        assert!(metadata.structure.has_struct_tree_root);
        assert_eq!(metadata.accessibility.language.as_deref(), Some("en-US"));
        assert_eq!(metadata.accessibility.mark_info_marked, Some(true));
        assert!(metadata.accessibility.has_role_map);
        assert_eq!(metadata.accessibility.structure_role_count, 1);
        assert!(metadata.accessibility.has_marked_content_references);
        assert_eq!(metadata.accessibility.marked_content_reference_count, 1);
        assert_eq!(metadata.accessibility.page_content_reference_count, 1);
        assert_eq!(metadata.accessibility.alt_text_count, 0);
        assert_eq!(metadata.accessibility.reading_order_warning_count, 0);
        assert!(!metadata.accessibility.truncated);
    }

    #[test]
    fn native_backend_should_report_tagged_visual_integrity_metadata() {
        let fixtures: &[(&[u8], &str, usize, usize)] = &[
            (
                include_bytes!("../../../fixtures/generated/tagged-report-visual-integrity.pdf")
                    as &[u8],
                "tagged report visual integrity",
                3,
                2,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-form-visual-integrity.pdf")
                    as &[u8],
                "tagged form visual integrity",
                2,
                1,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-office-alt-text.pdf") as &[u8],
                "tagged office alt text",
                3,
                2,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-invoice-reading-order.pdf")
                    as &[u8],
                "tagged invoice reading order",
                4,
                3,
            ),
            (
                include_bytes!("../../../fixtures/generated/tagged-structure-heavy-report.pdf")
                    as &[u8],
                "tagged structure heavy report",
                65,
                64,
            ),
        ];

        for &(bytes, label, minimum_role_count, minimum_marked_content_count) in fixtures {
            let metadata = DocumentMetadataBackend::inspect(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
            )
            .unwrap_or_else(|error| panic!("{label} fixture should inspect: {error}"));

            assert!(
                metadata.structure.has_mark_info,
                "{label} should report MarkInfo presence"
            );
            assert!(
                metadata.structure.has_struct_tree_root,
                "{label} should report StructTreeRoot presence"
            );
            assert_eq!(metadata.accessibility.language.as_deref(), Some("en-US"));
            assert_eq!(metadata.accessibility.mark_info_marked, Some(true));
            assert!(metadata.accessibility.has_role_map);
            assert!(
                metadata.accessibility.structure_role_count >= minimum_role_count,
                "{label} should report bounded structure roles"
            );
            assert!(metadata.accessibility.has_marked_content_references);
            assert!(
                metadata.accessibility.marked_content_reference_count
                    >= minimum_marked_content_count,
                "{label} should report marked content references"
            );
            assert_eq!(
                metadata.accessibility.marked_content_reference_count,
                metadata.accessibility.page_content_reference_count,
                "{label} should associate marked content with pages"
            );
            assert_eq!(
                metadata.accessibility.reading_order_warning_count, 0,
                "{label} should have no reading-order warnings"
            );
            assert!(!metadata.accessibility.truncated);
        }
    }

    #[test]
    fn native_backend_should_warn_when_tagged_reading_order_lacks_page_context() {
        let bytes = include_bytes!(
            "../../../fixtures/generated/tagged-reading-order-missing-page-context.pdf"
        );
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("tagged reading-order warning fixture should inspect");

        assert_eq!(metadata.accessibility.structure_role_count, 2);
        assert!(metadata.accessibility.has_marked_content_references);
        assert_eq!(metadata.accessibility.marked_content_reference_count, 1);
        assert_eq!(metadata.accessibility.page_content_reference_count, 0);
        assert_eq!(metadata.accessibility.reading_order_warning_count, 1);
        assert!(!metadata.accessibility.truncated);
    }

    #[test]
    fn native_backend_should_report_untagged_accessibility_defaults() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated untagged fixture should inspect");

        assert_eq!(metadata.accessibility.language, None);
        assert_eq!(metadata.accessibility.mark_info_marked, None);
        assert!(!metadata.accessibility.has_role_map);
        assert_eq!(metadata.accessibility.structure_role_count, 0);
        assert!(!metadata.accessibility.has_marked_content_references);
        assert_eq!(metadata.accessibility.marked_content_reference_count, 0);
        assert_eq!(metadata.accessibility.page_content_reference_count, 0);
        assert_eq!(metadata.accessibility.alt_text_count, 0);
        assert_eq!(metadata.accessibility.reading_order_warning_count, 0);
        assert!(!metadata.accessibility.truncated);
    }

    #[test]
    fn native_backend_should_report_malformed_tagged_structure_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/malformed-tagged-structure.pdf");
        let error =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect_err("malformed tagged structure should fail metadata inspection");

        assert_eq!(error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_report_signature_presence_without_validation() {
        let fixtures: &[(&[u8], &str)] = &[
            (
                include_bytes!("../../../fixtures/generated/digital-signature-appearance.pdf")
                    as &[u8],
                "digital signature appearance",
            ),
            (
                include_bytes!("../../../fixtures/generated/e-signature-contract-workflow.pdf")
                    as &[u8],
                "e-signature contract workflow",
            ),
            (
                include_bytes!("../../../fixtures/generated/e-signature-incremental-revision.pdf")
                    as &[u8],
                "e-signature incremental revision",
            ),
        ];

        for &(bytes, label) in fixtures {
            let metadata = DocumentMetadataBackend::inspect(
                &NativeBackend::new(),
                PdfSource::from_bytes(bytes),
            )
            .unwrap_or_else(|error| panic!("{label} fixture should inspect: {error}"));

            assert!(
                metadata.structure.has_signature_fields,
                "{label} should report signature-field presence"
            );
            assert!(
                metadata.structure.has_signature_byte_range,
                "{label} should report ByteRange presence without validation"
            );
        }
    }

    #[test]
    fn native_backend_should_report_embedded_file_and_portfolio_presence() {
        let embedded = include_bytes!("../../../fixtures/generated/embedded-source-file.pdf");
        let embedded_metadata = DocumentMetadataBackend::inspect(
            &NativeBackend::new(),
            PdfSource::from_bytes(embedded),
        )
        .expect("generated embedded source file fixture should inspect");
        assert!(embedded_metadata.structure.has_embedded_files);
        assert!(!embedded_metadata.structure.has_portfolio_collection);
        assert!(!embedded_metadata.structure.has_file_attachment_annotations);

        let portfolio = include_bytes!("../../../fixtures/generated/portfolio-embedded-files.pdf");
        let portfolio_metadata = DocumentMetadataBackend::inspect(
            &NativeBackend::new(),
            PdfSource::from_bytes(portfolio),
        )
        .expect("generated portfolio fixture should inspect");
        assert!(portfolio_metadata.structure.has_embedded_files);
        assert!(portfolio_metadata.structure.has_portfolio_collection);

        let attachment =
            include_bytes!("../../../fixtures/generated/file-attachment-annotation.pdf");
        let attachment_metadata = DocumentMetadataBackend::inspect(
            &NativeBackend::new(),
            PdfSource::from_bytes(attachment),
        )
        .expect("generated file attachment annotation fixture should inspect");
        assert!(
            attachment_metadata
                .structure
                .has_file_attachment_annotations
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
    fn native_backend_should_report_adversarial_truncated_header_as_malformed() {
        let bytes = include_bytes!("../../../fixtures/adversarial/truncated-header.pdf");
        let inspect_error =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect_err("truncated PDF should not inspect");
        let render_error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions::default(),
        )
        .expect_err("truncated PDF should not render");

        assert_eq!(inspect_error, ThumbnailError::Malformed);
        assert_eq!(render_error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_bound_adversarial_huge_image_dimensions() {
        let bytes = include_bytes!("../../../fixtures/adversarial/huge-image-dimensions.pdf");
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions {
                page_index: 0,
                max_edge: 32,
                background: pdfrust_thumbnail::Rgba::WHITE,
                output_format: pdfrust_thumbnail::OutputFormat::Rgba,
                timeout: std::time::Duration::from_millis(100),
                annotation_mode: AnnotationMode::Screen,
                form_appearance_mode: FormAppearanceMode::DocumentState,
            },
        )
        .expect_err("huge image dimensions should fail before allocation");

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

    fn assert_region_contains_low_intensity(
        thumbnail: &Thumbnail,
        x_range: std::ops::Range<u32>,
        y_range: std::ops::Range<u32>,
        maximum: u8,
    ) {
        for y in y_range {
            for x in x_range.clone() {
                let rgba = rgba_at(thumbnail, x, y);
                if rgba[3] == 255 && rgba[0] <= maximum && rgba[1] <= maximum && rgba[2] <= maximum
                {
                    return;
                }
            }
        }
        panic!("region should contain a low-intensity text pixel");
    }

    fn operator_entry<'a>(
        report: &'a OperatorCoverageReport,
        operator: &str,
    ) -> &'a OperatorCoverageEntry {
        report
            .operators
            .iter()
            .find(|entry| entry.operator == operator)
            .expect("operator should be present in coverage report")
    }

    fn assert_operator_status(
        report: &OperatorCoverageReport,
        operator: &str,
        status: OperatorSupportStatus,
    ) {
        assert_eq!(operator_entry(report, operator).status, status);
    }

    fn assert_unsupported_image_filter_fixture(bytes: &[u8]) {
        assert_unsupported_feature_fixture(bytes, buckets::IMAGE_FILTER);
    }

    fn assert_unsupported_feature_fixture(bytes: &[u8], bucket: &'static str) {
        let error = ThumbnailBackend::render(
            &NativeBackend::new(),
            PdfSource::from_bytes(bytes),
            &ThumbnailOptions::default(),
        )
        .expect_err("unsupported feature fixture should not render natively");

        assert_eq!(
            error.class(),
            pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
        );
        assert_eq!(error.unsupported_feature_bucket(), Some(bucket));
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
