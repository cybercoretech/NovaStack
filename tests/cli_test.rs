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
        .stdout(predicate::str::contains("src/auth/jwt.rs"))
        .stdout(predicate::str::contains("src/telemetry.rs"));
}

#[test]
fn test_dry_run_includes_dotfiles_and_skips_db_when_none() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.args([
        "new",
        "demo",
        "--dry-run",
        "--no-interactive",
        "--db",
        "none",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".env.example"))
        .stdout(predicate::str::contains(".gitignore"))
        .stdout(predicate::str::contains(".dockerignore"))
        .stdout(predicate::str::contains("src/db/").not())
        .stdout(predicate::str::contains("generate/").not());
}

fn dry_run(template: &str) -> String {
    let output = Command::cargo_bin("novastack")
        .unwrap()
        .args([
            "new",
            "demo",
            "--dry-run",
            "--no-interactive",
            "-t",
            template,
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_each_archetype_adds_its_own_files() {
    let rest = dry_run("rest");
    assert!(rest.contains("src/api/items.rs") && rest.contains("tests/api.rs"));

    let htmx = dry_run("htmx");
    assert!(htmx.contains("templates/index.html") && htmx.contains("src/web/mod.rs"));

    let grpc = dry_run("grpc");
    assert!(grpc.contains("build.rs") && grpc.contains("proto/greeter.proto"));

    let realtime = dry_run("realtime");
    assert!(realtime.contains("static/index.html") && realtime.contains("src/realtime/mod.rs"));

    let minimal = dry_run("minimal");
    for other in ["src/api/", "src/web/", "src/grpc/", "src/realtime/"] {
        assert!(
            !minimal.contains(other),
            "minimal should not include {other}"
        );
    }
}

#[test]
fn test_list_shows_templates() {
    let mut cmd = Command::cargo_bin("novastack").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Build Templates"))
        .stdout(predicate::str::contains("realtime"));
}

#[test]
fn test_list_json_is_valid_and_complete() {
    let output = Command::cargo_bin("novastack")
        .unwrap()
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let catalog: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = catalog["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["rest", "htmx", "grpc", "realtime", "minimal"]);
    assert!(catalog["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["name"] == "auth" && p["files"][0] == "src/auth/mod.rs"));
}
