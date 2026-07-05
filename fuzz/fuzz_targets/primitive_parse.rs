#![cfg_attr(fuzzing, no_main)]

#[cfg(not(fuzzing))]
use ferrugo_fuzz::run_target;
use ferrugo_syntax::{parse_primitive, parse_primitive_prefix, PdfBytes};
#[cfg(fuzzing)]
use libfuzzer_sys::fuzz_target;

#[cfg(not(fuzzing))]
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

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

fn fuzz_one(data: &[u8]) {
    let input = PdfBytes::new(data);
    let _ = parse_primitive(input);
    let _ = parse_primitive_prefix(input);
}
