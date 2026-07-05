#![cfg_attr(fuzzing, no_main)]

use ferrugo_fuzz::minimal_pdf_with_content;
#[cfg(not(fuzzing))]
use ferrugo_fuzz::run_target;
use ferrugo_object::{load_classic_document, load_modern_document, parse_indirect_object};
use ferrugo_syntax::PdfBytes;
#[cfg(fuzzing)]
use libfuzzer_sys::fuzz_target;

#[cfg(not(fuzzing))]
fn main() {
    run_target(
        "xref_load",
        fuzz_one,
        &[
            b"%PDF-1.7\nxref\n0 1\n0000000000 65535 f \ntrailer\n<< /Size 1 >>\nstartxref\n9\n%%EOF\n",
            b"1 0 obj\n<< /Length 3 >>\nstream\nabc\nendstream\nendobj\n",
            b"xref\n999999999999 1\n0000000000 00000 n \n",
            include_bytes!("../../fixtures/adversarial/truncated-header.pdf"),
        ],
    );
}

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

fn fuzz_one(data: &[u8]) {
    let input = PdfBytes::new(data);
    let _ = parse_indirect_object(input);
    let _ = load_classic_document(input);
    let _ = load_modern_document(input);

    let wrapped = minimal_pdf_with_content(data);
    let wrapped_input = PdfBytes::new(&wrapped);
    let _ = load_classic_document(wrapped_input);
}
