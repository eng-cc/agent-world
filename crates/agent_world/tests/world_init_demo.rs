use std::process::Command;

#[test]
fn world_init_demo_runs_summary_only() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "minimal"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: minimal"));
}
