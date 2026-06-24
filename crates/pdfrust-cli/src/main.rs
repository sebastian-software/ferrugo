#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use pdfrust_pdfium::PdfiumBackend;
use pdfrust_thumbnail::{
    PdfSource, Rgba, ThumbnailBackend, ThumbnailOptions, DEFAULT_MAX_EDGE, DEFAULT_PAGE_INDEX,
    DEFAULT_TIMEOUT,
};

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<OsString>) -> Result<(), CliError> {
    let command = args.first().and_then(|arg| arg.to_str());
    match command {
        Some("render") => render_command(&args[1..]),
        Some("--version" | "-V") => {
            println!("pdfrust-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help" | "-h") | None => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(CliError::Usage(format!("unknown command `{other}`"))),
    }
}

fn render_command(args: &[OsString]) -> Result<(), CliError> {
    let config = RenderConfig::parse(args)?;
    let backend = PdfiumBackend::from_env().map_err(|err| CliError::Backend(err.to_string()))?;
    let options = ThumbnailOptions {
        page_index: config.page_index,
        max_edge: config.max_edge,
        background: config.background,
        output_format: pdfrust_thumbnail::OutputFormat::Png,
        timeout: config.timeout,
    };
    let source = PdfSource::from_path(&config.input);
    let thumbnail = backend
        .render(source, &options)
        .map_err(|err| CliError::Render {
            class: err.class().as_str(),
            message: err.to_string(),
        })?;
    let png = encode_rgba_png(&thumbnail)?;
    fs::write(&config.output, png).map_err(|source| CliError::Io {
        path: config.output,
        source,
    })?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderConfig {
    input: PathBuf,
    output: PathBuf,
    page_index: u32,
    max_edge: u32,
    background: Rgba,
    timeout: Duration,
}

impl RenderConfig {
    fn parse(args: &[OsString]) -> Result<Self, CliError> {
        let mut input = None;
        let mut output = None;
        let mut page_index = DEFAULT_PAGE_INDEX;
        let mut max_edge = DEFAULT_MAX_EDGE;
        let mut background = Rgba::WHITE;
        let mut timeout = DEFAULT_TIMEOUT;

        let mut index = 0;
        while index < args.len() {
            let arg = args[index]
                .to_str()
                .ok_or_else(|| CliError::Usage("arguments must be valid UTF-8".to_string()))?;
            match arg {
                "--output" | "-o" => {
                    index += 1;
                    output = Some(required_path(args, index, "--output")?);
                }
                "--page-index" => {
                    index += 1;
                    page_index = parse_u32(args, index, "--page-index")?;
                }
                "--max-edge" => {
                    index += 1;
                    max_edge = parse_u32(args, index, "--max-edge")?;
                }
                "--background" => {
                    index += 1;
                    background = parse_background(required_str(args, index, "--background")?)?;
                }
                "--timeout" => {
                    index += 1;
                    let seconds = parse_u64(args, index, "--timeout")?;
                    timeout = Duration::from_secs(seconds);
                }
                value if value.starts_with('-') => {
                    return Err(CliError::Usage(format!("unknown option `{value}`")));
                }
                value => {
                    if input.replace(PathBuf::from(value)).is_some() {
                        return Err(CliError::Usage(
                            "only one input PDF is supported".to_string(),
                        ));
                    }
                }
            }
            index += 1;
        }

        if max_edge == 0 {
            return Err(CliError::Usage(
                "--max-edge must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            input: input.ok_or_else(|| CliError::Usage("missing input PDF".to_string()))?,
            output: output.ok_or_else(|| CliError::Usage("missing --output path".to_string()))?,
            page_index,
            max_edge,
            background,
            timeout,
        })
    }
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Backend(String),
    Render {
        class: &'static str,
        message: String,
    },
    Encode(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "usage error: {message}"),
            Self::Backend(message) => write!(f, "backend error: {message}"),
            Self::Render { class, message } => write!(f, "render error [{class}]: {message}"),
            Self::Encode(message) => write!(f, "PNG encode error: {message}"),
            Self::Io { path, source } => {
                write!(f, "failed to write `{}`: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CliError {}

fn required_path(args: &[OsString], index: usize, option: &str) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(required_str(args, index, option)?))
}

fn required_str<'a>(args: &'a [OsString], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn parse_u32(args: &[OsString], index: usize, option: &str) -> Result<u32, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an unsigned integer")))
}

fn parse_u64(args: &[OsString], index: usize, option: &str) -> Result<u64, CliError> {
    required_str(args, index, option)?
        .parse()
        .map_err(|_| CliError::Usage(format!("{option} must be an unsigned integer")))
}

fn parse_background(value: &str) -> Result<Rgba, CliError> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let parse_channel = |range: std::ops::Range<usize>| {
        let channel = hex.get(range).ok_or_else(|| {
            CliError::Usage("--background must be #RRGGBB or #RRGGBBAA".to_string())
        })?;
        u8::from_str_radix(channel, 16)
            .map_err(|_| CliError::Usage("--background must be #RRGGBB or #RRGGBBAA".to_string()))
    };
    match hex.len() {
        6 => Ok(Rgba {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: 255,
        }),
        8 => Ok(Rgba {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: parse_channel(6..8)?,
        }),
        _ => Err(CliError::Usage(
            "--background must be #RRGGBB or #RRGGBBAA".to_string(),
        )),
    }
}

fn encode_rgba_png(thumbnail: &pdfrust_thumbnail::Thumbnail) -> Result<Vec<u8>, CliError> {
    let width = thumbnail.width;
    let height = thumbnail.height;
    let row_len = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| CliError::Encode("row length overflow".to_string()))?;
    let filtered_len = row_len
        .checked_add(1)
        .and_then(|row| row.checked_mul(height as usize))
        .ok_or_else(|| CliError::Encode("image size overflow".to_string()))?;
    if thumbnail.bytes.len() != row_len * height as usize {
        return Err(CliError::Encode(
            "thumbnail buffer length does not match dimensions".to_string(),
        ));
    }

    let mut filtered = Vec::with_capacity(filtered_len);
    for row in thumbnail.bytes.chunks_exact(row_len) {
        filtered.push(0);
        filtered.extend_from_slice(row);
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_png_chunk(&mut png, b"IHDR", &ihdr)?;
    write_png_chunk(&mut png, b"IDAT", &zlib_store(&filtered)?)?;
    write_png_chunk(&mut png, b"IEND", &[])?;
    Ok(png)
}

fn zlib_store(data: &[u8]) -> Result<Vec<u8>, CliError> {
    let mut out = Vec::with_capacity(data.len() + 6 + (data.len() / 65_535) * 5);
    out.extend_from_slice(&[0x78, 0x01]);
    let block_count = data.chunks(65_535).count();
    for (block_index, block) in data.chunks(65_535).enumerate() {
        let final_block = block_index + 1 == block_count;
        out.push(final_block as u8);
        let len = u16::try_from(block.len())
            .map_err(|_| CliError::Encode("deflate block too large".to_string()))?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    Ok(out)
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) -> Result<(), CliError> {
    let length = u32::try_from(data.len())
        .map_err(|_| CliError::Encode("PNG chunk too large".to_string()))?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32(chunk_type.iter().chain(data.iter()).copied());
    out.extend_from_slice(&crc.to_be_bytes());
    Ok(())
}

fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = 1_u32;
    let mut b = 0_u32;
    for byte in data {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn crc32(bytes: impl IntoIterator<Item = u8>) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn print_usage() {
    println!(
        "Usage: pdfrust-cli render <input.pdf> --output <output.png> \
         [--page-index N] [--max-edge N] [--background #RRGGBB] [--timeout SECONDS]"
    );
}

#[cfg(test)]
mod tests {
    use pdfrust_thumbnail::{PixelFormat, Thumbnail};

    use super::*;

    #[test]
    fn render_config_should_apply_phase_0_defaults() {
        let config = RenderConfig::parse(&[
            OsString::from("fixtures/generated/text-page.pdf"),
            OsString::from("--output"),
            OsString::from("target/text-page.png"),
        ])
        .expect("valid config");

        assert_eq!(config.page_index, 0);
        assert_eq!(config.max_edge, 1024);
        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn parse_background_should_accept_rgb() {
        let color = parse_background("#102030").expect("valid color");

        assert_eq!(
            color,
            Rgba {
                r: 0x10,
                g: 0x20,
                b: 0x30,
                a: 0xff,
            }
        );
    }

    #[test]
    fn encode_rgba_png_should_write_png_signature() {
        let thumbnail = Thumbnail {
            width: 1,
            height: 1,
            stride: 4,
            pixel_format: PixelFormat::Rgba8,
            bytes: vec![255, 0, 0, 255],
        };

        let png = encode_rgba_png(&thumbnail).expect("valid PNG");

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn render_error_should_include_error_class() {
        let error = CliError::Render {
            class: "malformed",
            message: "PDF is malformed".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "render error [malformed]: PDF is malformed"
        );
    }
}
