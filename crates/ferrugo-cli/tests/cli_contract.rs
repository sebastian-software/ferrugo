use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn ferrugo<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_ferrugo"))
        .args(args)
        .output()
        .expect("ferrugo binary should run")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    workspace_root().join("fixtures/generated").join(name)
}

fn unique_target_path(label: &str, extension: &str) -> PathBuf {
    let dir = workspace_root().join("target/cli-contract");
    std::fs::create_dir_all(&dir).expect("target/cli-contract should be created");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    dir.join(format!(
        "{label}-{}-{nanos}.{extension}",
        std::process::id()
    ))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn built_cli_should_print_help_and_version_contract() {
    let help = ferrugo(["--help"]);
    assert!(
        help.status.success(),
        "help should succeed, stderr: {}",
        stderr(&help)
    );
    let help_stdout = stdout(&help);
    assert!(help_stdout.contains("Usage: ferrugo"));
    assert!(help_stdout.contains("render-native"));
    assert!(help_stdout.contains("trace-native"));

    let version = ferrugo(["--version"]);
    assert!(
        version.status.success(),
        "version should succeed, stderr: {}",
        stderr(&version)
    );
    assert_eq!(
        stdout(&version).trim(),
        format!("ferrugo {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn built_cli_should_render_with_default_and_explicit_native_commands() {
    let input = fixture("text-page.pdf");

    for command in ["render", "render-native"] {
        let output_path = unique_target_path(command, "png");
        let output = ferrugo([
            OsString::from(command),
            input.as_os_str().to_os_string(),
            OsString::from("--max-edge"),
            OsString::from("96"),
            OsString::from("--output"),
            output_path.as_os_str().to_os_string(),
        ]);

        assert!(
            output.status.success(),
            "{command} should succeed, stderr: {}",
            stderr(&output)
        );
        let png = std::fs::read(&output_path).expect("render should write PNG output");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));

        if command == "render" {
            assert!(stderr(&output).contains("render backend: native"));
        }
    }
}

#[test]
fn built_cli_should_emit_stable_json_shapes_for_report_commands() {
    let input = fixture("text-page.pdf");

    let trace = ferrugo([
        OsString::from("trace-native"),
        input.as_os_str().to_os_string(),
        OsString::from("--max-edge"),
        OsString::from("64"),
        OsString::from("--max-events"),
        OsString::from("8"),
    ]);
    assert!(
        trace.status.success(),
        "trace-native should succeed, stderr: {}",
        stderr(&trace)
    );
    let trace_stdout = stdout(&trace);
    assert!(trace_stdout.contains("\"trace_kind\": \"native-render-trace\""));
    assert!(trace_stdout.contains("\"render\": {\"status\":\"rendered\""));
    assert!(trace_stdout.contains("\"operator_summary\""));

    let fallback_summary = ferrugo([
        OsString::from("summarize-fallbacks"),
        input.as_os_str().to_os_string(),
        OsString::from("--max-edge"),
        OsString::from("64"),
    ]);
    assert!(
        fallback_summary.status.success(),
        "summarize-fallbacks should succeed, stderr: {}",
        stderr(&fallback_summary)
    );
    let summary_stdout = stdout(&fallback_summary);
    assert!(summary_stdout.contains("\"schema_version\": 1"));
    assert!(summary_stdout.contains("\"native_rendered\": 1"));
    assert!(summary_stdout.contains("\"fallback_required\": 0"));
}

#[test]
fn built_cli_should_report_durable_usage_failures() {
    let output = ferrugo([
        OsString::from("render-native"),
        fixture("text-page.pdf").as_os_str().to_os_string(),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("usage error: missing --output path"));
}

#[test]
fn built_cli_should_reject_removed_pdfium_binding_commands() {
    for command in [
        "render-pdfium",
        "render-isolated",
        "render-worker",
        "compare-metadata",
        "benchmark-pdfium",
    ] {
        let output = ferrugo([command]);

        assert!(!output.status.success());
        assert!(stderr(&output).contains("usage error: unknown command"));
    }
}
