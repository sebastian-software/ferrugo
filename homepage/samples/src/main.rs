//! Renders the homepage samples with the published Ferrugo crates.
//!
//! Run from the repository root (the inputs are committed fixtures):
//!
//! - `ferrugo-homepage-samples <previews-dir>` renders every preview, writes
//!   it as PNG into the directory and prints the facts as JSON;
//! - `ferrugo-homepage-samples --check <previews-dir>` renders again, prints
//!   the same JSON and fails when a committed preview differs from the new
//!   render by more than the repository's visual-diff thresholds.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ferrugo_native::{NativeBackend, NativeRenderLimits};
use ferrugo_thumbnail::{
    DocumentMetadataBackend, OutputFormat, PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError,
    ThumbnailOptions, DEFAULT_MAX_EDGE, DEFAULT_TIMEOUT, STABLE_UNSUPPORTED_FEATURE_BUCKETS,
};
use serde_json::{json, Value};

/// The scanned patent: a public U.S. federal document with large scanned pages.
const PATENT: &str = "fixtures/real-world/uspto-patent-6289801.pdf";

/// The preview sizes the opening specimen switches between.
const PREVIEW_EDGES: [u32; 2] = [160, 640];

/// Inputs whose outcome the homepage lists, in its order.
const OUTCOMES: [(&str, &str); 9] = [
    ("patent", PATENT),
    (
        "chrome-print",
        "fixtures/real-world/usagov-report-scams-chrome-print.pdf",
    ),
    ("data-brief", "fixtures/real-world/cdc-data-brief-492.pdf"),
    ("tax-form", "fixtures/real-world/irs-fw9.pdf"),
    ("invoice", "fixtures/generated/business-invoice-dense.pdf"),
    ("encrypted", "fixtures/generated/encrypted-placeholder.pdf"),
    (
        "dynamic-form",
        "fixtures/generated/xfa-dynamic-no-static-appearance.pdf",
    ),
    ("truncated", "fixtures/adversarial/truncated-header.pdf"),
    (
        "huge-image",
        "fixtures/adversarial/huge-image-dimensions.pdf",
    ),
];

/// The repository's default visual-diff thresholds
/// (docs/policies/cross-platform-determinism.md).
const MAX_MEAN_ABS_ERROR: f64 = 2.0;
const MAX_CHANGED_RATIO: f64 = 0.05;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (check, directory) = match args.as_slice() {
        [flag, dir] if flag == "--check" => (true, PathBuf::from(dir)),
        [dir] => (false, PathBuf::from(dir)),
        _ => {
            eprintln!("usage: ferrugo-homepage-samples [--check] <previews-dir>");
            return ExitCode::FAILURE;
        }
    };
    match run(check, &directory) {
        Ok(samples) => {
            println!("{samples}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(check: bool, directory: &Path) -> Result<Value, String> {
    let default = NativeBackend::new();
    let low_memory = NativeBackend::low_memory();

    let metadata = default
        .inspect(PdfSource::from_path(Path::new(PATENT)))
        .map_err(|error| format!("{PATENT}: {error}"))?;
    let first = metadata
        .first_page_size()
        .ok_or_else(|| format!("{PATENT} has no pages"))?;

    let mut previews = Vec::new();
    for edge in PREVIEW_EDGES {
        let thumbnail = render(&default, PATENT, edge)
            .map_err(|error| format!("{PATENT} at {edge}px: {error}"))?;
        let file = format!("patent-{edge}.png");
        let path = directory.join(&file);
        if check {
            compare(&path, &thumbnail)?;
        } else {
            fs::write(&path, encode(&thumbnail)?)
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
        previews.push(json!({
            "maxEdge": edge,
            "width": thumbnail.width,
            "height": thumbnail.height,
            "file": file,
            "lowMemory": outcome(&low_memory, PATENT, edge),
        }));
    }

    let outcomes: Vec<Value> = OUTCOMES
        .iter()
        .map(|(id, input)| {
            json!({
                "id": id,
                "input": input,
                "default": outcome(&default, input, DEFAULT_MAX_EDGE),
                "lowMemory": outcome(&low_memory, input, DEFAULT_MAX_EDGE),
            })
        })
        .collect();

    Ok(json!({
        "ferrugo": ferrugo_thumbnail::version(),
        "defaults": {
            "pageIndex": ThumbnailOptions::default().page_index,
            "maxEdge": DEFAULT_MAX_EDGE,
            "timeoutSeconds": DEFAULT_TIMEOUT.as_secs(),
        },
        "buckets": STABLE_UNSUPPORTED_FEATURE_BUCKETS,
        "profiles": {
            "default": limits(default.render_limits()),
            "lowMemory": limits(low_memory.render_limits()),
        },
        "preview": {
            "input": PATENT,
            "pages": metadata.page_count(),
            "pageWidth": first.width,
            "pageHeight": first.height,
            "producer": metadata.info.producer,
            "renders": previews,
        },
        "outcomes": outcomes,
    }))
}

fn render(
    backend: &NativeBackend,
    input: &str,
    max_edge: u32,
) -> Result<Thumbnail, ThumbnailError> {
    let options = ThumbnailOptions {
        max_edge,
        output_format: OutputFormat::Rgba,
        ..ThumbnailOptions::default()
    };
    backend.render(PdfSource::from_path(Path::new(input)), &options)
}

/// What a first-page render returns: the image size, or the error.
fn outcome(backend: &NativeBackend, input: &str, max_edge: u32) -> Value {
    match render(backend, input, max_edge) {
        Ok(thumbnail) => json!({
            "rendered": true,
            "width": thumbnail.width,
            "height": thumbnail.height,
        }),
        Err(error) => json!({
            "rendered": false,
            "class": error.class().as_str(),
            "bucket": error.unsupported_feature_bucket(),
            "message": error.to_string(),
        }),
    }
}

fn limits(limits: NativeRenderLimits) -> Value {
    json!({
        "maxPagePixels": limits.max_page_pixels,
        "maxImageBytes": limits.max_image_bytes,
        "maxTotalImageBytes": limits.max_total_image_bytes,
        "maxDisplayItems": limits.max_display_items,
    })
}

fn encode(thumbnail: &Thumbnail) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut bytes), thumbnail.width, thumbnail.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
        .write_image_data(&thumbnail.bytes)
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())?;
    Ok(bytes)
}

/// Fails when the committed preview no longer matches a fresh render.
fn compare(path: &Path, thumbnail: &Thumbnail) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("{}: image too large", path.display()))?;
    let mut committed = vec![0; size];
    let info = reader
        .next_frame(&mut committed)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if (info.width, info.height) != (thumbnail.width, thumbnail.height)
        || info.color_type != png::ColorType::Rgba
    {
        return Err(format!(
            "{} is {}x{}, the new render is {}x{}. Run pnpm sample:write.",
            path.display(),
            info.width,
            info.height,
            thumbnail.width,
            thumbnail.height
        ));
    }
    let fresh = &thumbnail.bytes;
    let committed = &committed[..info.buffer_size()];
    let total: u64 = committed
        .iter()
        .zip(fresh)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    let changed = committed
        .as_chunks::<4>()
        .0
        .iter()
        .zip(fresh.as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    let pixels = (thumbnail.width as usize) * (thumbnail.height as usize);
    let mean = total as f64 / fresh.len() as f64;
    let ratio = changed as f64 / pixels as f64;
    if mean > MAX_MEAN_ABS_ERROR || ratio > MAX_CHANGED_RATIO {
        return Err(format!(
            "{} differs from the new render (mean error {mean:.3}, changed {ratio:.4}). Run pnpm sample:write.",
            path.display()
        ));
    }
    Ok(())
}
