use ferrugo_fuzz::run_target;
use ferrugo_syntax::{parse_primitive, parse_primitive_prefix, PdfBytes};

fn main() {
    run_target(
        "primitive_parse",
        fuzz_one,
        &[
            b"[1 2 3]",
            b"<< /A [1 0 R (unterminated",
            b"/Name#ff#00",
            b"999999999999999999999999999999",
            include_bytes!("../../fixtures/adversarial/deep-primitive-array.input"),
        ],
    );
}

fn fuzz_one(data: &[u8]) {
    let input = PdfBytes::new(data);
    let _ = parse_primitive(input);
    let _ = parse_primitive_prefix(input);
}
