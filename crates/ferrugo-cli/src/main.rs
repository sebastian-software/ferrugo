#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use ferrugo_native::{
    scan_operator_coverage, NativeBackend, NativeDocumentSessionStats, NativeMemoryDiagnostics,
    NativePageCacheKey, NativePageCachePolicy, NativeRenderPhaseTimings, NativeRenderTrace,
    OperatorCoverageEntry, OperatorCoverageOptions, OperatorSupportStatus, StrokeShapeSummary,
};
#[cfg(feature = "pdfium")]
use ferrugo_pdfium::PdfiumBackend;
#[cfg(any(feature = "pdfium", test))]
use ferrugo_thumbnail::PageSize;
use ferrugo_thumbnail::{
    AnnotationMode, DocumentMetadata, DocumentMetadataBackend, PdfSource, Rgba, ThumbnailBackend,
    ThumbnailError, ThumbnailOptions, DEFAULT_MAX_EDGE, DEFAULT_PAGE_INDEX, DEFAULT_TIMEOUT,
};

const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);
const LOW_AMPLITUDE_VISUAL_DRIFT_MAX_DELTA: u8 = 8;
const LOW_AMPLITUDE_VISUAL_DRIFT_P95_MAX_DELTA: u8 = 4;
const LOW_P95_EDGE_DRIFT_MAX_MAE: f64 = 3.5;
const LOW_P95_EDGE_DRIFT_MAX_DELTA: u8 = 5;
const LOW_P95_EDGE_DRIFT_MAX_CHANGED_RATIO: f64 = 0.5;
#[cfg(not(feature = "pdfium"))]
const PDFIUM_FEATURE_MESSAGE: &str =
    "PDFium support is disabled; rebuild ferrugo-cli with --features pdfium";
const PDFIUM_RUNTIME_FALLBACK_REMOVED_MESSAGE: &str =
    "PDFium runtime fallback has been removed from render/render-auto; use render-pdfium or maintainer comparison commands with --features pdfium";
#[cfg(feature = "pdfium")]
const PDFIUM_RENDER_WORKER_ENV: &str = "FERRUGO_PDFIUM_RENDER_WORKER";
const DEFAULT_TRACE_MAX_EVENTS: usize = 256;
const TRACE_MAX_EVENTS_LIMIT: usize = 4096;

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
        Some("render-pdfium") => render_direct_command(&args[1..]),
        Some("render-worker") => render_worker_command(&args[1..]),
        Some("render-native") => render_native_command(&args[1..]),
        Some("render-isolated") => render_isolated_command(&args[1..]),
        Some("compare-metadata") => compare_metadata_command(&args[1..]),
        Some("summarize-fallbacks") => summarize_fallbacks_command(&args[1..]),
        Some("operator-coverage") => operator_coverage_command(&args[1..]),
        Some("trace-native") => trace_native_command(&args[1..]),
        Some("replay-operators") => replay_operators_command(&args[1..]),
        Some("extract-corpus-metadata") => extract_corpus_metadata_command(&args[1..]),
        Some("producer-regression-report") => producer_regression_report_command(&args[1..]),
        Some("classify-pdf20-usage") => classify_pdf20_usage_command(&args[1..]),
        Some("validate-local-corpus") => validate_local_corpus_command(&args[1..]),
        Some("benchmark-native") => benchmark_native_command(&args[1..]),
        Some("benchmark-batch-native") => benchmark_batch_native_command(&args[1..]),
        Some("benchmark-repeat-native") => benchmark_repeat_native_command(&args[1..]),
        Some("benchmark-pdfium") => benchmark_pdfium_command(&args[1..]),
        Some("benchmark-matrix") => benchmark_matrix_command(&args[1..]),
        Some("visual-diff") => visual_diff_command(&args[1..]),
        Some("visual-diff-poppler") => visual_diff_poppler_command(&args[1..]),
        Some("--version" | "-V") => {
            println!("ferrugo-cli {}", env!("CARGO_PKG_VERSION"));
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

fn render_worker_command(args: &[OsString]) -> Result<(), CliError> {
    #[cfg(not(feature = "pdfium"))]
    {
        let _ = args;
        return Err(pdfium_feature_disabled());
    }

    #[cfg(feature = "pdfium")]
    {
        if env::var_os(PDFIUM_RENDER_WORKER_ENV).is_none() {
            return Err(CliError::Usage(
                "render-worker is private maintainer tooling; use render-isolated".to_string(),
            ));
        }
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
        Err(err) if err.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported => {
            let reason = FallbackReason::from_native_error(&err);
            Err(CliError::Render {
                class: err.class().as_str(),
                message: format!(
                    "{} for {}; {PDFIUM_RUNTIME_FALLBACK_REMOVED_MESSAGE}",
                    err,
                    reason.as_str()
                ),
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
}

impl fmt::Display for AutoRenderBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native => f.write_str("native"),
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

#[derive(Debug, PartialEq, Eq)]
struct AutoRenderOutcome {
    thumbnail: ferrugo_thumbnail::Thumbnail,
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
        output_format: ferrugo_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
        annotation_mode: config.annotation_mode,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    }
}

fn parse_annotation_mode(value: &str) -> Result<AnnotationMode, CliError> {
    match value {
        "screen" => Ok(AnnotationMode::Screen),
        "print" => Ok(AnnotationMode::Print),
        _ => Err(CliError::Usage(format!(
            "invalid --annotation-mode `{value}`; expected screen or print"
        ))),
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
        output_format: ferrugo_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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

fn operator_coverage_command(args: &[OsString]) -> Result<(), CliError> {
    let config = OperatorCoverageConfig::parse(args)?;
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let report = scan_operator_coverage_corpus(&fixtures, manifest.as_ref(), &config);
    let json = operator_coverage_report_json(&report);

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

fn trace_native_command(args: &[OsString]) -> Result<(), CliError> {
    let config = TraceNativeConfig::parse(args)?;
    let json = native_render_trace_json(&config)?;
    write_optional_json(config.output.as_deref(), &json)
}

fn replay_operators_command(args: &[OsString]) -> Result<(), CliError> {
    let config = ReplayOperatorsConfig::parse(args)?;
    let trace = fs::read_to_string(&config.input).map_err(|source| CliError::ReadFile {
        path: config.input.clone(),
        source,
    })?;
    let json = replay_operator_trace_json(&trace)?;
    write_optional_json(config.output.as_deref(), &json)
}

fn write_optional_json(output: Option<&Path>, json: &str) -> Result<(), CliError> {
    if let Some(output) = output {
        fs::write(output, json).map_err(|source| CliError::Io {
            path: output.to_path_buf(),
            source,
        })?;
    } else {
        println!("{json}");
    }
    Ok(())
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

fn producer_regression_report_command(args: &[OsString]) -> Result<(), CliError> {
    let config = ProducerRegressionConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: ferrugo_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = read_corpus_manifest(&config.manifest)?;
    let fixtures = filter_fixtures_by_manifest(&fixtures, &manifest)?;
    let fixtures = filter_fixtures_by_family(&fixtures, Some(&manifest), &config.include_families)?;
    let native = config.native_profile.backend();
    let report = build_producer_regression_report(&native, &fixtures, &options, &manifest);
    let json = producer_regression_report_json(&report);

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

fn classify_pdf20_usage_command(args: &[OsString]) -> Result<(), CliError> {
    let config = Pdf20UsageConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: ferrugo_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let report = classify_pdf20_usage(&fixtures, manifest.as_ref(), &options)?;
    let json = pdf20_usage_report_json(&report);

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
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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

fn benchmark_matrix_command(args: &[OsString]) -> Result<(), CliError> {
    let config = BenchmarkMatrixConfig::parse(args)?;
    let output_path = config.output.clone();
    let markdown_path = config.markdown_report.clone();
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: ferrugo_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures = match manifest.as_ref() {
        Some(manifest) => filter_fixtures_by_manifest(&fixtures, manifest)?,
        None => fixtures,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    fs::create_dir_all(&config.artifact_dir).map_err(|source| CliError::Io {
        path: config.artifact_dir.clone(),
        source,
    })?;

    let records = benchmark_matrix_records(&fixtures, manifest.as_ref(), &options, &config)?;
    let report = benchmark_matrix_report(config, records);
    let json = benchmark_matrix_report_json(&report);

    if let Some(output) = output_path {
        fs::write(&output, &json).map_err(|source| CliError::Io {
            path: output,
            source,
        })?;
    } else {
        println!("{json}");
    }

    if let Some(markdown_path) = markdown_path {
        let markdown = benchmark_matrix_markdown_report(&report);
        fs::write(&markdown_path, markdown).map_err(|source| CliError::Io {
            path: markdown_path,
            source,
        })?;
    }

    Ok(())
}

fn benchmark_matrix_records(
    fixtures: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    config: &BenchmarkMatrixConfig,
) -> Result<Vec<BenchmarkMatrixRecord>, CliError> {
    let mut records = Vec::new();
    for backend in &config.backends {
        for mode in &config.modes {
            match (*backend, *mode) {
                (MatrixBackend::Native, MatrixMode::ColdProcess) => {
                    for fixture in fixtures {
                        records.push(benchmark_matrix_native_cold(
                            fixture, manifest, options, config,
                        )?);
                    }
                }
                (MatrixBackend::Native, MatrixMode::HotRender) => {
                    let native = config.native_profile.backend();
                    for fixture in fixtures {
                        records.push(benchmark_matrix_hot_backend(
                            &native,
                            MatrixBackend::Native,
                            native_backend_version(config.native_profile),
                            MatrixFixtureContext {
                                fixture,
                                manifest,
                                options,
                            },
                            config,
                            true,
                        ));
                    }
                }
                (MatrixBackend::Pdfium, MatrixMode::ColdProcess) => {
                    if pdfium_matrix_available() {
                        for fixture in fixtures {
                            records.push(benchmark_matrix_pdfium_cold(
                                fixture, manifest, options, config,
                            )?);
                        }
                    } else {
                        records.extend(missing_tool_records(
                            MatrixBackend::Pdfium,
                            MatrixMode::ColdProcess,
                            fixtures,
                            manifest,
                            options,
                            pdfium_missing_message(),
                        ));
                    }
                }
                (MatrixBackend::Pdfium, MatrixMode::HotRender) => {
                    append_pdfium_hot_records(&mut records, fixtures, manifest, options, config);
                }
                (MatrixBackend::Poppler, MatrixMode::ColdProcess) => {
                    if resolve_command_path(&config.pdftoppm).is_some() {
                        for fixture in fixtures {
                            records.push(benchmark_matrix_poppler_cold(
                                fixture, manifest, options, config,
                            )?);
                        }
                    } else {
                        records.extend(missing_tool_records(
                            MatrixBackend::Poppler,
                            MatrixMode::ColdProcess,
                            fixtures,
                            manifest,
                            options,
                            format!(
                                "`{}` was not found; set --pdftoppm or FERRUGO_POPPLER_PDFTOPPM",
                                config.pdftoppm.display()
                            ),
                        ));
                    }
                }
                (MatrixBackend::Poppler, MatrixMode::HotRender) => {
                    records.extend(not_applicable_records(
                        MatrixBackend::Poppler,
                        MatrixMode::HotRender,
                        fixtures,
                        manifest,
                        options,
                        "Poppler is measured as an external process only in this matrix",
                    ));
                }
            }
        }
    }
    Ok(records)
}

fn benchmark_matrix_native_cold(
    fixture: &Path,
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    config: &BenchmarkMatrixConfig,
) -> Result<BenchmarkMatrixRecord, CliError> {
    let executable = env::current_exe().map_err(|source| {
        CliError::Process(format!("failed to locate current executable: {source}"))
    })?;
    let artifact = matrix_artifact_path(
        &config.artifact_dir,
        MatrixBackend::Native,
        MatrixMode::ColdProcess,
        fixture,
        "png",
    );
    let _ = fs::remove_file(&artifact);
    let args = renderer_process_args("render-native", fixture, options, &artifact);
    let measurement = run_measured_process(&executable, &args, &[], options.timeout)?;
    Ok(cold_process_record(
        MatrixBackend::Native,
        native_backend_version(config.native_profile),
        command_line(&executable, &args),
        MatrixFixtureContext {
            fixture,
            manifest,
            options,
        },
        artifact.as_path(),
        measurement,
        true,
    ))
}

fn benchmark_matrix_pdfium_cold(
    fixture: &Path,
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    config: &BenchmarkMatrixConfig,
) -> Result<BenchmarkMatrixRecord, CliError> {
    let executable = env::current_exe().map_err(|source| {
        CliError::Process(format!("failed to locate current executable: {source}"))
    })?;
    let artifact = matrix_artifact_path(
        &config.artifact_dir,
        MatrixBackend::Pdfium,
        MatrixMode::ColdProcess,
        fixture,
        "png",
    );
    let _ = fs::remove_file(&artifact);
    let args = renderer_process_args("render-pdfium", fixture, options, &artifact);
    let measurement = run_measured_process(&executable, &args, &[], options.timeout)?;
    Ok(cold_process_record(
        MatrixBackend::Pdfium,
        pdfium_backend_version(),
        command_line(&executable, &args),
        MatrixFixtureContext {
            fixture,
            manifest,
            options,
        },
        artifact.as_path(),
        measurement,
        false,
    ))
}

fn benchmark_matrix_poppler_cold(
    fixture: &Path,
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    config: &BenchmarkMatrixConfig,
) -> Result<BenchmarkMatrixRecord, CliError> {
    let artifact = matrix_artifact_path(
        &config.artifact_dir,
        MatrixBackend::Poppler,
        MatrixMode::ColdProcess,
        fixture,
        "ppm",
    );
    let mut artifact_prefix = artifact.clone();
    artifact_prefix.set_extension("");
    let _ = fs::remove_file(&artifact);
    let page_number = options.page_index.saturating_add(1).to_string();
    let max_edge = options.max_edge.to_string();
    let args = vec![
        OsString::from("-q"),
        OsString::from("-cropbox"),
        OsString::from("-f"),
        OsString::from(page_number.as_str()),
        OsString::from("-l"),
        OsString::from(page_number.as_str()),
        OsString::from("-singlefile"),
        OsString::from("-scale-to"),
        OsString::from(max_edge.as_str()),
        fixture.as_os_str().to_os_string(),
        artifact_prefix.as_os_str().to_os_string(),
    ];
    let cache_dir = env::temp_dir().join(format!("ferrugo-matrix-poppler-{}", std::process::id()));
    fs::create_dir_all(&cache_dir).map_err(|source| CliError::Io {
        path: cache_dir.clone(),
        source,
    })?;
    let mut envs = vec![
        (OsString::from("HOME"), cache_dir.as_os_str().to_os_string()),
        (
            OsString::from("XDG_CACHE_HOME"),
            cache_dir.as_os_str().to_os_string(),
        ),
    ];
    if let Some(fontconfig_file) = poppler_fontconfig_file(&config.pdftoppm) {
        envs.push((
            OsString::from("FONTCONFIG_FILE"),
            fontconfig_file.as_os_str().to_os_string(),
        ));
    }
    let measurement = run_measured_process(&config.pdftoppm, &args, &envs, options.timeout)?;
    Ok(cold_process_record(
        MatrixBackend::Poppler,
        poppler_backend_version(&config.pdftoppm),
        command_line(&config.pdftoppm, &args),
        MatrixFixtureContext {
            fixture,
            manifest,
            options,
        },
        artifact.as_path(),
        measurement,
        false,
    ))
}

fn renderer_process_args(
    command: &str,
    fixture: &Path,
    options: &ThumbnailOptions,
    output: &Path,
) -> Vec<OsString> {
    vec![
        OsString::from(command),
        fixture.as_os_str().to_os_string(),
        OsString::from("--page-index"),
        OsString::from(options.page_index.to_string()),
        OsString::from("--max-edge"),
        OsString::from(options.max_edge.to_string()),
        OsString::from("--background"),
        OsString::from(background_hex(options.background)),
        OsString::from("--timeout"),
        OsString::from(options.timeout.as_secs().to_string()),
        OsString::from("--output"),
        output.as_os_str().to_os_string(),
    ]
}

fn cold_process_record(
    backend: MatrixBackend,
    backend_version: String,
    command: String,
    context: MatrixFixtureContext<'_>,
    output_path: &Path,
    measurement: ProcessMeasurement,
    unsupported_is_fallback: bool,
) -> BenchmarkMatrixRecord {
    let output = if measurement.success {
        matrix_output_from_path(output_path)
    } else {
        MatrixOutput::default()
    };
    let (status, error_class, error_message, fallback_bucket) = if measurement.success {
        (MatrixStatus::Rendered, None, None, None)
    } else {
        matrix_process_failure_outcome(
            &measurement.stderr,
            unsupported_is_fallback,
            measurement.exit_status,
        )
    };
    BenchmarkMatrixRecord {
        backend,
        backend_version,
        command,
        mode: MatrixMode::ColdProcess,
        fixture: normalize_manifest_path(context.fixture),
        family: matrix_family(context.fixture, context.manifest),
        page_index: context.options.page_index,
        status,
        exit_status: measurement.exit_status,
        timing: MatrixTiming {
            wall_ms: Some(measurement.wall_ms),
            warmup_iterations: 0,
            measured_iterations: 1,
            samples_ms: vec![measurement.wall_ms],
            mean_ms: Some(measurement.wall_ms),
            p50_ms: Some(measurement.wall_ms),
            p95_ms: Some(measurement.wall_ms),
            max_ms: Some(measurement.wall_ms),
        },
        output,
        memory: MatrixMemory {
            rss_start_bytes: None,
            rss_peak_bytes: measurement.peak_rss_bytes,
            rss_end_bytes: None,
            source: measurement.memory_source,
        },
        error_class,
        error_message,
        fallback_bucket,
    }
}

fn benchmark_matrix_hot_backend<B: ThumbnailBackend>(
    backend: &B,
    matrix_backend: MatrixBackend,
    backend_version: String,
    context: MatrixFixtureContext<'_>,
    config: &BenchmarkMatrixConfig,
    unsupported_is_fallback: bool,
) -> BenchmarkMatrixRecord {
    let mut last_thumbnail = None;
    for _ in 0..config.warmup {
        match backend.render(PdfSource::from_path(context.fixture), context.options) {
            Ok(thumbnail) => last_thumbnail = Some(thumbnail),
            Err(error) => {
                return hot_error_record(
                    matrix_backend,
                    backend_version,
                    context,
                    config,
                    error,
                    unsupported_is_fallback,
                );
            }
        }
    }

    let rss_start_bytes = current_rss_kib().map(kib_to_bytes);
    let mut samples = Vec::with_capacity(config.iterations);
    for _ in 0..config.iterations {
        let started = Instant::now();
        match backend.render(PdfSource::from_path(context.fixture), context.options) {
            Ok(thumbnail) => {
                samples.push(elapsed_ms(started.elapsed()));
                last_thumbnail = Some(thumbnail);
            }
            Err(error) => {
                return hot_error_record(
                    matrix_backend,
                    backend_version,
                    context,
                    config,
                    error,
                    unsupported_is_fallback,
                );
            }
        }
    }
    let rss_end_bytes = current_rss_kib().map(kib_to_bytes);
    let thumbnail = last_thumbnail.expect("iterations is validated as non-zero");
    let timing = matrix_timing_from_samples(config.warmup, samples);
    BenchmarkMatrixRecord {
        backend: matrix_backend,
        backend_version,
        command: format!("in-process {}", matrix_backend.as_str()),
        mode: MatrixMode::HotRender,
        fixture: normalize_manifest_path(context.fixture),
        family: matrix_family(context.fixture, context.manifest),
        page_index: context.options.page_index,
        status: MatrixStatus::Rendered,
        exit_status: None,
        timing,
        output: MatrixOutput {
            width: Some(thumbnail.width),
            height: Some(thumbnail.height),
            bytes: Some(thumbnail.bytes.len() as u64),
        },
        memory: MatrixMemory {
            rss_start_bytes,
            rss_peak_bytes: max_optional_u64(rss_start_bytes, rss_end_bytes),
            rss_end_bytes,
            source: "process-rss-sample",
        },
        error_class: None,
        error_message: None,
        fallback_bucket: None,
    }
}

fn hot_error_record(
    backend: MatrixBackend,
    backend_version: String,
    context: MatrixFixtureContext<'_>,
    config: &BenchmarkMatrixConfig,
    error: ThumbnailError,
    unsupported_is_fallback: bool,
) -> BenchmarkMatrixRecord {
    let fallback_reason = (unsupported_is_fallback
        && error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported)
        .then(|| FallbackReason::from_native_error(&error));
    BenchmarkMatrixRecord {
        backend,
        backend_version,
        command: format!("in-process {}", backend.as_str()),
        mode: MatrixMode::HotRender,
        fixture: normalize_manifest_path(context.fixture),
        family: matrix_family(context.fixture, context.manifest),
        page_index: context.options.page_index,
        status: if fallback_reason.is_some() {
            MatrixStatus::FallbackRequired
        } else {
            MatrixStatus::Error
        },
        exit_status: None,
        timing: MatrixTiming {
            wall_ms: None,
            warmup_iterations: config.warmup,
            measured_iterations: config.iterations,
            samples_ms: Vec::new(),
            mean_ms: None,
            p50_ms: None,
            p95_ms: None,
            max_ms: None,
        },
        output: MatrixOutput::default(),
        memory: MatrixMemory {
            rss_start_bytes: current_rss_kib().map(kib_to_bytes),
            rss_peak_bytes: None,
            rss_end_bytes: current_rss_kib().map(kib_to_bytes),
            source: "process-rss-sample",
        },
        error_class: Some(error.class().as_str().to_string()),
        error_message: Some(error.to_string()),
        fallback_bucket: fallback_reason.map(|reason| reason.category().to_string()),
    }
}

#[cfg(feature = "pdfium")]
fn append_pdfium_hot_records(
    records: &mut Vec<BenchmarkMatrixRecord>,
    fixtures: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    config: &BenchmarkMatrixConfig,
) {
    match PdfiumBackend::from_env() {
        Ok(pdfium) => {
            for fixture in fixtures {
                records.push(benchmark_matrix_hot_backend(
                    &pdfium,
                    MatrixBackend::Pdfium,
                    pdfium_backend_version(),
                    MatrixFixtureContext {
                        fixture,
                        manifest,
                        options,
                    },
                    config,
                    false,
                ));
            }
        }
        Err(error) => records.extend(missing_tool_records(
            MatrixBackend::Pdfium,
            MatrixMode::HotRender,
            fixtures,
            manifest,
            options,
            format!("PDFium backend unavailable: {error}"),
        )),
    }
}

#[cfg(not(feature = "pdfium"))]
fn append_pdfium_hot_records(
    records: &mut Vec<BenchmarkMatrixRecord>,
    fixtures: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    _config: &BenchmarkMatrixConfig,
) {
    records.extend(missing_tool_records(
        MatrixBackend::Pdfium,
        MatrixMode::HotRender,
        fixtures,
        manifest,
        options,
        pdfium_missing_message(),
    ));
}

fn missing_tool_records(
    backend: MatrixBackend,
    mode: MatrixMode,
    fixtures: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    message: String,
) -> Vec<BenchmarkMatrixRecord> {
    fixtures
        .iter()
        .map(|fixture| {
            let context = MatrixFixtureContext {
                fixture,
                manifest,
                options,
            };
            matrix_unavailable_record(
                backend,
                mode,
                context,
                MatrixStatus::MissingTool,
                "missing-tool",
                &message,
            )
        })
        .collect()
}

fn not_applicable_records(
    backend: MatrixBackend,
    mode: MatrixMode,
    fixtures: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
    message: &str,
) -> Vec<BenchmarkMatrixRecord> {
    fixtures
        .iter()
        .map(|fixture| {
            let context = MatrixFixtureContext {
                fixture,
                manifest,
                options,
            };
            matrix_unavailable_record(
                backend,
                mode,
                context,
                MatrixStatus::NotApplicable,
                "not-applicable",
                message,
            )
        })
        .collect()
}

fn matrix_unavailable_record(
    backend: MatrixBackend,
    mode: MatrixMode,
    context: MatrixFixtureContext<'_>,
    status: MatrixStatus,
    class: &str,
    message: &str,
) -> BenchmarkMatrixRecord {
    BenchmarkMatrixRecord {
        backend,
        backend_version: match backend {
            MatrixBackend::Native => native_backend_version(NativeProfile::Default),
            MatrixBackend::Pdfium => pdfium_backend_version(),
            MatrixBackend::Poppler => "pdftoppm".to_string(),
        },
        command: backend.as_str().to_string(),
        mode,
        fixture: normalize_manifest_path(context.fixture),
        family: matrix_family(context.fixture, context.manifest),
        page_index: context.options.page_index,
        status,
        exit_status: None,
        timing: MatrixTiming::default(),
        output: MatrixOutput::default(),
        memory: MatrixMemory {
            source: "unavailable",
            ..MatrixMemory::default()
        },
        error_class: Some(class.to_string()),
        error_message: Some(message.to_string()),
        fallback_bucket: None,
    }
}

fn benchmark_matrix_report(
    config: BenchmarkMatrixConfig,
    records: Vec<BenchmarkMatrixRecord>,
) -> BenchmarkMatrixReport {
    let summary = benchmark_matrix_summary(&records);
    let families = benchmark_matrix_family_summaries(&records);
    BenchmarkMatrixReport {
        platform: PlatformMetadata::current(),
        command: env::args().collect::<Vec<_>>().join(" "),
        config: BenchmarkMatrixReportConfig {
            input: normalize_manifest_path(&config.input),
            manifest: config
                .manifest
                .as_ref()
                .map(|path| normalize_manifest_path(path)),
            include_families: config.include_families.clone(),
            page_index: config.page_index,
            max_edge: config.max_edge,
            timeout_secs: config.timeout.as_secs(),
            iterations: config.iterations,
            warmup: config.warmup,
            backends: config.backends.clone(),
            modes: config.modes.clone(),
            native_profile: config.native_profile,
        },
        summary,
        families,
        records,
    }
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
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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

fn visual_diff_poppler_command(args: &[OsString]) -> Result<(), CliError> {
    let config = VisualDiffConfig::parse(args)?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: config.timeout,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    };
    let fixtures = pdf_inputs(&config.input)?;
    let manifest = match &config.manifest {
        Some(path) => Some(read_corpus_manifest(path)?),
        None => None,
    };
    let fixtures =
        filter_fixtures_by_family(&fixtures, manifest.as_ref(), &config.include_families)?;
    let native = NativeBackend::new();
    let report = poppler_visual_diff_report(
        &native,
        &fixtures,
        &options,
        manifest.as_ref(),
        config.thresholds,
    );
    let json = poppler_visual_diff_report_json(&report);

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
        .env(PDFIUM_RENDER_WORKER_ENV, "1")
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
    annotation_mode: AnnotationMode,
}

impl RenderConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut annotation_mode = AnnotationMode::default();

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
                "--annotation-mode" => {
                    index += 1;
                    annotation_mode =
                        parse_annotation_mode(required_str(args, index, "--annotation-mode")?)?;
                }
                "--allow-pdfium-fallback" => {
                    return Err(CliError::Usage(
                        PDFIUM_RUNTIME_FALLBACK_REMOVED_MESSAGE.to_string(),
                    ));
                }
                "--native-only" | "--no-pdfium-fallback" => {
                    // Accepted for compatibility; render and render-auto are always native-only.
                }
                "--deny-fallback-reason" => {
                    index += 1;
                    let _ = required_str(args, index, "--deny-fallback-reason")?;
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
            annotation_mode,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct OperatorCoverageConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    include_annotations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducerRegressionConfig {
    input: PathBuf,
    manifest: PathBuf,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    native_profile: NativeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceNativeConfig {
    input: PathBuf,
    output: Option<PathBuf>,
    page_index: u32,
    max_edge: u32,
    max_events: usize,
    include_annotations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayOperatorsConfig {
    input: PathBuf,
    output: Option<PathBuf>,
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

impl OperatorCoverageConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut include_annotations = true;

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
                "--no-annotations" => {
                    include_annotations = false;
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
            include_families,
            output,
            page_index,
            include_annotations,
        })
    }
}

impl ProducerRegressionConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
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
                    timeout = Duration::from_secs(parse_u64(args, index, "--timeout")?);
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
            manifest: manifest.ok_or_else(|| {
                CliError::Usage("--manifest is required for producer-regression-report".to_string())
            })?,
            include_families,
            output,
            page_index,
            max_edge,
            background,
            timeout,
            native_profile,
        })
    }
}

impl TraceNativeConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut max_events = DEFAULT_TRACE_MAX_EVENTS;
        let mut include_annotations = true;

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
                "--max-events" => {
                    index += 1;
                    max_events = parse_usize(args, index, "--max-events")?;
                }
                "--no-annotations" => {
                    include_annotations = false;
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
        if max_events > TRACE_MAX_EVENTS_LIMIT {
            return Err(CliError::Usage(format!(
                "--max-events must be <= {TRACE_MAX_EVENTS_LIMIT}"
            )));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input PDF".to_string()))?,
            output,
            page_index,
            max_edge,
            max_events,
            include_annotations,
        })
    }
}

impl ReplayOperatorsConfig {
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
                            "only one trace file is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing trace file".to_string()))?,
            output,
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
struct Pdf20UsageConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    output: Option<PathBuf>,
    include_families: Vec<String>,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
}

impl Pdf20UsageConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut output = None;
        let mut include_families = Vec::new();
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
            include_families,
            page_index,
            max_edge,
            background,
            timeout,
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
    pages_per_input: usize,
    max_workers: usize,
    max_in_flight_pixels: usize,
    max_p95_ms: u64,
    max_errors: usize,
    fail_on_budget: bool,
    native_profile: NativeProfile,
    cancel_after_jobs: Option<usize>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkMatrixConfig {
    input: PathBuf,
    manifest: Option<PathBuf>,
    include_families: Vec<String>,
    output: Option<PathBuf>,
    markdown_report: Option<PathBuf>,
    artifact_dir: PathBuf,
    pdftoppm: PathBuf,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
    iterations: usize,
    warmup: usize,
    backends: Vec<MatrixBackend>,
    modes: Vec<MatrixMode>,
    native_profile: NativeProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MatrixBackend {
    Native,
    Pdfium,
    Poppler,
}

impl MatrixBackend {
    const ALL: [Self; 3] = [Self::Native, Self::Pdfium, Self::Poppler];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Pdfium => "pdfium",
            Self::Poppler => "poppler",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MatrixMode {
    ColdProcess,
    HotRender,
}

impl MatrixMode {
    const ALL: [Self; 2] = [Self::ColdProcess, Self::HotRender];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ColdProcess => "cold-process",
            Self::HotRender => "hot-render",
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct VisualDiffThresholds {
    max_mean_abs_error: f64,
    max_p95_channel_delta: u8,
    max_changed_ratio: f64,
}

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
        let mut pages_per_input = 1;
        let mut max_workers = 2;
        let mut max_in_flight_pixels = 2 * 160 * 160;
        let mut max_p95_ms = 1000;
        let mut max_errors = 0;
        let mut fail_on_budget = false;
        let mut native_profile = NativeProfile::Default;
        let mut cancel_after_jobs = None;

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
                "--pages-per-input" => {
                    index += 1;
                    pages_per_input = parse_usize(args, index, "--pages-per-input")?;
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
                "--cancel-after-jobs" => {
                    index += 1;
                    cancel_after_jobs = Some(parse_usize(args, index, "--cancel-after-jobs")?);
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
        if pages_per_input == 0 {
            return Err(CliError::Usage(
                "--pages-per-input must be greater than zero".to_string(),
            ));
        }
        if pages_per_input > u32::MAX as usize {
            return Err(CliError::Usage(
                "--pages-per-input must fit in u32 page indices".to_string(),
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
            pages_per_input,
            max_workers,
            max_in_flight_pixels,
            max_p95_ms,
            max_errors,
            fail_on_budget,
            native_profile,
            cancel_after_jobs,
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

impl BenchmarkMatrixConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut manifest = None;
        let mut include_families = Vec::new();
        let mut output = None;
        let mut markdown_report = None;
        let mut artifact_dir = PathBuf::from("target/performance-matrix");
        let mut pdftoppm = env::var_os("FERRUGO_POPPLER_PDFTOPPM")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("pdftoppm"));
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = 160;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut iterations = 3;
        let mut warmup = 1;
        let mut backends = Vec::new();
        let mut modes = Vec::new();
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
                "--report" | "--markdown-report" => {
                    index += 1;
                    markdown_report = Some(required_path(args, index, arg)?);
                }
                "--artifact-dir" => {
                    index += 1;
                    artifact_dir = required_path(args, index, "--artifact-dir")?;
                }
                "--pdftoppm" => {
                    index += 1;
                    pdftoppm = required_path(args, index, "--pdftoppm")?;
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
                "--warmup" => {
                    index += 1;
                    warmup = parse_usize(args, index, "--warmup")?;
                }
                "--backend" => {
                    index += 1;
                    backends.push(parse_matrix_backend(required_str(
                        args,
                        index,
                        "--backend",
                    )?)?);
                }
                "--mode" => {
                    index += 1;
                    modes.push(parse_matrix_mode(required_str(args, index, "--mode")?)?);
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
        if backends.is_empty() {
            backends.extend(MatrixBackend::ALL);
        }
        if modes.is_empty() {
            modes.extend(MatrixMode::ALL);
        }
        dedup_sorted_matrix_backends(&mut backends);
        dedup_sorted_matrix_modes(&mut modes);

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input path".to_string()))?,
            manifest,
            include_families,
            output,
            markdown_report,
            artifact_dir,
            pdftoppm,
            page_index,
            max_edge,
            background,
            timeout,
            iterations,
            warmup,
            backends,
            modes,
            native_profile,
        })
    }
}

fn parse_matrix_backend(value: &str) -> Result<MatrixBackend, CliError> {
    match value {
        "native" | "rust-native" | "ferrugo" => Ok(MatrixBackend::Native),
        "pdfium" => Ok(MatrixBackend::Pdfium),
        "poppler" | "pdftoppm" => Ok(MatrixBackend::Poppler),
        _ => Err(CliError::Usage(format!(
            "unknown --backend `{value}`; expected `native`, `pdfium`, or `poppler`"
        ))),
    }
}

fn parse_matrix_mode(value: &str) -> Result<MatrixMode, CliError> {
    match value {
        "cold-process" | "cold" => Ok(MatrixMode::ColdProcess),
        "hot-render" | "hot" => Ok(MatrixMode::HotRender),
        _ => Err(CliError::Usage(format!(
            "unknown --mode `{value}`; expected `cold-process` or `hot-render`"
        ))),
    }
}

fn dedup_sorted_matrix_backends(backends: &mut Vec<MatrixBackend>) {
    backends.sort_unstable();
    backends.dedup();
}

fn dedup_sorted_matrix_modes(modes: &mut Vec<MatrixMode>) {
    modes.sort_unstable();
    modes.dedup();
}

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
struct ProducerRegressionReport {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: usize,
    producer_groups: BTreeMap<String, ProducerRegressionGroup>,
    family_groups: BTreeMap<String, ProducerRegressionGroup>,
    feature_groups: BTreeMap<String, ProducerRegressionGroup>,
    records: Vec<ProducerRegressionRecord>,
}

impl ProducerRegressionReport {
    fn new(total: usize) -> Self {
        Self {
            total,
            native_rendered: 0,
            fallback_required: 0,
            errors: 0,
            producer_groups: BTreeMap::new(),
            family_groups: BTreeMap::new(),
            feature_groups: BTreeMap::new(),
            records: Vec::with_capacity(total),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProducerRegressionGroup {
    total: usize,
    native_rendered: usize,
    fallback_required: usize,
    errors: BTreeMap<&'static str, usize>,
    fallback_categories: BTreeMap<&'static str, usize>,
    affected_features: BTreeMap<String, usize>,
    milestone_routes: BTreeMap<String, usize>,
}

impl ProducerRegressionGroup {
    fn record(
        &mut self,
        outcome: &ProducerRegressionOutcome,
        features: &[String],
        milestone_routes: &[String],
    ) {
        self.total += 1;
        match outcome {
            ProducerRegressionOutcome::NativeRendered => {
                self.native_rendered += 1;
            }
            ProducerRegressionOutcome::FallbackRequired { category, .. } => {
                self.fallback_required += 1;
                *self.fallback_categories.entry(category).or_insert(0) += 1;
                for feature in features {
                    *self.affected_features.entry(feature.clone()).or_insert(0) += 1;
                }
            }
            ProducerRegressionOutcome::Error { class, .. } => {
                *self.errors.entry(class).or_insert(0) += 1;
                for feature in features {
                    *self.affected_features.entry(feature.clone()).or_insert(0) += 1;
                }
            }
        }
        for route in milestone_routes {
            *self.milestone_routes.entry(route.clone()).or_insert(0) += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducerRegressionRecord {
    fixture_id: String,
    path_redacted: bool,
    family: String,
    producer: String,
    features: Vec<String>,
    milestone_routes: Vec<String>,
    outcome: ProducerRegressionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProducerRegressionOutcome {
    NativeRendered,
    FallbackRequired {
        reason: String,
        category: &'static str,
    },
    Error {
        class: &'static str,
        message: String,
    },
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
struct Pdf20UsageReport {
    total_scanned: usize,
    pdf20_documents: usize,
    native_rendered: usize,
    typed_unsupported: usize,
    errors: usize,
    feature_counts: BTreeMap<String, usize>,
    impact_counts: BTreeMap<&'static str, usize>,
    families: BTreeMap<String, Pdf20FamilySummary>,
    followups: Vec<Pdf20Followup>,
    fixtures: Vec<Pdf20UsageRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Pdf20FamilySummary {
    total: usize,
    pdf20_documents: usize,
    native_rendered: usize,
    typed_unsupported: usize,
    errors: usize,
}

impl Pdf20FamilySummary {
    fn record(&mut self, record: &Pdf20UsageRecord) {
        self.total += 1;
        if record.version.detected_pdf20 {
            self.pdf20_documents += 1;
        }
        match record.render {
            Pdf20RenderOutcome::NativeRendered => self.native_rendered += 1,
            Pdf20RenderOutcome::TypedUnsupported { .. } => self.typed_unsupported += 1,
            Pdf20RenderOutcome::Error { .. } => self.errors += 1,
            Pdf20RenderOutcome::NotPdf20 => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pdf20UsageRecord {
    path: String,
    family: String,
    manifest_features: Vec<String>,
    version: Pdf20VersionEvidence,
    features: Vec<Pdf20FeatureObservation>,
    render: Pdf20RenderOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pdf20VersionEvidence {
    header_version: Option<String>,
    catalog_version_20: bool,
    manifest_pdf20_tag: bool,
    detected_pdf20: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pdf20FeatureObservation {
    feature: &'static str,
    policy: &'static str,
    visual_impact: &'static str,
    bucket: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pdf20RenderOutcome {
    NotPdf20,
    NativeRendered,
    TypedUnsupported { bucket: &'static str },
    Error { class: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pdf20Followup {
    rank: usize,
    feature: String,
    observed_documents: usize,
    visual_impact: &'static str,
    bucket: Option<&'static str>,
    recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusOperatorCoverageReport {
    page_index: u32,
    include_annotations: bool,
    total: usize,
    scanned: usize,
    errors: usize,
    total_operators: usize,
    inline_images: usize,
    status_counts: OperatorStatusCounts,
    operators: BTreeMap<String, CorpusOperatorSummary>,
    families: BTreeMap<String, CorpusOperatorFamilySummary>,
    fixtures: Vec<CorpusOperatorCoverageRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OperatorStatusCounts {
    implemented: usize,
    partial: usize,
    unsupported: usize,
    ignored: usize,
}

impl OperatorStatusCounts {
    fn record(&mut self, status: OperatorSupportStatus, count: usize) {
        match status {
            OperatorSupportStatus::Implemented => self.implemented += count,
            OperatorSupportStatus::Partial => self.partial += count,
            OperatorSupportStatus::Unsupported => self.unsupported += count,
            OperatorSupportStatus::Ignored => self.ignored += count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusOperatorSummary {
    count: usize,
    status: OperatorSupportStatus,
    fallback_bucket: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CorpusOperatorFamilySummary {
    total: usize,
    scanned: usize,
    errors: usize,
    total_operators: usize,
    inline_images: usize,
    status_counts: OperatorStatusCounts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorpusOperatorCoverageRecord {
    path: String,
    family: String,
    outcome: CorpusOperatorCoverageOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CorpusOperatorCoverageOutcome {
    Scanned {
        streams_scanned: usize,
        total_operators: usize,
        inline_images: usize,
        operators: Vec<OperatorCoverageEntry>,
    },
    Error {
        class: &'static str,
        message: String,
    },
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
    pages_per_input: usize,
    elapsed_ms: f64,
    throughput_per_sec: f64,
    max_p95_ms: u64,
    max_errors: usize,
    isolation: BatchIsolationSummary,
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

#[derive(Debug, Clone, PartialEq)]
struct BenchmarkMatrixReport {
    platform: PlatformMetadata,
    command: String,
    config: BenchmarkMatrixReportConfig,
    summary: BenchmarkMatrixSummary,
    families: BTreeMap<String, BenchmarkMatrixFamilySummary>,
    records: Vec<BenchmarkMatrixRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkMatrixTimingReliability {
    rss_available: bool,
    pdfium_requested: bool,
    pdfium_available: bool,
    poppler_requested: bool,
    poppler_available: bool,
    hot_pdfium_comparison_available: bool,
    cold_reference_available: bool,
    caveats: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkMatrixReportConfig {
    input: String,
    manifest: Option<String>,
    include_families: Vec<String>,
    page_index: u32,
    max_edge: u32,
    timeout_secs: u64,
    iterations: usize,
    warmup: usize,
    backends: Vec<MatrixBackend>,
    modes: Vec<MatrixMode>,
    native_profile: NativeProfile,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BenchmarkMatrixSummary {
    total_records: usize,
    rendered: usize,
    fallback_required: usize,
    missing_tool: usize,
    not_applicable: usize,
    errors: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct BenchmarkMatrixFamilySummary {
    total: usize,
    rendered: usize,
    fallback_required: usize,
    missing_tool: usize,
    not_applicable: usize,
    errors: usize,
    native_hot_p95_ms: Option<f64>,
    native_cold_wall_ms: Option<f64>,
    pdfium_hot_p95_ms: Option<f64>,
    pdfium_cold_wall_ms: Option<f64>,
    poppler_cold_wall_ms: Option<f64>,
    ferrugo_to_pdfium_hot_ratio: Option<f64>,
    ferrugo_to_pdfium_cold_ratio: Option<f64>,
    ferrugo_to_poppler_cold_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
struct BenchmarkMatrixRecord {
    backend: MatrixBackend,
    backend_version: String,
    command: String,
    mode: MatrixMode,
    fixture: String,
    family: String,
    page_index: u32,
    status: MatrixStatus,
    exit_status: Option<i32>,
    timing: MatrixTiming,
    output: MatrixOutput,
    memory: MatrixMemory,
    error_class: Option<String>,
    error_message: Option<String>,
    fallback_bucket: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatrixStatus {
    Rendered,
    FallbackRequired,
    MissingTool,
    NotApplicable,
    Error,
}

impl MatrixStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Rendered => "rendered",
            Self::FallbackRequired => "fallback-required",
            Self::MissingTool => "missing-tool",
            Self::NotApplicable => "not-applicable",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct MatrixTiming {
    wall_ms: Option<f64>,
    warmup_iterations: usize,
    measured_iterations: usize,
    samples_ms: Vec<f64>,
    mean_ms: Option<f64>,
    p50_ms: Option<f64>,
    p95_ms: Option<f64>,
    max_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MatrixOutput {
    width: Option<u32>,
    height: Option<u32>,
    bytes: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MatrixMemory {
    rss_start_bytes: Option<u64>,
    rss_peak_bytes: Option<u64>,
    rss_end_bytes: Option<u64>,
    source: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct BatchMemorySummary {
    rss_start_kib: Option<u64>,
    rss_high_water_kib: Option<u64>,
    rss_end_kib: Option<u64>,
    max_in_flight_pixels: usize,
    max_output_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatchIsolationSummary {
    cache_policy: NativePageCachePolicy,
    cancel_after_jobs: Option<usize>,
    scheduled_jobs: usize,
    skipped_jobs: usize,
    cancelled: bool,
    backend_scope: &'static str,
    shared_document_state: bool,
    timeout_ms: u128,
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
    phase_timing_count: usize,
    phase_timings: Option<RepeatPhaseTimings>,
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
                if let Some(phase_timings) = record.phase_timings.as_ref() {
                    self.phase_timing_count += 1;
                    let sums = self
                        .phase_timings
                        .get_or_insert_with(RepeatPhaseTimings::default);
                    add_phase_timings(&mut sums.first, phase_timings.first);
                    add_phase_timings(&mut sums.repeat_mean, phase_timings.repeat_mean);
                }
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
        if let Some(phase_timings) = self.phase_timings.as_mut() {
            phase_timings.first = scale_phase_timings(phase_timings.first, self.phase_timing_count);
            phase_timings.repeat_mean =
                scale_phase_timings(phase_timings.repeat_mean, self.phase_timing_count);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RepeatBenchmarkRecord {
    path: String,
    family: String,
    page_index: u32,
    cache_key: NativePageCacheKey,
    session_stats: Option<NativeDocumentSessionStats>,
    timings_ms: Vec<f64>,
    phase_timings: Option<RepeatPhaseTimings>,
    budget_violations: Vec<&'static str>,
    outcome: RepeatBenchmarkOutcome,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct RepeatPhaseTimings {
    first: NativeRenderPhaseTimings,
    repeat_mean: NativeRenderPhaseTimings,
}

struct NativeDiagnosticBundle<'a> {
    path: &'a str,
    path_redacted: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlatformMetadata {
    os: &'static str,
    arch: &'static str,
    family: &'static str,
    endian: &'static str,
    pointer_width_bits: usize,
    rustc_version: Option<String>,
    logical_cpus: Option<usize>,
    cpu_brand: Option<String>,
    memory_bytes: Option<u64>,
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
            rustc_version: command_stdout("rustc", &["--version"]),
            logical_cpus: thread::available_parallelism()
                .ok()
                .map(std::num::NonZeroUsize::get),
            cpu_brand: host_cpu_brand(),
            memory_bytes: host_memory_bytes(),
        }
    }
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn host_cpu_brand() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        command_stdout("sysctl", &["-n", "machdep.cpu.brand_string"])
    }
    #[cfg(target_os = "linux")]
    {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
        cpuinfo.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.trim() == "model name")
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn host_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        command_stdout("sysctl", &["-n", "hw.memsize"])?
            .parse()
            .ok()
    }
    #[cfg(target_os = "linux")]
    {
        let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
        meminfo.lines().find_map(|line| {
            let value = line.strip_prefix("MemTotal:")?.trim();
            let kib = value.split_whitespace().next()?.parse::<u64>().ok()?;
            kib.checked_mul(1024)
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisualDiffError {
    class: &'static str,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
struct PopplerVisualDiffReport {
    platform: PlatformMetadata,
    thresholds: VisualDiffThresholds,
    total: usize,
    exact: usize,
    accepted_drift: usize,
    blockers: usize,
    native_errors: usize,
    reference_errors: usize,
    both_errors: usize,
    families: BTreeMap<String, PopplerFamilyVisualDiffSummary>,
    subsystems: BTreeMap<String, PopplerFamilyVisualDiffSummary>,
    fixtures: Vec<PopplerVisualDiffRecord>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PopplerFamilyVisualDiffSummary {
    total: usize,
    exact: usize,
    accepted_drift: usize,
    blockers: usize,
    native_errors: usize,
    reference_errors: usize,
    both_errors: usize,
}

impl PopplerFamilyVisualDiffSummary {
    fn record(&mut self, record: &PopplerVisualDiffRecord) {
        self.total += 1;
        match record.status {
            PopplerVisualDiffStatus::Exact => self.exact += 1,
            PopplerVisualDiffStatus::AcceptedDrift => self.accepted_drift += 1,
            PopplerVisualDiffStatus::Blocker => self.blockers += 1,
            PopplerVisualDiffStatus::NativeError => self.native_errors += 1,
            PopplerVisualDiffStatus::ReferenceError => self.reference_errors += 1,
            PopplerVisualDiffStatus::BothError => self.both_errors += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PopplerVisualDiffRecord {
    path: String,
    family: String,
    subsystem: &'static str,
    status: PopplerVisualDiffStatus,
    metrics: Option<VisualDiffMetrics>,
    comparison_error: Option<VisualDiffError>,
    native_error: Option<VisualDiffError>,
    reference_error: Option<VisualDiffError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopplerVisualDiffStatus {
    Exact,
    AcceptedDrift,
    Blocker,
    NativeError,
    ReferenceError,
    BothError,
}

impl PopplerVisualDiffStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::AcceptedDrift => "accepted_drift",
            Self::Blocker => "blocker",
            Self::NativeError => "native_error",
            Self::ReferenceError => "reference_error",
            Self::BothError => "both_error",
        }
    }
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

fn filter_fixtures_by_manifest(
    paths: &[PathBuf],
    manifest: &CorpusManifest,
) -> Result<Vec<PathBuf>, CliError> {
    let filtered = paths
        .iter()
        .filter(|path| {
            let path_key = normalize_manifest_path(path);
            manifest.entry_for_path(&path_key).is_some()
        })
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(CliError::Usage(
            "manifest matched no input fixtures".to_string(),
        ));
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
            Err(error) if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported => {
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

fn build_producer_regression_report(
    native: &NativeBackend,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: &CorpusManifest,
) -> ProducerRegressionReport {
    let mut report = ProducerRegressionReport::new(paths.len());

    for (index, path) in paths.iter().enumerate() {
        let path_key = normalize_manifest_path(path);
        let manifest_entry = manifest.entry_for_path(&path_key);
        let family = manifest_entry
            .map(|entry| entry.family.clone())
            .unwrap_or_else(|| "unclassified".to_string());
        let producer = manifest_entry
            .map(producer_key)
            .unwrap_or_else(|| "unclassified".to_string());
        let features = manifest_entry.map(feature_flags).unwrap_or_default();
        let outcome = match native.render(PdfSource::from_path(path), options) {
            Ok(_) => {
                report.native_rendered += 1;
                ProducerRegressionOutcome::NativeRendered
            }
            Err(error) if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported => {
                report.fallback_required += 1;
                let reason = FallbackReason::from_native_error(&error);
                ProducerRegressionOutcome::FallbackRequired {
                    reason: reason.as_str().to_string(),
                    category: reason.category(),
                }
            }
            Err(error) => {
                report.errors += 1;
                ProducerRegressionOutcome::Error {
                    class: error.class().as_str(),
                    message: error.to_string(),
                }
            }
        };
        let milestone_routes = producer_milestone_routes(manifest_entry, &outcome);
        let fixture_id = producer_fixture_id(index, path, manifest_entry);
        let path_redacted = manifest_entry.is_some_and(|entry| is_sensitive_fixture(path, entry));

        report
            .producer_groups
            .entry(producer.clone())
            .or_default()
            .record(&outcome, &features, &milestone_routes);
        report
            .family_groups
            .entry(family.clone())
            .or_default()
            .record(&outcome, &features, &milestone_routes);
        for feature in &features {
            report
                .feature_groups
                .entry(feature.clone())
                .or_default()
                .record(&outcome, &features, &milestone_routes);
        }

        report.records.push(ProducerRegressionRecord {
            fixture_id,
            path_redacted,
            family,
            producer,
            features,
            milestone_routes,
            outcome,
        });
    }

    report
}

fn producer_key(entry: &CorpusManifestEntry) -> String {
    entry
        .features
        .iter()
        .find_map(|feature| feature.strip_prefix("producer:"))
        .unwrap_or("unclassified")
        .to_string()
}

fn feature_flags(entry: &CorpusManifestEntry) -> Vec<String> {
    entry
        .features
        .iter()
        .filter(|feature| {
            !feature.starts_with("producer:")
                && !feature.starts_with("expected:")
                && !feature.starts_with("privacy:")
        })
        .cloned()
        .collect()
}

fn producer_fixture_id(
    index: usize,
    path: &Path,
    manifest_entry: Option<&CorpusManifestEntry>,
) -> String {
    match manifest_entry {
        Some(entry) if is_sensitive_fixture(path, entry) => format!("local-only-{index:04}"),
        _ => normalize_manifest_path(path),
    }
}

fn is_sensitive_fixture(path: &Path, entry: &CorpusManifestEntry) -> bool {
    let path_key = normalize_manifest_path(path);
    path_key.contains("fixtures/local-corpus/")
        || entry.source.contains("local-corpus")
        || entry.license.contains("local-review")
        || entry.license.contains("reference-only")
        || entry
            .features
            .iter()
            .any(|feature| matches!(feature.as_str(), "privacy:private" | "privacy:local-only"))
}

fn producer_milestone_routes(
    manifest_entry: Option<&CorpusManifestEntry>,
    outcome: &ProducerRegressionOutcome,
) -> Vec<String> {
    if matches!(outcome, ProducerRegressionOutcome::NativeRendered) {
        return Vec::new();
    }

    let mut routes = BTreeMap::<String, ()>::new();
    if let Some(entry) = manifest_entry {
        for feature in &entry.features {
            if let Some(route) = feature_milestone_route(feature) {
                routes.insert(route.to_string(), ());
            }
        }
    }
    if let ProducerRegressionOutcome::FallbackRequired { category, .. } = outcome {
        if let Some(route) = fallback_category_milestone_route(category) {
            routes.insert(route.to_string(), ());
        }
    }
    if routes.is_empty() {
        routes.insert("0190 manual regression triage".to_string(), ());
    }
    routes.into_keys().collect()
}

fn feature_milestone_route(feature: &str) -> Option<&'static str> {
    match feature {
        "optional-content" | "ocmd" => Some("0192 optional-content-ui-state"),
        "ccitt" | "jbig2" | "jpx" | "codec" => Some("0209 rust-native-image-codec"),
        "table" | "dense-table" | "dense-totals" | "ledger" | "spreadsheet" | "thin-strokes"
        | "small-text" => Some("0203 dense-office-table"),
        "chart" | "smartart" | "vector-effects" => {
            Some("0204 office-chart-smartart-vector-effects")
        }
        "acroform" | "checkbox" | "form" => Some("0206 form-appearance-update"),
        "annotation" | "comments" | "hyperlink" => Some("0207 annotation-fidelity"),
        "pdf-2.0" | "catalog-version" => Some("0181 pdf-2-0-feature-usage"),
        _ => None,
    }
}

fn fallback_category_milestone_route(category: &str) -> Option<&'static str> {
    match category {
        "graphics.optional-content" => Some("0192 optional-content-ui-state"),
        "image.filter" => Some("0209 rust-native-image-codec"),
        "annotations.forms" | "annotations.appearance" => Some("0206 form-appearance-update"),
        "graphics.color-management" => Some("0208 color-managed-print-preview"),
        _ => None,
    }
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
        let manifest_entry = manifest.and_then(|manifest| manifest.entry_for_path(&path_key));
        let path_redacted = manifest_entry.is_some_and(|entry| is_sensitive_fixture(path, entry));
        let diagnostic_path = diagnostic_fixture_id(index, path, manifest_entry);
        let diagnostics = native.memory_diagnostics();
        let bundle = native_diagnostic_bundle_json(NativeDiagnosticBundle {
            path: &diagnostic_path,
            path_redacted,
            manifest: manifest_entry,
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

fn diagnostic_fixture_id(
    index: usize,
    path: &Path,
    manifest_entry: Option<&CorpusManifestEntry>,
) -> String {
    match manifest_entry {
        Some(entry) if is_sensitive_fixture(path, entry) => format!("local-only-{index:04}"),
        _ => normalize_manifest_path(path),
    }
}

fn scan_operator_coverage_corpus(
    paths: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    config: &OperatorCoverageConfig,
) -> CorpusOperatorCoverageReport {
    let mut report = CorpusOperatorCoverageReport {
        page_index: config.page_index,
        include_annotations: config.include_annotations,
        total: paths.len(),
        scanned: 0,
        errors: 0,
        total_operators: 0,
        inline_images: 0,
        status_counts: OperatorStatusCounts::default(),
        operators: BTreeMap::new(),
        families: BTreeMap::new(),
        fixtures: Vec::with_capacity(paths.len()),
    };
    let options = OperatorCoverageOptions {
        page_index: config.page_index,
        include_annotations: config.include_annotations,
    };

    for path in paths {
        let path_key = normalize_manifest_path(path);
        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified")
            .to_string();
        report.families.entry(family.clone()).or_default().total += 1;

        let outcome = match fs::read(path) {
            Ok(bytes) => match scan_operator_coverage(&bytes, options) {
                Ok(coverage) => {
                    report.scanned += 1;
                    report.total_operators += coverage.total_operators;
                    report.inline_images += coverage.inline_images;
                    if let Some(family_summary) = report.families.get_mut(&family) {
                        family_summary.scanned += 1;
                        family_summary.total_operators += coverage.total_operators;
                        family_summary.inline_images += coverage.inline_images;
                    }
                    for entry in &coverage.operators {
                        report.status_counts.record(entry.status, entry.count);
                        if let Some(family_summary) = report.families.get_mut(&family) {
                            family_summary
                                .status_counts
                                .record(entry.status, entry.count);
                        }
                        let summary = report.operators.entry(entry.operator.clone()).or_insert(
                            CorpusOperatorSummary {
                                count: 0,
                                status: entry.status,
                                fallback_bucket: entry.fallback_bucket,
                            },
                        );
                        summary.count += entry.count;
                    }
                    CorpusOperatorCoverageOutcome::Scanned {
                        streams_scanned: coverage.streams_scanned,
                        total_operators: coverage.total_operators,
                        inline_images: coverage.inline_images,
                        operators: coverage.operators,
                    }
                }
                Err(error) => {
                    report.errors += 1;
                    if let Some(family_summary) = report.families.get_mut(&family) {
                        family_summary.errors += 1;
                    }
                    CorpusOperatorCoverageOutcome::Error {
                        class: error.class().as_str(),
                        message: error.to_string(),
                    }
                }
            },
            Err(error) => {
                report.errors += 1;
                if let Some(family_summary) = report.families.get_mut(&family) {
                    family_summary.errors += 1;
                }
                CorpusOperatorCoverageOutcome::Error {
                    class: "io",
                    message: error.to_string(),
                }
            }
        };

        report.fixtures.push(CorpusOperatorCoverageRecord {
            path: path_key,
            family,
            outcome,
        });
    }

    report
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

fn classify_pdf20_usage(
    paths: &[PathBuf],
    manifest: Option<&CorpusManifest>,
    options: &ThumbnailOptions,
) -> Result<Pdf20UsageReport, CliError> {
    let native = NativeBackend::new();
    let mut fixtures = Vec::with_capacity(paths.len());
    let mut feature_counts = BTreeMap::new();
    let mut impact_counts = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut pdf20_documents = 0usize;
    let mut native_rendered = 0usize;
    let mut typed_unsupported = 0usize;
    let mut errors = 0usize;

    for path in paths {
        let bytes = fs::read(path).map_err(|source| CliError::ReadFile {
            path: path.clone(),
            source,
        })?;
        let path_key = normalize_manifest_path(path);
        let manifest_entry = manifest.and_then(|manifest| manifest.entry_for_path(&path_key));
        let family = manifest_entry
            .map(|entry| entry.family.as_str())
            .unwrap_or("unclassified")
            .to_string();
        let manifest_features = manifest_entry
            .map(|entry| entry.features.clone())
            .unwrap_or_default();
        let version = pdf20_version_evidence(&bytes, manifest_entry);
        let features = pdf20_feature_observations(&bytes, &manifest_features, &version);
        let render = if version.detected_pdf20 {
            match native.render(PdfSource::from_path(path), options) {
                Ok(_) => {
                    native_rendered += 1;
                    Pdf20RenderOutcome::NativeRendered
                }
                Err(error)
                    if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported =>
                {
                    typed_unsupported += 1;
                    Pdf20RenderOutcome::TypedUnsupported {
                        bucket: FallbackReason::from_native_error(&error).category(),
                    }
                }
                Err(error) => {
                    errors += 1;
                    Pdf20RenderOutcome::Error {
                        class: error.class().as_str(),
                    }
                }
            }
        } else {
            Pdf20RenderOutcome::NotPdf20
        };

        if version.detected_pdf20 {
            pdf20_documents += 1;
            for feature in &features {
                *feature_counts
                    .entry(feature.feature.to_string())
                    .or_insert(0) += 1;
                *impact_counts.entry(feature.visual_impact).or_insert(0) += 1;
            }
        }

        let record = Pdf20UsageRecord {
            path: path_key,
            family,
            manifest_features,
            version,
            features,
            render,
        };
        families
            .entry(record.family.clone())
            .or_insert_with(Pdf20FamilySummary::default)
            .record(&record);
        fixtures.push(record);
    }

    let followups = pdf20_followups(&feature_counts);
    Ok(Pdf20UsageReport {
        total_scanned: paths.len(),
        pdf20_documents,
        native_rendered,
        typed_unsupported,
        errors,
        feature_counts,
        impact_counts,
        families,
        followups,
        fixtures,
    })
}

fn pdf20_version_evidence(
    bytes: &[u8],
    manifest: Option<&CorpusManifestEntry>,
) -> Pdf20VersionEvidence {
    let header_version = pdf_header_version(bytes);
    let catalog_version_20 =
        contains_bytes(bytes, b"/Version /2.0") || contains_bytes(bytes, b"/Version/2.0");
    let manifest_pdf20_tag =
        manifest.is_some_and(|entry| feature_present(&entry.features, "pdf-2.0"));
    let detected_pdf20 = manifest_pdf20_tag
        || catalog_version_20
        || header_version.as_deref().is_some_and(pdf20_or_later);

    Pdf20VersionEvidence {
        header_version,
        catalog_version_20,
        manifest_pdf20_tag,
        detected_pdf20,
    }
}

fn pdf20_feature_observations(
    bytes: &[u8],
    manifest_features: &[String],
    version: &Pdf20VersionEvidence,
) -> Vec<Pdf20FeatureObservation> {
    if !version.detected_pdf20 {
        return Vec::new();
    }

    let mut observations = Vec::new();
    observations.push(Pdf20FeatureObservation {
        feature: "pdf-2.0-version-marker",
        policy: "accept-existing-render-path",
        visual_impact: "visual-supported",
        bucket: None,
    });
    if version.catalog_version_20 || feature_present(manifest_features, "catalog-version") {
        observations.push(Pdf20FeatureObservation {
            feature: "catalog-version",
            policy: "accept-existing-render-path",
            visual_impact: "non-visual",
            bucket: None,
        });
    }
    if contains_bytes(bytes, b"/AF ")
        || contains_bytes(bytes, b"/AF[")
        || contains_bytes(bytes, b"/AFRelationship")
        || feature_present(manifest_features, "associated-files")
    {
        observations.push(Pdf20FeatureObservation {
            feature: "associated-files",
            policy: "ignore-metadata-only",
            visual_impact: "non-visual",
            bucket: None,
        });
    }
    if contains_bytes(bytes, b"/UseBlackPtComp")
        || feature_present(manifest_features, "black-point-compensation")
    {
        observations.push(Pdf20FeatureObservation {
            feature: "black-point-compensation",
            policy: "typed-unsupported",
            visual_impact: "visual-unsupported",
            bucket: Some(ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT),
        });
    }

    observations
}

fn pdf20_followups(feature_counts: &BTreeMap<String, usize>) -> Vec<Pdf20Followup> {
    let mut followups = Vec::new();
    if let Some(&count) = feature_counts.get("black-point-compensation") {
        followups.push(Pdf20Followup {
            rank: 1,
            feature: "black-point-compensation".to_string(),
            observed_documents: count,
            visual_impact: "visual-unsupported",
            bucket: Some(ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT),
            recommendation: "keep typed unsupported for 1.2 unless real-corpus frequency rises; implement only with color-threshold evidence",
        });
    }
    if let Some(&count) = feature_counts.get("associated-files") {
        followups.push(Pdf20Followup {
            rank: followups.len() + 1,
            feature: "associated-files".to_string(),
            observed_documents: count,
            visual_impact: "non-visual",
            bucket: None,
            recommendation:
                "keep accepted as metadata-only for thumbnails and preserve regression fixtures",
        });
    }
    if let Some(&count) = feature_counts.get("catalog-version") {
        followups.push(Pdf20Followup {
            rank: followups.len() + 1,
            feature: "catalog-version".to_string(),
            observed_documents: count,
            visual_impact: "non-visual",
            bucket: None,
            recommendation: "keep as a compatibility signal and continue accepting when render operators stay in supported paths",
        });
    }
    followups
}

fn pdf_header_version(bytes: &[u8]) -> Option<String> {
    let prefix = bytes.get(..bytes.len().min(16))?;
    let header = std::str::from_utf8(prefix).ok()?;
    let version = header
        .strip_prefix("%PDF-")?
        .chars()
        .take(3)
        .collect::<String>();
    (version.len() == 3).then_some(version)
}

fn pdf20_or_later(version: &str) -> bool {
    version
        .split_once('.')
        .and_then(|(major, _)| major.parse::<u8>().ok())
        .is_some_and(|major| major >= 2)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn feature_present(features: &[String], needle: &str) -> bool {
    features.iter().any(|feature| feature == needle)
}

#[derive(Debug, Clone)]
struct BatchJob {
    path: PathBuf,
    path_key: String,
    family: String,
    repetition: usize,
    page_index: u32,
}

fn benchmark_native_batch(
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    config: &BatchBenchmarkConfig,
) -> Result<BatchBenchmarkReport, CliError> {
    let workers = effective_batch_workers(config, options)?;
    let mut jobs = batch_jobs(
        paths,
        manifest,
        config.repetitions,
        config.page_index,
        config.pages_per_input,
    );
    let requested_jobs = jobs.len();
    let scheduled_jobs = config.cancel_after_jobs.map_or(requested_jobs, |limit| {
        let scheduled = requested_jobs.min(limit);
        jobs.truncate(scheduled);
        scheduled
    });
    let skipped_jobs = requested_jobs.saturating_sub(scheduled_jobs);
    let cancelled = skipped_jobs > 0;
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
                        benchmark_batch_job(config.native_profile.backend(), job, options)
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
        BatchIsolationSummary {
            cache_policy: NativePageCachePolicy::IsolatedRender,
            cancel_after_jobs: config.cancel_after_jobs,
            scheduled_jobs,
            skipped_jobs,
            cancelled,
            backend_scope: "per-job",
            shared_document_state: false,
            timeout_ms: options.timeout.as_millis(),
        },
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
    start_page_index: u32,
    pages_per_input: usize,
) -> Vec<BatchJob> {
    let mut jobs = Vec::new();
    for repetition in 0..repetitions {
        for path in paths {
            let path_key = normalize_manifest_path(path);
            let manifest_entry = manifest.and_then(|manifest| manifest.entry_for_path(&path_key));
            let family = manifest_entry
                .map(|entry| entry.family.as_str())
                .unwrap_or("unclassified")
                .to_string();
            let page_indices =
                batch_page_indices(start_page_index, pages_per_input, manifest_entry);
            for page_index in page_indices {
                jobs.push(BatchJob {
                    path: path.clone(),
                    path_key: path_key.clone(),
                    family: family.clone(),
                    repetition,
                    page_index,
                });
            }
        }
    }
    jobs
}

fn batch_page_indices(
    start_page_index: u32,
    pages_per_input: usize,
    manifest_entry: Option<&CorpusManifestEntry>,
) -> std::ops::Range<u32> {
    let requested_end = start_page_index.saturating_add(pages_per_input as u32);
    let Some(entry) = manifest_entry else {
        return start_page_index..requested_end;
    };
    let page_count = entry.page_count.min(u32::MAX as usize) as u32;
    if start_page_index >= page_count {
        return start_page_index..start_page_index.saturating_add(1);
    }
    start_page_index..requested_end.min(page_count)
}

fn benchmark_batch_job(
    native: NativeBackend,
    job: &BatchJob,
    options: &ThumbnailOptions,
) -> BatchBenchmarkRecord {
    let started = Instant::now();
    let mut options = *options;
    options.page_index = job.page_index;
    let outcome = match native.render(PdfSource::from_path(&job.path), &options) {
        Ok(thumbnail) => BatchBenchmarkOutcome::NativeRendered {
            width: thumbnail.width,
            height: thumbnail.height,
            output_bytes: thumbnail.bytes.len(),
        },
        Err(error) if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported => {
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
        page_index: job.page_index,
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
    isolation: BatchIsolationSummary,
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
        pages_per_input: config.pages_per_input,
        elapsed_ms,
        throughput_per_sec: total_jobs as f64 / elapsed_secs,
        max_p95_ms: config.max_p95_ms,
        max_errors: config.max_errors,
        isolation,
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
    let mut last_success;
    let first_started = Instant::now();
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let outcome = RepeatBenchmarkOutcome::Error {
                class: "io",
                message: error.to_string(),
            };
            return repeat_error_record(path_key, family, options.page_index, cache_key, outcome);
        }
    };
    let session = match native.document_session(&bytes, &[options.page_index]) {
        Ok(session) => session,
        Err(error) => {
            let outcome = repeat_error_outcome(error);
            return repeat_error_record(path_key, family, options.page_index, cache_key, outcome);
        }
    };
    let session_stats = session.stats();
    let mut phase_timings = Vec::with_capacity(config.repetitions);
    let mut first_phase_timings = NativeRenderPhaseTimings::default();
    match session.render_page_with_timings(options, &mut first_phase_timings) {
        Ok(thumbnail) => {
            timings_ms.push(elapsed_ms(first_started.elapsed()));
            phase_timings.push(first_phase_timings);
            last_success = thumbnail;
        }
        Err(error) => {
            let outcome = repeat_error_outcome(error);
            return repeat_error_record_with_session(
                path_key,
                family,
                options.page_index,
                cache_key,
                Some(session_stats),
                timings_ms,
                outcome,
            );
        }
    }

    for _ in 1..config.repetitions {
        let started = Instant::now();
        let mut repeat_phase_timings = NativeRenderPhaseTimings::default();
        match session.render_page_with_timings(options, &mut repeat_phase_timings) {
            Ok(thumbnail) => {
                timings_ms.push(elapsed_ms(started.elapsed()));
                phase_timings.push(repeat_phase_timings);
                last_success = thumbnail;
            }
            Err(error) => {
                let outcome = repeat_error_outcome(error);
                return repeat_error_record_with_session(
                    path_key,
                    family,
                    options.page_index,
                    cache_key,
                    Some(session_stats),
                    timings_ms,
                    outcome,
                );
            }
        }
    }

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
    let repeat_phase_timings = RepeatPhaseTimings {
        first: phase_timings[0],
        repeat_mean: mean_phase_timings(&phase_timings[1..]),
    };
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
        session_stats: Some(session_stats),
        timings_ms,
        phase_timings: Some(repeat_phase_timings),
        budget_violations,
        outcome: RepeatBenchmarkOutcome::NativeRendered {
            width: last_success.width,
            height: last_success.height,
            output_bytes: last_success.bytes.len(),
            first_ms,
            repeat_mean_ms,
            repeat_min_ms,
            repeat_max_ms,
            repeat_to_first_ratio,
        },
    }
}

fn repeat_error_outcome(error: ThumbnailError) -> RepeatBenchmarkOutcome {
    if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported {
        RepeatBenchmarkOutcome::FallbackRequired {
            reason: FallbackReason::from_native_error(&error),
            message: error.to_string(),
        }
    } else {
        RepeatBenchmarkOutcome::Error {
            class: error.class().as_str(),
            message: error.to_string(),
        }
    }
}

fn mean_phase_timings(values: &[NativeRenderPhaseTimings]) -> NativeRenderPhaseTimings {
    let mut sum = NativeRenderPhaseTimings::default();
    for timings in values {
        add_phase_timings(&mut sum, *timings);
    }
    scale_phase_timings(sum, values.len())
}

fn add_phase_timings(target: &mut NativeRenderPhaseTimings, addend: NativeRenderPhaseTimings) {
    target.load_xref_object += addend.load_xref_object;
    target.stream_decode += addend.stream_decode;
    target.content_tokenize += addend.content_tokenize;
    target.display_list_build += addend.display_list_build;
    target.resource_decode += addend.resource_decode;
    target.raster_paths += addend.raster_paths;
    target.raster_text += addend.raster_text;
    target.raster_images += addend.raster_images;
    target.output += addend.output;
    target.total += addend.total;
}

fn scale_phase_timings(
    timings: NativeRenderPhaseTimings,
    divisor: usize,
) -> NativeRenderPhaseTimings {
    let divisor = divisor as f64;
    NativeRenderPhaseTimings {
        load_xref_object: scale_duration(timings.load_xref_object, divisor),
        stream_decode: scale_duration(timings.stream_decode, divisor),
        content_tokenize: scale_duration(timings.content_tokenize, divisor),
        display_list_build: scale_duration(timings.display_list_build, divisor),
        resource_decode: scale_duration(timings.resource_decode, divisor),
        raster_paths: scale_duration(timings.raster_paths, divisor),
        raster_text: scale_duration(timings.raster_text, divisor),
        raster_images: scale_duration(timings.raster_images, divisor),
        output: scale_duration(timings.output, divisor),
        total: scale_duration(timings.total, divisor),
    }
}

fn scale_duration(duration: Duration, divisor: f64) -> Duration {
    Duration::from_secs_f64(duration.as_secs_f64() / divisor)
}

fn repeat_error_record(
    path: String,
    family: String,
    page_index: u32,
    cache_key: NativePageCacheKey,
    outcome: RepeatBenchmarkOutcome,
) -> RepeatBenchmarkRecord {
    repeat_error_record_with_session(
        path,
        family,
        page_index,
        cache_key,
        None,
        Vec::new(),
        outcome,
    )
}

fn repeat_error_record_with_session(
    path: String,
    family: String,
    page_index: u32,
    cache_key: NativePageCacheKey,
    session_stats: Option<NativeDocumentSessionStats>,
    timings_ms: Vec<f64>,
    outcome: RepeatBenchmarkOutcome,
) -> RepeatBenchmarkRecord {
    let budget_violations = match &outcome {
        RepeatBenchmarkOutcome::FallbackRequired { .. } => vec!["native_fallback"],
        RepeatBenchmarkOutcome::Error { .. } => vec!["render_error"],
        RepeatBenchmarkOutcome::NativeRendered { .. } => Vec::new(),
    };
    RepeatBenchmarkRecord {
        path,
        family,
        page_index,
        cache_key,
        session_stats,
        timings_ms,
        phase_timings: None,
        budget_violations,
        outcome,
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
        cache_policy: NativePageCachePolicy::DocumentSession,
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

#[derive(Debug, Clone, PartialEq)]
struct ProcessMeasurement {
    success: bool,
    exit_status: Option<i32>,
    wall_ms: f64,
    peak_rss_bytes: Option<u64>,
    memory_source: &'static str,
    stderr: String,
}

#[derive(Debug, Clone, Copy)]
struct MatrixFixtureContext<'a> {
    fixture: &'a Path,
    manifest: Option<&'a CorpusManifest>,
    options: &'a ThumbnailOptions,
}

fn run_measured_process(
    program: &Path,
    args: &[OsString],
    envs: &[(OsString, OsString)],
    timeout: Duration,
) -> Result<ProcessMeasurement, CliError> {
    let time_command = Path::new("/usr/bin/time");
    let use_time = time_l_is_usable(time_command);
    if use_time {
        let measurement = run_measured_process_once(program, args, envs, timeout, true)?;
        if !time_l_wrapper_blocked(&measurement.stderr) {
            return Ok(measurement);
        }
    }
    run_measured_process_once(program, args, envs, timeout, false)
}

fn time_l_is_usable(time_command: &Path) -> bool {
    if !time_command.is_file() {
        return false;
    }
    let Ok(output) = Command::new(time_command)
        .arg("-l")
        .arg("true")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
    else {
        return false;
    };
    output.status.success()
        && parse_time_l_peak_rss_bytes(&String::from_utf8_lossy(&output.stderr)).is_some()
}

fn run_measured_process_once(
    program: &Path,
    args: &[OsString],
    envs: &[(OsString, OsString)],
    timeout: Duration,
    use_time: bool,
) -> Result<ProcessMeasurement, CliError> {
    let time_command = Path::new("/usr/bin/time");
    let mut command = if use_time {
        let mut command = Command::new(time_command);
        command.arg("-l").arg(program).args(args);
        command
    } else {
        let mut command = Command::new(program);
        command.args(args);
        command
    };
    for (key, value) in envs {
        command.env(key, value);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let started = Instant::now();
    let mut child = command.spawn().map_err(|source| CliError::Io {
        path: if use_time {
            time_command.to_path_buf()
        } else {
            program.to_path_buf()
        },
        source,
    })?;

    let status = loop {
        if let Some(status) = child.try_wait().map_err(|source| {
            CliError::Process(format!("failed to poll benchmark process: {source}"))
        })? {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ProcessMeasurement {
                success: false,
                exit_status: None,
                wall_ms: elapsed_ms(started.elapsed()),
                peak_rss_bytes: None,
                memory_source: if use_time {
                    "usr-bin-time-l"
                } else {
                    "unavailable"
                },
                stderr: format!("process exceeded {}s timeout", timeout.as_secs()),
            });
        }
        thread::sleep(WORKER_POLL_INTERVAL);
    };

    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_end(&mut stderr);
    }
    if let Some(mut pipe) = child.stdout.take() {
        let mut stdout = Vec::new();
        let _ = pipe.read_to_end(&mut stdout);
    }
    let stderr = String::from_utf8_lossy(&stderr).into_owned();
    Ok(ProcessMeasurement {
        success: status.success(),
        exit_status: status.code(),
        wall_ms: elapsed_ms(started.elapsed()),
        peak_rss_bytes: use_time
            .then(|| parse_time_l_peak_rss_bytes(&stderr))
            .flatten(),
        memory_source: if use_time {
            "usr-bin-time-l"
        } else {
            "unavailable"
        },
        stderr,
    })
}

fn time_l_wrapper_blocked(stderr: &str) -> bool {
    stderr.contains("time: sysctl kern.clockrate")
        || stderr.contains("time: command terminated abnormally")
        || stderr.contains("Operation not permitted")
            && stderr.contains("maximum resident set size")
}

fn parse_time_l_peak_rss_bytes(stderr: &str) -> Option<u64> {
    stderr.lines().find_map(|line| {
        if !line.contains("maximum resident set size") {
            return None;
        }
        line.split_whitespace()
            .find_map(|token| token.parse::<u64>().ok())
    })
}

fn matrix_process_failure_outcome(
    stderr: &str,
    unsupported_is_fallback: bool,
    exit_status: Option<i32>,
) -> (MatrixStatus, Option<String>, Option<String>, Option<String>) {
    let class = parse_render_error_class(stderr).unwrap_or("process");
    let message = if stderr.trim().is_empty() {
        match exit_status {
            Some(status) => format!("process exited with status {status}"),
            None => "process failed without an exit status".to_string(),
        }
    } else {
        stderr.trim().to_string()
    };
    if unsupported_is_fallback && class == "unsupported" {
        (
            MatrixStatus::FallbackRequired,
            Some(class.to_string()),
            Some(message),
            parse_unsupported_feature_bucket(stderr)
                .map(str::to_string)
                .or_else(|| Some("native-unsupported".to_string())),
        )
    } else {
        (
            MatrixStatus::Error,
            Some(class.to_string()),
            Some(message),
            None,
        )
    }
}

fn parse_render_error_class(stderr: &str) -> Option<&str> {
    let marker = "render error [";
    let start = stderr.find(marker)? + marker.len();
    let rest = stderr.get(start..)?;
    let end = rest.find(']')?;
    rest.get(..end)
}

fn parse_unsupported_feature_bucket(stderr: &str) -> Option<&str> {
    let marker = "PDF feature is unsupported (";
    let start = stderr.find(marker)? + marker.len();
    let rest = stderr.get(start..)?;
    let end = rest.find(')')?;
    rest.get(..end)
}

fn matrix_timing_from_samples(warmup_iterations: usize, samples_ms: Vec<f64>) -> MatrixTiming {
    let mut sorted = samples_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let mean = if samples_ms.is_empty() {
        None
    } else {
        Some(samples_ms.iter().sum::<f64>() / samples_ms.len() as f64)
    };
    MatrixTiming {
        wall_ms: mean,
        warmup_iterations,
        measured_iterations: samples_ms.len(),
        samples_ms,
        mean_ms: mean,
        p50_ms: (!sorted.is_empty()).then(|| percentile(&sorted, 0.50)),
        p95_ms: (!sorted.is_empty()).then(|| percentile(&sorted, 0.95)),
        max_ms: sorted.last().copied(),
    }
}

fn matrix_output_from_path(path: &Path) -> MatrixOutput {
    let bytes = fs::metadata(path).ok().map(|metadata| metadata.len());
    let dimensions = match path.extension().and_then(|extension| extension.to_str()) {
        Some("png") => read_png_dimensions(path).ok().flatten(),
        Some("ppm") => read_ppm_dimensions(path).ok().flatten(),
        _ => None,
    };
    MatrixOutput {
        width: dimensions.map(|(width, _)| width),
        height: dimensions.map(|(_, height)| height),
        bytes,
    }
}

fn read_png_dimensions(path: &Path) -> Result<Option<(u32, u32)>, CliError> {
    let mut file = fs::File::open(path).map_err(|source| CliError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let mut header = [0_u8; 24];
    file.read_exact(&mut header)
        .map_err(|source| CliError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
    if &header[..8] != b"\x89PNG\r\n\x1a\n" || &header[12..16] != b"IHDR" {
        return Ok(None);
    }
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
    Ok(Some((width, height)))
}

fn read_ppm_dimensions(path: &Path) -> Result<Option<(u32, u32)>, CliError> {
    let bytes = fs::read(path).map_err(|source| CliError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let Some(tokens) = ppm_header_tokens(&bytes, 4) else {
        return Ok(None);
    };
    if tokens.first().map(String::as_str) != Some("P6") {
        return Ok(None);
    }
    let Some(width) = tokens.get(1).and_then(|value| value.parse::<u32>().ok()) else {
        return Ok(None);
    };
    let Some(height) = tokens.get(2).and_then(|value| value.parse::<u32>().ok()) else {
        return Ok(None);
    };
    Ok(Some((width, height)))
}

fn ppm_header_tokens(bytes: &[u8], limit: usize) -> Option<Vec<String>> {
    let mut tokens = Vec::with_capacity(limit);
    let mut index = 0;
    while index < bytes.len() && tokens.len() < limit {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        if bytes[index] == b'#' {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'#' {
            index += 1;
        }
        tokens.push(String::from_utf8_lossy(&bytes[start..index]).into_owned());
    }
    (tokens.len() >= limit).then_some(tokens)
}

fn benchmark_matrix_summary(records: &[BenchmarkMatrixRecord]) -> BenchmarkMatrixSummary {
    let mut summary = BenchmarkMatrixSummary {
        total_records: records.len(),
        ..BenchmarkMatrixSummary::default()
    };
    for record in records {
        match record.status {
            MatrixStatus::Rendered => summary.rendered += 1,
            MatrixStatus::FallbackRequired => summary.fallback_required += 1,
            MatrixStatus::MissingTool => summary.missing_tool += 1,
            MatrixStatus::NotApplicable => summary.not_applicable += 1,
            MatrixStatus::Error => summary.errors += 1,
        }
    }
    summary
}

fn benchmark_matrix_family_summaries(
    records: &[BenchmarkMatrixRecord],
) -> BTreeMap<String, BenchmarkMatrixFamilySummary> {
    let mut families = BTreeMap::new();
    for record in records {
        let summary = families
            .entry(record.family.clone())
            .or_insert_with(BenchmarkMatrixFamilySummary::default);
        summary.total += 1;
        match record.status {
            MatrixStatus::Rendered => summary.rendered += 1,
            MatrixStatus::FallbackRequired => summary.fallback_required += 1,
            MatrixStatus::MissingTool => summary.missing_tool += 1,
            MatrixStatus::NotApplicable => summary.not_applicable += 1,
            MatrixStatus::Error => summary.errors += 1,
        }
    }
    let family_names = families.keys().cloned().collect::<Vec<_>>();
    for family in family_names {
        if let Some(summary) = families.get_mut(&family) {
            summary.native_hot_p95_ms = matrix_family_timing(
                records,
                &family,
                MatrixBackend::Native,
                MatrixMode::HotRender,
                MatrixTimingSelector::P95,
            );
            summary.native_cold_wall_ms = matrix_family_timing(
                records,
                &family,
                MatrixBackend::Native,
                MatrixMode::ColdProcess,
                MatrixTimingSelector::Wall,
            );
            summary.pdfium_hot_p95_ms = matrix_family_timing(
                records,
                &family,
                MatrixBackend::Pdfium,
                MatrixMode::HotRender,
                MatrixTimingSelector::P95,
            );
            summary.pdfium_cold_wall_ms = matrix_family_timing(
                records,
                &family,
                MatrixBackend::Pdfium,
                MatrixMode::ColdProcess,
                MatrixTimingSelector::Wall,
            );
            summary.poppler_cold_wall_ms = matrix_family_timing(
                records,
                &family,
                MatrixBackend::Poppler,
                MatrixMode::ColdProcess,
                MatrixTimingSelector::Wall,
            );
            summary.ferrugo_to_pdfium_hot_ratio =
                ratio(summary.native_hot_p95_ms, summary.pdfium_hot_p95_ms);
            summary.ferrugo_to_pdfium_cold_ratio =
                ratio(summary.native_cold_wall_ms, summary.pdfium_cold_wall_ms);
            summary.ferrugo_to_poppler_cold_ratio =
                ratio(summary.native_cold_wall_ms, summary.poppler_cold_wall_ms);
        }
    }
    families
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatrixTimingSelector {
    Wall,
    P95,
}

fn matrix_family_timing(
    records: &[BenchmarkMatrixRecord],
    family: &str,
    backend: MatrixBackend,
    mode: MatrixMode,
    selector: MatrixTimingSelector,
) -> Option<f64> {
    let mut values = records
        .iter()
        .filter(|record| {
            record.family == family
                && record.backend == backend
                && record.mode == mode
                && record.status == MatrixStatus::Rendered
        })
        .filter_map(|record| match selector {
            MatrixTimingSelector::Wall => record.timing.wall_ms,
            MatrixTimingSelector::P95 => record.timing.p95_ms,
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(percentile(&values, 0.95))
}

fn ratio(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    let left = left?;
    let right = right?;
    (right > f64::EPSILON).then(|| left / right)
}

fn matrix_family(path: &Path, manifest: Option<&CorpusManifest>) -> String {
    manifest
        .and_then(|manifest| manifest.family_for_path(path))
        .unwrap_or("unclassified")
        .to_string()
}

fn matrix_artifact_path(
    artifact_dir: &Path,
    backend: MatrixBackend,
    mode: MatrixMode,
    fixture: &Path,
    extension: &str,
) -> PathBuf {
    let stem = fixture
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_artifact_stem)
        .unwrap_or_else(|| "fixture".to_string());
    artifact_dir
        .join(format!("{}-{}-{stem}", backend.as_str(), mode.as_str()))
        .with_extension(extension)
}

fn sanitize_artifact_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn command_line(program: &Path, args: &[OsString]) -> String {
    std::iter::once(program.as_os_str())
        .chain(args.iter().map(OsString::as_os_str))
        .map(|value| value.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

fn background_hex(background: Rgba) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        background.r, background.g, background.b, background.a
    )
}

const fn kib_to_bytes(kib: u64) -> u64 {
    kib * 1024
}

fn native_backend_version(profile: NativeProfile) -> String {
    format!(
        "ferrugo-native {} ({})",
        env!("CARGO_PKG_VERSION"),
        profile.as_str()
    )
}

fn pdfium_backend_version() -> String {
    #[cfg(feature = "pdfium")]
    {
        env::var("FERRUGO_PDFIUM_LIBRARY")
            .map(|path| format!("pdfium via {path}"))
            .unwrap_or_else(|_| "pdfium feature enabled; library not configured".to_string())
    }
    #[cfg(not(feature = "pdfium"))]
    {
        "pdfium feature disabled".to_string()
    }
}

fn poppler_backend_version(command: &Path) -> String {
    format!("pdftoppm {}", command.display())
}

#[cfg(feature = "pdfium")]
fn pdfium_matrix_available() -> bool {
    env::var_os("FERRUGO_PDFIUM_LIBRARY").is_some()
}

#[cfg(not(feature = "pdfium"))]
const fn pdfium_matrix_available() -> bool {
    false
}

fn pdfium_missing_message() -> String {
    #[cfg(feature = "pdfium")]
    {
        "PDFium feature is enabled, but FERRUGO_PDFIUM_LIBRARY is not set".to_string()
    }
    #[cfg(not(feature = "pdfium"))]
    {
        PDFIUM_FEATURE_MESSAGE.to_string()
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
        && error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported
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

fn poppler_visual_diff_report<N: ThumbnailBackend>(
    native: &N,
    paths: &[PathBuf],
    options: &ThumbnailOptions,
    manifest: Option<&CorpusManifest>,
    thresholds: VisualDiffThresholds,
) -> PopplerVisualDiffReport {
    let mut families = BTreeMap::new();
    let mut fixtures = Vec::with_capacity(paths.len());
    let mut exact = 0;
    let mut accepted_drift = 0;
    let mut blockers = 0;
    let mut native_errors = 0;
    let mut reference_errors = 0;
    let mut both_errors = 0;
    let mut subsystems = BTreeMap::new();

    for path in paths {
        let path_key = normalize_manifest_path(path);
        let family = manifest
            .and_then(|manifest| manifest.family_for_path(path))
            .unwrap_or("unclassified")
            .to_string();
        let record =
            poppler_visual_diff_fixture(native, path, options, path_key, family, thresholds);
        match record.status {
            PopplerVisualDiffStatus::Exact => exact += 1,
            PopplerVisualDiffStatus::AcceptedDrift => accepted_drift += 1,
            PopplerVisualDiffStatus::Blocker => blockers += 1,
            PopplerVisualDiffStatus::NativeError => native_errors += 1,
            PopplerVisualDiffStatus::ReferenceError => reference_errors += 1,
            PopplerVisualDiffStatus::BothError => both_errors += 1,
        }
        families
            .entry(record.family.clone())
            .or_insert_with(PopplerFamilyVisualDiffSummary::default)
            .record(&record);
        subsystems
            .entry(record.subsystem.to_string())
            .or_insert_with(PopplerFamilyVisualDiffSummary::default)
            .record(&record);
        fixtures.push(record);
    }

    PopplerVisualDiffReport {
        platform: PlatformMetadata::current(),
        thresholds,
        total: paths.len(),
        exact,
        accepted_drift,
        blockers,
        native_errors,
        reference_errors,
        both_errors,
        families,
        subsystems,
        fixtures,
    }
}

fn poppler_visual_diff_fixture<N: ThumbnailBackend>(
    native: &N,
    path: &Path,
    options: &ThumbnailOptions,
    path_key: String,
    family: String,
    thresholds: VisualDiffThresholds,
) -> PopplerVisualDiffRecord {
    let native_result = native.render(PdfSource::from_path(path), options);
    let target_dimensions = native_result
        .as_ref()
        .ok()
        .map(PopplerTargetDimensions::from);
    let reference_result = render_poppler_ppm(
        path,
        options,
        target_dimensions,
        PopplerScaleMode::TargetDimensions,
    );
    let subsystem = visual_diff_subsystem(path_key.as_str(), family.as_str());

    match (native_result, reference_result) {
        (Ok(native), Ok(reference)) => {
            let mut record = poppler_visual_diff_record_from_thumbnails(
                path_key, family, subsystem, thresholds, &native, reference,
            );
            if record.status == PopplerVisualDiffStatus::Blocker && target_dimensions.is_some() {
                if let Ok(reference) = render_poppler_ppm(
                    path,
                    options,
                    target_dimensions,
                    PopplerScaleMode::UniformMaxDimension,
                ) {
                    let alternate = poppler_visual_diff_record_from_thumbnails(
                        record.path.clone(),
                        record.family.clone(),
                        record.subsystem,
                        thresholds,
                        &native,
                        reference,
                    );
                    if alternate.status != PopplerVisualDiffStatus::Blocker {
                        record = alternate;
                    }
                }
            }
            record
        }
        (Err(native), Ok(_)) => PopplerVisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: PopplerVisualDiffStatus::NativeError,
            metrics: None,
            comparison_error: None,
            native_error: Some(VisualDiffError {
                class: native.class().as_str(),
                message: native.to_string(),
            }),
            reference_error: None,
        },
        (Ok(_), Err(reference)) => PopplerVisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: PopplerVisualDiffStatus::ReferenceError,
            metrics: None,
            comparison_error: None,
            native_error: None,
            reference_error: Some(reference_visual_error(reference)),
        },
        (Err(native), Err(reference)) => PopplerVisualDiffRecord {
            path: path_key,
            family,
            subsystem,
            status: PopplerVisualDiffStatus::BothError,
            metrics: None,
            comparison_error: None,
            native_error: Some(VisualDiffError {
                class: native.class().as_str(),
                message: native.to_string(),
            }),
            reference_error: Some(reference_visual_error(reference)),
        },
    }
}

fn poppler_visual_diff_record_from_thumbnails(
    path: String,
    family: String,
    subsystem: &'static str,
    thresholds: VisualDiffThresholds,
    native: &ferrugo_thumbnail::Thumbnail,
    reference: ferrugo_thumbnail::Thumbnail,
) -> PopplerVisualDiffRecord {
    let metrics = visual_diff_metrics(native, &reference);
    let comparison_error = metrics.is_none().then(|| VisualDiffError {
        class: "dimension_mismatch",
        message: format!(
            "native rendered {}x{} but poppler rendered {}x{}",
            native.width, native.height, reference.width, reference.height
        ),
    });
    let status = metrics
        .as_ref()
        .map(|metrics| classify_visual_diff(metrics, thresholds))
        .map(poppler_status_from_visual_status)
        .unwrap_or(PopplerVisualDiffStatus::Blocker);
    PopplerVisualDiffRecord {
        path,
        family,
        subsystem,
        status,
        metrics,
        comparison_error,
        native_error: None,
        reference_error: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PopplerTargetDimensions {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopplerScaleMode {
    TargetDimensions,
    UniformMaxDimension,
}

impl From<&ferrugo_thumbnail::Thumbnail> for PopplerTargetDimensions {
    fn from(thumbnail: &ferrugo_thumbnail::Thumbnail) -> Self {
        Self {
            width: thumbnail.width,
            height: thumbnail.height,
        }
    }
}

fn poppler_status_from_visual_status(status: VisualDiffStatus) -> PopplerVisualDiffStatus {
    match status {
        VisualDiffStatus::Exact => PopplerVisualDiffStatus::Exact,
        VisualDiffStatus::AcceptedDrift => PopplerVisualDiffStatus::AcceptedDrift,
        VisualDiffStatus::Blocker => PopplerVisualDiffStatus::Blocker,
        #[cfg(feature = "pdfium")]
        VisualDiffStatus::NativeError => PopplerVisualDiffStatus::NativeError,
        #[cfg(feature = "pdfium")]
        VisualDiffStatus::PdfiumError => PopplerVisualDiffStatus::ReferenceError,
        #[cfg(feature = "pdfium")]
        VisualDiffStatus::BothError => PopplerVisualDiffStatus::BothError,
    }
}

fn reference_visual_error(error: CliError) -> VisualDiffError {
    let class = match error {
        CliError::Render { class, .. } => class,
        CliError::Process(_) => "process",
        CliError::Io { .. } | CliError::ReadFile { .. } | CliError::ReadDir { .. } => "io",
        CliError::Usage(_) => "usage",
        CliError::Backend(_) => "backend",
        CliError::Compare(_) => "compare",
        CliError::Benchmark(_) => "benchmark",
        CliError::Encode(_) => "decode",
    };
    VisualDiffError {
        class,
        message: error.to_string(),
    }
}

fn render_poppler_ppm(
    path: &Path,
    options: &ThumbnailOptions,
    target_dimensions: Option<PopplerTargetDimensions>,
    scale_mode: PopplerScaleMode,
) -> Result<ferrugo_thumbnail::Thumbnail, CliError> {
    let command = env::var_os("FERRUGO_POPPLER_PDFTOPPM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("pdftoppm"));
    let temp_dir = env::temp_dir().join(format!(
        "ferrugo-poppler-{}-{}",
        std::process::id(),
        document_identity_hash(path)?
    ));
    let cache_dir = env::temp_dir().join(format!("ferrugo-poppler-cache-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(|source| CliError::Io {
        path: temp_dir.clone(),
        source,
    })?;
    fs::create_dir_all(&cache_dir).map_err(|source| CliError::Io {
        path: cache_dir.clone(),
        source,
    })?;
    let output_prefix = temp_dir.join("page");
    let output_path = output_prefix.with_extension("ppm");
    let page_number = options.page_index.saturating_add(1).to_string();
    let scale_args = poppler_scale_args(options.max_edge, target_dimensions, scale_mode);
    let mut poppler = Command::new(&command);
    poppler
        .arg("-q")
        .args(poppler_page_box_args())
        .arg("-f")
        .arg(page_number.as_str())
        .arg("-l")
        .arg(page_number.as_str())
        .arg("-singlefile")
        .args(scale_args.iter())
        .arg(path)
        .arg(&output_prefix)
        .env("HOME", &cache_dir)
        .env("XDG_CACHE_HOME", &cache_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(fontconfig_file) = poppler_fontconfig_file(&command) {
        poppler.env("FONTCONFIG_FILE", fontconfig_file);
    }
    let mut child = poppler.spawn().map_err(|source| CliError::Io {
        path: command.clone(),
        source,
    })?;
    let status = wait_for_child(&mut child, options.timeout)?;
    if !status.success() {
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_dir(&temp_dir);
        return Err(CliError::Process(format!(
            "`{}` exited with status {status}",
            command.display()
        )));
    }
    let ppm = fs::read(&output_path).map_err(|source| CliError::ReadFile {
        path: output_path.clone(),
        source,
    })?;
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_dir(&temp_dir);
    let thumbnail = decode_ppm_rgb_as_rgba(&ppm)?;
    match scale_mode {
        PopplerScaleMode::TargetDimensions => Ok(thumbnail),
        PopplerScaleMode::UniformMaxDimension => {
            normalize_poppler_target_dimensions(thumbnail, target_dimensions)
        }
    }
}

fn poppler_page_box_args() -> [OsString; 1] {
    [OsString::from("-cropbox")]
}

fn poppler_scale_args(
    max_edge: u32,
    target_dimensions: Option<PopplerTargetDimensions>,
    scale_mode: PopplerScaleMode,
) -> Vec<OsString> {
    match (target_dimensions, scale_mode) {
        (Some(dimensions), PopplerScaleMode::TargetDimensions) => vec![
            OsString::from("-scale-to-x"),
            OsString::from(dimensions.width.to_string()),
            OsString::from("-scale-to-y"),
            OsString::from(dimensions.height.to_string()),
        ],
        (target_dimensions, PopplerScaleMode::UniformMaxDimension) => {
            let scale_to = target_dimensions
                .map(|dimensions| dimensions.width.max(dimensions.height))
                .unwrap_or(max_edge);
            vec![
                OsString::from("-scale-to"),
                OsString::from(scale_to.to_string()),
            ]
        }
        (None, PopplerScaleMode::TargetDimensions) => {
            vec![
                OsString::from("-scale-to"),
                OsString::from(max_edge.to_string()),
            ]
        }
    }
}

fn normalize_poppler_target_dimensions(
    thumbnail: ferrugo_thumbnail::Thumbnail,
    target_dimensions: Option<PopplerTargetDimensions>,
) -> Result<ferrugo_thumbnail::Thumbnail, CliError> {
    let Some(target) = target_dimensions else {
        return Ok(thumbnail);
    };
    if thumbnail.width == target.width && thumbnail.height == target.height {
        return Ok(thumbnail);
    }

    let width_delta = thumbnail.width.abs_diff(target.width);
    let height_delta = thumbnail.height.abs_diff(target.height);
    if width_delta > 1 || height_delta > 1 {
        return Ok(thumbnail);
    }

    let mut bytes = vec![255; target.width as usize * target.height as usize * 4];
    let copy_width = thumbnail.width.min(target.width) as usize;
    let copy_height = thumbnail.height.min(target.height) as usize;
    let source_stride = thumbnail.stride;
    let target_stride = target.width as usize * 4;
    let row_bytes = copy_width * 4;
    for row in 0..copy_height {
        let source_start = row * source_stride;
        let target_start = row * target_stride;
        bytes[target_start..target_start + row_bytes]
            .copy_from_slice(&thumbnail.bytes[source_start..source_start + row_bytes]);
    }

    ferrugo_thumbnail::Thumbnail::rgba(target.width, target.height, bytes)
        .map_err(|err| CliError::Encode(err.to_string()))
}

fn wait_for_child(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, CliError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| {
            CliError::Process(format!("failed to poll child process: {source}"))
        })? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CliError::Render {
                class: "timeout",
                message: format!("pdftoppm exceeded {}s timeout", timeout.as_secs()),
            });
        }
        thread::sleep(WORKER_POLL_INTERVAL);
    }
}

fn poppler_fontconfig_file(command: &Path) -> Option<PathBuf> {
    if let Some(path) = env::var_os("FERRUGO_POPPLER_FONTCONFIG_FILE").map(PathBuf::from) {
        return path.is_file().then_some(path);
    }
    let executable = resolve_command_path(command)?;
    let dependencies_dir = executable.parent()?.parent()?;
    let fontconfig = dependencies_dir
        .join("native")
        .join("poppler")
        .join("poppler")
        .join("etc")
        .join("fonts")
        .join("fonts.conf");
    fontconfig.is_file().then_some(fontconfig)
}

fn resolve_command_path(command: &Path) -> Option<PathBuf> {
    if command.components().count() > 1 {
        return command.is_file().then(|| command.to_path_buf());
    }
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .map(|path| path.join(command))
        .find(|path| path.is_file())
}

fn decode_ppm_rgb_as_rgba(bytes: &[u8]) -> Result<ferrugo_thumbnail::Thumbnail, CliError> {
    let mut index = 0;
    let magic = next_ppm_token(bytes, &mut index)
        .ok_or_else(|| CliError::Encode("PPM is missing magic header".to_string()))?;
    if magic != b"P6" {
        return Err(CliError::Encode(
            "PPM decoder only supports binary P6 data".to_string(),
        ));
    }
    let width = parse_ppm_u32(bytes, &mut index, "width")?;
    let height = parse_ppm_u32(bytes, &mut index, "height")?;
    let max_value = parse_ppm_u32(bytes, &mut index, "max value")?;
    if max_value != 255 {
        return Err(CliError::Encode(
            "PPM decoder only supports 8-bit RGB data".to_string(),
        ));
    }
    if index >= bytes.len() || !bytes[index].is_ascii_whitespace() {
        return Err(CliError::Encode(
            "PPM header must be followed by raster data".to_string(),
        ));
    }
    index += 1;

    let pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| CliError::Encode("PPM dimensions overflow".to_string()))?;
    let expected_rgb_bytes = pixels
        .checked_mul(3)
        .ok_or_else(|| CliError::Encode("PPM byte count overflow".to_string()))?;
    let rgb = bytes
        .get(index..index + expected_rgb_bytes)
        .ok_or_else(|| CliError::Encode("PPM raster data is truncated".to_string()))?;
    if bytes.len() != index + expected_rgb_bytes {
        return Err(CliError::Encode(
            "PPM raster data has trailing bytes".to_string(),
        ));
    }

    let mut rgba = Vec::with_capacity(pixels * 4);
    for pixel in rgb.chunks_exact(3) {
        rgba.extend_from_slice(pixel);
        rgba.push(255);
    }
    ferrugo_thumbnail::Thumbnail::rgba(width, height, rgba)
        .map_err(|err| CliError::Encode(err.to_string()))
}

fn parse_ppm_u32(bytes: &[u8], index: &mut usize, field: &str) -> Result<u32, CliError> {
    let token = next_ppm_token(bytes, index)
        .ok_or_else(|| CliError::Encode(format!("PPM is missing {field}")))?;
    std::str::from_utf8(token)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| CliError::Encode(format!("PPM {field} is not an unsigned integer")))
}

fn next_ppm_token<'a>(bytes: &'a [u8], index: &mut usize) -> Option<&'a [u8]> {
    loop {
        while bytes.get(*index).is_some_and(u8::is_ascii_whitespace) {
            *index += 1;
        }
        if bytes.get(*index) != Some(&b'#') {
            break;
        }
        while bytes.get(*index).is_some_and(|byte| *byte != b'\n') {
            *index += 1;
        }
    }
    let start = *index;
    while bytes
        .get(*index)
        .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'#')
    {
        *index += 1;
    }
    (start != *index).then_some(&bytes[start..*index])
}

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

fn visual_diff_metrics(
    native: &ferrugo_thumbnail::Thumbnail,
    pdfium: &ferrugo_thumbnail::Thumbnail,
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
    let low_amplitude_distribution =
        metrics.p95_channel_delta <= LOW_AMPLITUDE_VISUAL_DRIFT_P95_MAX_DELTA;
    let low_p95_edge_drift = metrics.mean_abs_error <= LOW_P95_EDGE_DRIFT_MAX_MAE
        && metrics.p95_channel_delta <= LOW_P95_EDGE_DRIFT_MAX_DELTA
        && metrics.changed_ratio <= LOW_P95_EDGE_DRIFT_MAX_CHANGED_RATIO;
    if (metrics.mean_abs_error <= thresholds.max_mean_abs_error
        && (bounded_distribution || low_amplitude_field || low_amplitude_distribution))
        || low_p95_edge_drift
    {
        VisualDiffStatus::AcceptedDrift
    } else {
        VisualDiffStatus::Blocker
    }
}

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

fn parse_u8(args: &[OsString], index: usize, option: &str) -> Result<u8, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an integer between 0 and 255")))
}

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
            "\"max_total_font_program_bytes\":{},",
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
        diagnostics.max_total_font_program_bytes,
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
            "  \"path_redacted\": {},\n",
            "  \"manifest\": {},\n",
            "  \"telemetry\": {{\"collection\":\"none\",\"controlled_by\":\"application\",\"default_enabled\":false}},\n",
            "  \"privacy\": {{\"includes_pdf_bytes\":false,\"includes_rendered_pixels\":false,\"includes_document_info\":false,\"includes_text_samples\":false,\"includes_private_paths\":false,\"field_classes\":{{\"path\":\"sensitive\",\"manifest\":\"sensitive\",\"options\":\"safe\",\"metadata\":\"safe\",\"stages\":\"safe\",\"native_memory_diagnostics\":\"safe\"}},\"redaction\":\"share only after local review; private/local-only paths and manifest notes are redacted\"}},\n",
            "  \"options\": {},\n",
            "  \"metadata\": {},\n",
            "  \"stages\": [{{\"name\":\"metadata\",\"elapsed_ms\":{:.3},\"outcome\":{}}},{{\"name\":\"render_pipeline\",\"elapsed_ms\":{:.3},\"stage_hint\":{},\"outcome\":{}}}],\n",
            "  \"native_memory_diagnostics\": {}\n",
            "}}\n"
        ),
        json_string(bundle.path),
        bundle.path_redacted,
        diagnostic_manifest_entry_json(bundle.manifest, bundle.path_redacted),
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

fn diagnostic_manifest_entry_json(entry: Option<&CorpusManifestEntry>, redacted: bool) -> String {
    if redacted {
        "{\"status\":\"redacted\",\"reason\":\"privacy-sensitive-fixture\"}".to_string()
    } else {
        manifest_entry_json(entry)
    }
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
        ferrugo_thumbnail::OutputFormat::Png => "\"png\"",
        ferrugo_thumbnail::OutputFormat::Rgba => "\"rgba\"",
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
    let category = if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported {
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
            if error.class() == ferrugo_thumbnail::ThumbnailErrorClass::Unsupported {
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
        ferrugo_thumbnail::ThumbnailErrorClass::Malformed
        | ferrugo_thumbnail::ThumbnailErrorClass::Encrypted => "parser-or-object",
        ferrugo_thumbnail::ThumbnailErrorClass::Unsupported => {
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
        ferrugo_thumbnail::ThumbnailErrorClass::Timeout => "timeout",
        ferrugo_thumbnail::ThumbnailErrorClass::Internal => "internal",
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

fn pdf20_usage_report_json(report: &Pdf20UsageReport) -> String {
    let families = report
        .families
        .iter()
        .map(|(family, summary)| format!("{}:{}", json_string(family), pdf20_family_json(summary)))
        .collect::<Vec<_>>()
        .join(",");
    let followups = report
        .followups
        .iter()
        .map(pdf20_followup_json)
        .collect::<Vec<_>>()
        .join(",");
    let fixtures = report
        .fixtures
        .iter()
        .map(pdf20_usage_record_json)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"report_kind\": \"pdf-2-0-feature-usage\",\n",
            "  \"privacy\": \"no PDF bytes, rendered pixels, text samples, or stream operands\",\n",
            "  \"total_scanned\": {},\n",
            "  \"pdf20_documents\": {},\n",
            "  \"native_rendered\": {},\n",
            "  \"typed_unsupported\": {},\n",
            "  \"errors\": {},\n",
            "  \"feature_counts\": {},\n",
            "  \"visual_impact_counts\": {},\n",
            "  \"families\": {{{}}},\n",
            "  \"followups\": [{}],\n",
            "  \"fixtures\": [{}]\n",
            "}}\n"
        ),
        report.total_scanned,
        report.pdf20_documents,
        report.native_rendered,
        report.typed_unsupported,
        report.errors,
        string_count_map_json(&report.feature_counts),
        count_map_json(&report.impact_counts),
        families,
        followups,
        fixtures
    )
}

fn pdf20_family_json(summary: &Pdf20FamilySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"pdf20_documents\":{},",
            "\"native_rendered\":{},",
            "\"typed_unsupported\":{},",
            "\"errors\":{}",
            "}}"
        ),
        summary.total,
        summary.pdf20_documents,
        summary.native_rendered,
        summary.typed_unsupported,
        summary.errors
    )
}

fn pdf20_followup_json(followup: &Pdf20Followup) -> String {
    format!(
        concat!(
            "{{",
            "\"rank\":{},",
            "\"feature\":{},",
            "\"observed_documents\":{},",
            "\"visual_impact\":{},",
            "\"bucket\":{},",
            "\"recommendation\":{}",
            "}}"
        ),
        followup.rank,
        json_string(&followup.feature),
        followup.observed_documents,
        json_string(followup.visual_impact),
        optional_json_string(followup.bucket),
        json_string(followup.recommendation)
    )
}

fn pdf20_usage_record_json(record: &Pdf20UsageRecord) -> String {
    let features = record
        .features
        .iter()
        .map(pdf20_feature_observation_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"manifest_features\":{},",
            "\"version\":{},",
            "\"features\":[{}],",
            "\"render\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        json_string_array(&record.manifest_features),
        pdf20_version_json(&record.version),
        features,
        pdf20_render_outcome_json(record.render)
    )
}

fn pdf20_version_json(version: &Pdf20VersionEvidence) -> String {
    format!(
        concat!(
            "{{",
            "\"header_version\":{},",
            "\"catalog_version_20\":{},",
            "\"manifest_pdf20_tag\":{},",
            "\"detected_pdf20\":{}",
            "}}"
        ),
        optional_json_string(version.header_version.as_deref()),
        version.catalog_version_20,
        version.manifest_pdf20_tag,
        version.detected_pdf20
    )
}

fn pdf20_feature_observation_json(feature: &Pdf20FeatureObservation) -> String {
    format!(
        concat!(
            "{{",
            "\"feature\":{},",
            "\"policy\":{},",
            "\"visual_impact\":{},",
            "\"bucket\":{}",
            "}}"
        ),
        json_string(feature.feature),
        json_string(feature.policy),
        json_string(feature.visual_impact),
        optional_json_string(feature.bucket)
    )
}

fn pdf20_render_outcome_json(outcome: Pdf20RenderOutcome) -> String {
    match outcome {
        Pdf20RenderOutcome::NotPdf20 => "{\"status\":\"not_pdf20\"}".to_string(),
        Pdf20RenderOutcome::NativeRendered => "{\"status\":\"native_rendered\"}".to_string(),
        Pdf20RenderOutcome::TypedUnsupported { bucket } => format!(
            "{{\"status\":\"typed_unsupported\",\"bucket\":{}}}",
            json_string(bucket)
        ),
        Pdf20RenderOutcome::Error { class } => {
            format!("{{\"status\":\"error\",\"class\":{}}}", json_string(class))
        }
    }
}

fn operator_coverage_report_json(report: &CorpusOperatorCoverageReport) -> String {
    let operators = report
        .operators
        .iter()
        .map(|(operator, summary)| {
            format!(
                "{}:{}",
                json_string(operator),
                operator_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let families = report
        .families
        .iter()
        .map(|(family, summary)| {
            format!("{}:{}", json_string(family), operator_family_json(summary))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fixtures = report
        .fixtures
        .iter()
        .map(operator_coverage_record_json)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"page_index\": {},\n",
            "  \"include_annotations\": {},\n",
            "  \"summary\": {{\"total\":{},\"scanned\":{},\"errors\":{},\"total_operators\":{},\"inline_images\":{},\"status_counts\":{}}},\n",
            "  \"operators\": {{{}}},\n",
            "  \"families\": {{{}}},\n",
            "  \"fixtures\": [{}]\n",
            "}}\n"
        ),
        report.page_index,
        report.include_annotations,
        report.total,
        report.scanned,
        report.errors,
        report.total_operators,
        report.inline_images,
        operator_status_counts_json(&report.status_counts),
        operators,
        families,
        fixtures
    )
}

fn operator_summary_json(summary: &CorpusOperatorSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"count\":{},",
            "\"status\":{},",
            "\"fallback_bucket\":{}",
            "}}"
        ),
        summary.count,
        json_string(summary.status.as_str()),
        optional_json_string(summary.fallback_bucket)
    )
}

fn operator_family_json(summary: &CorpusOperatorFamilySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"scanned\":{},",
            "\"errors\":{},",
            "\"total_operators\":{},",
            "\"inline_images\":{},",
            "\"status_counts\":{}",
            "}}"
        ),
        summary.total,
        summary.scanned,
        summary.errors,
        summary.total_operators,
        summary.inline_images,
        operator_status_counts_json(&summary.status_counts)
    )
}

fn operator_status_counts_json(counts: &OperatorStatusCounts) -> String {
    format!(
        concat!(
            "{{",
            "\"implemented\":{},",
            "\"partial\":{},",
            "\"unsupported\":{},",
            "\"ignored\":{}",
            "}}"
        ),
        counts.implemented, counts.partial, counts.unsupported, counts.ignored
    )
}

fn operator_coverage_record_json(record: &CorpusOperatorCoverageRecord) -> String {
    format!(
        "{{\"path\":{},\"family\":{},\"outcome\":{}}}",
        json_string(&record.path),
        json_string(&record.family),
        operator_coverage_outcome_json(&record.outcome)
    )
}

fn operator_coverage_outcome_json(outcome: &CorpusOperatorCoverageOutcome) -> String {
    match outcome {
        CorpusOperatorCoverageOutcome::Scanned {
            streams_scanned,
            total_operators,
            inline_images,
            operators,
        } => {
            let operators = operators
                .iter()
                .map(operator_entry_json)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                concat!(
                    "{{",
                    "\"status\":\"scanned\",",
                    "\"streams_scanned\":{},",
                    "\"total_operators\":{},",
                    "\"inline_images\":{},",
                    "\"operators\":[{}]",
                    "}}"
                ),
                streams_scanned, total_operators, inline_images, operators
            )
        }
        CorpusOperatorCoverageOutcome::Error { class, message } => format!(
            "{{\"status\":\"error\",\"class\":{},\"message\":{}}}",
            json_string(class),
            json_string(message)
        ),
    }
}

fn operator_entry_json(entry: &OperatorCoverageEntry) -> String {
    format!(
        concat!(
            "{{",
            "\"operator\":{},",
            "\"count\":{},",
            "\"status\":{},",
            "\"fallback_bucket\":{}",
            "}}"
        ),
        json_string(&entry.operator),
        entry.count,
        json_string(entry.status.as_str()),
        optional_json_string(entry.fallback_bucket)
    )
}

fn native_render_trace_json(config: &TraceNativeConfig) -> Result<String, CliError> {
    let bytes = fs::read(&config.input).map_err(|source| CliError::ReadFile {
        path: config.input.clone(),
        source,
    })?;
    let native = NativeBackend::new();
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: Rgba::WHITE,
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: DEFAULT_TIMEOUT,
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    };
    let metadata = DocumentMetadataBackend::inspect(&native, PdfSource::from_bytes(&bytes));
    let coverage = scan_operator_coverage(
        &bytes,
        OperatorCoverageOptions {
            page_index: config.page_index,
            include_annotations: config.include_annotations,
        },
    );
    let render_trace = native.render_with_trace(PdfSource::from_bytes(&bytes), &options);

    let (coverage_json, operators, events_json, emitted_events, total_events, events_truncated) =
        match coverage {
            Ok(coverage) => {
                let (events_json, emitted_events, total_events, events_truncated) =
                    trace_operator_events_json(&coverage.operators, config.max_events);
                (
                    format!(
                        concat!(
                            "{{",
                            "\"status\":\"scanned\",",
                            "\"streams_scanned\":{},",
                            "\"total_operators\":{},",
                            "\"inline_images\":{},",
                            "\"operators\":[{}]",
                            "}}"
                        ),
                        coverage.streams_scanned,
                        coverage.total_operators,
                        coverage.inline_images,
                        coverage
                            .operators
                            .iter()
                            .map(operator_entry_json)
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    coverage.operators,
                    events_json,
                    emitted_events,
                    total_events,
                    events_truncated,
                )
            }
            Err(error) => (
                format!(
                    concat!(
                        "{{",
                        "\"status\":\"error\",",
                        "\"class\":{},",
                        "\"bucket\":{},",
                        "\"message\":{}",
                        "}}"
                    ),
                    json_string(error.class().as_str()),
                    optional_json_string(error.unsupported_feature_bucket()),
                    json_string(&error.to_string())
                ),
                Vec::new(),
                String::new(),
                0,
                0,
                false,
            ),
        };
    let operator_summary = trace_operator_summary_json(&operators);
    let metadata_json = trace_metadata_json(metadata);
    let phase_timings_json =
        trace_phase_timings_json(render_trace.as_ref().map(|trace| &trace.timings));
    let stroke_shape_summary_json =
        trace_stroke_shape_summary_json(render_trace.as_ref().map(|trace| &trace.stroke_shapes));
    let render_json = trace_render_outcome_json(render_trace);

    Ok(format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"trace_kind\": \"native-render-trace\",\n",
            "  \"privacy\": \"no document bytes, stream bytes, operands, text, or image samples\",\n",
            "  \"input\": {},\n",
            "  \"page_index\": {},\n",
            "  \"max_edge\": {},\n",
            "  \"include_annotations\": {},\n",
            "  \"max_events\": {},\n",
            "  \"events_emitted\": {},\n",
            "  \"events_total\": {},\n",
            "  \"events_truncated\": {},\n",
            "  \"metadata\": {},\n",
            "  \"render\": {},\n",
            "  \"phase_timings_ms\": {},\n",
            "  \"stroke_shape_summary\": {},\n",
            "  \"operator_coverage\": {},\n",
            "  \"operator_summary\": {},\n",
            "  \"events\": [{}]\n",
            "}}\n"
        ),
        json_string(&normalize_manifest_path(&config.input)),
        config.page_index,
        config.max_edge,
        config.include_annotations,
        config.max_events,
        emitted_events,
        total_events,
        events_truncated,
        metadata_json,
        render_json,
        phase_timings_json,
        stroke_shape_summary_json,
        coverage_json,
        operator_summary,
        events_json
    ))
}

fn trace_metadata_json(result: Result<DocumentMetadata, ThumbnailError>) -> String {
    match result {
        Ok(metadata) => {
            let first_page = metadata
                .first_page_size()
                .map(|size| format!("{{\"width\":{},\"height\":{}}}", size.width, size.height))
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{\"status\":\"ok\",\"page_count\":{},\"first_page_size\":{}}}",
                metadata.page_count(),
                first_page
            )
        }
        Err(error) => format!(
            "{{\"status\":\"error\",\"class\":{},\"bucket\":{},\"message\":{}}}",
            json_string(error.class().as_str()),
            optional_json_string(error.unsupported_feature_bucket()),
            json_string(&error.to_string())
        ),
    }
}

fn trace_render_outcome_json(result: Result<NativeRenderTrace, ThumbnailError>) -> String {
    match result {
        Ok(trace) => format!(
            concat!(
                "{{",
                "\"status\":\"rendered\",",
                "\"width\":{},",
                "\"height\":{},",
                "\"stride\":{},",
                "\"output_bytes\":{}",
                "}}"
            ),
            trace.thumbnail.width,
            trace.thumbnail.height,
            trace.thumbnail.stride,
            trace.thumbnail.bytes.len()
        ),
        Err(error) => format!(
            "{{\"status\":\"error\",\"class\":{},\"bucket\":{},\"message\":{}}}",
            json_string(error.class().as_str()),
            optional_json_string(error.unsupported_feature_bucket()),
            json_string(&error.to_string())
        ),
    }
}

fn trace_stroke_shape_summary_json(
    summary: Result<&StrokeShapeSummary, &ThumbnailError>,
) -> String {
    match summary {
        Ok(summary) => format!(
            concat!(
                "{{",
                "\"status\":\"measured\",",
                "\"stroked_items\":{},",
                "\"dashed_items\":{},",
                "\"row_bucket_candidate_items\":{},",
                "\"flattened_lines\":{},",
                "\"axis_aligned_lines\":{},",
                "\"row_index_refs\":{},",
                "\"max_lines_per_item\":{},",
                "\"max_row_index_refs_per_item\":{},",
                "\"line_count_buckets\":{{",
                "\"lt_32\":{},",
                "\"from_32_to_127\":{},",
                "\"ge_128\":{}",
                "}},",
                "\"pixel_x_span_buckets\":{{",
                "\"le_16\":{},",
                "\"le_32\":{},",
                "\"le_64\":{},",
                "\"gt_64\":{}",
                "}}",
                "}}"
            ),
            summary.stroked_items,
            summary.dashed_items,
            summary.row_bucket_candidate_items,
            summary.flattened_lines,
            summary.axis_aligned_lines,
            summary.row_index_refs,
            summary.max_lines_per_item,
            summary.max_row_index_refs_per_item,
            summary.line_count_buckets.lt_32,
            summary.line_count_buckets.from_32_to_127,
            summary.line_count_buckets.ge_128,
            summary.pixel_x_span_buckets.le_16,
            summary.pixel_x_span_buckets.le_32,
            summary.pixel_x_span_buckets.le_64,
            summary.pixel_x_span_buckets.gt_64
        ),
        Err(error) => format!(
            "{{\"status\":\"error\",\"class\":{},\"bucket\":{}}}",
            json_string(error.class().as_str()),
            optional_json_string(error.unsupported_feature_bucket())
        ),
    }
}

fn trace_phase_timings_json(timings: Result<&NativeRenderPhaseTimings, &ThumbnailError>) -> String {
    match timings {
        Ok(timings) => format!(
            concat!(
                "{{",
                "\"status\":\"measured\",",
                "\"load_xref_object\":{:.3},",
                "\"stream_decode\":{:.3},",
                "\"content_tokenize\":{:.3},",
                "\"display_list_build\":{:.3},",
                "\"resource_decode\":{:.3},",
                "\"raster_paths\":{:.3},",
                "\"raster_text\":{:.3},",
                "\"raster_images\":{:.3},",
                "\"output\":{:.3},",
                "\"total\":{:.3}",
                "}}"
            ),
            elapsed_ms(timings.load_xref_object),
            elapsed_ms(timings.stream_decode),
            elapsed_ms(timings.content_tokenize),
            elapsed_ms(timings.display_list_build),
            elapsed_ms(timings.resource_decode),
            elapsed_ms(timings.raster_paths),
            elapsed_ms(timings.raster_text),
            elapsed_ms(timings.raster_images),
            elapsed_ms(timings.output),
            elapsed_ms(timings.total)
        ),
        Err(error) => format!(
            "{{\"status\":\"error\",\"class\":{},\"bucket\":{}}}",
            json_string(error.class().as_str()),
            optional_json_string(error.unsupported_feature_bucket())
        ),
    }
}

fn trace_operator_summary_json(operators: &[OperatorCoverageEntry]) -> String {
    let entries = operators
        .iter()
        .map(|entry| format!("{}:{}", json_string(&entry.operator), entry.count))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{entries}}}")
}

fn trace_operator_events_json(
    operators: &[OperatorCoverageEntry],
    max_events: usize,
) -> (String, usize, usize, bool) {
    let total_events = operators.iter().map(|entry| entry.count).sum();
    let mut emitted = 0usize;
    let mut events = Vec::new();

    'outer: for entry in operators {
        for _ in 0..entry.count {
            if emitted == max_events {
                break 'outer;
            }
            events.push(format!(
                concat!(
                    "{{",
                    "\"seq\":{},",
                    "\"phase\":\"operator\",",
                    "\"operator\":{},",
                    "\"status\":{},",
                    "\"fallback_bucket\":{}",
                    "}}"
                ),
                emitted,
                json_string(&entry.operator),
                json_string(entry.status.as_str()),
                optional_json_string(entry.fallback_bucket)
            ));
            emitted += 1;
        }
    }

    (
        events.join(","),
        emitted,
        total_events,
        emitted < total_events,
    )
}

fn replay_operator_trace_json(trace: &str) -> Result<String, CliError> {
    if !trace.contains("\"trace_kind\": \"native-render-trace\"")
        && !trace.contains("\"trace_kind\":\"native-render-trace\"")
    {
        return Err(CliError::Usage(
            "trace file is not a native-render-trace".to_string(),
        ));
    }
    let counts = replay_operator_counts(trace);
    let events_replayed: usize = counts.values().sum();
    let operators = counts
        .iter()
        .map(|(operator, count)| format!("{}:{}", json_string(operator), count))
        .collect::<Vec<_>>()
        .join(",");

    Ok(format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"trace_kind\": \"operator-replay\",\n",
            "  \"events_replayed\": {},\n",
            "  \"operator_counts\": {{{}}}\n",
            "}}\n"
        ),
        events_replayed, operators
    ))
}

fn replay_operator_counts(trace: &str) -> BTreeMap<String, usize> {
    let marker = "\"phase\":\"operator\",\"operator\":";
    let mut counts = BTreeMap::new();
    let mut remaining = trace;

    while let Some(position) = remaining.find(marker) {
        let value = &remaining[position + marker.len()..];
        let Some((operator, consumed)) = parse_json_string_value(value) else {
            break;
        };
        *counts.entry(operator).or_insert(0) += 1;
        remaining = &value[consumed..];
    }

    counts
}

fn parse_json_string_value(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('"') {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    for (offset, character) in input[1..].char_indices() {
        if escaped {
            value.push(match character {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '"' => return Some((value, offset + 2)),
            other => value.push(other),
        }
    }
    None
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

fn benchmark_matrix_report_json(report: &BenchmarkMatrixReport) -> String {
    let records = report
        .records
        .iter()
        .map(benchmark_matrix_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"report_kind\": \"renderer-performance-matrix\",\n",
            "  \"privacy\": {{\"includes_pdf_bytes\":false,\"includes_rendered_pixels\":false}},\n",
            "  \"command\": {},\n",
            "  \"platform\": {},\n",
            "  \"config\": {},\n",
            "  \"timing_reliability\": {},\n",
            "  \"summary\": {},\n",
            "  \"families\": {},\n",
            "  \"records\": [{}]\n",
            "}}\n"
        ),
        json_string(&report.command),
        platform_metadata_json(&report.platform),
        benchmark_matrix_config_json(&report.config),
        benchmark_matrix_timing_reliability_json(&benchmark_matrix_timing_reliability(report)),
        benchmark_matrix_summary_json(&report.summary),
        benchmark_matrix_families_json(&report.families),
        records
    )
}

fn benchmark_matrix_config_json(config: &BenchmarkMatrixReportConfig) -> String {
    let backends = config
        .backends
        .iter()
        .map(|backend| json_string(backend.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    let modes = config
        .modes
        .iter()
        .map(|mode| json_string(mode.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"input\":{},",
            "\"manifest\":{},",
            "\"include_families\":{},",
            "\"page_index\":{},",
            "\"max_edge\":{},",
            "\"timeout_secs\":{},",
            "\"iterations\":{},",
            "\"warmup\":{},",
            "\"backends\":[{}],",
            "\"modes\":[{}],",
            "\"native_profile\":{}",
            "}}"
        ),
        json_string(&config.input),
        optional_json_string(config.manifest.as_deref()),
        json_string_array(&config.include_families),
        config.page_index,
        config.max_edge,
        config.timeout_secs,
        config.iterations,
        config.warmup,
        backends,
        modes,
        json_string(config.native_profile.as_str())
    )
}

fn benchmark_matrix_summary_json(summary: &BenchmarkMatrixSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total_records\":{},",
            "\"rendered\":{},",
            "\"fallback_required\":{},",
            "\"missing_tool\":{},",
            "\"not_applicable\":{},",
            "\"errors\":{}",
            "}}"
        ),
        summary.total_records,
        summary.rendered,
        summary.fallback_required,
        summary.missing_tool,
        summary.not_applicable,
        summary.errors
    )
}

fn benchmark_matrix_timing_reliability(
    report: &BenchmarkMatrixReport,
) -> BenchmarkMatrixTimingReliability {
    let pdfium_requested = report.config.backends.contains(&MatrixBackend::Pdfium);
    let poppler_requested = report.config.backends.contains(&MatrixBackend::Poppler);
    let hot_requested = report.config.modes.contains(&MatrixMode::HotRender);
    let cold_requested = report.config.modes.contains(&MatrixMode::ColdProcess);

    let rss_available = report.records.iter().any(|record| {
        record.memory.rss_start_bytes.is_some()
            || record.memory.rss_peak_bytes.is_some()
            || record.memory.rss_end_bytes.is_some()
    });
    let pdfium_available = report.records.iter().any(|record| {
        record.backend == MatrixBackend::Pdfium
            && !matches!(
                record.status,
                MatrixStatus::MissingTool | MatrixStatus::NotApplicable
            )
    });
    let poppler_available = report.records.iter().any(|record| {
        record.backend == MatrixBackend::Poppler
            && !matches!(
                record.status,
                MatrixStatus::MissingTool | MatrixStatus::NotApplicable
            )
    });
    let native_hot_available = report.records.iter().any(|record| {
        record.backend == MatrixBackend::Native
            && record.mode == MatrixMode::HotRender
            && record.status == MatrixStatus::Rendered
    });
    let pdfium_hot_available = report.records.iter().any(|record| {
        record.backend == MatrixBackend::Pdfium
            && record.mode == MatrixMode::HotRender
            && record.status == MatrixStatus::Rendered
    });
    let cold_reference_available = report.records.iter().any(|record| {
        matches!(
            record.backend,
            MatrixBackend::Pdfium | MatrixBackend::Poppler
        ) && record.mode == MatrixMode::ColdProcess
            && record.status == MatrixStatus::Rendered
    });

    let hot_pdfium_comparison_available = native_hot_available && pdfium_hot_available;
    let mut caveats = Vec::new();
    if !rss_available {
        caveats.push("rss-unavailable");
    }
    if pdfium_requested && !pdfium_available {
        caveats.push("pdfium-missing-tool");
    }
    if poppler_requested && cold_requested && !poppler_available {
        caveats.push("poppler-missing-tool");
    }
    if poppler_requested && hot_requested {
        caveats.push("poppler-hot-render-external-only");
    }
    if hot_requested && pdfium_requested && !hot_pdfium_comparison_available {
        caveats.push("pdfium-hot-reference-unavailable");
    } else if hot_requested && !pdfium_requested {
        caveats.push("pdfium-hot-reference-not-requested");
    }
    if cold_requested && !cold_reference_available {
        caveats.push("cold-reference-unavailable");
    }

    BenchmarkMatrixTimingReliability {
        rss_available,
        pdfium_requested,
        pdfium_available,
        poppler_requested,
        poppler_available,
        hot_pdfium_comparison_available,
        cold_reference_available,
        caveats,
    }
}

fn benchmark_matrix_timing_reliability_json(
    reliability: &BenchmarkMatrixTimingReliability,
) -> String {
    format!(
        concat!(
            "{{",
            "\"rss_available\":{},",
            "\"pdfium_requested\":{},",
            "\"pdfium_available\":{},",
            "\"poppler_requested\":{},",
            "\"poppler_available\":{},",
            "\"hot_pdfium_comparison_available\":{},",
            "\"cold_reference_available\":{},",
            "\"caveats\":{}",
            "}}"
        ),
        reliability.rss_available,
        reliability.pdfium_requested,
        reliability.pdfium_available,
        reliability.poppler_requested,
        reliability.poppler_available,
        reliability.hot_pdfium_comparison_available,
        reliability.cold_reference_available,
        json_str_array(&reliability.caveats)
    )
}

fn benchmark_matrix_families_json(
    families: &BTreeMap<String, BenchmarkMatrixFamilySummary>,
) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                benchmark_matrix_family_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn benchmark_matrix_family_json(summary: &BenchmarkMatrixFamilySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"rendered\":{},",
            "\"fallback_required\":{},",
            "\"missing_tool\":{},",
            "\"not_applicable\":{},",
            "\"errors\":{},",
            "\"native_hot_p95_ms\":{},",
            "\"native_cold_wall_ms\":{},",
            "\"pdfium_hot_p95_ms\":{},",
            "\"pdfium_cold_wall_ms\":{},",
            "\"poppler_cold_wall_ms\":{},",
            "\"ferrugo_to_pdfium_hot_ratio\":{},",
            "\"ferrugo_to_pdfium_cold_ratio\":{},",
            "\"ferrugo_to_poppler_cold_ratio\":{}",
            "}}"
        ),
        summary.total,
        summary.rendered,
        summary.fallback_required,
        summary.missing_tool,
        summary.not_applicable,
        summary.errors,
        optional_json_f64(summary.native_hot_p95_ms),
        optional_json_f64(summary.native_cold_wall_ms),
        optional_json_f64(summary.pdfium_hot_p95_ms),
        optional_json_f64(summary.pdfium_cold_wall_ms),
        optional_json_f64(summary.poppler_cold_wall_ms),
        optional_json_f64(summary.ferrugo_to_pdfium_hot_ratio),
        optional_json_f64(summary.ferrugo_to_pdfium_cold_ratio),
        optional_json_f64(summary.ferrugo_to_poppler_cold_ratio)
    )
}

fn benchmark_matrix_record_json(record: &BenchmarkMatrixRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"backend\":{},",
            "\"backend_version\":{},",
            "\"command\":{},",
            "\"mode\":{},",
            "\"fixture\":{},",
            "\"family\":{},",
            "\"page_index\":{},",
            "\"status\":{},",
            "\"exit_status\":{},",
            "\"timing\":{},",
            "\"output\":{},",
            "\"memory\":{},",
            "\"error_class\":{},",
            "\"error_message\":{},",
            "\"fallback_bucket\":{}",
            "}}"
        ),
        json_string(record.backend.as_str()),
        json_string(&record.backend_version),
        json_string(&record.command),
        json_string(record.mode.as_str()),
        json_string(&record.fixture),
        json_string(&record.family),
        record.page_index,
        json_string(record.status.as_str()),
        optional_json_i32(record.exit_status),
        matrix_timing_json(&record.timing),
        matrix_output_json(&record.output),
        matrix_memory_json(&record.memory),
        optional_json_string(record.error_class.as_deref()),
        optional_json_string(record.error_message.as_deref()),
        optional_json_string(record.fallback_bucket.as_deref())
    )
}

fn matrix_timing_json(timing: &MatrixTiming) -> String {
    format!(
        concat!(
            "{{",
            "\"wall_ms\":{},",
            "\"warmup_iterations\":{},",
            "\"measured_iterations\":{},",
            "\"samples_ms\":{},",
            "\"mean_ms\":{},",
            "\"p50_ms\":{},",
            "\"p95_ms\":{},",
            "\"max_ms\":{}",
            "}}"
        ),
        optional_json_f64(timing.wall_ms),
        timing.warmup_iterations,
        timing.measured_iterations,
        float_array_json(&timing.samples_ms),
        optional_json_f64(timing.mean_ms),
        optional_json_f64(timing.p50_ms),
        optional_json_f64(timing.p95_ms),
        optional_json_f64(timing.max_ms)
    )
}

fn matrix_output_json(output: &MatrixOutput) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"bytes\":{}}}",
        optional_json_u32(output.width),
        optional_json_u32(output.height),
        optional_json_u64(output.bytes)
    )
}

fn matrix_memory_json(memory: &MatrixMemory) -> String {
    format!(
        concat!(
            "{{",
            "\"rss_start_bytes\":{},",
            "\"rss_peak_bytes\":{},",
            "\"rss_end_bytes\":{},",
            "\"source\":{}",
            "}}"
        ),
        optional_json_u64(memory.rss_start_bytes),
        optional_json_u64(memory.rss_peak_bytes),
        optional_json_u64(memory.rss_end_bytes),
        json_string(memory.source)
    )
}

fn benchmark_matrix_markdown_report(report: &BenchmarkMatrixReport) -> String {
    let mut markdown = String::new();
    let timing_reliability = benchmark_matrix_timing_reliability(report);
    markdown.push_str("# Ferrugo Renderer Performance Matrix\n\n");
    markdown.push_str("Generated by `ferrugo-cli benchmark-matrix`.\n\n");
    markdown.push_str("## Timing Reliability\n\n");
    markdown.push_str("| Signal | Value |\n| --- | --- |\n");
    markdown.push_str(&format!(
        "| RSS samples available | {} |\n| PDFium requested | {} |\n| PDFium available | {} |\n| Poppler requested | {} |\n| Poppler available | {} |\n| Hot PDFium comparison available | {} |\n| Cold reference available | {} |\n\n",
        markdown_bool(timing_reliability.rss_available),
        markdown_bool(timing_reliability.pdfium_requested),
        markdown_bool(timing_reliability.pdfium_available),
        markdown_bool(timing_reliability.poppler_requested),
        markdown_bool(timing_reliability.poppler_available),
        markdown_bool(timing_reliability.hot_pdfium_comparison_available),
        markdown_bool(timing_reliability.cold_reference_available)
    ));
    if timing_reliability.caveats.is_empty() {
        markdown.push_str("Caveats: none.\n\n");
    } else {
        let caveats = timing_reliability
            .caveats
            .iter()
            .map(|caveat| format!("`{caveat}`"))
            .collect::<Vec<_>>()
            .join(", ");
        markdown.push_str(&format!("Caveats: {caveats}.\n\n"));
    }
    markdown.push_str("## Summary\n\n");
    markdown.push_str("| Metric | Count |\n| --- | ---: |\n");
    markdown.push_str(&format!(
        "| Records | {} |\n| Rendered | {} |\n| Fallback required | {} |\n| Missing tool | {} |\n| Not applicable | {} |\n| Errors | {} |\n\n",
        report.summary.total_records,
        report.summary.rendered,
        report.summary.fallback_required,
        report.summary.missing_tool,
        report.summary.not_applicable,
        report.summary.errors
    ));

    markdown.push_str("## Top 25 Slowest Ferrugo Fixtures\n\n");
    markdown.push_str("| Rank | Fixture | Family | Mode | Time ms | Status |\n| ---: | --- | --- | --- | ---: | --- |\n");
    for (index, record) in top_ferrugo_slowest_records(&report.records, 25)
        .iter()
        .enumerate()
    {
        markdown.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` | {} | `{}` |\n",
            index + 1,
            record.fixture,
            record.family,
            record.mode.as_str(),
            markdown_optional_ms(record.timing.p95_ms.or(record.timing.wall_ms)),
            record.status.as_str()
        ));
    }

    markdown.push_str("\n## Top 25 Largest Reference Gaps\n\n");
    markdown.push_str("| Rank | Fixture | Family | Native cold ms | Fastest reference | Reference ms | Gap |\n| ---: | --- | --- | ---: | --- | ---: | ---: |\n");
    for (index, gap) in benchmark_matrix_reference_gaps(&report.records, 25)
        .iter()
        .enumerate()
    {
        markdown.push_str(&format!(
            "| {} | `{}` | `{}` | {:.3} | `{}` | {:.3} | {:.2}x |\n",
            index + 1,
            gap.fixture,
            gap.family,
            gap.native_ms,
            gap.reference_backend.as_str(),
            gap.reference_ms,
            gap.ratio
        ));
    }

    markdown.push_str("\n## Top Memory High-Water Records\n\n");
    markdown.push_str("| Rank | Fixture | Family | Backend | Mode | Peak RSS bytes |\n| ---: | --- | --- | --- | --- | ---: |\n");
    for (index, record) in top_memory_records(&report.records, 25).iter().enumerate() {
        markdown.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` | `{}` | {} |\n",
            index + 1,
            record.fixture,
            record.family,
            record.backend.as_str(),
            record.mode.as_str(),
            record.memory.rss_peak_bytes.unwrap_or_default()
        ));
    }

    markdown.push_str("\n## Family Summary\n\n");
    markdown.push_str("| Family | Native hot p95 | PDFium hot p95 | Ferrugo/PDFium hot | Native cold | PDFium cold | Poppler cold | Errors |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for (family, summary) in &report.families {
        markdown.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} |\n",
            family,
            markdown_optional_ms(summary.native_hot_p95_ms),
            markdown_optional_ms(summary.pdfium_hot_p95_ms),
            markdown_optional_ratio(summary.ferrugo_to_pdfium_hot_ratio),
            markdown_optional_ms(summary.native_cold_wall_ms),
            markdown_optional_ms(summary.pdfium_cold_wall_ms),
            markdown_optional_ms(summary.poppler_cold_wall_ms),
            summary.errors
        ));
    }

    markdown.push_str("\n## Profiling Loop\n\n");
    markdown.push_str("- Profile the top 5 Ferrugo slow fixtures before changing renderer code.\n");
    markdown.push_str(
        "- Prefer `sample`, Instruments, or Samply on release builds with the same `--max-edge` and fixture set.\n",
    );
    markdown.push_str(
        "- Accept optimization PRs only with before/after matrix evidence, at least 10% target-fixture speedup or a clear memory win, and no new visual or fallback regression.\n",
    );
    markdown
}

fn top_ferrugo_slowest_records(
    records: &[BenchmarkMatrixRecord],
    limit: usize,
) -> Vec<&BenchmarkMatrixRecord> {
    let mut selected = records
        .iter()
        .filter(|record| record.backend == MatrixBackend::Native)
        .filter(|record| record.status == MatrixStatus::Rendered)
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        let left_ms = left
            .timing
            .p95_ms
            .or(left.timing.wall_ms)
            .unwrap_or_default();
        let right_ms = right
            .timing
            .p95_ms
            .or(right.timing.wall_ms)
            .unwrap_or_default();
        right_ms.total_cmp(&left_ms)
    });
    selected.truncate(limit);
    selected
}

#[derive(Debug, Clone, PartialEq)]
struct MatrixReferenceGap {
    fixture: String,
    family: String,
    native_ms: f64,
    reference_backend: MatrixBackend,
    reference_ms: f64,
    ratio: f64,
}

fn benchmark_matrix_reference_gaps(
    records: &[BenchmarkMatrixRecord],
    limit: usize,
) -> Vec<MatrixReferenceGap> {
    let mut gaps = Vec::new();
    for native in records.iter().filter(|record| {
        record.backend == MatrixBackend::Native
            && record.mode == MatrixMode::ColdProcess
            && record.status == MatrixStatus::Rendered
    }) {
        let Some(native_ms) = native.timing.wall_ms else {
            continue;
        };
        let fastest = records
            .iter()
            .filter(|record| {
                record.fixture == native.fixture
                    && record.mode == MatrixMode::ColdProcess
                    && matches!(
                        record.backend,
                        MatrixBackend::Pdfium | MatrixBackend::Poppler
                    )
                    && record.status == MatrixStatus::Rendered
            })
            .filter_map(|record| {
                record
                    .timing
                    .wall_ms
                    .map(|wall_ms| (record.backend, wall_ms))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right));
        let Some((reference_backend, reference_ms)) = fastest else {
            continue;
        };
        if reference_ms <= f64::EPSILON {
            continue;
        }
        gaps.push(MatrixReferenceGap {
            fixture: native.fixture.clone(),
            family: native.family.clone(),
            native_ms,
            reference_backend,
            reference_ms,
            ratio: native_ms / reference_ms,
        });
    }
    gaps.sort_by(|left, right| right.ratio.total_cmp(&left.ratio));
    gaps.truncate(limit);
    gaps
}

fn top_memory_records(
    records: &[BenchmarkMatrixRecord],
    limit: usize,
) -> Vec<&BenchmarkMatrixRecord> {
    let mut selected = records
        .iter()
        .filter(|record| record.memory.rss_peak_bytes.is_some())
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .memory
            .rss_peak_bytes
            .unwrap_or_default()
            .cmp(&left.memory.rss_peak_bytes.unwrap_or_default())
    });
    selected.truncate(limit);
    selected
}

fn markdown_optional_ms(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value:.3}"))
}

fn markdown_optional_ratio(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value:.2}x"))
}

fn markdown_bool(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
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
            "  \"config\": {{\"repetitions\":{},\"pages_per_input\":{},\"workers\":{},\"max_p95_ms\":{},\"max_errors\":{},\"max_in_flight_pixels\":{},\"cancel_after_jobs\":{}}},\n",
            "  \"summary\": {{\"total_inputs\":{},\"total_jobs\":{},\"native_rendered\":{},\"fallback_required\":{},\"errors\":{},\"budget_failures\":{},\"elapsed_ms\":{:.3},\"throughput_per_sec\":{:.3}}},\n",
            "  \"isolation\": {},\n",
            "  \"latency\": {},\n",
            "  \"memory\": {},\n",
            "  \"families\": {},\n",
            "  \"records\": [{}]\n",
            "}}\n"
        ),
        platform_metadata_json(&report.platform),
        report.repetitions,
        report.pages_per_input,
        report.workers,
        report.max_p95_ms,
        report.max_errors,
        report.memory.max_in_flight_pixels,
        optional_json_usize(report.isolation.cancel_after_jobs),
        report.total_inputs,
        report.total_jobs,
        report.native_rendered,
        report.fallback_required,
        report.errors,
        report.budget_failures,
        report.elapsed_ms,
        report.throughput_per_sec,
        batch_isolation_summary_json(&report.isolation),
        batch_latency_summary_json(&report.latency),
        batch_memory_summary_json(&report.memory),
        batch_family_map_json(&report.families),
        records
    )
}

fn batch_isolation_summary_json(summary: &BatchIsolationSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"cache_policy\":{},",
            "\"scheduled_jobs\":{},",
            "\"skipped_jobs\":{},",
            "\"cancelled\":{},",
            "\"backend_scope\":{},",
            "\"shared_document_state\":{},",
            "\"timeout_ms\":{}",
            "}}"
        ),
        native_page_cache_policy_json(summary.cache_policy),
        summary.scheduled_jobs,
        summary.skipped_jobs,
        summary.cancelled,
        json_string(summary.backend_scope),
        summary.shared_document_state,
        summary.timeout_ms
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
            "\"repeat_mean_ms\":{:.3},",
            "\"phase_timings_ms\":{}",
            "}}"
        ),
        summary.total,
        summary.native_rendered,
        summary.fallback_required,
        summary.errors,
        summary.budget_failures,
        summary.first_mean_ms,
        summary.repeat_mean_ms,
        repeat_family_phase_timings_json(summary.phase_timings.as_ref())
    )
}

fn repeat_family_phase_timings_json(phase_timings: Option<&RepeatPhaseTimings>) -> String {
    let Some(phase_timings) = phase_timings else {
        return "null".to_string();
    };
    format!(
        "{{\"first_mean\":{},\"repeat_mean\":{}}}",
        trace_phase_timings_json(Ok(&phase_timings.first)),
        trace_phase_timings_json(Ok(&phase_timings.repeat_mean))
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
            "\"session\":{},",
            "\"timings_ms\":{},",
            "\"phase_timings_ms\":{},",
            "\"budget_violations\":{},",
            "\"outcome\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        record.page_index,
        native_page_cache_key_json(&record.cache_key),
        native_document_session_stats_json(record.session_stats),
        float_array_json(&record.timings_ms),
        repeat_phase_timings_json(record.phase_timings.as_ref()),
        json_str_array(record.budget_violations.as_slice()),
        repeat_benchmark_outcome_json(&record.outcome)
    )
}

fn repeat_phase_timings_json(phase_timings: Option<&RepeatPhaseTimings>) -> String {
    let Some(phase_timings) = phase_timings else {
        return "null".to_string();
    };
    format!(
        "{{\"first\":{},\"repeat_mean\":{}}}",
        trace_phase_timings_json(Ok(&phase_timings.first)),
        trace_phase_timings_json(Ok(&phase_timings.repeat_mean))
    )
}

fn native_document_session_stats_json(stats: Option<NativeDocumentSessionStats>) -> String {
    let Some(stats) = stats else {
        return "null".to_string();
    };
    format!(
        concat!(
            "{{",
            "\"cache_policy\":{},",
            "\"input_bytes\":{},",
            "\"loaded_objects\":{},",
            "\"max_loaded_objects\":{},",
            "\"loaded_object_bytes\":{},",
            "\"max_loaded_object_bytes\":{},",
            "\"page_count\":{},",
            "\"first_page_only\":{}",
            "}}"
        ),
        native_page_cache_policy_json(stats.cache_policy),
        stats.input_bytes,
        stats.loaded_objects,
        stats.max_loaded_objects,
        stats.loaded_object_bytes,
        stats.max_loaded_object_bytes,
        stats.page_count,
        stats.first_page_only
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
            "\"pointer_width_bits\":{},",
            "\"rustc_version\":{},",
            "\"logical_cpus\":{},",
            "\"cpu_brand\":{},",
            "\"memory_bytes\":{}",
            "}}"
        ),
        json_string(platform.os),
        json_string(platform.arch),
        json_string(platform.family),
        json_string(platform.endian),
        platform.pointer_width_bits,
        optional_json_string(platform.rustc_version.as_deref()),
        optional_json_usize(platform.logical_cpus),
        optional_json_string(platform.cpu_brand.as_deref()),
        optional_json_u64(platform.memory_bytes)
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

fn poppler_visual_diff_report_json(report: &PopplerVisualDiffReport) -> String {
    let fixtures = report
        .fixtures
        .iter()
        .map(poppler_visual_diff_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"reference_backend\": \"poppler-pdftoppm\",\n",
            "  \"platform\": {},\n",
            "  \"thresholds\": {{\"max_mean_abs_error\":{:.3},\"max_p95_channel_delta\":{},\"max_changed_ratio\":{:.6}}},\n",
            "  \"summary\": {{\"total\":{},\"exact\":{},\"accepted_drift\":{},\"blockers\":{},\"native_errors\":{},\"reference_errors\":{},\"both_errors\":{}}},\n",
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
        report.reference_errors,
        report.both_errors,
        poppler_visual_diff_family_map_json(&report.families),
        poppler_visual_diff_family_map_json(&report.subsystems),
        fixtures
    )
}

fn poppler_visual_diff_family_map_json(
    families: &BTreeMap<String, PopplerFamilyVisualDiffSummary>,
) -> String {
    let values = families
        .iter()
        .map(|(family, summary)| {
            format!(
                "{}:{}",
                json_string(family),
                poppler_visual_diff_family_summary_json(summary)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn poppler_visual_diff_family_summary_json(summary: &PopplerFamilyVisualDiffSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"exact\":{},",
            "\"accepted_drift\":{},",
            "\"blockers\":{},",
            "\"native_errors\":{},",
            "\"reference_errors\":{},",
            "\"both_errors\":{}",
            "}}"
        ),
        summary.total,
        summary.exact,
        summary.accepted_drift,
        summary.blockers,
        summary.native_errors,
        summary.reference_errors,
        summary.both_errors
    )
}

fn poppler_visual_diff_record_json(record: &PopplerVisualDiffRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"path\":{},",
            "\"family\":{},",
            "\"subsystem\":{},",
            "\"status\":{},",
            "\"metrics\":{},",
            "\"comparison_error\":{},",
            "\"native_error\":{},",
            "\"reference_error\":{}",
            "}}"
        ),
        json_string(&record.path),
        json_string(&record.family),
        json_string(record.subsystem),
        json_string(record.status.as_str()),
        poppler_visual_diff_metrics_json(record.metrics.as_ref()),
        visual_diff_error_json(record.comparison_error.as_ref()),
        visual_diff_error_json(record.native_error.as_ref()),
        visual_diff_error_json(record.reference_error.as_ref())
    )
}

fn poppler_visual_diff_metrics_json(metrics: Option<&VisualDiffMetrics>) -> String {
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
                "\"reference_nonwhite_pixels\":{}",
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

fn producer_regression_report_json(report: &ProducerRegressionReport) -> String {
    let producer_groups = producer_regression_group_map_json(&report.producer_groups);
    let family_groups = producer_regression_group_map_json(&report.family_groups);
    let feature_groups = producer_regression_group_map_json(&report.feature_groups);
    let records = report
        .records
        .iter()
        .map(producer_regression_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": 1,\n",
            "  \"report_kind\": \"producer-regression-report\",\n",
            "  \"privacy\": \"no PDF bytes, rendered pixels, extracted text, private filenames, or document hashes\",\n",
            "  \"summary\": {{\"total\":{},\"native_rendered\":{},\"fallback_required\":{},\"errors\":{}}},\n",
            "  \"producer_groups\": {},\n",
            "  \"family_groups\": {},\n",
            "  \"feature_groups\": {},\n",
            "  \"records\": [{}]\n",
            "}}\n"
        ),
        report.total,
        report.native_rendered,
        report.fallback_required,
        report.errors,
        producer_groups,
        family_groups,
        feature_groups,
        records
    )
}

fn producer_regression_group_map_json(
    groups: &BTreeMap<String, ProducerRegressionGroup>,
) -> String {
    let values = groups
        .iter()
        .map(|(key, group)| {
            format!(
                "{}:{}",
                json_string(key),
                producer_regression_group_json(group)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{values}}}")
}

fn producer_regression_group_json(group: &ProducerRegressionGroup) -> String {
    let pass_rate = if group.total == 0 {
        0.0
    } else {
        group.native_rendered as f64 / group.total as f64
    };
    format!(
        concat!(
            "{{",
            "\"total\":{},",
            "\"native_rendered\":{},",
            "\"native_pass_rate\":{:.3},",
            "\"fallback_required\":{},",
            "\"fallback_categories\":{},",
            "\"errors\":{},",
            "\"affected_features\":{},",
            "\"milestone_routes\":{}",
            "}}"
        ),
        group.total,
        group.native_rendered,
        pass_rate,
        group.fallback_required,
        count_map_json(&group.fallback_categories),
        count_map_json(&group.errors),
        string_count_map_json(&group.affected_features),
        string_count_map_json(&group.milestone_routes)
    )
}

fn producer_regression_record_json(record: &ProducerRegressionRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"fixture_id\":{},",
            "\"path_redacted\":{},",
            "\"family\":{},",
            "\"producer\":{},",
            "\"features\":{},",
            "\"milestone_routes\":{},",
            "\"outcome\":{}",
            "}}"
        ),
        json_string(&record.fixture_id),
        record.path_redacted,
        json_string(&record.family),
        json_string(&record.producer),
        json_string_array(&record.features),
        json_string_array(&record.milestone_routes),
        producer_regression_outcome_json(&record.outcome)
    )
}

fn producer_regression_outcome_json(outcome: &ProducerRegressionOutcome) -> String {
    match outcome {
        ProducerRegressionOutcome::NativeRendered => "{\"status\":\"native_rendered\"}".to_string(),
        ProducerRegressionOutcome::FallbackRequired { reason, category } => format!(
            "{{\"status\":\"fallback_required\",\"reason\":{},\"category\":{}}}",
            json_string(reason),
            json_string(category)
        ),
        ProducerRegressionOutcome::Error { class, message } => format!(
            "{{\"status\":\"error\",\"class\":{},\"message\":{}}}",
            json_string(class),
            json_string(message)
        ),
    }
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
                "{{\"status\":\"success\",\"page_count\":{},\"pages\":[{}],\"info\":{},\"structure\":{},\"outlines\":{},\"page_labels\":{},\"accessibility\":{},\"archival\":{},\"optional_content\":{}}}",
                metadata.page_count(),
                pages,
                document_info_json(&metadata.info),
                document_structure_json(&metadata.structure),
                outline_metadata_json(&metadata.outlines),
                page_labels_metadata_json(&metadata.page_labels),
                accessibility_metadata_json(&metadata.accessibility),
                archival_metadata_json(&metadata.archival),
                optional_content_metadata_json(&metadata.optional_content)
            )
        }
        MetadataOutcome::Error { class, message } => format!(
            "{{\"status\":\"error\",\"error_class\":{},\"message\":{}}}",
            json_string(class),
            json_string(message)
        ),
    }
}

fn document_info_json(info: &ferrugo_thumbnail::DocumentInfo) -> String {
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

fn document_structure_json(structure: &ferrugo_thumbnail::DocumentStructure) -> String {
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

fn outline_metadata_json(outlines: &ferrugo_thumbnail::OutlineMetadata) -> String {
    format!(
        "{{\"has_outlines\":{},\"item_count\":{},\"truncated\":{}}}",
        outlines.has_outlines, outlines.item_count, outlines.truncated
    )
}

fn page_labels_metadata_json(page_labels: &ferrugo_thumbnail::PageLabelsMetadata) -> String {
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

fn accessibility_metadata_json(accessibility: &ferrugo_thumbnail::AccessibilityMetadata) -> String {
    format!(
        "{{\"language\":{},\"mark_info_marked\":{},\"has_role_map\":{},\"structure_role_count\":{},\"has_marked_content_references\":{},\"marked_content_reference_count\":{},\"page_content_reference_count\":{},\"alt_text_count\":{},\"reading_order_warning_count\":{},\"truncated\":{}}}",
        optional_json_string(accessibility.language.as_deref()),
        optional_json_bool(accessibility.mark_info_marked),
        accessibility.has_role_map,
        accessibility.structure_role_count,
        accessibility.has_marked_content_references,
        accessibility.marked_content_reference_count,
        accessibility.page_content_reference_count,
        accessibility.alt_text_count,
        accessibility.reading_order_warning_count,
        accessibility.truncated
    )
}

fn archival_metadata_json(archival: &ferrugo_thumbnail::ArchivalMetadata) -> String {
    format!(
        "{{\"pdfa_part\":{},\"pdfa_conformance\":{},\"has_output_intents\":{},\"conformance_validation_performed\":{}}}",
        optional_json_string(archival.pdfa_part.as_deref()),
        optional_json_string(archival.pdfa_conformance.as_deref()),
        archival.has_output_intents,
        archival.conformance_validation_performed
    )
}

fn optional_content_metadata_json(
    optional_content: &ferrugo_thumbnail::OptionalContentMetadata,
) -> String {
    format!(
        "{{\"has_oc_properties\":{},\"group_count\":{},\"has_default_configuration\":{},\"base_state\":{},\"default_on_count\":{},\"default_off_count\":{},\"has_usage_application\":{},\"has_unsupported_membership_policy\":{},\"has_direct_group_dictionary\":{},\"has_unsupported_behavior\":{}}}",
        optional_content.has_oc_properties,
        optional_content.group_count,
        optional_content.has_default_configuration,
        optional_content_base_state_json(optional_content.base_state),
        optional_content.default_on_count,
        optional_content.default_off_count,
        optional_content.has_usage_application,
        optional_content.has_unsupported_membership_policy,
        optional_content.has_direct_group_dictionary,
        optional_content.has_unsupported_behavior
    )
}

fn optional_content_base_state_json(
    base_state: ferrugo_thumbnail::OptionalContentBaseState,
) -> &'static str {
    match base_state {
        ferrugo_thumbnail::OptionalContentBaseState::Unspecified => "null",
        ferrugo_thumbnail::OptionalContentBaseState::On => "\"on\"",
        ferrugo_thumbnail::OptionalContentBaseState::Off => "\"off\"",
        ferrugo_thumbnail::OptionalContentBaseState::Unchanged => "\"unchanged\"",
    }
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

fn optional_json_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn optional_json_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn optional_json_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn optional_json_f64(value: Option<f64>) -> String {
    value.map_or_else(|| "null".to_string(), |value| format!("{value:.3}"))
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

fn encode_rgba_png(thumbnail: &ferrugo_thumbnail::Thumbnail) -> Result<Vec<u8>, CliError> {
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
        "Usage: ferrugo-cli <render|render-auto|render-native|render-pdfium|render-isolated|compare-metadata|summarize-fallbacks|operator-coverage|trace-native|replay-operators|extract-corpus-metadata|producer-regression-report|classify-pdf20-usage|validate-local-corpus|benchmark-native|benchmark-batch-native|benchmark-repeat-native|benchmark-pdfium|benchmark-matrix|visual-diff|visual-diff-poppler> <input.pdf> \
         [--output PATH] [--page-index N] [--max-edge N] [--background #RRGGBB] \
         [--timeout SECONDS] [--iterations N] [--warmup N] [--repetitions N] [--pages-per-input N] [--max-events N] [--max-workers N] [--max-in-flight-pixels N] [--cancel-after-jobs N] [--max-ms N] [--max-p95-ms N] [--max-first-ms N] [--max-repeat-mean-ms N] [--max-output-bytes N] \
         [--backend native|pdfium|poppler] [--mode cold-process|hot-render] [--report PATH] [--artifact-dir PATH] [--pdftoppm PATH] [--native-only] [--manifest PATH] [--include-family FAMILY] \
         [--diagnostics-dir PATH] [--allow-missing] [--annotation-mode screen|print] [--no-annotations] [--max-mae N] [--max-p95 N] [--max-changed-ratio N]"
    );
}

#[cfg(test)]
mod tests {
    use ferrugo_thumbnail::{PageMetadata, PixelFormat, Thumbnail};

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
        assert_eq!(config.annotation_mode, AnnotationMode::Screen);
    }

    #[test]
    fn render_config_should_accept_print_annotation_mode() {
        let config = RenderConfig::parse(&[
            OsString::from("fixtures/generated/text-page.pdf"),
            OsString::from("--output"),
            OsString::from("target/text-page.png"),
            OsString::from("--annotation-mode"),
            OsString::from("print"),
        ])
        .expect("valid config");

        assert_eq!(config.annotation_mode, AnnotationMode::Print);
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
        env::remove_var("FERRUGO_PDFIUM_LIBRARY");
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
        env::remove_var("FERRUGO_PDFIUM_LIBRARY");
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
        env::remove_var("FERRUGO_PDFIUM_LIBRARY");
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
            annotation_mode: AnnotationMode::Screen,
        };

        let outcome = render_auto_thumbnail(&config).expect("supported fixture should render");

        assert_eq!(outcome.backend, AutoRenderBackend::Native);
    }

    #[test]
    fn render_auto_thumbnail_should_return_native_unsupported_without_pdfium_fallback() {
        env::remove_var("FERRUGO_PDFIUM_LIBRARY");
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
            annotation_mode: AnnotationMode::Screen,
        };

        let error = render_auto_thumbnail(&config)
            .expect_err("auto mode should not retry unsupported documents through PDFium");

        assert_eq!(
            error.to_string(),
            format!(
                "render error [unsupported]: PDF feature is unsupported (graphics.optional-content) for graphics.optional-content; {PDFIUM_RUNTIME_FALLBACK_REMOVED_MESSAGE}"
            )
        );
    }

    #[test]
    fn render_config_should_reject_explicit_pdfium_fallback_flag() {
        let error = RenderConfig::parse(&[
            OsString::from("fixtures/generated/optional-content-ocmd.pdf"),
            OsString::from("--output"),
            OsString::from("target/ocmd.png"),
            OsString::from("--allow-pdfium-fallback"),
        ])
        .expect_err("runtime PDFium fallback flag should be rejected");

        assert_eq!(
            error.to_string(),
            format!("usage error: {PDFIUM_RUNTIME_FALLBACK_REMOVED_MESSAGE}")
        );
    }

    #[cfg(feature = "pdfium")]
    #[test]
    fn render_worker_should_reject_direct_cli_invocation() {
        env::remove_var(PDFIUM_RENDER_WORKER_ENV);

        let error = run(vec![OsString::from("render-worker")])
            .expect_err("render-worker should only be launched by render-isolated");

        assert_eq!(
            error.to_string(),
            "usage error: render-worker is private maintainer tooling; use render-isolated"
        );
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
    fn operator_coverage_config_should_accept_family_filters() {
        let config = OperatorCoverageConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/corpus-manifest.tsv"),
            OsString::from("--include-family"),
            OsString::from("office-export"),
            OsString::from("--page-index"),
            OsString::from("0"),
            OsString::from("--no-annotations"),
            OsString::from("--output"),
            OsString::from("target/operator-coverage.json"),
        ])
        .expect("valid operator coverage config");

        assert_eq!(config.include_families, vec!["office-export".to_string()]);
        assert_eq!(config.page_index, 0);
        assert!(!config.include_annotations);
        assert_eq!(
            config.output,
            Some(PathBuf::from("target/operator-coverage.json"))
        );
    }

    #[test]
    fn benchmark_matrix_config_should_default_to_all_backends_and_modes() {
        let config = BenchmarkMatrixConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/performance-matrix-manifest.tsv"),
            OsString::from("--output"),
            OsString::from("target/performance-matrix.json"),
        ])
        .expect("valid matrix benchmark config");

        assert_eq!(
            config.backends,
            vec![
                MatrixBackend::Native,
                MatrixBackend::Pdfium,
                MatrixBackend::Poppler
            ]
        );
        assert_eq!(
            config.modes,
            vec![MatrixMode::ColdProcess, MatrixMode::HotRender]
        );
    }

    #[test]
    fn benchmark_matrix_manifest_filter_should_drop_unlisted_fixtures() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest =
            read_corpus_manifest(&fixture_root.join("fixtures/performance-matrix-manifest.tsv"))
                .expect("performance matrix manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/text-page.pdf"),
            fixture_root.join("fixtures/generated/vector-paths.pdf"),
        ];

        let filtered =
            filter_fixtures_by_manifest(&paths, &manifest).expect("one fixture should match");

        assert_eq!(
            filtered,
            vec![fixture_root.join("fixtures/generated/text-page.pdf")]
        );
    }

    #[test]
    fn benchmark_matrix_config_should_accept_backend_modes_and_report_path() {
        let config = BenchmarkMatrixConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--backend"),
            OsString::from("poppler"),
            OsString::from("--backend"),
            OsString::from("native"),
            OsString::from("--mode"),
            OsString::from("hot-render"),
            OsString::from("--mode"),
            OsString::from("cold-process"),
            OsString::from("--warmup"),
            OsString::from("0"),
            OsString::from("--iterations"),
            OsString::from("2"),
            OsString::from("--report"),
            OsString::from("target/performance-matrix.md"),
            OsString::from("--artifact-dir"),
            OsString::from("target/perf-artifacts"),
        ])
        .expect("valid matrix benchmark config");

        assert_eq!(
            config.backends,
            vec![MatrixBackend::Native, MatrixBackend::Poppler]
        );
        assert_eq!(
            config.modes,
            vec![MatrixMode::ColdProcess, MatrixMode::HotRender]
        );
        assert_eq!(config.warmup, 0);
        assert_eq!(config.iterations, 2);
        assert_eq!(
            config.markdown_report,
            Some(PathBuf::from("target/performance-matrix.md"))
        );
    }

    #[test]
    fn benchmark_matrix_timing_should_calculate_distribution() {
        let timing = matrix_timing_from_samples(1, vec![4.0, 1.0, 9.0, 2.0]);

        assert_eq!(timing.warmup_iterations, 1);
        assert_eq!(timing.measured_iterations, 4);
        assert_eq!(timing.mean_ms, Some(4.0));
        assert_eq!(timing.p50_ms, Some(2.0));
        assert_eq!(timing.p95_ms, Some(9.0));
    }

    #[test]
    fn benchmark_matrix_json_should_include_missing_tool_and_not_applicable_statuses() {
        let fixture = PathBuf::from("fixtures/generated/text-page.pdf");
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
        };
        let mut records = missing_tool_records(
            MatrixBackend::Pdfium,
            MatrixMode::HotRender,
            std::slice::from_ref(&fixture),
            None,
            &options,
            "PDFium unavailable".to_string(),
        );
        records.extend(not_applicable_records(
            MatrixBackend::Poppler,
            MatrixMode::HotRender,
            &[fixture],
            None,
            &options,
            "external process only",
        ));
        let report = BenchmarkMatrixReport {
            platform: PlatformMetadata::current(),
            command: "ferrugo-cli benchmark-matrix".to_string(),
            config: BenchmarkMatrixReportConfig {
                input: "fixtures/generated".to_string(),
                manifest: None,
                include_families: Vec::new(),
                page_index: 0,
                max_edge: 120,
                timeout_secs: 5,
                iterations: 1,
                warmup: 0,
                backends: vec![MatrixBackend::Pdfium, MatrixBackend::Poppler],
                modes: vec![MatrixMode::HotRender],
                native_profile: NativeProfile::Default,
            },
            summary: benchmark_matrix_summary(&records),
            families: benchmark_matrix_family_summaries(&records),
            records,
        };

        let json = benchmark_matrix_report_json(&report);
        let reliability = benchmark_matrix_timing_reliability(&report);
        let markdown = benchmark_matrix_markdown_report(&report);

        assert!(!reliability.rss_available);
        assert!(reliability.pdfium_requested);
        assert!(!reliability.pdfium_available);
        assert!(reliability.poppler_requested);
        assert!(!reliability.poppler_available);
        assert!(!reliability.hot_pdfium_comparison_available);
        assert_eq!(
            reliability.caveats,
            vec![
                "rss-unavailable",
                "pdfium-missing-tool",
                "poppler-hot-render-external-only",
                "pdfium-hot-reference-unavailable",
            ]
        );
        assert!(json.contains("\"timing_reliability\""));
        assert!(json.contains("\"pdfium_available\":false"));
        assert!(json.contains("\"poppler-hot-render-external-only\""));
        assert!(json.contains("\"status\":\"missing-tool\""));
        assert!(json.contains("\"status\":\"not-applicable\""));
        assert!(json.contains("\"report_kind\": \"renderer-performance-matrix\""));
        assert!(markdown.contains("## Timing Reliability"));
        assert!(markdown.contains("Hot PDFium comparison available"));
    }

    #[test]
    fn operator_coverage_should_aggregate_fixture_operators() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = vec![
            fixture_root.join("fixtures/generated/vector-paths.pdf"),
            fixture_root.join("fixtures/generated/inline-image.pdf"),
        ];
        let config = OperatorCoverageConfig {
            input: fixture_root.join("fixtures/generated"),
            manifest: None,
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            include_annotations: true,
        };

        let report = scan_operator_coverage_corpus(&paths, None, &config);

        assert_eq!(report.total, 2);
        assert_eq!(report.scanned, 2);
        assert_eq!(report.errors, 0);
        assert!(report.operators.contains_key("S"));
        assert!(report.operators.contains_key("BI"));
    }

    #[test]
    fn operator_coverage_json_should_include_status_counts() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = vec![fixture_root.join("fixtures/generated/inline-image.pdf")];
        let config = OperatorCoverageConfig {
            input: fixture_root.join("fixtures/generated/inline-image.pdf"),
            manifest: None,
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            include_annotations: true,
        };
        let report = scan_operator_coverage_corpus(&paths, None, &config);
        let json = operator_coverage_report_json(&report);

        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"status_counts\""));
        assert!(json.contains("\"BI\""));
        assert!(json.contains("\"implemented\""));
    }

    #[test]
    fn producer_regression_config_should_require_manifest_and_accept_filters() {
        let missing_manifest = ProducerRegressionConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--include-family"),
            OsString::from("unsupported-boundary"),
        ])
        .expect_err("producer regression reports require manifest metadata");

        assert_eq!(
            missing_manifest.to_string(),
            "usage error: --manifest is required for producer-regression-report"
        );

        let config = ProducerRegressionConfig::parse(&[
            OsString::from("fixtures/generated"),
            OsString::from("--manifest"),
            OsString::from("fixtures/producer-compatibility-manifest.tsv"),
            OsString::from("--include-family"),
            OsString::from("unsupported-boundary"),
            OsString::from("--max-edge"),
            OsString::from("120"),
            OsString::from("--output"),
            OsString::from("target/producer-regression.json"),
        ])
        .expect("valid producer regression config");

        assert_eq!(
            config.manifest,
            PathBuf::from("fixtures/producer-compatibility-manifest.tsv")
        );
        assert_eq!(
            config.include_families,
            vec!["unsupported-boundary".to_string()]
        );
        assert_eq!(config.max_edge, 120);
        assert_eq!(
            config.output,
            Some(PathBuf::from("target/producer-regression.json"))
        );
    }

    #[test]
    fn producer_regression_report_should_group_failures_by_producer_and_route() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/producer-compatibility-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/optional-content-ocmd.pdf"),
            fixture_root.join("fixtures/generated/unsupported-ccitt-image.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
        };

        let report =
            build_producer_regression_report(&NativeBackend::new(), &paths, &options, &manifest);
        let json = producer_regression_report_json(&report);

        assert_eq!(report.total, 2);
        assert_eq!(report.native_rendered, 0);
        assert_eq!(report.fallback_required, 2);
        assert_eq!(
            report
                .producer_groups
                .get("layered-presentation-export")
                .and_then(|group| group.fallback_categories.get("graphics.optional-content")),
            Some(&1)
        );
        assert_eq!(
            report
                .producer_groups
                .get("fax-scanner-export")
                .and_then(|group| group.fallback_categories.get("image.filter")),
            Some(&1)
        );
        assert!(json.contains("\"report_kind\": \"producer-regression-report\""));
        assert!(json.contains("\"0192 optional-content-ui-state\""));
        assert!(json.contains("\"0209 rust-native-image-codec\""));
        assert!(json.contains(
            "\"privacy\": \"no PDF bytes, rendered pixels, extracted text, private filenames, or document hashes\""
        ));
        assert!(!json.contains("%PDF"));
    }

    #[test]
    fn producer_regression_manifest_filter_should_drop_unlisted_fixtures() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/producer-compatibility-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/text-page.pdf"),
            fixture_root.join("fixtures/generated/optional-content-ocmd.pdf"),
        ];

        let filtered =
            filter_fixtures_by_manifest(&paths, &manifest).expect("manifest row should match");

        assert_eq!(
            filtered,
            vec![fixture_root.join("fixtures/generated/optional-content-ocmd.pdf")]
        );
    }

    #[test]
    fn producer_regression_fixture_id_should_redact_private_paths() {
        let entry = CorpusManifestEntry {
            path: "fixtures/local-corpus/customer-statement.pdf".to_string(),
            family: "statement".to_string(),
            source: "fixtures/local-corpus/metadata.toml".to_string(),
            license: "local-review-only".to_string(),
            page_count: 1,
            features: vec![
                "producer:private-accounting-export".to_string(),
                "privacy:private".to_string(),
                "table".to_string(),
            ],
            notes: "private aggregate sample".to_string(),
        };
        let path = PathBuf::from("fixtures/local-corpus/customer-statement.pdf");

        assert!(is_sensitive_fixture(&path, &entry));
        assert_eq!(
            producer_fixture_id(7, &path, Some(&entry)),
            "local-only-0007"
        );
    }

    #[test]
    fn trace_native_config_should_bound_event_count() {
        let config = TraceNativeConfig::parse(&[
            OsString::from("fixtures/generated/vector-paths.pdf"),
            OsString::from("--page-index"),
            OsString::from("0"),
            OsString::from("--max-edge"),
            OsString::from("160"),
            OsString::from("--max-events"),
            OsString::from("12"),
            OsString::from("--no-annotations"),
            OsString::from("--output"),
            OsString::from("target/native-trace.json"),
        ])
        .expect("valid trace config");

        assert_eq!(config.page_index, 0);
        assert_eq!(config.max_edge, 160);
        assert_eq!(config.max_events, 12);
        assert!(!config.include_annotations);
        assert_eq!(
            config.output,
            Some(PathBuf::from("target/native-trace.json"))
        );

        let too_large = TraceNativeConfig::parse(&[
            OsString::from("fixtures/generated/vector-paths.pdf"),
            OsString::from("--max-events"),
            OsString::from((TRACE_MAX_EVENTS_LIMIT + 1).to_string()),
        ]);
        assert!(too_large.is_err());
    }

    #[test]
    fn native_trace_json_should_omit_document_bytes_and_bound_events() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = TraceNativeConfig {
            input: fixture_root.join("fixtures/generated/vector-paths.pdf"),
            output: None,
            page_index: 0,
            max_edge: 160,
            max_events: 2,
            include_annotations: true,
        };

        let json = native_render_trace_json(&config).expect("trace should render");

        assert!(json.contains("\"trace_kind\": \"native-render-trace\""));
        assert!(json.contains("\"privacy\""));
        assert!(json.contains("\"events_emitted\": 2"));
        assert!(json.contains("\"events_truncated\": true"));
        assert!(json.contains("\"phase_timings_ms\""));
        assert!(json.contains("\"load_xref_object\""));
        assert!(json.contains("\"display_list_build\""));
        assert!(json.contains("\"raster_paths\""));
        assert!(json.contains("\"total\""));
        assert!(json.contains("\"stroke_shape_summary\""));
        assert!(json.contains("\"flattened_lines\""));
        assert!(json.contains("\"pixel_x_span_buckets\""));
        assert!(json.contains("\"operator_summary\""));
        assert!(!json.contains("stream\n"));
        assert!(!json.contains("ferrugo thumbnail fixture"));
    }

    #[test]
    fn replay_operator_trace_should_count_bounded_events() {
        let trace = concat!(
            "{",
            "\"trace_kind\":\"native-render-trace\",",
            "\"events\":[",
            "{\"seq\":0,\"phase\":\"operator\",\"operator\":\"q\",\"status\":\"implemented\",\"fallback_bucket\":null},",
            "{\"seq\":1,\"phase\":\"operator\",\"operator\":\"S\",\"status\":\"implemented\",\"fallback_bucket\":null},",
            "{\"seq\":2,\"phase\":\"operator\",\"operator\":\"S\",\"status\":\"implemented\",\"fallback_bucket\":null}",
            "]",
            "}"
        );

        let json = replay_operator_trace_json(trace).expect("trace should replay");

        assert!(json.contains("\"trace_kind\": \"operator-replay\""));
        assert!(json.contains("\"events_replayed\": 3"));
        assert!(json.contains("\"q\":1"));
        assert!(json.contains("\"S\":2"));
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
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        assert!(bundle.contains("\"telemetry\": {\"collection\":\"none\""));
        assert!(bundle.contains("\"includes_pdf_bytes\":false"));
        assert!(bundle.contains("\"includes_rendered_pixels\":false"));
        assert!(bundle.contains("\"includes_text_samples\":false"));
        assert!(bundle.contains("\"includes_private_paths\":false"));
        assert!(bundle.contains("\"field_classes\""));
        assert!(bundle.contains("\"stage_hint\":\"display-list-or-raster\""));
        assert!(bundle.contains("\"category\":\"graphics.optional-content\""));
        assert!(!bundle.contains("%PDF"));

        let private_entry = CorpusManifestEntry {
            path: "fixtures/local-corpus/private/customer/invoice.pdf".to_string(),
            family: "invoice".to_string(),
            source: "local-corpus".to_string(),
            license: "local-review-only".to_string(),
            page_count: 1,
            features: vec!["privacy:private".to_string(), "invoice".to_string()],
            notes: "customer name and account number".to_string(),
        };
        let private_error = ThumbnailError::unsupported_feature("image.filter");
        let diagnostics = NativeBackend::new().memory_diagnostics();
        let private_bundle = native_diagnostic_bundle_json(NativeDiagnosticBundle {
            path: "local-only-0000",
            path_redacted: true,
            manifest: Some(&private_entry),
            options: &options,
            metadata: Err(&private_error),
            metadata_ms: 1.0,
            render_error: &private_error,
            render_ms: 1.0,
            diagnostics: &diagnostics,
        });
        assert!(private_bundle.contains("\"path\": \"local-only-0000\""));
        assert!(private_bundle.contains("\"path_redacted\": true"));
        assert!(private_bundle.contains("\"status\":\"redacted\""));
        assert!(!private_bundle.contains("customer/invoice.pdf"));
        assert!(!private_bundle.contains("customer name"));
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
        assert!(json.contains("\"marked_content_reference_count\":1"));
        assert!(json.contains("\"page_content_reference_count\":1"));
        assert!(json.contains("\"alt_text_count\":0"));
        assert!(json.contains("\"reading_order_warning_count\":0"));
    }

    #[test]
    fn pdf20_usage_should_classify_version_features_and_typed_boundary() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/pdf20-compatibility-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![
            fixture_root.join("fixtures/generated/pdf20-basic-office.pdf"),
            fixture_root.join("fixtures/generated/pdf20-associated-files.pdf"),
            fixture_root.join("fixtures/generated/pdf20-black-point-compensation.pdf"),
        ];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Png,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
        };

        let report =
            classify_pdf20_usage(&paths, Some(&manifest), &options).expect("usage should scan");

        assert_eq!(report.pdf20_documents, 3);
        assert_eq!(report.native_rendered, 2);
        assert_eq!(report.typed_unsupported, 1);
        assert_eq!(
            report.feature_counts.get("black-point-compensation"),
            Some(&1)
        );
        assert_eq!(report.impact_counts.get("visual-unsupported"), Some(&1));
        assert_eq!(
            report
                .families
                .get("unsupported-color-management")
                .map(|family| family.typed_unsupported),
            Some(1)
        );
        assert_eq!(report.followups[0].feature, "black-point-compensation");
        assert_eq!(
            report.followups[0].bucket,
            Some(ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT)
        );
    }

    #[test]
    fn pdf20_usage_json_should_include_privacy_and_policy_fields() {
        let report = Pdf20UsageReport {
            total_scanned: 1,
            pdf20_documents: 1,
            native_rendered: 0,
            typed_unsupported: 1,
            errors: 0,
            feature_counts: BTreeMap::from([("black-point-compensation".to_string(), 1)]),
            impact_counts: BTreeMap::from([("visual-unsupported", 1)]),
            families: BTreeMap::from([(
                "report".to_string(),
                Pdf20FamilySummary {
                    total: 1,
                    pdf20_documents: 1,
                    native_rendered: 0,
                    typed_unsupported: 1,
                    errors: 0,
                },
            )]),
            followups: vec![Pdf20Followup {
                rank: 1,
                feature: "black-point-compensation".to_string(),
                observed_documents: 1,
                visual_impact: "visual-unsupported",
                bucket: Some(
                    ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT,
                ),
                recommendation: "keep typed unsupported for 1.2 unless real-corpus frequency rises; implement only with color-threshold evidence",
            }],
            fixtures: vec![Pdf20UsageRecord {
                path: "fixtures/generated/pdf20-black-point-compensation.pdf".to_string(),
                family: "report".to_string(),
                manifest_features: vec!["pdf-2.0".to_string()],
                version: Pdf20VersionEvidence {
                    header_version: Some("2.0".to_string()),
                    catalog_version_20: true,
                    manifest_pdf20_tag: true,
                    detected_pdf20: true,
                },
                features: vec![Pdf20FeatureObservation {
                    feature: "black-point-compensation",
                    policy: "typed-unsupported",
                    visual_impact: "visual-unsupported",
                    bucket: Some(
                        ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT,
                    ),
                }],
                render: Pdf20RenderOutcome::TypedUnsupported {
                    bucket: ferrugo_thumbnail::unsupported_feature_buckets::GRAPHICS_COLOR_MANAGEMENT,
                },
            }],
        };

        let json = pdf20_usage_report_json(&report);

        assert!(json.contains("\"report_kind\": \"pdf-2-0-feature-usage\""));
        assert!(json.contains(
            "\"privacy\": \"no PDF bytes, rendered pixels, text samples, or stream operands\""
        ));
        assert!(json.contains("\"policy\":\"typed-unsupported\""));
        assert!(json.contains("\"bucket\":\"graphics.color-management\""));
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
            OsString::from("--pages-per-input"),
            OsString::from("4"),
            OsString::from("--max-workers"),
            OsString::from("4"),
            OsString::from("--max-in-flight-pixels"),
            OsString::from("25600"),
            OsString::from("--cancel-after-jobs"),
            OsString::from("2"),
        ])
        .expect("valid batch benchmark config");
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 160,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
        };

        assert_eq!(config.repetitions, 3);
        assert_eq!(config.pages_per_input, 4);
        assert_eq!(config.max_workers, 4);
        assert_eq!(config.cancel_after_jobs, Some(2));
        assert_eq!(
            effective_batch_workers(&config, &options).expect("workers"),
            1
        );
        assert_eq!(config.native_profile, NativeProfile::Default);
    }

    #[test]
    fn batch_benchmark_should_reject_unschedulable_pixel_budget() {
        let config = BatchBenchmarkConfig {
            input: PathBuf::from("fixtures/generated"),
            manifest: None,
            include_families: Vec::new(),
            output: None,
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            timeout: Duration::from_secs(5),
            repetitions: 1,
            pages_per_input: 1,
            max_workers: 2,
            max_in_flight_pixels: 120 * 120 - 1,
            max_p95_ms: 60_000,
            max_errors: 0,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
            cancel_after_jobs: None,
        };
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
        };

        let error = effective_batch_workers(&config, &options)
            .expect_err("one render job must fit inside the pixel budget");

        assert_eq!(
            error.to_string(),
            "benchmark budget failure: batch memory budget cannot schedule one render job"
        );
    }

    #[test]
    fn batch_jobs_should_expand_pages_in_stable_order_with_manifest_bounds() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/shared-resource-cache-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![fixture_root.join("fixtures/generated/long-document-navigation-deck.pdf")];

        let jobs = batch_jobs(&paths, Some(&manifest), 2, 1, 3);

        let observed = jobs
            .iter()
            .map(|job| (job.repetition, job.page_index, job.family.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            observed,
            vec![
                (0, 1, "long-document-shared"),
                (0, 2, "long-document-shared"),
                (0, 3, "long-document-shared"),
                (1, 1, "long-document-shared"),
                (1, 2, "long-document-shared"),
                (1, 3, "long-document-shared"),
            ]
        );
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
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
            pages_per_input: 1,
            max_workers: 2,
            max_in_flight_pixels: 120 * 120 * 2,
            max_p95_ms: 60_000,
            max_errors: 2,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
            cancel_after_jobs: None,
        };

        let report = benchmark_native_batch(&paths, &options, Some(&manifest), &config)
            .expect("batch benchmark should run");
        let json = batch_benchmark_report_json(&report);

        assert_eq!(report.total_inputs, 2);
        assert_eq!(report.total_jobs, 4);
        assert_eq!(report.pages_per_input, 1);
        assert_eq!(report.native_rendered, 2);
        assert_eq!(report.fallback_required, 2);
        assert_eq!(report.errors, 0);
        assert_eq!(report.budget_failures, 0);
        assert_eq!(report.workers, 2);
        assert_eq!(
            report.isolation.cache_policy,
            NativePageCachePolicy::IsolatedRender
        );
        assert_eq!(report.isolation.scheduled_jobs, 4);
        assert_eq!(report.isolation.skipped_jobs, 0);
        assert!(!report.isolation.cancelled);
        assert_eq!(report.isolation.backend_scope, "per-job");
        assert!(!report.isolation.shared_document_state);
        assert_eq!(report.isolation.timeout_ms, 5_000);
        assert!(report.throughput_per_sec > 0.0);
        assert!(report.latency.p95_ms >= report.latency.p50_ms);
        assert!(report.memory.max_output_bytes > 0);
        assert!(json.contains("\"throughput_per_sec\""));
        assert!(json.contains("\"pages_per_input\":1"));
        assert!(json.contains("\"isolation\""));
        assert!(json.contains("\"backend_scope\":\"per-job\""));
        assert!(json.contains("\"latency\""));
        assert!(json.contains("\"memory\""));
        assert!(json.contains("\"page_index\":0"));
        assert!(json.contains("\"status\":\"fallback_required\""));
        assert!(json.contains("\"category\":\"graphics.optional-content\""));
    }

    #[test]
    fn batch_benchmark_should_report_cooperative_cancellation_boundary() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest_path = fixture_root.join("fixtures/shared-resource-cache-manifest.tsv");
        let manifest = read_corpus_manifest(&manifest_path).expect("manifest should parse");
        let paths = vec![fixture_root.join("fixtures/generated/long-document-navigation-deck.pdf")];
        let options = ThumbnailOptions {
            page_index: 0,
            max_edge: 120,
            background: Rgba::WHITE,
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
            repetitions: 1,
            pages_per_input: 6,
            max_workers: 2,
            max_in_flight_pixels: 120 * 120 * 2,
            max_p95_ms: 60_000,
            max_errors: 0,
            fail_on_budget: false,
            native_profile: NativeProfile::Default,
            cancel_after_jobs: Some(3),
        };

        let report = benchmark_native_batch(&paths, &options, Some(&manifest), &config)
            .expect("batch benchmark should report cancellation");
        let json = batch_benchmark_report_json(&report);

        assert_eq!(report.total_jobs, 3);
        assert_eq!(report.native_rendered, 3);
        assert_eq!(report.isolation.scheduled_jobs, 3);
        assert_eq!(report.isolation.skipped_jobs, 3);
        assert!(report.isolation.cancelled);
        assert_eq!(report.isolation.cancel_after_jobs, Some(3));
        assert_eq!(
            report
                .records
                .iter()
                .map(|record| (record.repetition, record.page_index))
                .collect::<Vec<_>>(),
            vec![(0, 0), (0, 1), (0, 2)]
        );
        assert!(json.contains("\"cancel_after_jobs\":3"));
        assert!(json.contains("\"skipped_jobs\":3"));
        assert!(json.contains("\"cancelled\":true"));
        assert!(json.contains("\"shared_document_state\":false"));
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
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        assert_eq!(report.cache_policy, NativePageCachePolicy::DocumentSession);
        assert_eq!(
            report.records[0]
                .session_stats
                .expect("record should expose session stats")
                .cache_policy,
            NativePageCachePolicy::DocumentSession
        );
        assert!(json.contains("\"name\":\"document-session\""));
        assert!(json.contains("\"session\""));
        assert!(json.contains("\"loaded_objects\""));
        assert!(json.contains("\"cache_key\""));
        assert!(json.contains("\"phase_timings_ms\""));
        assert!(json.contains("\"phase_timings_ms\":{\"first_mean\""));
        assert!(json.contains("\"resource_decode\""));
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
            output_format: ferrugo_thumbnail::OutputFormat::Rgba,
            timeout: Duration::from_secs(5),
            annotation_mode: AnnotationMode::Screen,
            form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
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
        assert!(json.contains("\"rustc_version\":"));
        assert!(json.contains("\"logical_cpus\":"));
        assert!(json.contains("\"cpu_brand\":"));
        assert!(json.contains("\"memory_bytes\":"));
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
        metadata.outlines = ferrugo_thumbnail::OutlineMetadata {
            has_outlines: true,
            item_count: 2,
            truncated: false,
        };
        metadata
            .page_labels
            .labels
            .push(ferrugo_thumbnail::PageLabel {
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
        assert!(json.contains("\"optional_content\""));
        assert!(json.contains("\"has_oc_properties\":false"));
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
        let output = Path::new("target/ferrugo-thumbnails/text-page.png");

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

        let thin_high_delta_tail = VisualDiffMetrics {
            max_channel_delta: 64,
            ..metrics
        };

        assert_eq!(
            classify_visual_diff(&thin_high_delta_tail, VisualDiffThresholds::default()),
            VisualDiffStatus::AcceptedDrift
        );

        let high_delta = VisualDiffMetrics {
            max_channel_delta: 64,
            p95_channel_delta: 9,
            ..metrics
        };

        assert_eq!(
            classify_visual_diff(&high_delta, VisualDiffThresholds::default()),
            VisualDiffStatus::Blocker
        );
    }

    #[test]
    fn visual_diff_metrics_should_accept_low_p95_edge_drift() {
        let metrics = VisualDiffMetrics {
            width: 160,
            height: 90,
            changed_pixels: 4_450,
            changed_ratio: 0.309028,
            mean_abs_error: 3.3,
            p95_channel_delta: 5,
            max_channel_delta: 216,
            native_nonwhite_pixels: 14_400,
            pdfium_nonwhite_pixels: 14_400,
        };

        assert_eq!(
            classify_visual_diff(&metrics, VisualDiffThresholds::default()),
            VisualDiffStatus::AcceptedDrift
        );

        assert_eq!(
            classify_visual_diff(
                &VisualDiffMetrics {
                    p95_channel_delta: 6,
                    ..metrics
                },
                VisualDiffThresholds::default(),
            ),
            VisualDiffStatus::Blocker
        );

        assert_eq!(
            classify_visual_diff(
                &VisualDiffMetrics {
                    changed_ratio: 0.75,
                    ..metrics
                },
                VisualDiffThresholds::default(),
            ),
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
    fn poppler_scale_args_should_use_native_target_dimensions_for_primary_reference() {
        let args = poppler_scale_args(
            160,
            Some(PopplerTargetDimensions {
                width: 160,
                height: 87,
            }),
            PopplerScaleMode::TargetDimensions,
        );

        assert_eq!(
            args,
            vec![
                OsString::from("-scale-to-x"),
                OsString::from("160"),
                OsString::from("-scale-to-y"),
                OsString::from("87"),
            ]
        );
    }

    #[test]
    fn poppler_scale_args_should_use_uniform_native_max_dimension_for_fallback_reference() {
        let args = poppler_scale_args(
            160,
            Some(PopplerTargetDimensions {
                width: 87,
                height: 160,
            }),
            PopplerScaleMode::UniformMaxDimension,
        );

        assert_eq!(
            args,
            vec![OsString::from("-scale-to"), OsString::from("160")]
        );
    }

    #[test]
    fn poppler_scale_args_should_fallback_to_max_edge_without_native_render() {
        let args = poppler_scale_args(160, None, PopplerScaleMode::TargetDimensions);

        assert_eq!(
            args,
            vec![OsString::from("-scale-to"), OsString::from("160")]
        );
    }

    #[test]
    fn poppler_page_box_args_should_match_native_cropbox_policy() {
        assert_eq!(poppler_page_box_args(), [OsString::from("-cropbox")]);
    }

    #[test]
    fn poppler_target_normalization_should_crop_one_pixel_rounding_drift() {
        let thumbnail = Thumbnail::rgba(
            2,
            3,
            vec![
                1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255, 13, 14, 15, 255, 16, 17,
                18, 255,
            ],
        )
        .expect("valid thumbnail");

        let normalized = normalize_poppler_target_dimensions(
            thumbnail,
            Some(PopplerTargetDimensions {
                width: 2,
                height: 2,
            }),
        )
        .expect("normalization should succeed");

        assert_eq!(normalized.width, 2);
        assert_eq!(normalized.height, 2);
        assert_eq!(
            normalized.bytes,
            vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255,]
        );
    }

    #[test]
    fn poppler_target_normalization_should_pad_one_pixel_rounding_drift() {
        let thumbnail = Thumbnail::rgba(1, 1, vec![1, 2, 3, 255]).expect("valid thumbnail");

        let normalized = normalize_poppler_target_dimensions(
            thumbnail,
            Some(PopplerTargetDimensions {
                width: 2,
                height: 1,
            }),
        )
        .expect("normalization should succeed");

        assert_eq!(normalized.width, 2);
        assert_eq!(normalized.height, 1);
        assert_eq!(normalized.bytes, vec![1, 2, 3, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn ppm_decoder_should_convert_binary_rgb_to_rgba() {
        let ppm = b"P6\n# generated by test\n2 1\n255\n\x00\x01\x02\xfd\xfe\xff";
        let thumbnail = decode_ppm_rgb_as_rgba(ppm).expect("valid PPM");

        assert_eq!(thumbnail.width, 2);
        assert_eq!(thumbnail.height, 1);
        assert_eq!(thumbnail.bytes, vec![0, 1, 2, 255, 253, 254, 255, 255]);
    }

    #[test]
    fn ppm_decoder_should_reject_non_8_bit_rgb() {
        let error =
            decode_ppm_rgb_as_rgba(b"P6\n1 1\n65535\n\x00\x00\x00").expect_err("invalid PPM");

        assert_eq!(
            error.to_string(),
            "PNG encode error: PPM decoder only supports 8-bit RGB data"
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
