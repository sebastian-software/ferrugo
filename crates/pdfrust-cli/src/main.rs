#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(feature = "pdfium")]
use std::process::{Child, Stdio};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

use pdfrust_native::{
    NativeBackend, NativeMemoryDiagnostics, NativePageCacheKey, NativePageCachePolicy,
};
#[cfg(feature = "pdfium")]
use pdfrust_pdfium::PdfiumBackend;
#[cfg(any(feature = "pdfium", test))]
use pdfrust_thumbnail::PageSize;
use pdfrust_thumbnail::{
    DocumentMetadata, DocumentMetadataBackend, PdfSource, Rgba, ThumbnailBackend, ThumbnailError,
    ThumbnailOptions, DEFAULT_MAX_EDGE, DEFAULT_PAGE_INDEX, DEFAULT_TIMEOUT,
};

#[cfg(feature = "pdfium")]
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(any(feature = "pdfium", test))]
const LOW_AMPLITUDE_VISUAL_DRIFT_MAX_DELTA: u8 = 8;
#[cfg(not(feature = "pdfium"))]
const PDFIUM_FEATURE_MESSAGE: &str =
    "PDFium support is disabled; rebuild pdfrust-cli with --features pdfium";

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
        Some("summarize-fallbacks") => summarize_fallbacks_command(&args[1..]),
        Some("extract-corpus-metadata") => extract_corpus_metadata_command(&args[1..]),
        Some("validate-local-corpus") => validate_local_corpus_command(&args[1..]),
        Some("benchmark-native") => benchmark_native_command(&args[1..]),
        Some("benchmark-batch-native") => benchmark_batch_native_command(&args[1..]),
        Some("benchmark-repeat-native") => benchmark_repeat_native_command(&args[1..]),
        Some("benchmark-pdfium") => benchmark_pdfium_command(&args[1..]),
        Some("visual-diff") => visual_diff_command(&args[1..]),
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
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        render_direct_command_pdfium(args)
    }
}

#[cfg(feature = "pdfium")]
fn render_direct_command_pdfium(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    render_direct(config)
}

#[cfg(feature = "pdfium")]
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
    eprintln!("render backend: {}", outcome.backend);
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
        Err(err) if err.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported => {
            let reason = FallbackReason::from_native_error(&err);
            if !config.fallback_policy.allows(reason) {
                return Err(CliError::Render {
                    class: err.class().as_str(),
                    message: format!(
                        "PDFium fallback not enabled for {}; pass --allow-pdfium-fallback to opt in",
                        reason.as_str()
                    ),
                });
            }
            #[cfg(not(feature = "pdfium"))]
            {
                Err(CliError::Render {
                    class: err.class().as_str(),
                    message: format!("{} for {}", PDFIUM_FEATURE_MESSAGE, reason.as_str()),
                })
            }
            #[cfg(feature = "pdfium")]
            {
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
                    backend: AutoRenderBackend::PdfiumFallback { reason },
                })
            }
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
    #[cfg_attr(not(feature = "pdfium"), allow(dead_code))]
    PdfiumFallback {
        reason: FallbackReason,
    },
}

impl fmt::Display for AutoRenderBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native => f.write_str("native"),
            Self::PdfiumFallback { reason } => {
                write!(
                    f,
                    "pdfium fallback_reason={} fallback_category={}",
                    reason.as_str(),
                    reason.category()
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FallbackReason {
    NativeUnsupported,
    NativeUnsupportedFeature(&'static str),
}

impl FallbackReason {
    fn from_native_error(error: &ThumbnailError) -> Self {
        error
            .unsupported_feature_bucket()
            .map(Self::NativeUnsupportedFeature)
            .unwrap_or(Self::NativeUnsupported)
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::NativeUnsupported => "native.unsupported",
            Self::NativeUnsupportedFeature(bucket) => bucket,
        }
    }

    const fn category(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackPolicy {
    allow_pdfium: bool,
    denied_reasons: Vec<String>,
}

impl FallbackPolicy {
    fn allows(&self, reason: FallbackReason) -> bool {
        self.allow_pdfium
            && !self
                .denied_reasons
                .iter()
                .any(|denied| denied == reason.as_str())
    }
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        Self {
            allow_pdfium: env_flag("PDFRUST_ALLOW_PDFIUM_FALLBACK")
                && !env_flag("PDFRUST_NATIVE_ONLY"),
            denied_reasons: env_list("PDFRUST_DENY_FALLBACK_REASONS"),
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
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        render_isolated_command_pdfium(args)
    }
}

#[cfg(feature = "pdfium")]
fn render_isolated_command_pdfium(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    render_isolated(config)
}

fn compare_metadata_command(args: &[OsString]) -> Result<(), CliError> {
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        compare_metadata_command_pdfium(args)
    }
}

#[cfg(feature = "pdfium")]
fn compare_metadata_command_pdfium(args: &[OsString]) -> Result<(), CliError> {
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

fn summarize_fallbacks_command(args: &[OsString]) -> Result<(), CliError> {
    let config = FallbackSummaryConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let native = config.native_profile.backend();
    let summary = summarize_native_fallbacks(&native, &fixtures, &options, manifest.as_ref());
    let json = fallback_summary_json(&summary);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if let Some(diagnostics_dir) = &config.diagnostics_dir {
        write_native_diagnostic_bundles(
            &native,
            &fixtures,
            &options,
            manifest.as_ref(),
            diagnostics_dir,
        )?;
    }

    if config.fail_on_fallback && summary.fallback_required > 0 {
        Err(CliError::Compare(format!(
            "{} native fallback(s) required",
            summary.fallback_required
        )))
    } else {
        Ok(())
    }
}

fn extract_corpus_metadata_command(args: &[OsString]) -> Result<(), CliError> {
    let config = CorpusMetadataConfig::parse(args)?;
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let records = extract_native_corpus_metadata(&fixtures, manifest.as_ref());
    let json = corpus_metadata_json(&records);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    Ok(())
}

fn validate_local_corpus_command(args: &[OsString]) -> Result<(), CliError> {
    let config = LocalCorpusValidationConfig::parse(args)?;
    if config.allow_missing && !config.input.exists() {
        println!("{}", local_corpus_missing_json(&config.input));
        return Ok(());
    }

    let content = fs::read_to_string(&config.input).map_err(|source| CliError::ReadFile {
        path: config.input.clone(),
        source,
    })?;
    let report = validate_local_corpus_metadata(&content)?;
    println!("{}", local_corpus_validation_json(&report));
    Ok(())
}

fn benchmark_native_command(args: &[OsString]) -> Result<(), CliError> {
    let config = BenchmarkConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let native = config.native_profile.backend();
    let report = benchmark_backend(
        &native,
        "rust-native",
        &fixtures,
        &options,
        manifest.as_ref(),
        &config,
        true,
    );
    write_benchmark_report(config, report)
}

fn benchmark_batch_native_command(args: &[OsString]) -> Result<(), CliError> {
    let config = BatchBenchmarkConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let report = benchmark_native_batch(&fixtures, &options, manifest.as_ref(), &config)?;
    let json = batch_benchmark_report_json(&report);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if config.fail_on_budget && report.budget_failures > 0 {
        Err(CliError::Benchmark(format!(
            "{} batch benchmark budget failure(s)",
            report.budget_failures
        )))
    } else {
        Ok(())
    }
}

fn benchmark_repeat_native_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RepeatBenchmarkConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let report = benchmark_native_repeat(&fixtures, &options, manifest.as_ref(), &config)?;
    let json = repeat_benchmark_report_json(&report);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if config.fail_on_budget && report.budget_failures > 0 {
        Err(CliError::Benchmark(format!(
            "{} repeated-render benchmark budget failure(s)",
            report.budget_failures
        )))
    } else {
        Ok(())
    }
}

fn benchmark_pdfium_command(args: &[OsString]) -> Result<(), CliError> {
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        benchmark_pdfium_command_enabled(args)
    }
}

#[cfg(feature = "pdfium")]
fn benchmark_pdfium_command_enabled(args: &[OsString]) -> Result<(), CliError> {
    let config = BenchmarkConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let pdfium = PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
    let report = benchmark_backend(
        &pdfium,
        "pdfium",
        &fixtures,
        &options,
        manifest.as_ref(),
        &config,
        false,
    );
    write_benchmark_report(config, report)
}

fn visual_diff_command(args: &[OsString]) -> Result<(), CliError> {
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        visual_diff_command_enabled(args)
    }
}

#[cfg(feature = "pdfium")]
fn visual_diff_command_enabled(args: &[OsString]) -> Result<(), CliError> {
    let config = VisualDiffConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let native = NativeBackend::new();
    let pdfium = PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
    let report = visual_diff_report(
        &native,
        &pdfium,
        &fixtures,
        &options,
        manifest.as_ref(),
        config.thresholds,
    );
    let json = visual_diff_report_json(&report);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    Ok(())
}

#[cfg(not(feature = "pdfium"))]
fn pdfium_feature_disabled() -> CliError {
    CliError::Usage(PDFIUM_FEATURE_MESSAGE.to_string())
}

fn write_benchmark_report(
    config: BenchmarkConfig,
    report: BenchmarkReport,
) -> Result<(), CliError> {
    let json = benchmark_report_json(&report);

    if let Some(output) = config.output {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if config.fail_on_budget && report.budget_failures > 0 {
        Err(CliError::Benchmark(format!(
            "{} benchmark budget failure(s)",
            report.budget_failures
        )))
    } else {
        Ok(())
    }
}

#[cfg(feature = "pdfium")]
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

#[cfg(feature = "pdfium")]
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

#[cfg(feature = "pdfium")]
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

#[cfg(feature = "pdfium")]
fn terminate_worker(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = read_worker_stderr(child);
}

#[cfg(feature = "pdfium")]
fn read_worker_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr.trim().to_string()
}

#[cfg(any(feature = "pdfium", test))]
fn worker_failure(stderr: String, fallback: String) -> CliError {
    parse_worker_render_error(&stderr).unwrap_or_else(|| {
        let message = if stderr.is_empty() { fallback } else { stderr };
        CliError::Render {
            class: "internal",
            message,
        }
    })
}

#[cfg(any(feature = "pdfium", test))]
fn parse_worker_render_error(stderr: &str) -> Option<CliError> {
    let rest = stderr.strip_prefix("render error [")?;
    let (class, message) = rest.split_once("]: ")?;
    Some(CliError::Render {
        class: stable_error_class(class),
        message: message.to_string(),
    })
}

#[cfg(any(feature = "pdfium", test))]
fn stable_error_class(class: &str) -> &'static str {
    match class {
        "encrypted" => "encrypted",
        "malformed" => "malformed",
        "unsupported" => "unsupported",
        "timeout" => "timeout",
        _ => "internal",
    }
}

#[cfg(feature = "pdfium")]
fn timeout_error() -> CliError {
    CliError::Render {
        class: ThumbnailError::Timeout.class().as_str(),
        message: ThumbnailError::Timeout.to_string(),
    }
}

#[cfg(any(feature = "pdfium", test))]
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
    fallback_policy: FallbackPolicy,
}

impl RenderConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut fallback_policy = FallbackPolicy::default();

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
                "--allow-pdfium-fallback" => {
                    fallback_policy.allow_pdfium = true;
                }
                "--native-only" | "--no-pdfium-fallback" => {
                    fallback_policy.allow_pdfium = false;
                }
                "--deny-fallback-reason" => {
                    index += 1;
                    fallback_policy
                        .denied_reasons
                        .push(required_str(args, index, "--deny-fallback-reason")?.to_string());
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
            fallback_policy,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(feature = "pdfium"), allow(dead_code))]
struct CompareMetadataConfig {
    input: PathBuf,
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackSummaryConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    output: Option<PathBuf>,
    diagnostics_dir: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    fail_on_fallback: bool,
    include_families: Vec<String>,
    native_profile: NativeProfile,
}

impl FallbackSummaryConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut output = None;
        let mut diagnostics_dir = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut fail_on_fallback = false;
        let mut include_families = Vec::new();
        let mut native_profile = NativeProfile::Default;

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
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                "--diagnostics-dir" => {
                    index += 1;
                    diagnostics_dir = Some(required_path(args, index, "--diagnostics-dir")?);
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
                "--fail-on-fallback" => {
                    fail_on_fallback = true;
                }
                "--include-family" => {
                    index += 1;
                    include_families
                        .push(required_str(args, index, "--include-family")?.to_string());
                }
                "--native-profile" => {
                    index += 1;
                    native_profile =
                        parse_native_profile(required_str(args, index, "--native-profile")?)?;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
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
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            output,
            diagnostics_dir,
            page_index,
            max_edge,
            background,
            timeout,
            fail_on_fallback,
            include_families,
            native_profile,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeProfile {
    Default,
    LowMemory,
}

impl NativeProfile {
    fn backend(self) -> NativeBackend {
        match self {
            Self::Default => NativeBackend::new(),
            Self::LowMemory => NativeBackend::low_memory(),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::LowMemory => "low-memory",
        }
    }
}

fn parse_native_profile(value: &str) -> Result<NativeProfile, CliError> {
    match value {
        "default" => Ok(NativeProfile::Default),
        "low-memory" => Ok(NativeProfile::LowMemory),
        _ => Err(CliError::Usage(format!(
            "unknown --native-profile `{value}`; expected `default` or `low-memory`"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusMetadataConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl CorpusMetadataConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
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
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            output,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalCorpusValidationConfig {
    input: PathBuf,
    allow_missing: bool,
}

impl LocalCorpusValidationConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut allow_missing = false;

        let mut index = 0;
        while index < args.len() {
            let arg = args[index]
                .to_str()
                .ok_or_else(|| CliError::Usage("arguments must be valid UTF-8".to_string()))?;
            match arg {
                "--allow-missing" => {
                    allow_missing = true;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one local corpus metadata path is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        Ok(Self {
            input: input
                .ok_or_else(|| CliError::Usage("missing local corpus metadata path".to_string()))?,
            allow_missing,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    iterations: usize,
    max_ms: u64,
    max_output_bytes: usize,
    fail_on_budget: bool,
    native_profile: NativeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchBenchmarkConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    repetitions: usize,
    max_workers: usize,
    max_in_flight_pixels: usize,
    max_p95_ms: u64,
    max_errors: usize,
    fail_on_budget: bool,
    native_profile: NativeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepeatBenchmarkConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    repetitions: usize,
    max_first_ms: u64,
    max_repeat_mean_ms: u64,
    max_errors: usize,
    fail_on_budget: bool,
    native_profile: NativeProfile,
}

#[cfg(feature = "pdfium")]
#[derive(Debug, Clone, PartialEq)]
struct VisualDiffConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    thresholds: VisualDiffThresholds,
}

#[cfg(any(feature = "pdfium", test))]
#[derive(Debug, Clone, Copy, PartialEq)]
struct VisualDiffThresholds {
    max_mean_abs_error: f64,
    max_p95_channel_delta: u8,
    max_changed_ratio: f64,
}

#[cfg(any(feature = "pdfium", test))]
impl Default for VisualDiffThresholds {
    fn default() -> Self {
        Self {
            max_mean_abs_error: 2.0,
            max_p95_channel_delta: 16,
            max_changed_ratio: 0.05,
        }
    }
}

impl BenchmarkConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = 160;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut iterations = 1;
        let mut max_ms = 250;
        let mut max_output_bytes = 4 * 160 * 160;
        let mut fail_on_budget = false;
        let mut native_profile = NativeProfile::Default;

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
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                "--include-family" => {
                    index += 1;
                    include_families
                        .push(required_str(args, index, "--include-family")?.to_string());
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
                "--iterations" => {
                    index += 1;
                    iterations = parse_usize(args, index, "--iterations")?;
                }
                "--max-ms" => {
                    index += 1;
                    max_ms = parse_u64(args, index, "--max-ms")?;
                }
                "--max-output-bytes" => {
                    index += 1;
                    max_output_bytes = parse_usize(args, index, "--max-output-bytes")?;
                }
                "--fail-on-budget" => {
                    fail_on_budget = true;
                }
                "--native-profile" => {
                    index += 1;
                    native_profile =
                        parse_native_profile(required_str(args, index, "--native-profile")?)?;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
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
        if iterations == 0 {
            return Err(CliError::Usage(
                "--iterations must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            include_families,
            output,
            page_index,
            max_edge,
            background,
            timeout,
            iterations,
            max_ms,
            max_output_bytes,
            fail_on_budget,
            native_profile,
        })
    }
}

impl BatchBenchmarkConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = 160;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut repetitions = 2;
        let mut max_workers = 2;
        let mut max_in_flight_pixels = 2 * 160 * 160;
        let mut max_p95_ms = 1000;
        let mut max_errors = 0;
        let mut fail_on_budget = false;
        let mut native_profile = NativeProfile::Default;

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
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                "--include-family" => {
                    index += 1;
                    include_families
                        .push(required_str(args, index, "--include-family")?.to_string());
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
                "--repetitions" => {
                    index += 1;
                    repetitions = parse_usize(args, index, "--repetitions")?;
                }
                "--max-workers" => {
                    index += 1;
                    max_workers = parse_usize(args, index, "--max-workers")?;
                }
                "--max-in-flight-pixels" => {
                    index += 1;
                    max_in_flight_pixels = parse_usize(args, index, "--max-in-flight-pixels")?;
                }
                "--max-p95-ms" => {
                    index += 1;
                    max_p95_ms = parse_u64(args, index, "--max-p95-ms")?;
                }
                "--max-errors" => {
                    index += 1;
                    max_errors = parse_usize(args, index, "--max-errors")?;
                }
                "--fail-on-budget" => {
                    fail_on_budget = true;
                }
                "--native-profile" => {
                    index += 1;
                    native_profile =
                        parse_native_profile(required_str(args, index, "--native-profile")?)?;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
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
        if repetitions == 0 {
            return Err(CliError::Usage(
                "--repetitions must be greater than zero".to_string(),
            ));
        }
        if max_workers == 0 {
            return Err(CliError::Usage(
                "--max-workers must be greater than zero".to_string(),
            ));
        }
        if max_in_flight_pixels == 0 {
            return Err(CliError::Usage(
                "--max-in-flight-pixels must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            include_families,
            output,
            page_index,
            max_edge,
            background,
            timeout,
            repetitions,
            max_workers,
            max_in_flight_pixels,
            max_p95_ms,
            max_errors,
            fail_on_budget,
            native_profile,
        })
    }
}

impl RepeatBenchmarkConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = 160;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut repetitions = 3;
        let mut max_first_ms = 1000;
        let mut max_repeat_mean_ms = 1000;
        let mut max_errors = 0;
        let mut fail_on_budget = false;
        let mut native_profile = NativeProfile::Default;

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
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                "--include-family" => {
                    index += 1;
                    include_families
                        .push(required_str(args, index, "--include-family")?.to_string());
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
                "--repetitions" => {
                    index += 1;
                    repetitions = parse_usize(args, index, "--repetitions")?;
                }
                "--max-first-ms" => {
                    index += 1;
                    max_first_ms = parse_u64(args, index, "--max-first-ms")?;
                }
                "--max-repeat-mean-ms" => {
                    index += 1;
                    max_repeat_mean_ms = parse_u64(args, index, "--max-repeat-mean-ms")?;
                }
                "--max-errors" => {
                    index += 1;
                    max_errors = parse_usize(args, index, "--max-errors")?;
                }
                "--fail-on-budget" => {
                    fail_on_budget = true;
                }
                "--native-profile" => {
                    index += 1;
                    native_profile =
                        parse_native_profile(required_str(args, index, "--native-profile")?)?;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
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
        if repetitions < 2 {
            return Err(CliError::Usage(
                "--repetitions must be at least 2 for repeated-render benchmarks".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            include_families,
            output,
            page_index,
            max_edge,
            background,
            timeout,
            repetitions,
            max_first_ms,
            max_repeat_mean_ms,
            max_errors,
            fail_on_budget,
            native_profile,
        })
    }
}

#[cfg(feature = "pdfium")]
impl VisualDiffConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = 160;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut thresholds = VisualDiffThresholds::default();

        let mut index = 0;
        while index < args.len() {
            let arg = args[index]
                .to_str()
                .ok_or_else(|| CliError::Usage("arguments must be valid UTF-8".to_string()))?;
            match arg {
                "--manifest" => {
                    index += 1;
                    manifest = Some(required_path(args, index, "--manifest")?);
                }
                "--include-family" => {
                    index += 1;
                    include_families
                        .push(required_str(args, index, "--include-family")?.to_string());
                }
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
                "--max-mae" => {
                    index += 1;
                    thresholds.max_mean_abs_error = parse_f64(args, index, "--max-mae")?;
                }
                "--max-p95" => {
                    index += 1;
                    thresholds.max_p95_channel_delta = parse_u8(args, index, "--max-p95")?;
                }
                "--max-changed-ratio" => {
                    index += 1;
                    thresholds.max_changed_ratio = parse_f64(args, index, "--max-changed-ratio")?;
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input path is supported".to_string(),
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
        if thresholds.max_changed_ratio < 0.0 || thresholds.max_changed_ratio > 1.0 {
            return Err(CliError::Usage(
                "--max-changed-ratio must be between 0.0 and 1.0".to_string(),
            ));
        }
        if thresholds.max_mean_abs_error < 0.0 {
            return Err(CliError::Usage(
                "--max-mae must be greater than or equal to zero".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            include_families,
            output,
            page_index,
            max_edge,
            background,
            timeout,
            thresholds,
        })
    }
}

#[cfg(any(feature = "pdfium", test))]
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
    Success(Box<DocumentMetadata>),
    Error {
        class: &'static str,
        message: String,
    },
}

impl MetadataOutcome {
    fn from_result(result: Result<DocumentMetadata, ThumbnailError>) -> Self {
        match result {
            Ok(metadata) => Self::Success(Box::new(metadata)),
            Err(error) => Self::Error {
                class: error.class().as_str(),
                message: error.to_string(),
            },
        }
    }
}

#[cfg_attr(not(feature = "pdfium"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
struct MetadataComparison {
    matches: bool,
    pdfium: MetadataOutcome,
    native: MetadataOutcome,
    mismatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackSummary {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: BTreeMap<&'static str, usize>,
    fallback_categories: BTreeMap<&'static str, usize>,
    families: BTreeMap<String, FamilyFallbackSummary>,
}

impl FallbackSummary {
    fn new(total: usize) -> Self {
        Self {
            total,
            native_rendered: 0,
            fallback_required: 0,
            errors: BTreeMap::new(),
            fallback_categories: BTreeMap::new(),
            families: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct FamilyFallbackSummary {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: BTreeMap<&'static str, usize>,
    fallback_categories: BTreeMap<&'static str, usize>,
}

impl FamilyFallbackSummary {
    fn record(&mut self, outcome: CorpusOutcome) {
        self.total += 1;
        match outcome {
            CorpusOutcome::NativeRendered => {
                self.native_rendered += 1;
            }
            CorpusOutcome::FallbackRequired(reason) => {
                self.fallback_required += 1;
                *self
                    .fallback_categories
                    .entry(reason.category())
                    .or_insert(0) += 1;
            }
            CorpusOutcome::Error(class) => {
                *self.errors.entry(class).or_insert(0) += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorpusOutcome {
    NativeRendered,
    FallbackRequired(FallbackReason),
    Error(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusManifest {
    entries_by_path: BTreeMap<String, CorpusManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusManifestEntry {
    path: String,
    family: String,
    source: String,
    license: String,
    page_count: usize,
    features: Vec<String>,
    notes: String,
}

#[derive(Debug, Clone, PartialEq)]
struct CorpusMetadataRecord {
    path: String,
    manifest: Option<CorpusManifestEntry>,
    metadata: MetadataOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalCorpusValidationReport {
    sample_count: usize,
    document_count: usize,
    categories: BTreeMap<String, usize>,
    privacy: BTreeMap<String, usize>,
    synthetic_replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalCorpusSample {
    id: String,
    category: String,
    privacy: String,
    permission: String,
    redaction_state: String,
    source_note: String,
    count: usize,
    page_count_range: String,
    features: Vec<String>,
    synthetic_replacement: Option<String>,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalCorpusValue {
    String(String),
    Integer(usize),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
struct BenchmarkReport {
    backend: &'static str,
    platform: PlatformMetadata,
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    budget_failures: usize,
    iterations: usize,
    max_ms: u64,
    max_output_bytes: usize,
    families: BTreeMap<String, FamilyBenchmarkSummary>,
    fixtures: Vec<BenchmarkRecord>,
}

#[derive(Debug, Clone, PartialEq)]
struct BatchBenchmarkReport {
    platform: PlatformMetadata,
    total_inputs: usize,
    total_jobs: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    budget_failures: usize,
    workers: usize,
    repetitions: usize,
    elapsed_ms: f64,
    throughput_per_sec: f64,
    max_p95_ms: u64,
    max_errors: usize,
    memory: BatchMemorySummary,
    latency: BatchLatencySummary,
    families: BTreeMap<String, BatchFamilySummary>,
    records: Vec<BatchBenchmarkRecord>,
}

#[derive(Debug, Clone, PartialEq)]
struct RepeatBenchmarkReport {
    platform: PlatformMetadata,
    cache_policy: NativePageCachePolicy,
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    budget_failures: usize,
    repetitions: usize,
    max_first_ms: u64,
    max_repeat_mean_ms: u64,
    max_errors: usize,
    families: BTreeMap<String, RepeatFamilySummary>,
    records: Vec<RepeatBenchmarkRecord>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct BatchMemorySummary {
    rss_start_kib: Option<u64>,
    rss_high_water_kib: Option<u64>,
    rss_end_kib: Option<u64>,
    max_in_flight_pixels: usize,
    max_output_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct BatchLatencySummary {
    mean_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct BatchFamilySummary {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    mean_ms: f64,
    max_ms: f64,
}

impl BatchFamilySummary {
    fn record(&mut self, record: &BatchBenchmarkRecord) {
        self.total += 1;
        self.mean_ms += record.elapsed_ms;
        self.max_ms = self.max_ms.max(record.elapsed_ms);
        match &record.outcome {
            BatchBenchmarkOutcome::NativeRendered { .. } => self.native_rendered += 1,
            BatchBenchmarkOutcome::FallbackRequired { .. } => self.fallback_required += 1,
            BatchBenchmarkOutcome::Error { .. } => self.errors += 1,
        }
    }

    fn finish(&mut self) {
        if self.total > 0 {
            self.mean_ms /= self.total as f64;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BatchBenchmarkRecord {
    path: String,
    family: String,
    repetition: usize,
    page_index: u32,
    elapsed_ms: f64,
    outcome: BatchBenchmarkOutcome,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct RepeatFamilySummary {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    budget_failures: usize,
    first_mean_ms: f64,
    repeat_mean_ms: f64,
}

impl RepeatFamilySummary {
    fn record(&mut self, record: &RepeatBenchmarkRecord) {
        self.total += 1;
        self.budget_failures += usize::from(!record.budget_violations.is_empty());
        match &record.outcome {
            RepeatBenchmarkOutcome::NativeRendered {
                first_ms,
                repeat_mean_ms,
                ..
            } => {
                self.native_rendered += 1;
                self.first_mean_ms += first_ms;
                self.repeat_mean_ms += repeat_mean_ms;
            }
            RepeatBenchmarkOutcome::FallbackRequired { .. } => self.fallback_required += 1,
            RepeatBenchmarkOutcome::Error { .. } => self.errors += 1,
        }
    }

    fn finish(&mut self) {
        if self.native_rendered > 0 {
            self.first_mean_ms /= self.native_rendered as f64;
            self.repeat_mean_ms /= self.native_rendered as f64;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RepeatBenchmarkRecord {
    path: String,
    family: String,
    page_index: u32,
    cache_key: NativePageCacheKey,
    timings_ms: Vec<f64>,
    budget_violations: Vec<&'static str>,
    outcome: RepeatBenchmarkOutcome,
}

struct NativeDiagnosticBundle<'a> {
    path: &'a str,
    manifest: Option<&'a CorpusManifestEntry>,
    options: &'a ThumbnailOptions,
    metadata: Result<&'a DocumentMetadata, &'a ThumbnailError>,
    metadata_ms: f64,
    render_error: &'a ThumbnailError,
    render_ms: f64,
    diagnostics: &'a NativeMemoryDiagnostics,
}

#[derive(Debug, Clone, PartialEq)]
enum BatchBenchmarkOutcome {
    NativeRendered {
        width: u32,
        height: u32,
        output_bytes: usize,
    },
    FallbackRequired {
        reason: FallbackReason,
        message: String,
    },
    Error {
        class: &'static str,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum RepeatBenchmarkOutcome {
    NativeRendered {
        width: u32,
        height: u32,
        output_bytes: usize,
        first_ms: f64,
        repeat_mean_ms: f64,
        repeat_min_ms: f64,
        repeat_max_ms: f64,
        repeat_to_first_ratio: f64,
    },
    FallbackRequired {
        reason: FallbackReason,
        message: String,
    },
    Error {
        class: &'static str,
        message: String,
    },
}

#[cfg(feature = "pdfium")]
#[derive(Debug, Clone, PartialEq)]
struct VisualDiffReport {
    platform: PlatformMetadata,
    thresholds: VisualDiffThresholds,
    total: usize,
    exact: usize,
    accepted_drift: usize,
    blockers: usize,
    native_errors: usize,
    pdfium_errors: usize,
    both_errors: usize,
    families: BTreeMap<String, FamilyVisualDiffSummary>,
    subsystems: BTreeMap<String, FamilyVisualDiffSummary>,
    fixtures: Vec<VisualDiffRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlatformMetadata {
    os: &'static str,
    arch: &'static str,
    family: &'static str,
    endian: &'static str,
    pointer_width_bits: usize,
}

impl PlatformMetadata {
    fn current() -> Self {
        let endian = if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        };
        Self {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            family: std::env::consts::FAMILY,
            endian,
            pointer_width_bits: std::mem::size_of::<usize>() * 8,
        }
    }
}

#[cfg(feature = "pdfium")]
#[derive(Debug, Clone, Default, PartialEq)]
struct FamilyVisualDiffSummary {
    total: usize,
    exact: usize,
    accepted_drift: usize,
    blockers: usize,
    native_errors: usize,
    pdfium_errors: usize,
    both_errors: usize,
}

#[cfg(feature = "pdfium")]
impl FamilyVisualDiffSummary {
    fn record(&mut self, record: &VisualDiffRecord) {
        self.total += 1;
        match record.status {
            VisualDiffStatus::Exact => self.exact += 1,
            VisualDiffStatus::AcceptedDrift => self.accepted_drift += 1,
            VisualDiffStatus::Blocker => self.blockers += 1,
            VisualDiffStatus::NativeError => self.native_errors += 1,
            VisualDiffStatus::PdfiumError => self.pdfium_errors += 1,
            VisualDiffStatus::BothError => self.both_errors += 1,
        }
    }
}

#[cfg(feature = "pdfium")]
#[derive(Debug, Clone, PartialEq)]
struct VisualDiffRecord {
    path: String,
    family: String,
    subsystem: &'static str,
    status: VisualDiffStatus,
    metrics: Option<VisualDiffMetrics>,
    native_error: Option<VisualDiffError>,
    pdfium_error: Option<VisualDiffError>,
}

#[cfg(any(feature = "pdfium", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualDiffStatus {
    Exact,
    AcceptedDrift,
    Blocker,
    #[cfg(feature = "pdfium")]
    NativeError,
    #[cfg(feature = "pdfium")]
    PdfiumError,
    #[cfg(feature = "pdfium")]
    BothError,
}

#[cfg(feature = "pdfium")]
impl VisualDiffStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::AcceptedDrift => "accepted_drift",
            Self::Blocker => "blocker",
            Self::NativeError => "native_error",
            Self::PdfiumError => "pdfium_error",
            Self::BothError => "both_error",
        }
    }
}

#[cfg(any(feature = "pdfium", test))]
#[derive(Debug, Clone, PartialEq)]
struct VisualDiffMetrics {
    width: u32,
    height: u32,
    changed_pixels: usize,
    changed_ratio: f64,
    mean_abs_error: f64,
    p95_channel_delta: u8,
    max_channel_delta: u8,
    native_nonwhite_pixels: usize,
    pdfium_nonwhite_pixels: usize,
}

#[cfg(feature = "pdfium")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct VisualDiffError {
    class: &'static str,
    message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FamilyBenchmarkSummary {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    budget_failures: usize,
    total_ms: f64,
    max_ms: f64,
    total_output_bytes: usize,
}

impl FamilyBenchmarkSummary {
    fn record(&mut self, record: &BenchmarkRecord) {
        self.total += 1;
        self.budget_failures += usize::from(!record.budget_violations.is_empty());
        match &record.outcome {
            BenchmarkOutcome::NativeRendered {
                mean_ms,
                output_bytes,
                ..
            } => {
                self.native_rendered += 1;
                self.total_ms += *mean_ms;
                self.max_ms = self.max_ms.max(*mean_ms);
                self.total_output_bytes += *output_bytes;
            }
            BenchmarkOutcome::FallbackRequired { .. } => {
                self.fallback_required += 1;
            }
            BenchmarkOutcome::Error { .. } => {
                self.errors += 1;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BenchmarkRecord {
    path: String,
    family: String,
    budget_violations: Vec<&'static str>,
    outcome: BenchmarkOutcome,
}

#[derive(Debug, Clone, PartialEq)]
enum BenchmarkOutcome {
    NativeRendered {
        width: u32,
        height: u32,
        output_bytes: usize,
        mean_ms: f64,
    },
    FallbackRequired {
        reason: FallbackReason,
        mean_ms: f64,
    },
    Error {
        class: &'static str,
        message: String,
        mean_ms: f64,
    },
}

#[cfg(any(feature = "pdfium", test))]
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

fn pdf_inputs(input: &Path) -> Result<Vec<PathBuf>, CliError> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }
    if !input.is_dir() {
        return Err(CliError::Usage(format!(
            "input path `{}` is not a file or directory",
            input.display()
        )));
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(input).map_err(|source| CliError::ReadDir {
        path: input.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::ReadDir {
            path: input.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("pdf") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn filter_fixtures_by_family(
    paths: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    include_families: &[String],
) -> Result<Vec<PathBuf>, CliError> {
    if include_families.is_empty() {
        return Ok(paths.to_vec());
    }
    let manifest = manifest
        .ok_or_else(|| CliError::Usage("--include-family requires --manifest".to_string()))?;
    let filtered = paths
        .iter()
        .filter(|path| {
            manifest
                .family_for_path(path)
                .is_some_and(|family| include_families.iter().any(|allowed| allowed == family))
        })
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(CliError::Usage(format!(
            "--include-family matched no fixtures: {}",
            include_families.join(",")
        )));
    }
    Ok(filtered)
}

fn summarize_native_fallbacks(
    native: &NativeBackend,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
) -> FallbackSummary {
    let mut summary = FallbackSummary::new(paths.len());

    for path in paths {
        let outcome = match native.render(PdfSource::from_path(path), options) {
            Ok(_) => {
                summary.native_rendered += 1;
                CorpusOutcome::NativeRendered
            }
            Err(error) if error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported => {
                summary.fallback_required += 1;
                let reason = FallbackReason::from_native_error(&error);
                *summary
                    .fallback_categories
                    .entry(reason.category())
                    .or_insert(0) += 1;
                CorpusOutcome::FallbackRequired(reason)
            }
            Err(error) => {
                let class = error.class().as_str();
                *summary.errors.entry(class).or_insert(0) += 1;
                CorpusOutcome::Error(class)
            }
        };

        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified");
        summary
            .families
            .entry(family.to_string())
            .or_default()
            .record(outcome);
    }

    summary
}

fn write_native_diagnostic_bundles(
    native: &NativeBackend,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    diagnostics_dir: &Path,
) -> Result<usize, CliError> {
    fs::create_dir_all(diagnostics_dir).map_err(|source| CliError::Io {
        path: diagnostics_dir.to_path_buf(),
        source,
    })?;
    let mut written = 0;
    for (index, path) in paths.iter().enumerate() {
        let metadata_started = Instant::now();
        let metadata = native.inspect(PdfSource::from_path(path));
        let metadata_ms = elapsed_ms(metadata_started.elapsed());

        let render_started = Instant::now();
        let render = native.render(PdfSource::from_path(path), options);
        let render_ms = elapsed_ms(render_started.elapsed());
        let Err(render_error) = render else {
            continue;
        };

        let path_key = normalize_manifest_path(path);
        let diagnostics = native.memory_diagnostics();
        let bundle = native_diagnostic_bundle_json(NativeDiagnosticBundle {
            path: &path_key,
            manifest: manifest.and_then(|manifest| manifest.entry_for_path(&path_key)),
            options,
            metadata: metadata.as_ref(),
            metadata_ms,
            render_error: &render_error,
            render_ms,
            diagnostics: &diagnostics,
        });
        let output_path = diagnostic_bundle_path(diagnostics_dir, index, &path_key);
        fs::write(&output_path, bundle).map_err(|source| CliError::Io {
            path: output_path,
            source,
        })?;
        written += 1;
    }
    Ok(written)
}

fn diagnostic_bundle_path(dir: &Path, index: usize, path_key: &str) -> PathBuf {
    let mut slug = path_key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let slug = slug.trim_matches('-');
    let slug = if slug.is_empty() { "fixture" } else { slug };
    let slug = slug.chars().take(96).collect::<String>();
    dir.join(format!("{index:04}-{slug}.diagnostics.json"))
}

fn extract_native_corpus_metadata(
    paths: &[PathBuf],
    manifest: Option<&CorpusManifest>,
) -> Vec<CorpusMetadataRecord> {
    let native = NativeBackend::new();
    paths
        .iter()
        .map(|path| {
            let path_key = normalize_manifest_path(path);
            let manifest = manifest
                .and_then(|manifest| manifest.entry_for_path(path_key.as_str()))
                .cloned();
            let metadata = MetadataOutcome::from_result(native.inspect(PdfSource::from_path(path)));
            CorpusMetadataRecord {
                path: path_key,
                manifest,
                metadata,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct BatchJob {
    path: PathBuf,
    path_key: String,
    family: String,
    repetition: usize,
}

fn benchmark_native_batch(
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    config: &BatchBenchmarkConfig,
) -> Result<BatchBenchmarkReport, CliError> {
    let workers = effective_batch_workers(config, options)?;
    let jobs = batch_jobs(paths, manifest, config.repetitions);
    let mut records = Vec::with_capacity(jobs.len());
    let mut memory = BatchMemorySummary {
        rss_start_kib: current_rss_kib(),
        rss_high_water_kib: None,
        rss_end_kib: None,
        max_in_flight_pixels: config.max_in_flight_pixels,
        max_output_bytes: 0,
    };
    memory.rss_high_water_kib = memory.rss_start_kib;
    let started = Instant::now();

    for chunk in jobs.chunks(workers) {
        let batch = thread::scope(|scope| {
            chunk
                .iter()
                .map(|job| {
                    scope.spawn(move || {
                        benchmark_batch_job(
                            config.native_profile.backend(),
                            job,
                            options,
                            config.page_index,
                        )
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| CliError::Benchmark("batch worker panicked".to_string()))
                })
                .collect::<Result<Vec<_>, CliError>>()
        })?;
        for record in batch {
            if let BatchBenchmarkOutcome::NativeRendered { output_bytes, .. } = &record.outcome {
                memory.max_output_bytes = memory.max_output_bytes.max(*output_bytes);
            }
            records.push(record);
        }
        memory.rss_high_water_kib = max_optional_u64(memory.rss_high_water_kib, current_rss_kib());
    }

    memory.rss_end_kib = current_rss_kib();
    memory.rss_high_water_kib = max_optional_u64(memory.rss_high_water_kib, memory.rss_end_kib);
    Ok(batch_report_from_records(
        paths.len(),
        workers,
        config,
        records,
        memory,
        elapsed_ms(started.elapsed()),
    ))
}

fn effective_batch_workers(
    config: &BatchBenchmarkConfig,
    options: &ThumbnailOptions,
) -> Result<usize, CliError> {
    let pixels_per_job = (options.max_edge as usize)
        .checked_mul(options.max_edge as usize)
        .ok_or_else(|| CliError::Benchmark("batch max-edge pixel budget overflow".to_string()))?;
    if pixels_per_job == 0 {
        return Err(CliError::Benchmark(
            "batch max-edge pixel budget must be non-zero".to_string(),
        ));
    }
    let memory_limited_workers = config.max_in_flight_pixels / pixels_per_job;
    if memory_limited_workers == 0 {
        return Err(CliError::Benchmark(
            "batch memory budget cannot schedule one render job".to_string(),
        ));
    }
    Ok(config.max_workers.min(memory_limited_workers).max(1))
}

fn batch_jobs(
    paths: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    repetitions: usize,
) -> Vec<BatchJob> {
    let mut jobs = Vec::with_capacity(paths.len() * repetitions);
    for repetition in 0..repetitions {
        for path in paths {
            let path_key = normalize_manifest_path(path);
            let family = manifest
                .and_then(|manifest| manifest.family_for_path(path))
                .unwrap_or("unclassified")
                .to_string();
            jobs.push(BatchJob {
                path: path.clone(),
                path_key,
                family,
                repetition,
            });
        }
    }
    jobs
}

fn benchmark_batch_job(
    native: NativeBackend,
    job: &BatchJob,
    options: &ThumbnailOptions,
    page_index: u32,
) -> BatchBenchmarkRecord {
    let started = Instant::now();
    let outcome = match native.render(PdfSource::from_path(&job.path), options) {
        Ok(thumbnail) => BatchBenchmarkOutcome::NativeRendered {
            width: thumbnail.width,
            height: thumbnail.height,
            output_bytes: thumbnail.bytes.len(),
        },
        Err(error) if error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported => {
            BatchBenchmarkOutcome::FallbackRequired {
                reason: FallbackReason::from_native_error(&error),
                message: error.to_string(),
            }
        }
        Err(error) => BatchBenchmarkOutcome::Error {
            class: error.class().as_str(),
            message: error.to_string(),
        },
    };

    BatchBenchmarkRecord {
        path: job.path_key.clone(),
        family: job.family.clone(),
        repetition: job.repetition,
        page_index,
        elapsed_ms: elapsed_ms(started.elapsed()),
        outcome,
    }
}

fn batch_report_from_records(
    total_inputs: usize,
    workers: usize,
    config: &BatchBenchmarkConfig,
    records: Vec<BatchBenchmarkRecord>,
    memory: BatchMemorySummary,
    elapsed_ms: f64,
) -> BatchBenchmarkReport {
    let mut native_rendered = 0;
    let mut fallback_required = 0;
    let mut errors = 0;
    let mut families = BTreeMap::new();
    for record in &records {
        match &record.outcome {
            BatchBenchmarkOutcome::NativeRendered { .. } => native_rendered += 1,
            BatchBenchmarkOutcome::FallbackRequired { .. } => fallback_required += 1,
            BatchBenchmarkOutcome::Error { .. } => errors += 1,
        }
        families
            .entry(record.family.clone())
            .or_insert_with(BatchFamilySummary::default)
            .record(record);
    }
    for summary in families.values_mut() {
        summary.finish();
    }
    let latency = batch_latency_summary(&records);
    let mut budget_failures = 0;
    if latency.p95_ms > config.max_p95_ms as f64 {
        budget_failures += 1;
    }
    if errors + fallback_required > config.max_errors {
        budget_failures += 1;
    }
    let elapsed_secs = (elapsed_ms / 1000.0).max(f64::EPSILON);
    let total_jobs = records.len();

    BatchBenchmarkReport {
        platform: PlatformMetadata::current(),
        total_inputs,
        total_jobs,
        native_rendered,
        fallback_required,
        errors,
        budget_failures,
        workers,
        repetitions: config.repetitions,
        elapsed_ms,
        throughput_per_sec: total_jobs as f64 / elapsed_secs,
        max_p95_ms: config.max_p95_ms,
        max_errors: config.max_errors,
        memory,
        latency,
        families,
        records,
    }
}

fn benchmark_native_repeat(
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    config: &RepeatBenchmarkConfig,
) -> Result<RepeatBenchmarkReport, CliError> {
    let native = config.native_profile.backend();
    let mut records = Vec::with_capacity(paths.len());
    for path in paths {
        let document_identity = document_identity_hash(path)?;
        let cache_key = NativePageCacheKey::from_options(
            document_identity,
            options,
            config.native_profile.as_str(),
        );
        let path_key = normalize_manifest_path(path);
        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified")
            .to_string();
        records.push(benchmark_repeat_fixture(
            &native, path, options, config, path_key, family, cache_key,
        ));
    }

    Ok(repeat_report_from_records(config, records))
}

fn benchmark_repeat_fixture(
    native: &NativeBackend,
    path: &Path,
    options: &ThumbnailOptions,
    config: &RepeatBenchmarkConfig,
    path_key: String,
    family: String,
    cache_key: NativePageCacheKey,
) -> RepeatBenchmarkRecord {
    let mut timings_ms = Vec::with_capacity(config.repetitions);
    let mut last_success = None;
    for _ in 0..config.repetitions {
        let started = Instant::now();
        match native.render(PdfSource::from_path(path), options) {
            Ok(thumbnail) => {
                timings_ms.push(elapsed_ms(started.elapsed()));
                last_success = Some(thumbnail);
            }
            Err(error) => {
                let outcome =
                    if error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported {
                        RepeatBenchmarkOutcome::FallbackRequired {
                            reason: FallbackReason::from_native_error(&error),
                            message: error.to_string(),
                        }
                    } else {
                        RepeatBenchmarkOutcome::Error {
                            class: error.class().as_str(),
                            message: error.to_string(),
                        }
                    };
                let budget_violations = match &outcome {
                    RepeatBenchmarkOutcome::FallbackRequired { .. } => vec!["native_fallback"],
                    RepeatBenchmarkOutcome::Error { .. } => vec!["render_error"],
                    RepeatBenchmarkOutcome::NativeRendered { .. } => Vec::new(),
                };
                return RepeatBenchmarkRecord {
                    path: path_key,
                    family,
                    page_index: options.page_index,
                    cache_key,
                    timings_ms,
                    budget_violations,
                    outcome,
                };
            }
        }
    }

    let thumbnail = last_success.expect("repetitions is validated as at least two");
    let first_ms = timings_ms[0];
    let repeat_values = &timings_ms[1..];
    let repeat_mean_ms = repeat_values.iter().sum::<f64>() / repeat_values.len() as f64;
    let repeat_min_ms = repeat_values
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .expect("repeat values are non-empty");
    let repeat_max_ms = repeat_values
        .iter()
        .copied()
        .max_by(f64::total_cmp)
        .expect("repeat values are non-empty");
    let repeat_to_first_ratio = repeat_mean_ms / first_ms.max(f64::EPSILON);
    let mut budget_violations = Vec::new();
    if first_ms > config.max_first_ms as f64 {
        budget_violations.push("first_render_time");
    }
    if repeat_mean_ms > config.max_repeat_mean_ms as f64 {
        budget_violations.push("repeat_mean_time");
    }

    RepeatBenchmarkRecord {
        path: path_key,
        family,
        page_index: options.page_index,
        cache_key,
        timings_ms,
        budget_violations,
        outcome: RepeatBenchmarkOutcome::NativeRendered {
            width: thumbnail.width,
            height: thumbnail.height,
            output_bytes: thumbnail.bytes.len(),
            first_ms,
            repeat_mean_ms,
            repeat_min_ms,
            repeat_max_ms,
            repeat_to_first_ratio,
        },
    }
}

fn repeat_report_from_records(
    config: &RepeatBenchmarkConfig,
    records: Vec<RepeatBenchmarkRecord>,
) -> RepeatBenchmarkReport {
    let mut native_rendered = 0;
    let mut fallback_required = 0;
    let mut errors = 0;
    let mut budget_failures = 0;
    let mut families = BTreeMap::new();
    for record in &records {
        match &record.outcome {
            RepeatBenchmarkOutcome::NativeRendered { .. } => native_rendered += 1,
            RepeatBenchmarkOutcome::FallbackRequired { .. } => fallback_required += 1,
            RepeatBenchmarkOutcome::Error { .. } => errors += 1,
        }
        budget_failures += usize::from(!record.budget_violations.is_empty());
        families
            .entry(record.family.clone())
            .or_insert_with(RepeatFamilySummary::default)
            .record(record);
    }
    if errors + fallback_required > config.max_errors {
        budget_failures += 1;
    }
    for summary in families.values_mut() {
        summary.finish();
    }

    RepeatBenchmarkReport {
        platform: PlatformMetadata::current(),
        cache_policy: NativePageCachePolicy::IsolatedRender,
        total: records.len(),
        native_rendered,
        fallback_required,
        errors,
        budget_failures,
        repetitions: config.repetitions,
        max_first_ms: config.max_first_ms,
        max_repeat_mean_ms: config.max_repeat_mean_ms,
        max_errors: config.max_errors,
        families,
        records,
    }
}

fn document_identity_hash(path: &Path) -> Result<u64, CliError> {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut file = fs::File::open(path).map_err(|source| CliError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = FNV_OFFSET;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| CliError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    Ok(hash)
}

fn batch_latency_summary(records: &[BatchBenchmarkRecord]) -> BatchLatencySummary {
    if records.is_empty() {
        return BatchLatencySummary::default();
    }
    let mut values = records
        .iter()
        .map(|record| record.elapsed_ms)
        .collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    let total = values.iter().sum::<f64>();
    BatchLatencySummary {
        mean_ms: total / values.len() as f64,
        p50_ms: percentile(&values, 0.50),
        p95_ms: percentile(&values, 0.95),
        max_ms: *values.last().expect("values is non-empty"),
    }
}

fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    let index = ((sorted_values.len() as f64 * percentile).ceil() as usize)
        .saturating_sub(1)
        .min(sorted_values.len() - 1);
    sorted_values[index]
}

fn current_rss_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", pid.as_str()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
}

const fn max_optional_u64(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left > right { left } else { right }),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn benchmark_backend<B: ThumbnailBackend>(
    backend: &B,
    backend_name: &'static str,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    config: &BenchmarkConfig,
    unsupported_is_fallback: bool,
) -> BenchmarkReport {
    let mut families = BTreeMap::new();
    let mut fixtures = Vec::with_capacity(paths.len());
    let mut native_rendered = 0;
    let mut fallback_required = 0;
    let mut errors = 0;
    let mut budget_failures = 0;

    for path in paths {
        let path_key = normalize_manifest_path(path);
        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified")
            .to_string();
        let record = benchmark_fixture(
            backend,
            path,
            options,
            config,
            path_key,
            family,
            unsupported_is_fallback,
        );
        match record.outcome {
            BenchmarkOutcome::NativeRendered { .. } => native_rendered += 1,
            BenchmarkOutcome::FallbackRequired { .. } => fallback_required += 1,
            BenchmarkOutcome::Error { .. } => errors += 1,
        }
        budget_failures += usize::from(!record.budget_violations.is_empty());
        families
            .entry(record.family.clone())
            .or_insert_with(FamilyBenchmarkSummary::default)
            .record(&record);
        fixtures.push(record);
    }

    BenchmarkReport {
        backend: backend_name,
        platform: PlatformMetadata::current(),
        total: paths.len(),
        native_rendered,
        fallback_required,
        errors,
        budget_failures,
        iterations: config.iterations,
        max_ms: config.max_ms,
        max_output_bytes: config.max_output_bytes,
        families,
        fixtures,
    }
}

fn benchmark_fixture<B: ThumbnailBackend>(
    backend: &B,
    path: &Path,
    options: &ThumbnailOptions,
    config: &BenchmarkConfig,
    path_key: String,
    family: String,
    unsupported_is_fallback: bool,
) -> BenchmarkRecord {
    let started = Instant::now();
    let mut last_success = None;
    for _ in 0..config.iterations {
        match backend.render(PdfSource::from_path(path), options) {
            Ok(thumbnail) => last_success = Some(thumbnail),
            Err(error) => {
                let mean_ms = elapsed_mean_ms(started.elapsed(), config.iterations);
                let (outcome, mut budget_violations) =
                    benchmark_error_outcome(error, mean_ms, unsupported_is_fallback);
                if matches!(outcome, BenchmarkOutcome::FallbackRequired { .. }) {
                    budget_violations.push("native_fallback");
                } else {
                    budget_violations.push("render_error");
                }
                return BenchmarkRecord {
                    path: path_key,
                    family,
                    budget_violations,
                    outcome,
                };
            }
        }
    }

    let thumbnail = last_success.expect("iterations is validated as non-zero");
    let mean_ms = elapsed_mean_ms(started.elapsed(), config.iterations);
    let output_bytes = thumbnail.bytes.len();
    let mut budget_violations = Vec::new();
    if mean_ms > config.max_ms as f64 {
        budget_violations.push("render_time");
    }
    if output_bytes > config.max_output_bytes {
        budget_violations.push("output_bytes");
    }

    BenchmarkRecord {
        path: path_key,
        family,
        budget_violations,
        outcome: BenchmarkOutcome::NativeRendered {
            width: thumbnail.width,
            height: thumbnail.height,
            output_bytes,
            mean_ms,
        },
    }
}

fn benchmark_error_outcome(
    error: ThumbnailError,
    mean_ms: f64,
    unsupported_is_fallback: bool,
) -> (BenchmarkOutcome, Vec<&'static str>) {
    if unsupported_is_fallback
        && error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported
    {
        (
            BenchmarkOutcome::FallbackRequired {
                reason: FallbackReason::from_native_error(&error),
                mean_ms,
            },
            Vec::new(),
        )
    } else {
        (
            BenchmarkOutcome::Error {
                class: error.class().as_str(),
                message: error.to_string(),
                mean_ms,
            },
            Vec::new(),
        )
    }
}

#[cfg(feature = "pdfium")]
fn visual_diff_report<N: ThumbnailBackend, P: ThumbnailBackend>(
    native: &N,
    pdfium: &P,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    thresholds: VisualDiffThresholds,
) -> VisualDiffReport {
    let mut families = BTreeMap::new();
    let mut fixtures = Vec::with_capacity(paths.len());
    let mut exact = 0;
    let mut accepted_drift = 0;
    let mut blockers = 0;
    let mut native_errors = 0;
    let mut pdfium_errors = 0;
    let mut both_errors = 0;
    let mut subsystems = BTreeMap::new();

    for path in paths {
        let path_key = normalize_manifest_path(path);
        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified")
            .to_string();
        let record =
            visual_diff_fixture(native, pdfium, path, options, path_key, family, thresholds);
        match record.status {
            VisualDiffStatus::Exact => exact += 1,
            VisualDiffStatus::AcceptedDrift => accepted_drift += 1,
            VisualDiffStatus::Blocker => blockers += 1,
            VisualDiffStatus::NativeError => native_errors += 1,
            VisualDiffStatus::PdfiumError => pdfium_errors += 1,
            VisualDiffStatus::BothError => both_errors += 1,
        }
        families
            .entry(record.family.clone())
            .or_insert_with(FamilyVisualDiffSummary::default)
            .record(&record);
        subsystems
            .entry(record.subsystem.to_string())
            .or_insert_with(FamilyVisualDiffSummary::default)
            .record(&record);
        fixtures.push(record);
    }

    VisualDiffReport {
        platform: PlatformMetadata::current(),
        thresholds,
        total: paths.len(),
        exact,
        accepted_drift,
        blockers,
        native_errors,
        pdfium_errors,
        both_errors,
        families,
        subsystems,
        fixtures,
    }
}

#[cfg(feature = "pdfium")]
fn visual_diff_fixture<N: ThumbnailBackend, P: ThumbnailBackend>(
    native: &N,
    pdfium: &P,
    path: &Path,
    options: &ThumbnailOptions,
    path_key: String,
    family: String,
    thresholds: VisualDiffThresholds,
) -> VisualDiffRecord {
    let native_result = native.render(PdfSource::from_path(path), options);
    let pdfium_result = pdfium.render(PdfSource::from_path(path), options);
    let subsystem = visual_diff_subsystem(path_key.as_str(), family.as_str());

    match (native_result, pdfium_result) {
        (Ok(native), Ok(pdfium)) => {
            let metrics = visual_diff_metrics(&native, &pdfium);
            let status = metrics
                .as_ref()
                .map(|metrics| classify_visual_diff(metrics, thresholds))
                .unwrap_or(VisualDiffStatus::Blocker);
            VisualDiffRecord {
                path: path_key,
                family,
                subsystem,
                status,
                metrics,
                native_error: None,
                pdfium_error: None,
            }
        }
        (Err(native), Ok(_)) => VisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: VisualDiffStatus::NativeError,
            metrics: None,
            native_error: Some(VisualDiffError {
                class: native.class().as_str(),
                message: native.to_string(),
            }),
            pdfium_error: None,
        },
        (Ok(_), Err(pdfium)) => VisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: VisualDiffStatus::PdfiumError,
            metrics: None,
            native_error: None,
            pdfium_error: Some(VisualDiffError {
                class: pdfium.class().as_str(),
                message: pdfium.to_string(),
            }),
        },
        (Err(native), Err(pdfium)) => VisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: VisualDiffStatus::BothError,
            metrics: None,
            native_error: Some(VisualDiffError {
                class: native.class().as_str(),
                message: native.to_string(),
            }),
            pdfium_error: Some(VisualDiffError {
                class: pdfium.class().as_str(),
                message: pdfium.to_string(),
            }),
        },
    }
}

#[cfg(any(feature = "pdfium", test))]
fn visual_diff_subsystem(path: &str, family: &str) -> &'static str {
    let path = path.rsplit('/').next().unwrap_or(path);
    if family == "secure-document" || path.contains("encrypted") {
        return "document-security";
    }
    if path.contains("optional-content") {
        return "optional-content";
    }
    if path.contains("acroform")
        || path.contains("annotation")
        || path.contains("widget")
        || path.contains("signature")
    {
        return "annotations-forms";
    }
    if path.contains("font")
        || path.contains("text")
        || path.contains("cid")
        || path.contains("cjk")
        || path.contains("tounicode")
        || path.contains("encoding")
        || path.contains("rtl")
    {
        return "text-fonts";
    }
    if path.contains("image")
        || path.contains("scanned")
        || path.contains("dct")
        || path.contains("cmyk")
        || path.contains("indexed")
        || path.contains("predictor")
    {
        return "images-color";
    }
    if path.contains("transparency") || path.contains("blend") || path.contains("soft-mask") {
        return "transparency";
    }
    if path.contains("gradient")
        || path.contains("shading")
        || path.contains("vector")
        || path.contains("path")
        || path.contains("stroke")
        || path.contains("line-")
        || path.contains("pattern")
    {
        return "vector-graphics";
    }
    if path.contains("page")
        || path.contains("crop")
        || path.contains("rotation")
        || path.contains("rotated")
        || path.contains("user-unit")
        || path.contains("userunit")
    {
        return "page-geometry";
    }
    if path.contains("hybrid") || path.contains("xref") || path.contains("incremental") {
        return "document-structure";
    }
    "rendering-core"
}

#[cfg(any(feature = "pdfium", test))]
fn visual_diff_metrics(
    native: &pdfrust_thumbnail::Thumbnail,
    pdfium: &pdfrust_thumbnail::Thumbnail,
) -> Option<VisualDiffMetrics> {
    if native.width != pdfium.width || native.height != pdfium.height {
        return None;
    }

    let mut changed_pixels = 0;
    let mut native_nonwhite_pixels = 0;
    let mut pdfium_nonwhite_pixels = 0;
    let mut channel_sum = 0usize;
    let mut max_channel_delta = 0u8;
    let mut histogram = [0usize; 256];
    let mut channels = 0usize;

    for (native_pixel, pdfium_pixel) in native
        .bytes
        .chunks_exact(4)
        .zip(pdfium.bytes.chunks_exact(4))
    {
        let mut pixel_changed = false;
        if native_pixel != [255, 255, 255, 255] {
            native_nonwhite_pixels += 1;
        }
        if pdfium_pixel != [255, 255, 255, 255] {
            pdfium_nonwhite_pixels += 1;
        }

        for channel in 0..3 {
            let delta = native_pixel[channel].abs_diff(pdfium_pixel[channel]);
            if delta > 0 {
                pixel_changed = true;
            }
            max_channel_delta = max_channel_delta.max(delta);
            channel_sum += usize::from(delta);
            histogram[usize::from(delta)] += 1;
            channels += 1;
        }

        if pixel_changed {
            changed_pixels += 1;
        }
    }

    let pixels = (native.width as usize).checked_mul(native.height as usize)?;
    Some(VisualDiffMetrics {
        width: native.width,
        height: native.height,
        changed_pixels,
        changed_ratio: changed_pixels as f64 / pixels as f64,
        mean_abs_error: channel_sum as f64 / channels as f64,
        p95_channel_delta: percentile_delta(&histogram, channels, 0.95),
        max_channel_delta,
        native_nonwhite_pixels,
        pdfium_nonwhite_pixels,
    })
}

#[cfg(any(feature = "pdfium", test))]
fn classify_visual_diff(
    metrics: &VisualDiffMetrics,
    thresholds: VisualDiffThresholds,
) -> VisualDiffStatus {
    if metrics.changed_pixels == 0 && metrics.max_channel_delta == 0 {
        return VisualDiffStatus::Exact;
    }
    let bounded_distribution = metrics.p95_channel_delta <= thresholds.max_p95_channel_delta
        && metrics.changed_ratio <= thresholds.max_changed_ratio;
    let low_amplitude_field = metrics.max_channel_delta <= LOW_AMPLITUDE_VISUAL_DRIFT_MAX_DELTA;
    if metrics.mean_abs_error <= thresholds.max_mean_abs_error
        && (bounded_distribution || low_amplitude_field)
    {
        VisualDiffStatus::AcceptedDrift
    } else {
        VisualDiffStatus::Blocker
    }
}

#[cfg(any(feature = "pdfium", test))]
fn percentile_delta(histogram: &[usize; 256], total: usize, percentile: f64) -> u8 {
    if total == 0 {
        return 0;
    }
    let target = ((total as f64 * percentile).ceil() as usize).max(1);
    let mut cumulative = 0usize;
    for (delta, count) in histogram.iter().enumerate() {
        cumulative += count;
        if cumulative >= target {
            return delta as u8;
        }
    }
    u8::MAX
}

fn elapsed_mean_ms(duration: Duration, iterations: usize) -> f64 {
    duration.as_secs_f64() * 1000.0 / iterations as f64
}

fn elapsed_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

impl CorpusManifest {
    fn family_for_path(&self, path: &Path) -> Option<&str> {
        let path = normalize_manifest_path(path);
        self.entry_for_path(path.as_str())
            .map(|entry| entry.family.as_str())
    }

    fn entry_for_path(&self, path: &str) -> Option<&CorpusManifestEntry> {
        self.entries_by_path.get(path)
    }
}

fn read_corpus_manifest(path: &Path) -> Result<CorpusManifest, CliError> {
    let content = fs::read_to_string(path).map_err(|source| CliError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let mut entries_by_path = BTreeMap::new();
    for (line_index, line) in content.lines().enumerate() {
        if line_index == 0 {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 7 {
            return Err(CliError::Usage(format!(
                "manifest line {} must have 7 tab-separated columns",
                line_index + 1
            )));
        }
        let page_count = columns[4].parse().map_err(|_| {
            CliError::Usage(format!(
                "manifest line {} page_count must be an unsigned integer",
                line_index + 1
            ))
        })?;
        let entry = CorpusManifestEntry {
            path: columns[0].to_string(),
            family: columns[1].to_string(),
            source: columns[2].to_string(),
            license: columns[3].to_string(),
            page_count,
            features: columns[5]
                .split(',')
                .filter_map(|feature| {
                    let feature = feature.trim();
                    (!feature.is_empty()).then(|| feature.to_string())
                })
                .collect(),
            notes: columns[6].to_string(),
        };
        entries_by_path.insert(entry.path.clone(), entry);
    }

    Ok(CorpusManifest { entries_by_path })
}

fn validate_local_corpus_metadata(content: &str) -> Result<LocalCorpusValidationReport, CliError> {
    let mut root = BTreeMap::new();
    let mut samples = Vec::new();
    let mut current = None;

    for (line_index, raw_line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[sample]]" {
            if let Some(sample) = current.take() {
                samples.push(local_corpus_sample(sample)?);
            }
            current = Some(BTreeMap::new());
            continue;
        }
        if line.starts_with('[') {
            return Err(CliError::Usage(format!(
                "local corpus metadata line {line_number} uses unsupported table `{line}`"
            )));
        }

        let (key, value) = line.split_once('=').ok_or_else(|| {
            CliError::Usage(format!(
                "local corpus metadata line {line_number} must use `key = value`"
            ))
        })?;
        let key = key.trim();
        reject_private_local_corpus_key(key, line_number)?;
        let value = parse_local_corpus_value(value.trim(), line_number)?;
        if let Some(sample) = current.as_mut() {
            sample.insert(key.to_string(), value);
        } else {
            root.insert(key.to_string(), value);
        }
    }

    if let Some(sample) = current.take() {
        samples.push(local_corpus_sample(sample)?);
    }
    validate_local_corpus_root(&root)?;
    if samples.is_empty() {
        return Err(CliError::Usage(
            "local corpus metadata must contain at least one [[sample]]".to_string(),
        ));
    }

    let mut categories = BTreeMap::new();
    let mut privacy = BTreeMap::new();
    let mut document_count = 0usize;
    let mut synthetic_replacements = 0usize;
    for sample in &samples {
        document_count = document_count.saturating_add(sample.count);
        *categories.entry(sample.category.clone()).or_insert(0) += sample.count;
        *privacy.entry(sample.privacy.clone()).or_insert(0) += sample.count;
        if sample.synthetic_replacement.is_some() {
            synthetic_replacements += 1;
        }
    }

    Ok(LocalCorpusValidationReport {
        sample_count: samples.len(),
        document_count,
        categories,
        privacy,
        synthetic_replacements,
    })
}

fn validate_local_corpus_root(root: &BTreeMap<String, LocalCorpusValue>) -> Result<(), CliError> {
    for key in root.keys() {
        if !["schema_version", "review_date", "reviewer", "notes"].contains(&key.as_str()) {
            return Err(CliError::Usage(format!(
                "local corpus metadata root key `{key}` is not supported"
            )));
        }
    }
    match root.get("schema_version") {
        Some(LocalCorpusValue::Integer(1)) => Ok(()),
        Some(_) => Err(CliError::Usage(
            "local corpus metadata schema_version must be 1".to_string(),
        )),
        None => Err(CliError::Usage(
            "local corpus metadata must declare schema_version = 1".to_string(),
        )),
    }
}

fn local_corpus_sample(
    mut values: BTreeMap<String, LocalCorpusValue>,
) -> Result<LocalCorpusSample, CliError> {
    for key in values.keys() {
        if ![
            "id",
            "category",
            "privacy",
            "permission",
            "redaction_state",
            "source_note",
            "count",
            "page_count_range",
            "features",
            "synthetic_replacement",
            "status",
        ]
        .contains(&key.as_str())
        {
            return Err(CliError::Usage(format!(
                "local corpus sample key `{key}` is not supported"
            )));
        }
    }

    let id = required_local_string(&mut values, "id")?;
    validate_local_identifier(&id, "id")?;
    let category = required_local_string(&mut values, "category")?;
    validate_local_choice(
        "category",
        &category,
        &[
            "invoice",
            "report",
            "scanned-packet",
            "form",
            "statement",
            "browser-export",
            "office-export",
            "presentation",
            "secure-document",
            "malformed-recovery",
        ],
    )?;
    let privacy = required_local_string(&mut values, "privacy")?;
    validate_local_choice(
        "privacy",
        &privacy,
        &[
            "public-redistributable",
            "public-reference-only",
            "private",
            "synthetic-reduced",
        ],
    )?;
    let permission = required_local_string(&mut values, "permission")?;
    validate_local_choice(
        "permission",
        &permission,
        &[
            "redistributable",
            "reference-only",
            "local-review-only",
            "generated",
        ],
    )?;
    let redaction_state = required_local_string(&mut values, "redaction_state")?;
    validate_local_choice(
        "redaction_state",
        &redaction_state,
        &["none", "anonymized", "not-shareable", "reduced-to-fixture"],
    )?;
    let source_note = required_local_string(&mut values, "source_note")?;
    validate_private_safe_text("source_note", &source_note)?;
    let count = required_local_integer(&mut values, "count")?;
    if count == 0 {
        return Err(CliError::Usage(
            "local corpus sample count must be greater than zero".to_string(),
        ));
    }
    let page_count_range = required_local_string(&mut values, "page_count_range")?;
    validate_local_choice(
        "page_count_range",
        &page_count_range,
        &["1", "2-10", "11-50", "50+", "unknown"],
    )?;
    let features = required_local_string_array(&mut values, "features")?;
    if features.is_empty() {
        return Err(CliError::Usage(
            "local corpus sample features must not be empty".to_string(),
        ));
    }
    for feature in &features {
        validate_local_tag("features", feature)?;
    }
    let synthetic_replacement = optional_local_string(&mut values, "synthetic_replacement")?;
    if let Some(path) = &synthetic_replacement {
        validate_synthetic_replacement(path)?;
    }
    let status = required_local_string(&mut values, "status")?;
    validate_local_choice(
        "status",
        &status,
        &["candidate", "reviewed", "blocked", "reduced"],
    )?;

    Ok(LocalCorpusSample {
        id,
        category,
        privacy,
        permission,
        redaction_state,
        source_note,
        count,
        page_count_range,
        features,
        synthetic_replacement,
        status,
    })
}

fn parse_local_corpus_value(raw: &str, line_number: usize) -> Result<LocalCorpusValue, CliError> {
    if let Some(value) = parse_local_string(raw) {
        return Ok(LocalCorpusValue::String(value));
    }
    if let Some(inner) = raw
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        if inner.trim().is_empty() {
            return Ok(LocalCorpusValue::StringArray(Vec::new()));
        }
        let mut values = Vec::new();
        for item in inner.split(',') {
            let item = parse_local_string(item.trim()).ok_or_else(|| {
                CliError::Usage(format!(
                    "local corpus metadata line {line_number} arrays must contain quoted strings"
                ))
            })?;
            values.push(item);
        }
        return Ok(LocalCorpusValue::StringArray(values));
    }
    let value = raw.parse().map_err(|_| {
        CliError::Usage(format!(
            "local corpus metadata line {line_number} values must be strings, string arrays, or unsigned integers"
        ))
    })?;
    Ok(LocalCorpusValue::Integer(value))
}

fn parse_local_string(raw: &str) -> Option<String> {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
}

fn required_local_string(
    values: &mut BTreeMap<String, LocalCorpusValue>,
    key: &str,
) -> Result<String, CliError> {
    match values.remove(key) {
        Some(LocalCorpusValue::String(value)) => Ok(value),
        Some(_) => Err(CliError::Usage(format!(
            "local corpus sample `{key}` must be a string"
        ))),
        None => Err(CliError::Usage(format!(
            "local corpus sample is missing `{key}`"
        ))),
    }
}

fn optional_local_string(
    values: &mut BTreeMap<String, LocalCorpusValue>,
    key: &str,
) -> Result<Option<String>, CliError> {
    match values.remove(key) {
        Some(LocalCorpusValue::String(value)) if value == "none-yet" => Ok(None),
        Some(LocalCorpusValue::String(value)) => Ok(Some(value)),
        Some(_) => Err(CliError::Usage(format!(
            "local corpus sample `{key}` must be a string"
        ))),
        None => Ok(None),
    }
}

fn required_local_integer(
    values: &mut BTreeMap<String, LocalCorpusValue>,
    key: &str,
) -> Result<usize, CliError> {
    match values.remove(key) {
        Some(LocalCorpusValue::Integer(value)) => Ok(value),
        Some(_) => Err(CliError::Usage(format!(
            "local corpus sample `{key}` must be an unsigned integer"
        ))),
        None => Err(CliError::Usage(format!(
            "local corpus sample is missing `{key}`"
        ))),
    }
}

fn required_local_string_array(
    values: &mut BTreeMap<String, LocalCorpusValue>,
    key: &str,
) -> Result<Vec<String>, CliError> {
    match values.remove(key) {
        Some(LocalCorpusValue::StringArray(value)) => Ok(value),
        Some(_) => Err(CliError::Usage(format!(
            "local corpus sample `{key}` must be a string array"
        ))),
        None => Err(CliError::Usage(format!(
            "local corpus sample is missing `{key}`"
        ))),
    }
}

fn reject_private_local_corpus_key(key: &str, line_number: usize) -> Result<(), CliError> {
    if [
        "path",
        "filename",
        "file_name",
        "hash",
        "sha256",
        "text_excerpt",
        "screenshot",
        "rendered_output",
    ]
    .contains(&key)
    {
        Err(CliError::Usage(format!(
            "local corpus metadata line {line_number} key `{key}` is disallowed for private safety"
        )))
    } else {
        Ok(())
    }
}

fn validate_local_identifier(field: &str, value: &str) -> Result<(), CliError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(CliError::Usage(format!(
            "local corpus `{field}` must use lowercase letters, digits, and dashes only"
        )));
    }
    Ok(())
}

fn validate_local_tag(field: &str, value: &str) -> Result<(), CliError> {
    if value.is_empty()
        || value.len() > 48
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b':'
        })
    {
        return Err(CliError::Usage(format!(
            "local corpus `{field}` tag `{value}` must use lowercase tag characters only"
        )));
    }
    Ok(())
}

fn validate_local_choice(field: &str, value: &str, allowed: &[&str]) -> Result<(), CliError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(CliError::Usage(format!(
            "local corpus `{field}` value `{value}` is not supported"
        )))
    }
}

fn validate_private_safe_text(field: &str, value: &str) -> Result<(), CliError> {
    let lowercase = value.to_ascii_lowercase();
    let has_forbidden_marker = lowercase.contains('@')
        || lowercase.contains('/')
        || lowercase.contains('\\')
        || lowercase.contains(".pdf")
        || has_long_hex_run(&lowercase);
    if value.is_empty() || value.len() > 160 || has_forbidden_marker {
        return Err(CliError::Usage(format!(
            "local corpus `{field}` must be anonymized aggregate text"
        )));
    }
    Ok(())
}

fn has_long_hex_run(value: &str) -> bool {
    let mut run = 0usize;
    for byte in value.bytes() {
        if byte.is_ascii_hexdigit() {
            run += 1;
            if run >= 32 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn validate_synthetic_replacement(value: &str) -> Result<(), CliError> {
    if value.starts_with("fixtures/generated/") && value.ends_with(".pdf") {
        Ok(())
    } else {
        Err(CliError::Usage(
            "local corpus synthetic_replacement must point to fixtures/generated/*.pdf or use \"none-yet\""
                .to_string(),
        ))
    }
}

fn normalize_manifest_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    path.find("fixtures/")
        .map(|index| path[index..].to_string())
        .unwrap_or(path)
}

#[cfg(any(feature = "pdfium", test))]
fn page_sizes_match(expected: PageSize, actual: PageSize) -> bool {
    const EPSILON: f64 = 0.01;
    (expected.width - actual.width).abs() <= EPSILON
        && (expected.height - actual.height).abs() <= EPSILON
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    #[cfg_attr(not(feature = "pdfium"), allow(dead_code))]
    Backend(String),
    #[cfg_attr(not(feature = "pdfium"), allow(dead_code))]
    Process(String),
    Render {
        class: &'static str,
        message: String,
    },
    Compare(String),
    Benchmark(String),
    Encode(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadDir {
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
            Self::Benchmark(message) => write!(f, "benchmark budget failure: {message}"),
            Self::Encode(message) => write!(f, "PNG encode error: {message}"),
            Self::Io { path, source } => {
                write!(f, "failed to write `{}`: {source}", path.display())
            }
            Self::ReadFile { path, source } => {
                write!(f, "failed to read `{}`: {source}", path.display())
            }
            Self::ReadDir { path, source } => {
                write!(f, "failed to read `{}`: {source}", path.display())
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

#[cfg(feature = "pdfium")]
fn parse_u8(args: &[OsString], index: usize, option: &str) -> Result<u8, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an integer between 0 and 255")))
}

#[cfg(feature = "pdfium")]
fn parse_f64(args: &[OsString], index: usize, option: &str) -> Result<f64, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be a number")))
}

fn parse_usize(args: &[OsString], index: usize, option: &str) -> Result<usize, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an unsigned integer")))
}

fn env_flag(name: &str) -> bool {
    env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn env_list(name: &str) -> Vec<String> {
    env::var(name)
        .map(|value| {
            value
                .split(',')
                .filter_map(|item| {
                    let item = item.trim();
                    (!item.is_empty()).then(|| item.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
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

#[cfg(any(feature = "pdfium", test))]
fn format_background(color: Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        color.r, color.g, color.b, color.a
    )
}

#[cfg(any(feature = "pdfium", test))]
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
            "\"max_total_image_bytes\":{},",
            "\"max_font_program_bytes\":{},",
            "\"max_cmap_bytes\":{},",
            "\"max_text_run_bytes\":{},",
            "\"max_display_items\":{},",
            "\"max_font_fallback_cache_entries\":{},",
            "\"max_transparency_group_pixels\":{},",
            "\"max_flattened_segments\":{},",
            "\"max_pattern_tiles\":{},",
            "\"max_pattern_cell_cache_entries\":{},",
            "\"spooling_enabled\":{},",
            "\"max_spool_bytes\":{}",
            "}}"
        ),
        diagnostics.max_page_pixels,
        diagnostics.max_image_bytes,
        diagnostics.max_total_image_bytes,
        diagnostics.max_font_program_bytes,
        diagnostics.max_cmap_bytes,
        diagnostics.max_text_run_bytes,
        diagnostics.max_display_items,
        diagnostics.max_font_fallback_cache_entries,
        diagnostics.max_transparency_group_pixels,
        diagnostics.max_flattened_segments,
        diagnostics.max_pattern_tiles,
        diagnostics.max_pattern_cell_cache_entries,
        diagnostics.spooling_enabled,
        diagnostics.max_spool_bytes
    )
}

fn native_diagnostic_bundle_json(bundle: NativeDiagnosticBundle<'_>) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"backend\": \"rust-native\",\n",
            "  \"path\": {},\n",
            "  \"manifest\": {},\n",
            "  \"privacy\": {{\"includes_pdf_bytes\":false,\"includes_rendered_pixels\":false,\"includes_document_info\":false,\"redaction\":\"review path and manifest notes before sharing outside the trust boundary\"}},\n",
            "  \"options\": {},\n",
            "  \"metadata\": {},\n",
            "  \"stages\": [{{\"name\":\"metadata\",\"elapsed_ms\":{:.3},\"outcome\":{}}},{{\"name\":\"render_pipeline\",\"elapsed_ms\":{:.3},\"stage_hint\":{},\"outcome\":{}}}],\n",
            "  \"native_memory_diagnostics\": {}\n",
            "}}\n"
        ),
        json_string(bundle.path),
        manifest_entry_json(bundle.manifest),
        thumbnail_options_json(bundle.options),
        safe_metadata_json(bundle.metadata),
        bundle.metadata_ms,
        metadata_stage_outcome_json(bundle.metadata),
        bundle.render_ms,
        json_string(error_stage_hint(bundle.render_error)),
        thumbnail_error_json(bundle.render_error),
        native_memory_diagnostics_json(bundle.diagnostics)
    )
}

fn thumbnail_options_json(options: &ThumbnailOptions) -> String {
    format!(
        concat!(
            "{{",
            "\"page_index\":{},",
            "\"max_edge\":{},",
            "\"background\":[{},{},{},{}],",
            "\"output_format\":{},",
            "\"timeout_ms\":{}",
            "}}"
        ),
        options.page_index,
        options.max_edge,
        options.background.r,
        options.background.g,
        options.background.b,
        options.background.a,
        output_format_name(options),
        options.timeout.as_millis()
    )
}

fn output_format_name(options: &ThumbnailOptions) -> &'static str {
    match options.output_format {
        pdfrust_thumbnail::OutputFormat::Png => "\"png\"",
        pdfrust_thumbnail::OutputFormat::Rgba => "\"rgba\"",
    }
}

fn safe_metadata_json(metadata: Result<&DocumentMetadata, &ThumbnailError>) -> String {
    match metadata {
        Ok(metadata) => {
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
        Err(error) => format!(
            "{{\"status\":\"error\",\"error\":{}}}",
            thumbnail_error_json(error)
        ),
    }
}

fn metadata_stage_outcome_json(metadata: Result<&DocumentMetadata, &ThumbnailError>) -> String {
    match metadata {
        Ok(_) => "{\"status\":\"success\"}".to_string(),
        Err(error) => thumbnail_error_json(error),
    }
}

fn thumbnail_error_json(error: &ThumbnailError) -> String {
    let class = error.class().as_str();
    let category = if error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported {
        format!(
            ",\"category\":{}",
            json_string(FallbackReason::from_native_error(error).category())
        )
    } else {
        String::new()
    };
    format!(
        "{{\"status\":{},\"class\":{},\"message\":{}{}}}",
        json_string(
            if error.class() == pdfrust_thumbnail::ThumbnailErrorClass::Unsupported {
                "fallback_required"
            } else {
                "error"
            }
        ),
        json_string(class),
        json_string(&error.to_string()),
        category
    )
}

fn error_stage_hint(error: &ThumbnailError) -> &'static str {
    match error.class() {
        pdfrust_thumbnail::ThumbnailErrorClass::Malformed
        | pdfrust_thumbnail::ThumbnailErrorClass::Encrypted => "parser-or-object",
        pdfrust_thumbnail::ThumbnailErrorClass::Unsupported => {
            let reason = FallbackReason::from_native_error(error);
            let category = reason.category();
            if category.starts_with("renderer.memory") {
                "raster-or-memory-budget"
            } else if category.starts_with("text.") || category.starts_with("graphics.") {
                "display-list-or-raster"
            } else if category.starts_with("image.") {
                "resource-decode-or-raster"
            } else {
                "unsupported-boundary"
            }
        }
        pdfrust_thumbnail::ThumbnailErrorClass::Timeout => "timeout",
        pdfrust_thumbnail::ThumbnailErrorClass::Internal => "internal",
    }
}

fn fallback_summary_json(summary: &FallbackSummary) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"total\": {},\n",
            "  \"native_rendered\": {},\n",
            "  \"fallback_required\": {},\n",
            "  \"fallback_categories\": {},\n",
            "  \"errors\": {},\n",
            "  \"families\": {}\n",
            "}}\n"
        ),
        summary.total,
        summary.native_rendered,
        summary.fallback_required,
        count_map_json(&summary.fallback_categories),
        count_map_json(&summary.errors),
        family_summary_map_json(&summary.families)
    )
}

fn corpus_metadata_json(records: &[CorpusMetadataRecord]) -> String {
    let fixtures = records
        .iter()
        .map(corpus_metadata_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"total\": {},\n",
            "  \"fixtures\": [{}]\n",
            "}}\n"
        ),
        records.len(),
        fixtures
    )
}

fn corpus_metadata_record_json(record: &CorpusMetadataRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"manifest\":{},",
            "\"metadata\":{}",
            "}}"
        ),
        json_string(&record.path),
        manifest_entry_json(record.manifest.as_ref()),
        metadata_outcome_json(&record.metadata)
    )
}

fn local_corpus_missing_json(path: &Path) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"status\": \"missing\",\n",
            "  \"path\": {},\n",
            "  \"sample_count\": 0,\n",
            "  \"document_count\": 0\n",
            "}}\n"
        ),
        json_string(&path.to_string_lossy())
    )
}

fn local_corpus_validation_json(report: &LocalCorpusValidationReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"status\": \"valid\",\n",
            "  \"sample_count\": {},\n",
            "  \"document_count\": {},\n",
            "  \"categories\": {},\n",
            "  \"privacy\": {},\n",
            "  \"synthetic_replacements\": {}\n",
            "}}\n"
        ),
        report.sample_count,
        report.document_count,
        string_count_map_json(&report.categories),
        string_count_map_json(&report.privacy),
        report.synthetic_replacements
    )
}

fn benchmark_report_json(report: &BenchmarkReport) -> String {
    let fixtures = report
        .fixtures
        .iter()
        .map(benchmark_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"backend\": {},\n",
            "  \"platform\": {},\n",
            "  \"config\": {{\"iterations\":{},\"max_ms\":{},\"max_output_bytes\":{}}},\n",
            "  \"summary\": {{\"total\":{},\"native_rendered\":{},\"fallback_required\":{},\"errors\":{},\"budget_failures\":{}}},\n",
            "  \"families\": {},\n",
            "  \"fixtures\": [{}]\n",
            "}}\n"
        ),
        json_string(report.backend),
        platform_metadata_json(&report.platform),
        report.iterations,
        report.max_ms,
        report.max_output_bytes,
        report.total,
        report.native_rendered,
        report.fallback_required,
        report.errors,
        report.budget_failures,
        benchmark_family_map_json(&report.families),
        fixtures
    )
}

fn batch_benchmark_report_json(report: &BatchBenchmarkReport) -> String {
    let records = report
        .records
        .iter()
        .map(batch_benchmark_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"backend\": \"rust-native\",\n",
            "  \"platform\": {},\n",
            "  \"config\": {{\"repetitions\":{},\"workers\":{},\"max_p95_ms\":{},\"max_errors\":{},\"max_in_flight_pixels\":{}}},\n",
            "  \"summary\": {{\"total_inputs\":{},\"total_jobs\":{},\"native_rendered\":{},\"fallback_required\":{},\"errors\":{},\"budget_failures\":{},\"elapsed_ms\":{:.3},\"throughput_per_sec\":{:.3}}},\n",
            "  \"latency\": {},\n",
            "  \"memory\": {},\n",
            "  \"families\": {},\n",
            "  \"records\": [{}]\n",
            "}}\n"
        ),
        platform_metadata_json(&report.platform),
        report.repetitions,
        report.workers,
        report.max_p95_ms,
        report.max_errors,
        report.memory.max_in_flight_pixels,
        report.total_inputs,
        report.total_jobs,
        report.native_rendered,
        report.fallback_required,
        report.errors,
        report.budget_failures,
        report.elapsed_ms,
        report.throughput_per_sec,
        batch_latency_summary_json(&report.latency),
        batch_memory_summary_json(&report.memory),
        batch_family_map_json(&report.families),
        records
    )
}

fn batch_latency_summary_json(summary: &BatchLatencySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"mean_ms\":{:.3},",
            "\"p50_ms\":{:.3},",
            "\"p95_ms\":{:.3},",
            "\"max_ms\":{:.3}",
            "}}"
        ),
        summary.mean_ms, summary.p50_ms, summary.p95_ms, summary.max_ms
    )
}

fn batch_memory_summary_json(summary: &BatchMemorySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"rss_start_kib\":{},",
            "\"rss_high_water_kib\":{},",
            "\"rss_end_kib\":{},",
            "\"max_in_flight_pixels\":{},",
            "\"max_output_bytes\":{}",
            "}}"
        ),
        optional_json_u64(summary.rss_start_kib),
        optional_json_u64(summary.rss_high_water_kib),
        optional_json_u64(summary.rss_end_kib),
        summary.max_in_flight_pixels,
        summary.max_output_bytes
    )
}

fn batch_family_map_json(families: &BTreeMap<String, BatchFamilySummary>) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                batch_family_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn batch_family_summary_json(summary: &BatchFamilySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"native_rendered\":{},",
            "\"fallback_required\":{},",
            "\"errors\":{},",
            "\"mean_ms\":{:.3},",
            "\"max_ms\":{:.3}",
            "}}"
        ),
        summary.total,
        summary.native_rendered,
        summary.fallback_required,
        summary.errors,
        summary.mean_ms,
        summary.max_ms
    )
}

fn batch_benchmark_record_json(record: &BatchBenchmarkRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"repetition\":{},",
            "\"page_index\":{},",
            "\"elapsed_ms\":{:.3},",
            "\"outcome\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        record.repetition,
        record.page_index,
        record.elapsed_ms,
        batch_benchmark_outcome_json(&record.outcome)
    )
}

fn batch_benchmark_outcome_json(outcome: &BatchBenchmarkOutcome) -> String {
    match outcome {
        BatchBenchmarkOutcome::NativeRendered {
            width,
            height,
            output_bytes,
        } => format!(
            concat!(
                "{{",
                "\"status\":\"native_rendered\",",
                "\"width\":{},",
                "\"height\":{},",
                "\"output_bytes\":{}",
                "}}"
            ),
            width, height, output_bytes
        ),
        BatchBenchmarkOutcome::FallbackRequired { reason, message } => format!(
            concat!(
                "{{",
                "\"status\":\"fallback_required\",",
                "\"reason\":{},",
                "\"category\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_string(reason.as_str()),
            json_string(reason.category()),
            json_string(message)
        ),
        BatchBenchmarkOutcome::Error { class, message } => format!(
            concat!(
                "{{",
                "\"status\":\"error\",",
                "\"class\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_string(class),
            json_string(message)
        ),
    }
}

fn repeat_benchmark_report_json(report: &RepeatBenchmarkReport) -> String {
    let records = report
        .records
        .iter()
        .map(repeat_benchmark_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"backend\": \"rust-native\",\n",
            "  \"platform\": {},\n",
            "  \"cache_policy\": {},\n",
            "  \"config\": {{\"repetitions\":{},\"max_first_ms\":{},\"max_repeat_mean_ms\":{},\"max_errors\":{}}},\n",
            "  \"summary\": {{\"total\":{},\"native_rendered\":{},\"fallback_required\":{},\"errors\":{},\"budget_failures\":{}}},\n",
            "  \"families\": {},\n",
            "  \"records\": [{}]\n",
            "}}\n"
        ),
        platform_metadata_json(&report.platform),
        native_page_cache_policy_json(report.cache_policy),
        report.repetitions,
        report.max_first_ms,
        report.max_repeat_mean_ms,
        report.max_errors,
        report.total,
        report.native_rendered,
        report.fallback_required,
        report.errors,
        report.budget_failures,
        repeat_family_map_json(&report.families),
        records
    )
}

fn native_page_cache_policy_json(policy: NativePageCachePolicy) -> String {
    format!(
        "{{\"name\":{},\"permits_disk_persistence\":{}}}",
        json_string(policy.as_str()),
        policy.permits_disk_persistence()
    )
}

fn repeat_family_map_json(families: &BTreeMap<String, RepeatFamilySummary>) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                repeat_family_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn repeat_family_summary_json(summary: &RepeatFamilySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"native_rendered\":{},",
            "\"fallback_required\":{},",
            "\"errors\":{},",
            "\"budget_failures\":{},",
            "\"first_mean_ms\":{:.3},",
            "\"repeat_mean_ms\":{:.3}",
            "}}"
        ),
        summary.total,
        summary.native_rendered,
        summary.fallback_required,
        summary.errors,
        summary.budget_failures,
        summary.first_mean_ms,
        summary.repeat_mean_ms
    )
}

fn repeat_benchmark_record_json(record: &RepeatBenchmarkRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"page_index\":{},",
            "\"cache_key\":{},",
            "\"timings_ms\":{},",
            "\"budget_violations\":{},",
            "\"outcome\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        record.page_index,
        native_page_cache_key_json(&record.cache_key),
        float_array_json(&record.timings_ms),
        json_str_array(record.budget_violations.as_slice()),
        repeat_benchmark_outcome_json(&record.outcome)
    )
}

fn native_page_cache_key_json(cache_key: &NativePageCacheKey) -> String {
    format!(
        concat!(
            "{{",
            "\"document_identity\":\"{:016x}\",",
            "\"page_index\":{},",
            "\"max_edge\":{},",
            "\"background\":[{},{},{},{}],",
            "\"renderer_version\":{},",
            "\"native_profile\":{}",
            "}}"
        ),
        cache_key.document_identity,
        cache_key.page_index,
        cache_key.max_edge,
        cache_key.background[0],
        cache_key.background[1],
        cache_key.background[2],
        cache_key.background[3],
        json_string(cache_key.renderer_version),
        json_string(cache_key.native_profile)
    )
}

fn float_array_json(values: &[f64]) -> String {
    let values = values
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn repeat_benchmark_outcome_json(outcome: &RepeatBenchmarkOutcome) -> String {
    match outcome {
        RepeatBenchmarkOutcome::NativeRendered {
            width,
            height,
            output_bytes,
            first_ms,
            repeat_mean_ms,
            repeat_min_ms,
            repeat_max_ms,
            repeat_to_first_ratio,
        } => format!(
            concat!(
                "{{",
                "\"status\":\"native_rendered\",",
                "\"width\":{},",
                "\"height\":{},",
                "\"output_bytes\":{},",
                "\"first_ms\":{:.3},",
                "\"repeat_mean_ms\":{:.3},",
                "\"repeat_min_ms\":{:.3},",
                "\"repeat_max_ms\":{:.3},",
                "\"repeat_to_first_ratio\":{:.3}",
                "}}"
            ),
            width,
            height,
            output_bytes,
            first_ms,
            repeat_mean_ms,
            repeat_min_ms,
            repeat_max_ms,
            repeat_to_first_ratio
        ),
        RepeatBenchmarkOutcome::FallbackRequired { reason, message } => format!(
            concat!(
                "{{",
                "\"status\":\"fallback_required\",",
                "\"reason\":{},",
                "\"category\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_string(reason.as_str()),
            json_string(reason.category()),
            json_string(message)
        ),
        RepeatBenchmarkOutcome::Error { class, message } => format!(
            concat!(
                "{{",
                "\"status\":\"error\",",
                "\"class\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_string(class),
            json_string(message)
        ),
    }
}

fn platform_metadata_json(platform: &PlatformMetadata) -> String {
    format!(
        concat!(
            "{{",
            "\"os\":{},",
            "\"arch\":{},",
            "\"family\":{},",
            "\"endian\":{},",
            "\"pointer_width_bits\":{}",
            "}}"
        ),
        json_string(platform.os),
        json_string(platform.arch),
        json_string(platform.family),
        json_string(platform.endian),
        platform.pointer_width_bits
    )
}

fn benchmark_family_map_json(families: &BTreeMap<String, FamilyBenchmarkSummary>) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                benchmark_family_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn benchmark_family_summary_json(summary: &FamilyBenchmarkSummary) -> String {
    let mean_ms = if summary.native_rendered == 0 {
        0.0
    } else {
        summary.total_ms / summary.native_rendered as f64
    };
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"native_rendered\":{},",
            "\"fallback_required\":{},",
            "\"errors\":{},",
            "\"budget_failures\":{},",
            "\"mean_ms\":{:.3},",
            "\"max_ms\":{:.3},",
            "\"output_bytes\":{}",
            "}}"
        ),
        summary.total,
        summary.native_rendered,
        summary.fallback_required,
        summary.errors,
        summary.budget_failures,
        mean_ms,
        summary.max_ms,
        summary.total_output_bytes
    )
}

fn benchmark_record_json(record: &BenchmarkRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"budget_violations\":{},",
            "\"outcome\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        json_str_array(record.budget_violations.as_slice()),
        benchmark_outcome_json(&record.outcome)
    )
}

#[cfg(feature = "pdfium")]
fn visual_diff_report_json(report: &VisualDiffReport) -> String {
    let fixtures = report
        .fixtures
        .iter()
        .map(visual_diff_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"platform\": {},\n",
            "  \"thresholds\": {{\"max_mean_abs_error\":{:.3},\"max_p95_channel_delta\":{},\"max_changed_ratio\":{:.6}}},\n",
            "  \"summary\": {{\"total\":{},\"exact\":{},\"accepted_drift\":{},\"blockers\":{},\"native_errors\":{},\"pdfium_errors\":{},\"both_errors\":{}}},\n",
            "  \"families\": {},\n",
            "  \"subsystems\": {},\n",
            "  \"fixtures\": [{}]\n",
            "}}\n"
        ),
        platform_metadata_json(&report.platform),
        report.thresholds.max_mean_abs_error,
        report.thresholds.max_p95_channel_delta,
        report.thresholds.max_changed_ratio,
        report.total,
        report.exact,
        report.accepted_drift,
        report.blockers,
        report.native_errors,
        report.pdfium_errors,
        report.both_errors,
        visual_diff_family_map_json(&report.families),
        visual_diff_family_map_json(&report.subsystems),
        fixtures
    )
}

#[cfg(feature = "pdfium")]
fn visual_diff_family_map_json(families: &BTreeMap<String, FamilyVisualDiffSummary>) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                visual_diff_family_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

#[cfg(feature = "pdfium")]
fn visual_diff_family_summary_json(summary: &FamilyVisualDiffSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"exact\":{},",
            "\"accepted_drift\":{},",
            "\"blockers\":{},",
            "\"native_errors\":{},",
            "\"pdfium_errors\":{},",
            "\"both_errors\":{}",
            "}}"
        ),
        summary.total,
        summary.exact,
        summary.accepted_drift,
        summary.blockers,
        summary.native_errors,
        summary.pdfium_errors,
        summary.both_errors
    )
}

#[cfg(feature = "pdfium")]
fn visual_diff_record_json(record: &VisualDiffRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"subsystem\":{},",
            "\"status\":{},",
            "\"metrics\":{},",
            "\"native_error\":{},",
            "\"pdfium_error\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        json_string(record.subsystem),
        json_string(record.status.as_str()),
        visual_diff_metrics_json(record.metrics.as_ref()),
        visual_diff_error_json(record.native_error.as_ref()),
        visual_diff_error_json(record.pdfium_error.as_ref())
    )
}

#[cfg(feature = "pdfium")]
fn visual_diff_metrics_json(metrics: Option<&VisualDiffMetrics>) -> String {
    match metrics {
        Some(metrics) => format!(
            concat!(
                "{{",
                "\"width\":{},",
                "\"height\":{},",
                "\"changed_pixels\":{},",
                "\"changed_ratio\":{:.6},",
                "\"mean_abs_error\":{:.3},",
                "\"p95_channel_delta\":{},",
                "\"max_channel_delta\":{},",
                "\"native_nonwhite_pixels\":{},",
                "\"pdfium_nonwhite_pixels\":{}",
                "}}"
            ),
            metrics.width,
            metrics.height,
            metrics.changed_pixels,
            metrics.changed_ratio,
            metrics.mean_abs_error,
            metrics.p95_channel_delta,
            metrics.max_channel_delta,
            metrics.native_nonwhite_pixels,
            metrics.pdfium_nonwhite_pixels
        ),
        None => "null".to_string(),
    }
}

#[cfg(feature = "pdfium")]
fn visual_diff_error_json(error: Option<&VisualDiffError>) -> String {
    match error {
        Some(error) => format!(
            "{{\"class\":{},\"message\":{}}}",
            json_string(error.class),
            json_string(&error.message)
        ),
        None => "null".to_string(),
    }
}

fn benchmark_outcome_json(outcome: &BenchmarkOutcome) -> String {
    match outcome {
        BenchmarkOutcome::NativeRendered {
            width,
            height,
            output_bytes,
            mean_ms,
        } => format!(
            "{{\"status\":\"native_rendered\",\"width\":{},\"height\":{},\"output_bytes\":{},\"mean_ms\":{:.3}}}",
            width, height, output_bytes, mean_ms
        ),
        BenchmarkOutcome::FallbackRequired { reason, mean_ms } => format!(
            "{{\"status\":\"fallback_required\",\"reason\":{},\"mean_ms\":{:.3}}}",
            json_string(reason.as_str()),
            mean_ms
        ),
        BenchmarkOutcome::Error {
            class,
            message,
            mean_ms,
        } => format!(
            "{{\"status\":\"error\",\"error_class\":{},\"message\":{},\"mean_ms\":{:.3}}}",
            json_string(class),
            json_string(message),
            mean_ms
        ),
    }
}

fn manifest_entry_json(entry: Option<&CorpusManifestEntry>) -> String {
    match entry {
        Some(entry) => format!(
            concat!(
                "{{",
                "\"status\":\"matched\",",
                "\"family\":{},",
                "\"source\":{},",
                "\"license\":{},",
                "\"page_count\":{},",
                "\"features\":{},",
                "\"notes\":{}",
                "}}"
            ),
            json_string(&entry.family),
            json_string(&entry.source),
            json_string(&entry.license),
            entry.page_count,
            json_string_array(&entry.features),
            json_string(&entry.notes)
        ),
        None => "{\"status\":\"missing\"}".to_string(),
    }
}

fn count_map_json(counts: &BTreeMap<&'static str, usize>) -> String {
    let values = counts
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), value))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn string_count_map_json(counts: &BTreeMap<String, usize>) -> String {
    let values = counts
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), value))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn family_summary_map_json(families: &BTreeMap<String, FamilyFallbackSummary>) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!("{}:{}", json_string(family), family_summary_json(summary))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn family_summary_json(summary: &FamilyFallbackSummary) -> String {
    let pass_rate = if summary.total == 0 {
        0.0
    } else {
        summary.native_rendered as f64 / summary.total as f64
    };
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"native_rendered\":{},",
            "\"native_pass_rate\":{:.3},",
            "\"fallback_required\":{},",
            "\"fallback_categories\":{},",
            "\"errors\":{}",
            "}}"
        ),
        summary.total,
        summary.native_rendered,
        pass_rate,
        summary.fallback_required,
        count_map_json(&summary.fallback_categories),
        count_map_json(&summary.errors)
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
                "{{\"status\":\"success\",\"page_count\":{},\"pages\":[{}],\"info\":{},\"structure\":{},\"outlines\":{},\"page_labels\":{},\"accessibility\":{}}}",
                metadata.page_count(),
                pages,
                document_info_json(&metadata.info),
                document_structure_json(&metadata.structure),
                outline_metadata_json(&metadata.outlines),
                page_labels_metadata_json(&metadata.page_labels),
                accessibility_metadata_json(&metadata.accessibility)
            )
        }
        MetadataOutcome::Error { class, message } => format!(
            "{{\"status\":\"error\",\"error_class\":{},\"message\":{}}}",
            json_string(class),
            json_string(message)
        ),
    }
}

fn document_info_json(info: &pdfrust_thumbnail::DocumentInfo) -> String {
    format!(
        "{{\"title\":{},\"author\":{},\"subject\":{},\"keywords\":{},\"creator\":{},\"producer\":{},\"creation_date\":{},\"modification_date\":{}}}",
        optional_json_string(info.title.as_deref()),
        optional_json_string(info.author.as_deref()),
        optional_json_string(info.subject.as_deref()),
        optional_json_string(info.keywords.as_deref()),
        optional_json_string(info.creator.as_deref()),
        optional_json_string(info.producer.as_deref()),
        optional_json_string(info.creation_date.as_deref()),
        optional_json_string(info.modification_date.as_deref())
    )
}

fn document_structure_json(structure: &pdfrust_thumbnail::DocumentStructure) -> String {
    format!(
        "{{\"has_xmp_metadata\":{},\"has_mark_info\":{},\"has_struct_tree_root\":{},\"has_named_destinations\":{},\"has_signature_fields\":{},\"has_signature_byte_range\":{},\"has_embedded_files\":{},\"has_portfolio_collection\":{},\"has_file_attachment_annotations\":{}}}",
        structure.has_xmp_metadata,
        structure.has_mark_info,
        structure.has_struct_tree_root,
        structure.has_named_destinations,
        structure.has_signature_fields,
        structure.has_signature_byte_range,
        structure.has_embedded_files,
        structure.has_portfolio_collection,
        structure.has_file_attachment_annotations
    )
}

fn outline_metadata_json(outlines: &pdfrust_thumbnail::OutlineMetadata) -> String {
    format!(
        "{{\"has_outlines\":{},\"item_count\":{},\"truncated\":{}}}",
        outlines.has_outlines, outlines.item_count, outlines.truncated
    )
}

fn page_labels_metadata_json(page_labels: &pdfrust_thumbnail::PageLabelsMetadata) -> String {
    let labels = page_labels
        .labels
        .iter()
        .map(|label| {
            format!(
                "{{\"page_index\":{},\"label\":{}}}",
                label.page_index,
                json_string(&label.label)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"truncated\":{},\"labels\":[{}]}}",
        page_labels.truncated, labels
    )
}

fn accessibility_metadata_json(accessibility: &pdfrust_thumbnail::AccessibilityMetadata) -> String {
    format!(
        "{{\"language\":{},\"mark_info_marked\":{},\"has_role_map\":{},\"structure_role_count\":{},\"has_marked_content_references\":{},\"truncated\":{}}}",
        optional_json_string(accessibility.language.as_deref()),
        optional_json_bool(accessibility.mark_info_marked),
        accessibility.has_role_map,
        accessibility.structure_role_count,
        accessibility.has_marked_content_references,
        accessibility.truncated
    )
}

fn optional_json_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_string(), json_string)
}

fn optional_json_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    }
}

fn optional_json_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_str_array(values: &[&str]) -> String {
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
        "Usage: pdfrust-cli <render|render-auto|render-native|render-pdfium|render-isolated|compare-metadata|summarize-fallbacks|extract-corpus-metadata|validate-local-corpus|benchmark-native|benchmark-batch-native|benchmark-repeat-native|benchmark-pdfium|visual-diff> <input.pdf> \
         [--output PATH] [--page-index N] [--max-edge N] [--background #RRGGBB] \
         [--timeout SECONDS] [--iterations N] [--repetitions N] [--max-workers N] [--max-in-flight-pixels N] [--max-ms N] [--max-p95-ms N] [--max-first-ms N] [--max-repeat-mean-ms N] [--max-output-bytes N] \
         [--allow-pdfium-fallback] [--native-only] [--deny-fallback-reason BUCKET] [--manifest PATH] [--include-family FAMILY] \
         [--diagnostics-dir PATH] [--allow-missing] [--max-mae N] [--max-p95 N] [--max-changed-ratio N]"
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

    #[cfg(not(feature = "pdfium"))]
    #[test]
    fn pdfium_commands_should_report_disabled_in_native_only_build() {
        for command in [
            "render-pdfium",
            "render-isolated",
            "compare-metadata",
            "benchmark-pdfium",
        ] {
            let error = run(vec![OsString::from(command)])
                .expect_err("PDFium command should be disabled without feature");

            assert_eq!(
                error.to_string(),
                format!("usage error: {PDFIUM_FEATURE_MESSAGE}")
            );
        }
    }

    #[test]
    fn render_auto_command_should_use_native_for_supported_fixture() {
        env::remove_var("PDFRUST_PDFIUM_LIBRARY");
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
        env::remove_var("PDFRUST_PDFIUM_LIBRARY");
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
        env::remove_var("PDFRUST_PDFIUM_LIBRARY");
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
            fallback_policy: FallbackPolicy::default(),
        };

        let outcome = render_auto_thumbnail(&config).expect("supported fixture should render");

        assert_eq!(outcome.backend, AutoRenderBackend::Native);
    }

    #[test]
    fn render_auto_thumbnail_should_require_explicit_fallback() {
        env::remove_var("PDFRUST_PDFIUM_LIBRARY");
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/generated/optional-content-ocmd.pdf");
        let output = PathBuf::from("target/unused-default-fallback.png");
        let config = RenderConfig {
            input,
            output,
            page_index: 0,
            max_edge: 220,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
            fallback_policy: FallbackPolicy::default(),
        };

        let error = render_auto_thumbnail(&config)
            .expect_err("default auto mode should require explicit PDFium fallback");

        assert_eq!(
            error.to_string(),
            "render error [unsupported]: PDFium fallback not enabled for graphics.optional-content; pass --allow-pdfium-fallback to opt in"
        );
    }

    #[test]
    fn render_config_should_accept_explicit_pdfium_fallback_flag() {
        let config = RenderConfig::parse(&[
            OsString::from("fixtures/generated/optional-content-ocmd.pdf"),
            OsString::from("--output"),
            OsString::from("target/ocmd.png"),
            OsString::from("--allow-pdfium-fallback"),
        ])
        .expect("valid config");

        assert!(config
            .fallback_policy
            .allows(FallbackReason::NativeUnsupportedFeature(
                "graphics.optional-content"
            )));
    }

    #[test]
    fn fallback_summary_config_should_accept_family_filters() {
        let config = FallbackSummaryConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--include-family"),
            OsString::from("browser-print"),
            OsString::from("--include-family"),
            OsString::from("report"),
            OsString::from("--fail-on-fallback"),
            OsString::from("--diagnostics-dir"),
            OsString::from("target/diagnostics"),
        ])
        .expect("valid fallback summary config");

        assert_eq!(
            config.include_families,
            vec!["browser-print".to_string(), "report".to_string()]
        );
        assert!(config.fail_on_fallback);
        assert_eq!(
            config.diagnostics_dir,
            Some(PathBuf::from("target/diagnostics"))
        );
    }

    #[test]
    fn fallback_reason_should_use_native_feature_bucket() {
        let error = ThumbnailError::unsupported_feature("graphics.optional-content");

        let reason = FallbackReason::from_native_error(&error);

        assert_eq!(
            reason,
            FallbackReason::NativeUnsupportedFeature("graphics.optional-content")
        );
        assert_eq!(reason.category(), "graphics.optional-content");
    }

    #[test]
    fn fallback_summary_should_count_native_and_fallback_categories() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
        let paths = vec![
            fixture_root.join("vector-paths.pdf"),
            fixture_root.join("optional-content-ocmd.pdf"),
            fixture_root.join("encrypted-placeholder.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
        };

        let native = NativeBackend::new();
        let summary = summarize_native_fallbacks(&native, &paths, &options, None);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.native_rendered, 1);
        assert_eq!(summary.fallback_required, 1);
        assert_eq!(
            summary.fallback_categories.get("graphics.optional-content"),
            Some(&1)
        );
        assert_eq!(summary.errors.get("encrypted"), Some(&1));
        assert_eq!(
            summary
                .families
                .get("unclassified")
                .map(|family| family.total),
            Some(3)
        );
    }

    #[test]
    fn fallback_summary_should_filter_by_manifest_family() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/vector-paths.pdf"),
            fixture_root.join("fixtures/generated/optional-content-ocmd.pdf"),
        ];
        let filtered =
            filter_fixtures_by_family(&paths, Some(&manifest), &[String::from("browser-print")])
                .expect("browser-print fixture should match");
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
        };

        let native = NativeBackend::new();
        let summary = summarize_native_fallbacks(&native, &filtered, &options, Some(&manifest));

        assert_eq!(filtered.len(), 1);
        assert_eq!(summary.total, 1);
        assert_eq!(summary.native_rendered, 1);
        assert_eq!(summary.fallback_required, 0);
        assert_eq!(
            summary
                .families
                .get("browser-print")
                .map(|family| family.total),
            Some(1)
        );
        assert!(!summary.families.contains_key("presentation"));
    }

    #[test]
    fn fallback_summary_family_filter_should_require_manifest() {
        let error = filter_fixtures_by_family(
            &[PathBuf::from("fixtures/generated/vector-paths.pdf")],
            None,
            &[String::from("browser-print")],
        )
        .expect_err("family filter should require manifest");

        assert_eq!(
            error.to_string(),
            "usage error: --include-family requires --manifest"
        );
    }

    #[test]
    fn fallback_summary_json_should_emit_stable_counts() {
        let mut summary = FallbackSummary::new(2);
        summary.native_rendered = 1;
        summary.fallback_required = 1;
        summary
            .fallback_categories
            .insert("graphics.optional-content", 1);
        summary
            .families
            .entry("presentation".to_string())
            .or_default()
            .record(CorpusOutcome::FallbackRequired(
                FallbackReason::NativeUnsupportedFeature("graphics.optional-content"),
            ));

        let json = fallback_summary_json(&summary);

        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"native_rendered\": 1"));
        assert!(json.contains("\"fallback_required\": 1"));
        assert!(json.contains("\"graphics.optional-content\":1"));
        assert!(json.contains("\"presentation\""));
        assert!(json.contains("\"native_pass_rate\":0.000"));
    }

    #[test]
    fn diagnostic_bundles_should_exclude_private_bytes_and_include_typed_failure() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = vec![fixture_root.join("fixtures/generated/optional-content-ocmd.pdf")];
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let output_dir = fixture_root
            .join("target/diagnostic-bundle-test")
            .join(std::process::id().to_string());
        let _ = fs::remove_dir_all(&output_dir);
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
        };

        let written = write_native_diagnostic_bundles(
            &NativeBackend::new(),
            &paths,
            &options,
            Some(&manifest),
            &output_dir,
        )
        .expect("diagnostic bundle should write");
        let entries = fs::read_dir(&output_dir)
            .expect("diagnostic dir should exist")
            .collect::<Result<Vec<_>, _>>()
            .expect("diagnostic dir should list");
        let bundle = fs::read_to_string(entries[0].path()).expect("bundle should be readable");

        assert_eq!(written, 1);
        assert_eq!(entries.len(), 1);
        assert!(bundle.contains("\"includes_pdf_bytes\":false"));
        assert!(bundle.contains("\"includes_rendered_pixels\":false"));
        assert!(bundle.contains("\"stage_hint\":\"display-list-or-raster\""));
        assert!(bundle.contains("\"category\":\"graphics.optional-content\""));
        assert!(!bundle.contains("%PDF"));
    }

    #[test]
    fn corpus_manifest_should_assign_fixture_families() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let input = PathBuf::from("fixtures/generated/optional-content-ocmd.pdf");
        let entry = manifest
            .entry_for_path("fixtures/generated/optional-content-ocmd.pdf")
            .expect("fixture should have manifest entry");

        assert_eq!(manifest.family_for_path(&input), Some("presentation"));
        assert_eq!(entry.source, "scripts/generate_fixtures.py");
        assert_eq!(entry.license, "MIT OR Apache-2.0");
        assert_eq!(entry.page_count, 1);
        assert!(entry.features.iter().any(|feature| feature == "ocmd"));
    }

    #[test]
    fn corpus_metadata_json_should_include_manifest_and_page_size() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![fixture_root.join("fixtures/generated/tagged-accessibility-metadata.pdf")];

        let records = extract_native_corpus_metadata(&paths, Some(&manifest));
        let json = corpus_metadata_json(&records);

        assert_eq!(records.len(), 1);
        assert!(json.contains("\"family\":\"office-export\""));
        assert!(json.contains("\"source\":\"scripts/generate_fixtures.py\""));
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"width\":220.000"));
        assert!(json.contains("\"height\":140.000"));
        assert!(json.contains("\"accessibility\":"));
        assert!(json.contains("\"language\":\"en-US\""));
        assert!(json.contains("\"mark_info_marked\":true"));
        assert!(json.contains("\"structure_role_count\":1"));
        assert!(json.contains("\"has_marked_content_references\":true"));
    }

    #[test]
    fn local_corpus_metadata_should_validate_aggregate_samples() {
        let report = validate_local_corpus_metadata(
            r#"
schema_version = 1
review_date = "2026-06-25"
reviewer = "local"

[[sample]]
id = "invoice-export-private"
category = "invoice"
privacy = "private"
permission = "local-review-only"
redaction_state = "reduced-to-fixture"
source_note = "internal invoice export"
count = 3
page_count_range = "2-10"
features = ["text", "tables", "embedded-fonts"]
synthetic_replacement = "fixtures/generated/office-table.pdf"
status = "reviewed"
"#,
        )
        .expect("aggregate local corpus metadata should validate");

        assert_eq!(report.sample_count, 1);
        assert_eq!(report.document_count, 3);
        assert_eq!(report.categories.get("invoice"), Some(&3));
        assert_eq!(report.privacy.get("private"), Some(&3));
        assert_eq!(report.synthetic_replacements, 1);
    }

    #[test]
    fn local_corpus_validation_should_allow_missing_when_requested() {
        let config = LocalCorpusValidationConfig::parse(&[
            OsString::from("fixtures/local-corpus/metadata.toml"),
            OsString::from("--allow-missing"),
        ])
        .expect("valid config");
        let json = local_corpus_missing_json(&config.input);

        assert!(config.allow_missing);
        assert!(json.contains("\"status\": \"missing\""));
        assert!(json.contains("\"document_count\": 0"));
    }

    #[test]
    fn local_corpus_metadata_should_reject_private_path_fields() {
        let error = validate_local_corpus_metadata(
            r#"
schema_version = 1

[[sample]]
id = "invoice-export-private"
category = "invoice"
privacy = "private"
permission = "local-review-only"
redaction_state = "not-shareable"
source_note = "internal invoice export"
count = 1
page_count_range = "1"
features = ["text"]
path = "fixtures/local-corpus/customer.pdf"
status = "candidate"
"#,
        )
        .expect_err("private path fields should be rejected");

        assert!(error
            .to_string()
            .contains("key `path` is disallowed for private safety"));
    }

    #[test]
    fn benchmark_config_should_apply_smoke_defaults() {
        let config = BenchmarkConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--output"),
            OsString::from("target/benchmark.json"),
        ])
        .expect("valid benchmark config");

        assert_eq!(config.input, PathBuf::from("fixtures/generated"));
        assert_eq!(config.max_edge, 160);
        assert_eq!(config.iterations, 1);
        assert_eq!(config.max_ms, 250);
        assert_eq!(config.max_output_bytes, 4 * 160 * 160);
        assert_eq!(
            config.manifest,
            Some(PathBuf::from("fixtures/corpus-manifest.tsv"))
        );
        assert!(config.include_families.is_empty());
        assert_eq!(config.native_profile, NativeProfile::Default);
    }

    #[test]
    fn benchmark_config_should_accept_family_filters() {
        let config = BenchmarkConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--include-family"),
            OsString::from("invoice"),
            OsString::from("--include-family"),
            OsString::from("statement"),
        ])
        .expect("valid benchmark config");

        assert_eq!(
            config.include_families,
            vec!["invoice".to_string(), "statement".to_string()]
        );
    }

    #[test]
    fn benchmark_config_should_accept_low_memory_native_profile() {
        let config = BenchmarkConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--native-profile"),
            OsString::from("low-memory"),
        ])
        .expect("valid benchmark config");

        assert_eq!(config.native_profile, NativeProfile::LowMemory);
    }

    #[test]
    fn batch_benchmark_config_should_apply_defaults_and_worker_budget() {
        let config = BatchBenchmarkConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--repetitions"),
            OsString::from("3"),
            OsString::from("--max-workers"),
            OsString::from("4"),
            OsString::from("--max-in-flight-pixels"),
            OsString::from("25600"),
        ])
        .expect("valid batch benchmark config");
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 160,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
        };

        assert_eq!(config.repetitions, 3);
        assert_eq!(config.max_workers, 4);
        assert_eq!(
            effective_batch_workers(&config, &options).expect("workers"),
            1
        );
        assert_eq!(config.native_profile, NativeProfile::Default);
    }

    #[test]
    fn batch_benchmark_should_report_throughput_latency_memory_and_typed_errors() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/text-page.pdf"),
            fixture_root.join("fixtures/generated/optional-content-ocmd.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
        };
        let config = BatchBenchmarkConfig {
            input: fixture_root.join("fixtures/generated"),
            manifest: Some(manifest_path),
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
            repetitions: 2,
            max_workers: 2,
            max_in_flight_pixels: 120 * 120 * 2,
            max_p95_ms: 60_000,
            max_errors: 2,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
        };

        let report = benchmark_native_batch(&paths, &options, Some(&manifest), &config)
            .expect("batch benchmark should run");
        let json = batch_benchmark_report_json(&report);

        assert_eq!(report.total_inputs, 2);
        assert_eq!(report.total_jobs, 4);
        assert_eq!(report.native_rendered, 2);
        assert_eq!(report.fallback_required, 2);
        assert_eq!(report.errors, 0);
        assert_eq!(report.budget_failures, 0);
        assert_eq!(report.workers, 2);
        assert!(report.throughput_per_sec > 0.0);
        assert!(report.latency.p95_ms >= report.latency.p50_ms);
        assert!(report.memory.max_output_bytes > 0);
        assert!(json.contains("\"throughput_per_sec\""));
        assert!(json.contains("\"latency\""));
        assert!(json.contains("\"memory\""));
        assert!(json.contains("\"page_index\":0"));
        assert!(json.contains("\"status\":\"fallback_required\""));
        assert!(json.contains("\"category\":\"graphics.optional-content\""));
    }

    #[test]
    fn repeat_benchmark_config_should_apply_defaults_and_budgets() {
        let config = RepeatBenchmarkConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--repetitions"),
            OsString::from("4"),
            OsString::from("--max-first-ms"),
            OsString::from("900"),
            OsString::from("--max-repeat-mean-ms"),
            OsString::from("800"),
            OsString::from("--native-profile"),
            OsString::from("low-memory"),
        ])
        .expect("valid repeated benchmark config");

        assert_eq!(config.input, PathBuf::from("fixtures/generated"));
        assert_eq!(config.repetitions, 4);
        assert_eq!(config.max_first_ms, 900);
        assert_eq!(config.max_repeat_mean_ms, 800);
        assert_eq!(config.native_profile, NativeProfile::LowMemory);
    }

    #[test]
    fn repeat_benchmark_should_report_cache_policy_keys_and_repeated_timings() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/text-page.pdf"),
            fixture_root.join("fixtures/generated/vector-paths.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
        };
        let config = RepeatBenchmarkConfig {
            input: fixture_root.join("fixtures/generated"),
            manifest: Some(manifest_path),
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
            repetitions: 3,
            max_first_ms: 60_000,
            max_repeat_mean_ms: 60_000,
            max_errors: 0,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
        };

        let report = benchmark_native_repeat(&paths, &options, Some(&manifest), &config)
            .expect("repeated benchmark should run");
        let json = repeat_benchmark_report_json(&report);

        assert_eq!(report.total, 2);
        assert_eq!(report.native_rendered, 2);
        assert_eq!(report.fallback_required, 0);
        assert_eq!(report.errors, 0);
        assert_eq!(report.records[0].timings_ms.len(), 3);
        assert_ne!(
            report.records[0].cache_key.document_identity,
            report.records[1].cache_key.document_identity
        );
        assert!(json.contains("\"name\":\"isolated-render\""));
        assert!(json.contains("\"cache_key\""));
        assert!(json.contains("\"repeat_mean_ms\""));
    }

    #[test]
    fn benchmark_native_should_group_results_and_budget_failures() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/corpus-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/vector-paths.pdf"),
            fixture_root.join("fixtures/generated/optional-content-ocmd.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: pdfrust_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
        };
        let config = BenchmarkConfig {
            input: fixture_root.join("fixtures/generated"),
            manifest: Some(manifest_path),
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
            iterations: 1,
            max_ms: 60_000,
            max_output_bytes: 1,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
        };

        let native = NativeBackend::new();
        let report = benchmark_backend(
            &native,
            "rust-native",
            &paths,
            &options,
            Some(&manifest),
            &config,
            true,
        );
        let json = benchmark_report_json(&report);

        assert_eq!(report.total, 2);
        assert_eq!(report.native_rendered, 1);
        assert_eq!(report.fallback_required, 1);
        assert_eq!(report.budget_failures, 2);
        assert_eq!(report.platform, PlatformMetadata::current());
        assert!(json.contains("\"backend\": \"rust-native\""));
        assert!(json.contains("\"platform\": {"));
        assert!(json.contains("\"pointer_width_bits\":"));
        assert!(json.contains("\"family\":\"browser-print\""));
        assert!(json.contains("\"family\":\"presentation\""));
        assert!(json.contains("\"output_bytes\""));
        assert!(json.contains("\"native_fallback\""));
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
            MetadataOutcome::Success(Box::new(metadata.clone())),
            MetadataOutcome::Success(Box::new(metadata)),
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
            MetadataOutcome::Success(Box::new(pdfium)),
            MetadataOutcome::Success(Box::new(native)),
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
        let mut metadata = DocumentMetadata::new(vec![PageMetadata {
            index: 0,
            size: PageSize {
                width: 300.0,
                height: 160.0,
            },
        }]);
        metadata.info.title = Some("Metadata Fixture".to_string());
        metadata.structure.has_xmp_metadata = true;
        metadata.outlines = pdfrust_thumbnail::OutlineMetadata {
            has_outlines: true,
            item_count: 2,
            truncated: false,
        };
        metadata
            .page_labels
            .labels
            .push(pdfrust_thumbnail::PageLabel {
                page_index: 0,
                label: "A-1".to_string(),
            });
        let comparison = compare_metadata_results(
            MetadataOutcome::Success(Box::new(metadata.clone())),
            MetadataOutcome::Success(Box::new(metadata)),
        );

        let json = comparison_json(Path::new("fixtures/generated/text-page.pdf"), &comparison);

        assert!(json.contains("\"status\": \"match\""));
        assert!(json.contains("\"page_count\":1"));
        assert!(json.contains("\"title\":\"Metadata Fixture\""));
        assert!(json.contains("\"has_xmp_metadata\":true"));
        assert!(json.contains("\"item_count\":2"));
        assert!(json.contains("\"label\":\"A-1\""));
        assert!(json.contains("\"rust_native_memory\""));
        assert!(json.contains("\"max_page_pixels\":16777216"));
        assert!(json.contains("\"max_total_image_bytes\":134217728"));
        assert!(json.contains("\"spooling_enabled\":false"));
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
    fn visual_diff_metrics_should_classify_exact_and_drift() {
        let exact_a = Thumbnail::rgba(1, 1, vec![10, 20, 30, 255]).expect("valid thumbnail");
        let exact_b = Thumbnail::rgba(1, 1, vec![10, 20, 30, 255]).expect("valid thumbnail");
        let exact_metrics = visual_diff_metrics(&exact_a, &exact_b).expect("same dimensions");

        assert_eq!(
            classify_visual_diff(&exact_metrics, VisualDiffThresholds::default()),
            VisualDiffStatus::Exact
        );

        let drift = Thumbnail::rgba(1, 1, vec![11, 21, 30, 255]).expect("valid thumbnail");
        let drift_metrics = visual_diff_metrics(&exact_a, &drift).expect("same dimensions");

        assert_eq!(drift_metrics.changed_pixels, 1);
        assert_eq!(drift_metrics.max_channel_delta, 1);
        assert_eq!(
            classify_visual_diff(
                &drift_metrics,
                VisualDiffThresholds {
                    max_changed_ratio: 1.0,
                    ..VisualDiffThresholds::default()
                },
            ),
            VisualDiffStatus::AcceptedDrift
        );
        assert_eq!(
            classify_visual_diff(
                &drift_metrics,
                VisualDiffThresholds {
                    max_mean_abs_error: 0.0,
                    max_p95_channel_delta: 0,
                    max_changed_ratio: 0.0,
                },
            ),
            VisualDiffStatus::Blocker
        );
    }

    #[test]
    fn visual_diff_metrics_should_accept_low_amplitude_field_drift() {
        let metrics = VisualDiffMetrics {
            width: 120,
            height: 120,
            changed_pixels: 14_400,
            changed_ratio: 1.0,
            mean_abs_error: 1.25,
            p95_channel_delta: 4,
            max_channel_delta: 5,
            native_nonwhite_pixels: 14_400,
            pdfium_nonwhite_pixels: 14_400,
        };

        assert_eq!(
            classify_visual_diff(&metrics, VisualDiffThresholds::default()),
            VisualDiffStatus::AcceptedDrift
        );

        let high_delta = VisualDiffMetrics {
            max_channel_delta: 64,
            ..metrics
        };

        assert_eq!(
            classify_visual_diff(&high_delta, VisualDiffThresholds::default()),
            VisualDiffStatus::Blocker
        );
    }

    #[test]
    fn visual_diff_subsystem_should_group_common_renderer_areas() {
        assert_eq!(
            visual_diff_subsystem("fixtures/generated/acroform-text-field.pdf", "form"),
            "annotations-forms"
        );
        assert_eq!(
            visual_diff_subsystem("fixtures/generated/shaped-rtl-text.pdf", "unclassified"),
            "text-fonts"
        );
        assert_eq!(
            visual_diff_subsystem("fixtures/generated/cmyk-image.pdf", "unclassified"),
            "images-color"
        );
        assert_eq!(
            visual_diff_subsystem("fixtures/generated/vector-stress.pdf", "report"),
            "vector-graphics"
        );
        assert_eq!(
            visual_diff_subsystem(
                "fixtures/generated/encrypted-placeholder.pdf",
                "secure-document",
            ),
            "document-security"
        );
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
