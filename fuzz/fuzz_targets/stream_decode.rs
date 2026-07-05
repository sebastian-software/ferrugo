#![cfg_attr(fuzzing, no_main)]

#[cfg(not(fuzzing))]
use ferrugo_fuzz::run_target;
use ferrugo_object::{parse_indirect_object, ObjectValue, StreamDecodeOptions};
use ferrugo_syntax::PdfBytes;
#[cfg(fuzzing)]
use libfuzzer_sys::fuzz_target;

#[cfg(not(fuzzing))]
fn main() {
    run_target(
        "stream_decode",
        fuzz_one,
        &[
            b"abc",
            b"\x78\x9c\x4b\x4c\x4a\x06\x00\x02\x4d\x01\x27",
            b"\x80\x10H\x86B\x82\x21\x1c\x20",
            b"\x02abc\xffx\x80",
            b"<< /Filter /FlateDecode >>",
            b"<< /Filter /LZWDecode /DecodeParms << /EarlyChange 0 >> >>",
            b"<< /Filter /RunLengthDecode >>",
            b"/ASCIIHexDecode",
        ],
    );
}

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

fn fuzz_one(data: &[u8]) {
    let cases = [
        stream_object(data, b""),
        stream_object(data, b"/Filter /ASCIIHexDecode"),
        stream_object(data, b"/Filter /FlateDecode"),
        stream_object(data, b"/Filter /LZWDecode"),
        stream_object(
            data,
            b"/Filter /LZWDecode /DecodeParms << /EarlyChange 0 >>",
        ),
        stream_object(data, b"/Filter /RunLengthDecode"),
        stream_object(data, b"/Filter [/ASCIIHexDecode /FlateDecode]"),
    ];
    for case in cases {
        let Ok(object) = parse_indirect_object(PdfBytes::new(&case)) else {
            continue;
        };
        let ObjectValue::Stream(stream) = object.value else {
            continue;
        };
        let _ = stream.decode_with_options(StreamDecodeOptions {
            max_decoded_len: 4096,
            initial_capacity: None,
        });
    }
}

fn stream_object(data: &[u8], filter: &[u8]) -> Vec<u8> {
    let mut object = Vec::new();
    object.extend_from_slice(b"1 0 obj\n<< /Length ");
    object.extend_from_slice(data.len().to_string().as_bytes());
    if !filter.is_empty() {
        object.push(b' ');
        object.extend_from_slice(filter);
    }
    object.extend_from_slice(b" >>\nstream\n");
    object.extend_from_slice(data);
    object.extend_from_slice(b"\nendstream\nendobj\n");
    object
}
