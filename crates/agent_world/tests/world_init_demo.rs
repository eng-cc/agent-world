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

#[test]
fn world_init_demo_runs_llm_bootstrap_summary() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "llm_bootstrap"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: llm_bootstrap"));
}

#[test]
fn world_init_demo_runs_asteroid_fragment_summary() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "asteroid_fragment_bootstrap"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: asteroid_fragment_bootstrap"));
    assert!(stdout.contains("asteroid_fragment_fragments:"));
}

#[test]
fn world_init_demo_runs_asteroid_fragment_twin_summary() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "asteroid_fragment_twin_region_bootstrap"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: asteroid_fragment_twin_region_bootstrap"));
    assert!(stdout.contains("asteroid_fragment_fragments:"));
}

#[test]
fn world_init_demo_runs_asteroid_fragment_triad_summary() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "asteroid_fragment_triad_region_bootstrap"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: asteroid_fragment_triad_region_bootstrap"));
    assert!(stdout.contains("asteroid_fragment_fragments:"));
}

#[test]
fn world_init_demo_runs_triad_summary() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let output = Command::new(bin)
        .args(["--summary-only", "triad_region_bootstrap"])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: triad_region_bootstrap"));
}

#[test]
fn world_init_demo_runs_from_scenario_file() {
    let bin = env!("CARGO_BIN_EXE_world_init_demo");
    let scenario_path = format!("{}/scenarios/minimal.json", env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(bin)
        .args(["--summary-only", "--scenario-file", &scenario_path])
        .output()
        .expect("run world_init_demo");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scenario: minimal"));
}
