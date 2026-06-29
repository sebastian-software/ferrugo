use ferrugo_fuzz::{minimal_pdf_with_content, run_target};
use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    AnnotationMode, DocumentMetadataBackend, FormAppearanceMode, PdfSource, Rgba,
    ThumbnailBackend, ThumbnailOptions,
};
use std::time::Duration;

fn main() {
    run_target(
        "render_setup",
        fuzz_one,
        &[
            b"q 0 0 8 8 re f Q",
            b"q 1 0 0 1 0 0 cm /Missing Do Q",
            b"BT /F1 12 Tf (hello) Tj ET",
            b"BI /W 1 /H 1 /BPC 8 ID x EI",
            include_bytes!("../../fixtures/adversarial/truncated-header.pdf"),
            include_bytes!("../../fixtures/adversarial/huge-image-dimensions.pdf"),
        ],
    );
}

fn fuzz_one(data: &[u8]) {
    let backend = NativeBackend::new();
    let options = ThumbnailOptions {
        page_index: 0,
        max_edge: 32,
        background: Rgba::WHITE,
        output_format: ferrugo_thumbnail::OutputFormat::Rgba,
        timeout: Duration::from_millis(100),
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: FormAppearanceMode::DocumentState,
    };

    let _ = DocumentMetadataBackend::inspect(&backend, PdfSource::from_bytes(data));
    let _ = ThumbnailBackend::render(&backend, PdfSource::from_bytes(data), &options);

    let wrapped = minimal_pdf_with_content(data);
    let _ = DocumentMetadataBackend::inspect(&backend, PdfSource::from_bytes(&wrapped));
    let _ = ThumbnailBackend::render(&backend, PdfSource::from_bytes(&wrapped), &options);
}
