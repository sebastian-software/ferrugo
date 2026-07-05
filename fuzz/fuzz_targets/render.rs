#![cfg_attr(fuzzing, no_main)]

use ferrugo_fuzz::minimal_pdf_with_content;
#[cfg(not(fuzzing))]
use ferrugo_fuzz::run_target;
use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    AnnotationMode, FormAppearanceMode, PdfSource, Rgba, ThumbnailBackend, ThumbnailOptions,
};
#[cfg(fuzzing)]
use libfuzzer_sys::fuzz_target;
use std::time::Duration;

#[cfg(not(fuzzing))]
fn main() {
    run_target(
        "render",
        fuzz_one,
        &[
            include_bytes!("../../fixtures/adversarial/truncated-header.pdf"),
            include_bytes!("../../fixtures/adversarial/huge-image-dimensions.pdf"),
            include_bytes!("../../fixtures/generated/image-xobject.pdf"),
            include_bytes!("../../fixtures/generated/lzw-image-xobject.pdf"),
            include_bytes!("../../fixtures/generated/runlength-image-xobject.pdf"),
            include_bytes!("../../fixtures/generated/ccitt-g3-1d-image-mask.pdf"),
            include_bytes!("../../fixtures/generated/ccitt-g4-devicegray-blackis1.pdf"),
        ],
    );
}

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

fn fuzz_one(data: &[u8]) {
    let backend = NativeBackend::new();
    let options = ThumbnailOptions {
        page_index: 0,
        max_edge: 48,
        background: Rgba::WHITE,
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: Duration::from_millis(100),
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: FormAppearanceMode::DocumentState,
    };

    let _ = ThumbnailBackend::render(&backend, PdfSource::from_bytes(data), &options);

    let wrapped = minimal_pdf_with_content(data);
    let _ = ThumbnailBackend::render(&backend, PdfSource::from_bytes(&wrapped), &options);
}
