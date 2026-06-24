use std::process::ExitCode;

use pdfrust_pdfium::PdfiumBackend;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let backend = PdfiumBackend::from_env()?;
    let probe = backend.smoke_test()?;
    println!(
        "initialized={} last_error={} library={}",
        probe.initialized,
        probe.last_error,
        probe.library_path.display()
    );
    Ok(())
}
