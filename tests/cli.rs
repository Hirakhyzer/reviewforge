use std::fs;
use std::process::Command;

#[test]
fn sample_diff_generates_markdown_report() {
    let root = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO_BIN_EXE_reviewforge"))
        .args(["analyze", "--diff", &format!("{root}/examples/sample.diff"), "--context", "Fixes #42"])
        .output()
        .expect("run reviewforge");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Code Review Report"));
    assert!(stdout.contains("#42"));
    assert!(stdout.contains("Reviewer checklist"));
}

#[test]
fn html_output_is_written() {
    let root = env!("CARGO_MANIFEST_DIR");
    let path = std::env::temp_dir().join(format!("reviewforge-{}.html", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_reviewforge"))
        .args(["analyze", "--diff", &format!("{root}/examples/sample.diff"), "--format", "html", "--output", path.to_str().expect("utf8")])
        .output()
        .expect("run reviewforge");
    assert!(output.status.success());
    let html = fs::read_to_string(&path).expect("read report");
    assert!(html.contains("<!doctype html>"));
    let _ = fs::remove_file(path);
}
