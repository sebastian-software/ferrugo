use ferrugo_content::tokenize_content;
use ferrugo_fuzz::run_target;
use ferrugo_syntax::PdfBytes;

fn main() {
    run_target(
        "content_tokenize",
        fuzz_one,
        &[
            b"q 1 0 0 1 0 0 cm Q",
            b"BI /W 1 /H 1 /BPC 8 ID x EI",
            b"BT /F1 12 Tf (unterminated Tj ET",
            b"/Name /Other 1 2 3 Do",
            include_bytes!("../../fixtures/adversarial/unterminated-inline-image.content"),
        ],
    );
}

fn fuzz_one(data: &[u8]) {
    for token in tokenize_content(PdfBytes::new(data)) {
        let _ = token;
    }
}
