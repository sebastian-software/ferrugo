//! Example that handles native unsupported-feature errors.

use std::env;
use std::path::PathBuf;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    unsupported_feature_buckets, PdfSource, ThumbnailBackend, ThumbnailErrorClass, ThumbnailOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/generated/text-page.pdf"));

    let backend = NativeBackend::low_memory();
    let result = backend.render(PdfSource::from_path(&input), &ThumbnailOptions::default());

    match result {
        Ok(thumbnail) => {
            println!("rendered {}x{}", thumbnail.width, thumbnail.height);
        }
        Err(error) if error.class() == ThumbnailErrorClass::Unsupported => {
            let bucket = error
                .unsupported_feature_bucket()
                .unwrap_or(unsupported_feature_buckets::NATIVE_UNSUPPORTED);
            println!("native renderer unsupported bucket: {bucket}");
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}
