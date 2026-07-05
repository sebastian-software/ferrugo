//! Node-API bindings for Ferrugo native thumbnail rendering.

#![deny(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)
)]

use std::time::Duration;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    AnnotationMode, DocumentMetadata, DocumentMetadataBackend, FormAppearanceMode, OutputFormat,
    PdfSource, PixelFormat, Rgba, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions,
};
use napi::bindgen_prelude::{AsyncTask, Buffer, Env, JsObjectValue, Task, ToNapiValue};
use napi::{Error, Result, Status};
use napi_derive::napi;

/// Options accepted by [`render`].
#[napi(object)]
pub struct RenderOptions {
    /// Zero-based page index. Defaults to `0`.
    pub page_index: Option<u32>,
    /// Maximum output width or height in pixels. Defaults to the Rust facade default.
    pub max_edge: Option<u32>,
    /// RGBA background used for transparent pages. Defaults to opaque white.
    pub background: Option<RgbaOptions>,
    /// Output encoding. Supported values are `rgba` and `png`.
    pub output_format: Option<String>,
    /// Per-render timeout in milliseconds.
    pub timeout_ms: Option<u32>,
    /// Annotation visibility mode. Supported values are `screen` and `print`.
    pub annotation_mode: Option<String>,
    /// AcroForm appearance handling. Supported values are `document-state` and
    /// `requested-mutation`.
    pub form_appearance_mode: Option<String>,
}

/// RGBA background channels.
#[napi(object)]
pub struct RgbaOptions {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

/// Thumbnail returned by [`render`].
#[napi(object)]
pub struct RenderedThumbnail {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Number of bytes between adjacent raw RGBA rows, or `0` for encoded PNG.
    pub stride: u32,
    /// Byte payload format, currently `rgba8` or `png`.
    pub pixel_format: String,
    /// Requested output format, currently `rgba` or `png`.
    pub output_format: String,
    /// Thumbnail bytes as a Node.js `Buffer`.
    pub data: Buffer,
}

/// Metadata returned by [`inspect`].
#[napi(object)]
pub struct InspectedDocument {
    /// Number of pages resolved by the native parser.
    pub page_count: u32,
    /// First page size when available.
    pub first_page: Option<PageSizeInfo>,
    /// Document information dictionary fields.
    pub info: DocumentInfoInfo,
    /// Non-rendering catalog and structure signals.
    pub structure: DocumentStructureInfo,
    /// Tagged-PDF and accessibility-related signals.
    pub accessibility: AccessibilityInfo,
    /// Optional content group and layer-state signals.
    pub optional_content: OptionalContentInfo,
}

/// PDF page size in user-space units.
#[napi(object)]
pub struct PageSizeInfo {
    /// Page width.
    pub width: f64,
    /// Page height.
    pub height: f64,
}

/// Document information dictionary fields.
#[napi(object)]
pub struct DocumentInfoInfo {
    /// `/Title`.
    pub title: Option<String>,
    /// `/Author`.
    pub author: Option<String>,
    /// `/Subject`.
    pub subject: Option<String>,
    /// `/Keywords`.
    pub keywords: Option<String>,
    /// `/Creator`.
    pub creator: Option<String>,
    /// `/Producer`.
    pub producer: Option<String>,
    /// `/CreationDate`.
    pub creation_date: Option<String>,
    /// `/ModDate`.
    pub modification_date: Option<String>,
}

/// Non-rendering catalog and structure signals.
#[napi(object)]
pub struct DocumentStructureInfo {
    /// Catalog contains XMP metadata.
    pub has_xmp_metadata: bool,
    /// Catalog contains `/MarkInfo`.
    pub has_mark_info: bool,
    /// Catalog contains `/StructTreeRoot`.
    pub has_struct_tree_root: bool,
    /// Catalog names expose embedded files.
    pub has_embedded_files: bool,
    /// Catalog declares a portfolio collection.
    pub has_portfolio_collection: bool,
    /// Document exposes at least one AcroForm signature field.
    pub has_signature_fields: bool,
}

/// Tagged-PDF and accessibility signals.
#[napi(object)]
pub struct AccessibilityInfo {
    /// Catalog language value when present.
    pub language: Option<String>,
    /// `/MarkInfo /Marked` value when present.
    pub mark_info_marked: Option<bool>,
    /// Structure tree root exposes a role map.
    pub has_role_map: bool,
    /// Number of structure roles reached before the traversal budget.
    pub structure_role_count: u32,
    /// Structure traversal stopped because the item budget was reached.
    pub truncated: bool,
}

/// Optional content group and layer-state signals.
#[napi(object)]
pub struct OptionalContentInfo {
    /// Catalog contains `/OCProperties`.
    pub has_oc_properties: bool,
    /// Number of optional content groups listed in `/OCProperties /OCGs`.
    pub group_count: u32,
    /// Default optional-content base state.
    pub base_state: String,
    /// Metadata found behavior native rendering does not implement interactively.
    pub has_unsupported_behavior: bool,
}

#[derive(Debug, Clone)]
struct NodeThumbnailOptions {
    options: ThumbnailOptions,
}

impl NodeThumbnailOptions {
    fn from_options(options: Option<RenderOptions>) -> Result<Self> {
        let mut thumbnail_options = ThumbnailOptions::default();
        if let Some(options) = options {
            if let Some(page_index) = options.page_index {
                thumbnail_options.page_index = page_index;
            }
            if let Some(max_edge) = options.max_edge {
                if max_edge == 0 {
                    return Err(invalid_arg("maxEdge must be greater than zero"));
                }
                thumbnail_options.max_edge = max_edge;
            }
            if let Some(background) = options.background {
                thumbnail_options.background = Rgba {
                    r: background.r,
                    g: background.g,
                    b: background.b,
                    a: background.a,
                };
            }
            if let Some(output_format) = options.output_format {
                thumbnail_options.output_format = parse_output_format(&output_format)?;
            }
            if let Some(timeout_ms) = options.timeout_ms {
                thumbnail_options.timeout = Duration::from_millis(u64::from(timeout_ms));
            }
            if let Some(annotation_mode) = options.annotation_mode {
                thumbnail_options.annotation_mode = parse_annotation_mode(&annotation_mode)?;
            }
            if let Some(form_appearance_mode) = options.form_appearance_mode {
                thumbnail_options.form_appearance_mode =
                    parse_form_appearance_mode(&form_appearance_mode)?;
            }
        }
        Ok(Self {
            options: thumbnail_options,
        })
    }
}

/// Worker task backing [`render`].
pub struct RenderTask {
    input: Vec<u8>,
    options: NodeThumbnailOptions,
}

impl Task for RenderTask {
    type Output = Thumbnail;
    type JsValue = RenderedThumbnail;

    fn compute(&mut self) -> Result<Self::Output> {
        NativeBackend::new()
            .render(PdfSource::from_bytes(&self.input), &self.options.options)
            .map_err(map_thumbnail_error)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(rendered_thumbnail(output))
    }

    fn reject(&mut self, env: Env, err: Error) -> Result<Self::JsValue> {
        Err(js_error_with_taxonomy(env, err))
    }
}

/// Worker task backing [`inspect`].
pub struct InspectTask {
    input: Vec<u8>,
}

impl Task for InspectTask {
    type Output = DocumentMetadata;
    type JsValue = InspectedDocument;

    fn compute(&mut self) -> Result<Self::Output> {
        NativeBackend::new()
            .inspect(PdfSource::from_bytes(&self.input))
            .map_err(map_thumbnail_error)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        inspected_document(output)
    }

    fn reject(&mut self, env: Env, err: Error) -> Result<Self::JsValue> {
        Err(js_error_with_taxonomy(env, err))
    }
}

/// Renders one PDF page on the Node-API worker pool.
#[napi(ts_return_type = "Promise<RenderedThumbnail>")]
pub fn render(
    env: Env,
    input: Buffer,
    options: Option<RenderOptions>,
) -> Result<AsyncTask<RenderTask>> {
    let options = match NodeThumbnailOptions::from_options(options) {
        Ok(options) => options,
        Err(err) => return Err(js_invalid_arg_error(env, err)),
    };
    Ok(AsyncTask::new(RenderTask {
        input: input.as_ref().to_vec(),
        options,
    }))
}

/// Inspects document metadata on the Node-API worker pool.
#[napi(ts_return_type = "Promise<InspectedDocument>")]
pub fn inspect(input: Buffer) -> AsyncTask<InspectTask> {
    AsyncTask::new(InspectTask {
        input: input.as_ref().to_vec(),
    })
}

fn rendered_thumbnail(thumbnail: Thumbnail) -> RenderedThumbnail {
    RenderedThumbnail {
        width: thumbnail.width,
        height: thumbnail.height,
        stride: thumbnail.stride as u32,
        pixel_format: pixel_format_name(thumbnail.pixel_format).to_string(),
        output_format: output_format_name(thumbnail.output_format).to_string(),
        data: thumbnail.bytes.into(),
    }
}

fn inspected_document(metadata: DocumentMetadata) -> Result<InspectedDocument> {
    Ok(InspectedDocument {
        page_count: u32::try_from(metadata.page_count())
            .map_err(|_| invalid_arg("document page count exceeds u32"))?,
        first_page: metadata.first_page_size().map(|size| PageSizeInfo {
            width: size.width,
            height: size.height,
        }),
        info: DocumentInfoInfo {
            title: metadata.info.title,
            author: metadata.info.author,
            subject: metadata.info.subject,
            keywords: metadata.info.keywords,
            creator: metadata.info.creator,
            producer: metadata.info.producer,
            creation_date: metadata.info.creation_date,
            modification_date: metadata.info.modification_date,
        },
        structure: DocumentStructureInfo {
            has_xmp_metadata: metadata.structure.has_xmp_metadata,
            has_mark_info: metadata.structure.has_mark_info,
            has_struct_tree_root: metadata.structure.has_struct_tree_root,
            has_embedded_files: metadata.structure.has_embedded_files,
            has_portfolio_collection: metadata.structure.has_portfolio_collection,
            has_signature_fields: metadata.structure.has_signature_fields,
        },
        accessibility: AccessibilityInfo {
            language: metadata.accessibility.language,
            mark_info_marked: metadata.accessibility.mark_info_marked,
            has_role_map: metadata.accessibility.has_role_map,
            structure_role_count: u32::try_from(metadata.accessibility.structure_role_count)
                .map_err(|_| invalid_arg("structure role count exceeds u32"))?,
            truncated: metadata.accessibility.truncated,
        },
        optional_content: OptionalContentInfo {
            has_oc_properties: metadata.optional_content.has_oc_properties,
            group_count: u32::try_from(metadata.optional_content.group_count)
                .map_err(|_| invalid_arg("optional content group count exceeds u32"))?,
            base_state: metadata.optional_content.base_state.name().to_string(),
            has_unsupported_behavior: metadata.optional_content.has_unsupported_behavior,
        },
    })
}

fn parse_output_format(value: &str) -> Result<OutputFormat> {
    match value {
        "rgba" | "rgba8" => Ok(OutputFormat::Rgba),
        "png" => Ok(OutputFormat::Png),
        _ => Err(invalid_arg("outputFormat must be `rgba` or `png`")),
    }
}

fn parse_annotation_mode(value: &str) -> Result<AnnotationMode> {
    match value {
        "screen" => Ok(AnnotationMode::Screen),
        "print" => Ok(AnnotationMode::Print),
        _ => Err(invalid_arg("annotationMode must be `screen` or `print`")),
    }
}

fn parse_form_appearance_mode(value: &str) -> Result<FormAppearanceMode> {
    match value {
        "document-state" => Ok(FormAppearanceMode::DocumentState),
        "requested-mutation" => Ok(FormAppearanceMode::RequestedMutation),
        _ => Err(invalid_arg(
            "formAppearanceMode must be `document-state` or `requested-mutation`",
        )),
    }
}

fn output_format_name(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Rgba => "rgba",
        OutputFormat::Png => "png",
    }
}

fn pixel_format_name(format: PixelFormat) -> &'static str {
    match format {
        PixelFormat::Rgba8 => "rgba8",
        PixelFormat::Png => "png",
    }
}

fn map_thumbnail_error(error: ThumbnailError) -> Error {
    let class = error.class().as_str();
    let bucket = match &error {
        ThumbnailError::UnsupportedFeature(bucket) => Some(*bucket),
        _ => None,
    };
    let reason = match bucket {
        Some(bucket) => format!("{}|{}|{}", class, bucket, error),
        None => format!("{}||{}", class, error),
    };
    Error::new(Status::GenericFailure, reason)
}

fn js_error_with_taxonomy(env: Env, err: Error) -> Error {
    let (code, bucket, message) = parse_mapped_error_reason(&err.reason);
    let mut error = match env.create_error(Error::new(Status::GenericFailure, message)) {
        Ok(error) => error,
        Err(_) => return err,
    };
    if error.set_named_property("code", code).is_err() {
        return err;
    }
    if let Some(bucket) = bucket {
        if error.set_named_property("bucket", bucket).is_err() {
            return err;
        }
    }
    match error.into_unknown(&env) {
        Ok(error) => Error::from(error),
        Err(_) => err,
    }
}

fn parse_mapped_error_reason(reason: &str) -> (&str, Option<&str>, String) {
    let mut parts = reason.splitn(3, '|');
    let code = parts.next().unwrap_or("internal");
    let bucket = parts.next().filter(|value| !value.is_empty());
    let message = parts.next().unwrap_or(reason).to_string();
    (code, bucket, message)
}

fn invalid_arg(reason: &str) -> Error {
    Error::new(Status::InvalidArg, reason.to_string())
}

/// Gives synchronous option-validation failures the same `code` property
/// shape that asynchronous render and inspect errors carry.
fn js_invalid_arg_error(env: Env, err: Error) -> Error {
    let reason = format!("invalid-argument||{}", err.reason);
    js_error_with_taxonomy(env, Error::new(Status::InvalidArg, reason))
}

trait OptionalContentBaseStateName {
    fn name(self) -> &'static str;
}

impl OptionalContentBaseStateName for ferrugo_thumbnail::OptionalContentBaseState {
    fn name(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::On => "on",
            Self::Off => "off",
            Self::Unchanged => "unchanged",
        }
    }
}
