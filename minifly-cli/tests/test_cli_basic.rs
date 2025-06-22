use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Local Fly.io development simulator"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("apps"))
        .stdout(predicate::str::contains("machine"))
        .stdout(predicate::str::contains("serve"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("minifly"));
}

#[test]
fn test_cli_apps_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("apps").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage Fly apps"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("destroy"));
}

#[test]
fn test_cli_machine_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("machine").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage Fly machines"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"));
}

#[test]
fn test_cli_volume_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("volume").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage Fly volumes"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("destroy"));
}

#[test]
fn test_cli_init_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("init").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize a new Fly app"));
}

#[test]
fn test_cli_serve_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("serve").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start the Minifly API server"))
        .stdout(predicate::str::contains("Options:"))
        .stdout(predicate::str::contains("--port"));
}

#[test]
fn test_cli_status_help() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("status").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show the status of Minifly"));
}

#[test]
fn test_cli_env_var_api_base_url() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.env("MINIFLY_API_BASE_URL", "http://custom:9999");
    cmd.arg("apps").arg("list");
    
    // Should fail to connect to custom URL
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"))
        .stderr(predicate::str::contains("9999").or(predicate::str::contains("custom")));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("invalid-command");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_cli_missing_required_args() {
    let mut cmd = Command::cargo_bin("minifly").unwrap();
    cmd.arg("apps").arg("create");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}