use std::env;
use std::path::PathBuf;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    OutputFormat, PdfSource, ThumbnailBackend, ThumbnailOptions, DEFAULT_MAX_EDGE,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/generated/text-page.pdf"));

    let backend = NativeBackend::new();
    let options = ThumbnailOptions {
        max_edge: 256.min(DEFAULT_MAX_EDGE),
        output_format: OutputFormat::Rgba,
        ..ThumbnailOptions::default()
    };
    let thumbnail = backend.render(PdfSource::from_path(&input), &options)?;

    println!(
        "{}x{} {} bytes from {}",
        thumbnail.width,
        thumbnail.height,
        thumbnail.bytes.len(),
        backend.backend_name()
    );

    Ok(())
}
