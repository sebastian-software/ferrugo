use std::env;
use std::path::PathBuf;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{DocumentMetadataBackend, PdfSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/generated/text-page.pdf"));

    let backend = NativeBackend::new();
    let metadata = backend.inspect(PdfSource::from_path(&input))?;

    match metadata.first_page_size() {
        Some(size) => {
            println!(
                "{} pages, first page {:.1}x{:.1}",
                metadata.page_count(),
                size.width,
                size.height
            );
        }
        None => println!("0 pages"),
    }

    Ok(())
}
