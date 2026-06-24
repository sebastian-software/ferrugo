#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use pdfrust_native::{NativeBackend, NativeMemoryDiagnostics};
use pdfrust_pdfium::PdfiumBackend;
use pdfrust_thumbnail::{
    DocumentMetadata, DocumentMetadataBackend, PageSize, PdfSource, Rgba, ThumbnailBackend,
    ThumbnailError, ThumbnailOptions, DEFAULT_MAX_EDGE, DEFAULT_PAGE_INDEX, DEFAULT_TIMEOUT,
};

const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<OsString>) -> Result<(), CliError> {
    let command = args.first().and_then(|arg| arg.to_str());
    match command {
        Some("render") | Some("render-auto") => render_auto_command(&args[1..]),
        Some("render-pdfium") | Some("render-worker") => render_direct_command(&args[1..]),
        Some("render-native") => render_native_command(&args[1..]),
        Some("render-isolated") => render_isolated_command(&args[1..]),
        Some("compare-metadata") => compare_metadata_command(&args[1..]),
        Some("--version" | "-V") => {
            println!("pdfrust-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help" | "-h") | None => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(CliError::Usage(format!("unknown command `{other}`"))),
    }
}

fn render_direct_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    render_direct(config)
}

fn render_direct(config: RenderConfig) -> Result<(), CliError> {
    let backend = PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
    let options = thumbnail_options(&config);
    let source = PdfSource::from_path(&config.input);
    let thumbnail = backend
        .render(source, &options)
        .map_err(|err| CliError::Render {
            class: err.class().as_str(),
            message: err.to_string(),
        })?;
    let png = encode_rgba_png(&thumbnail)?;
    fs::write(&config.output, png).map_err(|source| CliError::Io {
        path: config.output,
        source,
    })?;
    Ok(())
}

fn render_auto_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    render_auto(config)
}

fn render_auto(config: RenderConfig) -> Result<(), CliError> {
    let outcome = render_auto_thumbnail(&config)?;
    eprintln!("render backend: {}", outcome.backend.as_str());
    let png = encode_rgba_png(&outcome.thumbnail)?;
    fs::write(&config.output, png).map_err(|source| CliError::Io {
        path: config.output,
        source,
    })?;
    Ok(())
}

fn render_auto_thumbnail(config: &RenderConfig) -> Result<AutoRenderOutcome, CliError> {
    let options = thumbnail_options(config);
    let source = PdfSource::from_path(&config.input);
    let native = NativeBackend::new();
    match native.render(source, &options) {
        Ok(thumbnail) => Ok(AutoRenderOutcome {
            thumbnail,
            backend: AutoRenderBackend::Native,
        }),
        Err(ThumbnailError::Unsupported) => {
            let pdfium =
                PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
            let thumbnail = pdfium
                .render(PdfSource::from_path(&config.input), &options)
                .map_err(|err| CliError::Render {
                    class: err.class().as_str(),
                    message: err.to_string(),
                })?;
            Ok(AutoRenderOutcome {
                thumbnail,
                backend: AutoRenderBackend::PdfiumFallback,
            })
        }
        Err(err) => Err(CliError::Render {
            class: err.class().as_str(),
            message: err.to_string(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutoRenderBackend {
    Native,
    PdfiumFallback,
}

impl AutoRenderBackend {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::PdfiumFallback => "pdfium fallback_reason=unsupported",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AutoRenderOutcome {
    thumbnail: pdfrust_thumbnail::Thumbnail,
    backend: AutoRenderBackend,
}

fn render_native_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    let backend = NativeBackend::new();
    let options = thumbnail_options(&config);
    let source = PdfSource::from_path(&config.input);
    let thumbnail = backend
        .render(source, &options)
        .map_err(|err| CliError::Render {
            class: err.class().as_str(),
            message: err.to_string(),
        })?;
    let png = encode_rgba_png(&thumbnail)?;
    fs::write(&config.output, png).map_err(|source| CliError::Io {
        path: config.output,
        source,
    })?;
    Ok(())
}

fn thumbnail_options(config: &RenderConfig) -> ThumbnailOptions {
    ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
    }
}

fn render_isolated_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    render_isolated(config)
}

fn compare_metadata_command(args: &[OsString]) -> Result<(), CliError> {
    let config = CompareMetadataConfig::parse(args)?;
    let pdfium = PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
    let native = NativeBackend::new();
    let pdfium_result = pdfium.inspect(PdfSource::from_path(&config.input));
    let native_result = native.inspect(PdfSource::from_path(&config.input));
    let comparison = compare_metadata_results(
        MetadataOutcome::from_result(pdfium_result),
        MetadataOutcome::from_result(native_result),
    );
    let json = comparison_json(&config.input, &comparison);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if comparison.matches {
        Ok(())
    } else {
        Err(CliError::Compare(comparison.mismatches.join("; ")))
    }
}

fn render_isolated(config: RenderConfig) -> Result<(), CliError> {
    let executable = env::current_exe().map_err(|source| {
        CliError::Process(format!("failed to locate current executable: {source}"))
    })?;
    let temp_output = temporary_output_path(&config.output);
    let _ = fs::remove_file(&temp_output);

    let mut child = Command::new(executable)
        .arg("render-worker")
        .args(worker_args(&config, &temp_output))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| CliError::Process(format!("failed to spawn render worker: {source}")))?;

    match wait_for_worker(&mut child, config.timeout) {
        Ok(()) => {
            fs::rename(&temp_output, &config.output).map_err(|source| CliError::Io {
                path: config.output,
                source,
            })?;
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_output);
            Err(error)
        }
    }
}

fn worker_args(config: &RenderConfig, output: &Path) -> Vec<OsString> {
    vec![
        config.input.as_os_str().to_owned(),
        OsString::from("--output"),
        output.as_os_str().to_owned(),
        OsString::from("--page-index"),
        OsString::from(config.page_index.to_string()),
        OsString::from("--max-edge"),
        OsString::from(config.max_edge.to_string()),
        OsString::from("--background"),
        OsString::from(format_background(config.background)),
        OsString::from("--timeout"),
        OsString::from(config.timeout.as_secs().to_string()),
    ]
}

fn wait_for_worker(child: &mut Child, timeout: Duration) -> Result<(), CliError> {
    if timeout.is_zero() {
        terminate_worker(child);
        return Err(timeout_error());
    }

    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| CliError::Process("timeout deadline overflow".to_string()))?;

    loop {
        if let Some(status) = child.try_wait().map_err(|source| {
            CliError::Process(format!("failed to poll render worker: {source}"))
        })? {
            let stderr = read_worker_stderr(child);
            return if status.success() {
                Ok(())
            } else {
                Err(worker_failure(stderr, status.to_string()))
            };
        }

        let now = Instant::now();
        if now >= deadline {
            terminate_worker(child);
            return Err(timeout_error());
        }

        thread::sleep((deadline - now).min(WORKER_POLL_INTERVAL));
    }
}

fn terminate_worker(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = read_worker_stderr(child);
}

fn read_worker_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr.trim().to_string()
}

fn worker_failure(stderr: String, fallback: String) -> CliError {
    parse_worker_render_error(&stderr).unwrap_or_else(|| {
        let message = if stderr.is_empty() { fallback } else { stderr };
        CliError::Render {
            class: "internal",
            message,
        }
    })
}

fn parse_worker_render_error(stderr: &str) -> Option<CliError> {
    let rest = stderr.strip_prefix("render error [")?;
    let (class, message) = rest.split_once("]: ")?;
    Some(CliError::Render {
        class: stable_error_class(class),
        message: message.to_string(),
    })
}

fn stable_error_class(class: &str) -> &'static str {
    match class {
        "encrypted" => "encrypted",
        "malformed" => "malformed",
        "unsupported" => "unsupported",
        "timeout" => "timeout",
        _ => "internal",
    }
}

fn timeout_error() -> CliError {
    CliError::Render {
        class: ThumbnailError::Timeout.class().as_str(),
        message: ThumbnailError::Timeout.to_string(),
    }
}

fn temporary_output_path(output: &Path) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("thumbnail.png");
    parent.join(format!(".{file_name}.{}.tmp", std::process::id()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderConfig {
    input: PathBuf,
    output: PathBuf,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
}

impl RenderConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;

        let mut index = 0;
        while index < args.len() {
            let arg = args[index]
                .to_str()
                .ok_or_else(|| CliError::Usage("arguments must be valid UTF-8".to_string()))?;
            match arg {
                "--output" | "-o" => {
                    index += 1;
                    output = Some(required_path(args, index, "--output")?);
                }
                "--page-index" => {
                    index += 1;
                    page_index = parse_u32(args, index, "--page-index")?;
                }
                "--max-edge" => {
                    index += 1;
                    max_edge = parse_u32(args, index, "--max-edge")?;
                }
                "--background" => {
                    index += 1;
                    background = parse_background(required_str(args, index, "--background")?)?;
                }
                "--timeout" => {
                    index += 1;
                    let seconds = parse_u64(args, index, "--timeout")?;
                    timeout = Duration::from_secs(seconds);
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input PDF is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        if max_edge == 0 {
            return Err(CliError::Usage(
                "--max-edge must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input PDF".to_string()))?,
            output: output.ok_or_else(|| CliError::Usage("missing --output path".to_string()))?,
            page_index,
            max_edge,
            background,
            timeout,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompareMetadataConfig {
    input: PathBuf,
    output: Option<PathBuf>,
}

impl CompareMetadataConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;

        let mut index = 0;
        while index < args.len() {
            let arg = args[index]
                .to_str()
                .ok_or_else(|| CliError::Usage("arguments must be valid UTF-8".to_string()))?;
            match arg {
                "--output" | "-o" => {
                    index += 1;
                    output = Some(required_path(args, index, "--output")?);
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input PDF is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input PDF".to_string()))?,
            output,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MetadataOutcome {
    Success(DocumentMetadata),
    Error {
        class: &'static str,
        message: String,
    },
}

impl MetadataOutcome {
    fn from_result(result: Result<DocumentMetadata, ThumbnailError>) -> Self {
        match result {
            Ok(metadata) => Self::Success(metadata),
            Err(error) => Self::Error {
                class: error.class().as_str(),
                message: error.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MetadataComparison {
    matches: bool,
    pdfium: MetadataOutcome,
    native: MetadataOutcome,
    mismatches: Vec<String>,
}

fn compare_metadata_results(
    pdfium: MetadataOutcome,
    native: MetadataOutcome,
) -> MetadataComparison {
    let mut mismatches = Vec::new();
    match (&pdfium, &native) {
        (MetadataOutcome::Success(expected), MetadataOutcome::Success(actual)) => {
            if expected.page_count() != actual.page_count() {
                mismatches.push(format!(
                    "page_count expected {} from pdfium but rust-native returned {}",
                    expected.page_count(),
                    actual.page_count()
                ));
            }
            let shared_pages = expected.pages.len().min(actual.pages.len());
            for index in 0..shared_pages {
                let expected_size = expected.pages[index].size;
                let actual_size = actual.pages[index].size;
                if !page_sizes_match(expected_size, actual_size) {
                    mismatches.push(format!(
                        "page {index} size expected {:.3}x{:.3} from pdfium but rust-native returned {:.3}x{:.3}",
                        expected_size.width,
                        expected_size.height,
                        actual_size.width,
                        actual_size.height
                    ));
                }
            }
        }
        (
            MetadataOutcome::Error {
                class: expected, ..
            },
            MetadataOutcome::Error { class: actual, .. },
        ) => {
            if expected != actual {
                mismatches.push(format!(
                    "error_class expected {expected} from pdfium but rust-native returned {actual}"
                ));
            }
        }
        (MetadataOutcome::Success(_), MetadataOutcome::Error { class, message }) => {
            mismatches.push(format!(
                "pdfium inspected metadata but rust-native returned {class}: {message}"
            ));
        }
        (MetadataOutcome::Error { class, message }, MetadataOutcome::Success(_)) => {
            mismatches.push(format!(
                "pdfium returned {class}: {message} but rust-native inspected metadata"
            ));
        }
    }

    MetadataComparison {
        matches: mismatches.is_empty(),
        pdfium,
        native,
        mismatches,
    }
}

fn page_sizes_match(expected: PageSize, actual: PageSize) -> bool {
    const EPSILON: f64 = 0.01;
    (expected.width - actual.width).abs() <= EPSILON
        && (expected.height - actual.height).abs() <= EPSILON
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Backend(String),
    Process(String),
    Render {
        class: &'static str,
        message: String,
    },
    Compare(String),
    Encode(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "usage error: {message}"),
            Self::Backend(message) => write!(f, "backend error: {message}"),
            Self::Process(message) => write!(f, "process error: {message}"),
            Self::Render { class, message } => write!(f, "render error [{class}]: {message}"),
            Self::Compare(message) => write!(f, "metadata comparison mismatch: {message}"),
            Self::Encode(message) => write!(f, "PNG encode error: {message}"),
            Self::Io { path, source } => {
                write!(f, "failed to write `{}`: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CliError {}

fn required_path(args: &[OsString], index: usize, option: &str) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(required_str(args, index, option)?))
}

fn required_str<'a>(args: &'a [OsString], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn parse_u32(args: &[OsString], index: usize, option: &str) -> Result<u32, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an unsigned integer")))
}

fn parse_u64(args: &[OsString], index: usize, option: &str) -> Result<u64, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an unsigned integer")))
}

fn parse_background(value: &str) -> Result<Rgba, CliError> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let parse_channel = |range: std::ops::Range<usize>| {
        let channel = hex.get(range).ok_or_else(|| {
            CliError::Usage("--background must be #RRGGBB or #RRGGBBAA".to_string())
        })?;
        u8::from_str_radix(channel, 16)
            .map_err(|_| CliError::Usage("--background must be #RRGGBB or #RRGGBBAA".to_string()))
    };
    match hex.len() {
        6 => Ok(Rgba {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: 255,
        }),
        8 => Ok(Rgba {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: parse_channel(6..8)?,
        }),
        _ => Err(CliError::Usage(
            "--background must be #RRGGBB or #RRGGBBAA".to_string(),
        )),
    }
}

fn format_background(color: Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        color.r, color.g, color.b, color.a
    )
}

fn comparison_json(input: &Path, comparison: &MetadataComparison) -> String {
    let status = if comparison.matches {
        "match"
    } else {
        "mismatch"
    };
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"fixture\": {{\n",
            "    \"path\": {}\n",
            "  }},\n",
            "  \"comparison\": {{\n",
            "    \"oracle\": \"pdfium\",\n",
            "    \"candidate\": \"rust-native\",\n",
            "    \"status\": {},\n",
            "    \"mismatches\": {}\n",
            "  }},\n",
            "  \"pdfium\": {},\n",
            "  \"rust_native\": {},\n",
            "  \"rust_native_memory\": {}\n",
            "}}\n"
        ),
        json_string(&input.to_string_lossy()),
        json_string(status),
        json_string_array(&comparison.mismatches),
        metadata_outcome_json(&comparison.pdfium),
        metadata_outcome_json(&comparison.native),
        native_memory_diagnostics_json(&NativeBackend::new().memory_diagnostics())
    )
}

fn native_memory_diagnostics_json(diagnostics: &NativeMemoryDiagnostics) -> String {
    format!(
        concat!(
            "{{",
            "\"max_page_pixels\":{},",
            "\"max_image_bytes\":{},",
            "\"max_font_program_bytes\":{},",
            "\"max_cmap_bytes\":{},",
            "\"max_text_run_bytes\":{},",
            "\"max_display_items\":{}",
            "}}"
        ),
        diagnostics.max_page_pixels,
        diagnostics.max_image_bytes,
        diagnostics.max_font_program_bytes,
        diagnostics.max_cmap_bytes,
        diagnostics.max_text_run_bytes,
        diagnostics.max_display_items
    )
}

fn metadata_outcome_json(outcome: &MetadataOutcome) -> String {
    match outcome {
        MetadataOutcome::Success(metadata) => {
            let pages = metadata
                .pages
                .iter()
                .map(|page| {
                    format!(
                        "{{\"index\":{},\"width\":{:.3},\"height\":{:.3}}}",
                        page.index, page.size.width, page.size.height
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"status\":\"success\",\"page_count\":{},\"pages\":[{}]}}",
                metadata.page_count(),
                pages
            )
        }
        MetadataOutcome::Error { class, message } => format!(
            "{{\"status\":\"error\",\"error_class\":{},\"message\":{}}}",
            json_string(class),
            json_string(message)
        ),
    }
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

fn encode_rgba_png(thumbnail: &pdfrust_thumbnail::Thumbnail) -> Result<Vec<u8>, CliError> {
    let width = thumbnail.width;
    let height = thumbnail.height;
    let row_len = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| CliError::Encode("row length overflow".to_string()))?;
    let filtered_len = row_len
        .checked_add(1)
        .and_then(|row| row.checked_mul(height as usize))
        .ok_or_else(|| CliError::Encode("image size overflow".to_string()))?;
    if thumbnail.bytes.len() != row_len * height as usize {
        return Err(CliError::Encode(
            "thumbnail buffer length does not match dimensions".to_string(),
        ));
    }

    let mut filtered = Vec::with_capacity(filtered_len);
    for row in thumbnail.bytes.chunks_exact(row_len) {
        filtered.push(0);
        filtered.extend_from_slice(row);
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_png_chunk(&mut png, b"IHDR", &ihdr)?;
    write_png_chunk(&mut png, b"IDAT", &zlib_store(&filtered)?)?;
    write_png_chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

fn zlib_store(data: &[u8]) -> Result<Vec<u8>, CliError> {
    let mut out = Vec::with_capacity(data.len() + 6 + (data.len() / 65_535) * 5);
    out.extend_from_slice(&[0x78, 0x01]);
    let block_count = data.chunks(65_535).count();
    for (block_index, block) in data.chunks(65_535).enumerate() {
        let final_block = block_index + 1 == block_count;
        out.push(final_block as u8);
        let len = u16::try_from(block.len())
            .map_err(|_| CliError::Encode("deflate block too large".to_string()))?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    Ok(out)
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) -> Result<(), CliError> {
    let length = u32::try_from(data.len())
        .map_err(|_| CliError::Encode("PNG chunk too large".to_string()))?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32(chunk_type.iter().chain(data.iter()).copied());
    out.extend_from_slice(&crc.to_be_bytes());
    Ok(())
}

fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = 1_u32;
    let mut b = 0_u32;
    for byte in data {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn crc32(bytes: impl IntoIterator<Item = u8>) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn print_usage() {
    println!(
        "Usage: pdfrust-cli <render|render-auto|render-native|render-pdfium|render-isolated|compare-metadata> <input.pdf> \
         [--output PATH] [--page-index N] [--max-edge N] [--background #RRGGBB] \
         [--timeout SECONDS]"
    );
}

#[cfg(test)]
mod tests {
    use pdfrust_thumbnail::{PageMetadata, PixelFormat, Thumbnail};

    use super::*;

    #[test]
    fn render_config_should_apply_phase_0_defaults() {
        let config = RenderConfig::parse(&[
            OsString::from("fixtures/generated/text-page.pdf"),
            OsString::from("--output"),
            OsString::from("target/text-page.png"),
        ])
        .expect("valid config");

        assert_eq!(config.page_index, 0);
        assert_eq!(config.max_edge, 1024);
        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn render_native_command_should_write_generated_vector_png() {
        let output =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/native-vector-test.png");
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/vector-paths.pdf");
        fs::create_dir_all(output.parent().expect("output parent"))
            .expect("test target directory should be created");
        let _ = fs::remove_file(&output);

        run(vec![
            OsString::from("render-native"),
            input.as_os_str().to_os_string(),
            OsString::from("--max-edge"),
            OsString::from("220"),
            OsString::from("--output"),
            output.as_os_str().to_os_string(),
        ])
        .expect("native vector render should succeed");

        let png = fs::read(&output).expect("native PNG should be written");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn render_auto_command_should_use_native_for_supported_fixture() {
        env::remove_var(pdfrust_pdfium::PDFIUM_LIBRARY_ENV);
        let output =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/auto-native-vector-test.png");
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/vector-paths.pdf");
        fs::create_dir_all(output.parent().expect("output parent"))
            .expect("test target directory should be created");
        let _ = fs::remove_file(&output);

        run(vec![
            OsString::from("render-auto"),
            input.as_os_str().to_os_string(),
            OsString::from("--max-edge"),
            OsString::from("220"),
            OsString::from("--output"),
            output.as_os_str().to_os_string(),
        ])
        .expect("auto render should use native without requiring PDFium");

        let png = fs::read(&output).expect("auto PNG should be written");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn render_command_should_default_to_auto_mode() {
        env::remove_var(pdfrust_pdfium::PDFIUM_LIBRARY_ENV);
        let output =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/default-auto-vector-test.png");
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/vector-paths.pdf");
        fs::create_dir_all(output.parent().expect("output parent"))
            .expect("test target directory should be created");
        let _ = fs::remove_file(&output);

        run(vec![
            OsString::from("render"),
            input.as_os_str().to_os_string(),
            OsString::from("--max-edge"),
            OsString::from("220"),
            OsString::from("--output"),
            output.as_os_str().to_os_string(),
        ])
        .expect("default render should use auto mode");

        let png = fs::read(&output).expect("default auto PNG should be written");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn render_auto_thumbnail_should_report_native_backend_choice() {
        env::remove_var(pdfrust_pdfium::PDFIUM_LIBRARY_ENV);
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/vector-paths.pdf");
        let output = PathBuf::from("target/unused-auto-choice.png");
        let config = RenderConfig {
            input,
            output,
            page_index: 0,
            max_edge: 220,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
        };

        let outcome = render_auto_thumbnail(&config).expect("supported fixture should render");

        assert_eq!(outcome.backend, AutoRenderBackend::Native);
    }

    #[test]
    fn compare_metadata_config_should_accept_optional_output() {
        let config = CompareMetadataConfig::parse(&[
            OsString::from("fixtures/generated/text-page.pdf"),
            OsString::from("--output"),
            OsString::from("target/metadata.json"),
        ])
        .expect("valid config");

        assert_eq!(
            config.input,
            PathBuf::from("fixtures/generated/text-page.pdf")
        );
        assert_eq!(config.output, Some(PathBuf::from("target/metadata.json")));
    }

    #[test]
    fn metadata_comparison_should_match_equal_page_metadata() {
        let metadata = DocumentMetadata::new(vec![PageMetadata {
            index: 0,
            size: PageSize {
                width: 300.0,
                height: 160.0,
            },
        }]);

        let comparison = compare_metadata_results(
            MetadataOutcome::Success(metadata.clone()),
            MetadataOutcome::Success(metadata),
        );

        assert!(comparison.matches);
        assert!(comparison.mismatches.is_empty());
    }

    #[test]
    fn metadata_comparison_should_report_page_size_mismatch() {
        let pdfium = DocumentMetadata::new(vec![PageMetadata {
            index: 0,
            size: PageSize {
                width: 300.0,
                height: 160.0,
            },
        }]);
        let native = DocumentMetadata::new(vec![PageMetadata {
            index: 0,
            size: PageSize {
                width: 301.0,
                height: 160.0,
            },
        }]);

        let comparison = compare_metadata_results(
            MetadataOutcome::Success(pdfium),
            MetadataOutcome::Success(native),
        );

        assert!(!comparison.matches);
        assert_eq!(comparison.mismatches.len(), 1);
        assert!(comparison.mismatches[0].contains("page 0 size expected"));
    }

    #[test]
    fn metadata_comparison_should_match_equal_error_classes() {
        let comparison = compare_metadata_results(
            MetadataOutcome::Error {
                class: "malformed",
                message: "PDF is malformed".to_string(),
            },
            MetadataOutcome::Error {
                class: "malformed",
                message: "different backend text".to_string(),
            },
        );

        assert!(comparison.matches);
    }

    #[test]
    fn comparison_json_should_include_match_status() {
        let metadata = DocumentMetadata::new(vec![PageMetadata {
            index: 0,
            size: PageSize {
                width: 300.0,
                height: 160.0,
            },
        }]);
        let comparison = compare_metadata_results(
            MetadataOutcome::Success(metadata.clone()),
            MetadataOutcome::Success(metadata),
        );

        let json = comparison_json(Path::new("fixtures/generated/text-page.pdf"), &comparison);

        assert!(json.contains("\"status\": \"match\""));
        assert!(json.contains("\"page_count\":1"));
        assert!(json.contains("\"rust_native_memory\""));
        assert!(json.contains("\"max_page_pixels\":16777216"));
    }

    #[test]
    fn parse_background_should_accept_rgb() {
        let color = parse_background("#102030").expect("valid color");

        assert_eq!(
            color,
            Rgba {
                r: 0x10,
                g: 0x20,
                b: 0x30,
                a: 0xff,
            }
        );
    }

    #[test]
    fn format_background_should_emit_rgba_hex() {
        let color = Rgba {
            r: 0x10,
            g: 0x20,
            b: 0x30,
            a: 0x40,
        };

        assert_eq!(format_background(color), "#10203040");
    }

    #[test]
    fn temporary_output_path_should_stay_next_to_target() {
        let output = Path::new("target/pdfrust-thumbnails/text-page.png");

        let temporary = temporary_output_path(output);

        assert_eq!(temporary.parent(), output.parent());
        assert!(temporary
            .file_name()
            .and_then(|name| name.to_str())
            .expect("file name")
            .starts_with(".text-page.png."));
    }

    #[test]
    fn worker_failure_should_preserve_render_error_class() {
        let error = worker_failure(
            "render error [malformed]: PDF is malformed".to_string(),
            "fallback".to_string(),
        );

        assert_eq!(
            error.to_string(),
            "render error [malformed]: PDF is malformed"
        );
    }

    #[test]
    fn encode_rgba_png_should_write_png_signature() {
        let thumbnail = Thumbnail {
            width: 1,
            height: 1,
            stride: 4,
            pixel_format: PixelFormat::Rgba8,
            bytes: vec![255, 0, 0, 255],
        };

        let png = encode_rgba_png(&thumbnail).expect("valid PNG");

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn render_error_should_include_error_class() {
        let error = CliError::Render {
            class: "malformed",
            message: "PDF is malformed".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "render error [malformed]: PDF is malformed"
        );
    }
}
