use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NovaStack scaffolds modular"));
}

#[test]
fn test_cli_list() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available Plugins"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("otel"))
        .stdout(predicate::str::contains("redis"))
        .stdout(predicate::str::contains("workers"));
}

#[test]
fn test_cli_doctor() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NovaStack System Diagnostics"));
}

#[test]
fn test_scaffold_dry_run() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.args([
        "new",
        "demo-test-api",
        "--dry-run",
        "--no-interactive",
        "--db",
        "postgres",
        "--plugins",
        "auth,otel",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[DRY RUN MODE]"))
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("src/main.rs"));
}
