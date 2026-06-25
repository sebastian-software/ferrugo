use std::fs;
use std::path::PathBuf;

pub type Target = fn(&[u8]);

pub fn run_target(name: &str, target: Target, seeds: &[&[u8]]) {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|arg| arg == "--smoke") {
        run_smoke(name, target, seeds);
        return;
    }

    for arg in args {
        let path = PathBuf::from(arg);
        let input = fs::read(&path).unwrap_or_else(|error| {
            panic!("failed to read fuzz input {}: {error}", path.display())
        });
        target(&input);
    }
}

pub fn run_smoke(name: &str, target: Target, seeds: &[&[u8]]) {
    let mut cases = 0usize;
    for seed in seeds.iter().copied() {
        for input in mutated_inputs(seed) {
            target(&input);
            cases += 1;
        }
    }
    for seed in common_seeds() {
        for input in mutated_inputs(seed) {
            target(&input);
            cases += 1;
        }
    }
    println!("{name}: {cases} smoke cases completed");
}

pub fn common_seeds() -> impl Iterator<Item = &'static [u8]> {
    const COMMON_SEEDS: &[&[u8]] = &[
        b"",
        b"\0",
        b"%PDF-1.7\n",
        b"null",
        b"<< /Length 4 >>",
        b"1 0 obj\n<< /Length 4 >>\nstream\ndata\nendstream\nendobj\n",
        b"BI /W 1 /H 1 /BPC 8 ID \xff EI",
        b"q 1 0 0 1 0 0 cm /Im0 Do Q",
        b"999999999999999999999999999999 0 R",
        &[0xff; 32],
    ];
    COMMON_SEEDS.iter().copied()
}

pub fn mutated_inputs(seed: &[u8]) -> Vec<Vec<u8>> {
    let capped = if seed.len() > 4096 {
        &seed[..4096]
    } else {
        seed
    };
    let mut cases = Vec::with_capacity(12);
    cases.push(capped.to_vec());
    cases.push([capped, b"\n"].concat());
    cases.push([b"<<", capped, b">>"].concat());
    cases.push([b"[", capped, b"]"].concat());
    cases.push([b"1 0 obj\n", capped, b"\nendobj\n"].concat());
    cases.push(minimal_pdf_with_content(capped));

    let mut reversed = capped.to_vec();
    reversed.reverse();
    cases.push(reversed);

    let mut toggled = capped.to_vec();
    for (index, byte) in toggled.iter_mut().enumerate() {
        if index % 3 == 0 {
            *byte ^= 0xff;
        }
    }
    cases.push(toggled);

    let half = capped.len() / 2;
    cases.push(capped[..half].to_vec());

    let repeat_count = if capped.is_empty() {
        1
    } else {
        4096usize.saturating_div(capped.len()).clamp(1, 16)
    };
    cases.push(capped.repeat(repeat_count));

    let mut sparse = Vec::with_capacity(capped.len().saturating_mul(2).min(4096));
    for byte in capped.iter().copied().take(2048) {
        sparse.push(byte);
        sparse.push(0);
    }
    cases.push(sparse);

    cases
}

pub fn minimal_pdf_with_content(content: &[u8]) -> Vec<u8> {
    let mut pdf = Vec::new();
    let mut offsets = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.7\n");

    push_object(
        &mut pdf,
        &mut offsets,
        1,
        b"<< /Type /Catalog /Pages 2 0 R >>",
    );
    push_object(
        &mut pdf,
        &mut offsets,
        2,
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    );
    push_object(
        &mut pdf,
        &mut offsets,
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 16 16] /Contents 4 0 R >>",
    );

    offsets.push(pdf.len());
    pdf.extend_from_slice(b"4 0 obj\n<< /Length ");
    pdf.extend_from_slice(content.len().to_string().as_bytes());
    pdf.extend_from_slice(b" >>\nstream\n");
    pdf.extend_from_slice(content);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    let startxref = pdf.len();
    pdf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(b"trailer\n<< /Root 1 0 R /Size 5 >>\nstartxref\n");
    pdf.extend_from_slice(startxref.to_string().as_bytes());
    pdf.extend_from_slice(b"\n%%EOF\n");
    pdf
}

fn push_object(pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, number: u32, body: &[u8]) {
    offsets.push(pdf.len());
    pdf.extend_from_slice(number.to_string().as_bytes());
    pdf.extend_from_slice(b" 0 obj\n");
    pdf.extend_from_slice(body);
    pdf.extend_from_slice(b"\nendobj\n");
}
