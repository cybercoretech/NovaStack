use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn scaffold(tmp: &TempDir, name: &str) {
    Command::cargo_bin("novastack")
        .expect("novastack binary")
        .args(["new", name, "--no-interactive", "--skip-hooks", "--db", "none"])
        .args(["--out", tmp.path().to_str().unwrap()])
        .assert()
        .success();
}

fn generate(project: &Path, kind: &str, name: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("novastack")
        .expect("novastack binary")
        .args(["generate", kind, name])
        .current_dir(project)
        .assert()
}

#[test]
fn generate_module_creates_and_registers_it() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "myproj");
    let project = tmp.path().join("myproj");

    generate(&project, "module", "UserProfile").success();

    assert!(project.join("src/user_profile/mod.rs").exists());
    let lib = fs::read_to_string(project.join("src/lib.rs")).unwrap();
    assert!(lib.contains("pub mod user_profile;"));
    let readme = fs::read_to_string(project.join("README.md")).unwrap();
    assert!(readme.contains("## Generated Components"));
    assert!(readme.contains("- module: user_profile"));
}

#[test]
fn generate_handler_registers_module_and_route() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "myproj");
    let project = tmp.path().join("myproj");

    generate(&project, "handler", "order-items").success();

    assert!(project.join("src/handlers/order_items.rs").exists());
    let handlers = fs::read_to_string(project.join("src/handlers/mod.rs")).unwrap();
    assert!(handlers.contains("pub mod order_items;"));
    let router = fs::read_to_string(project.join("src/router.rs")).unwrap();
    assert!(router.contains(".nest(\"/order-items\", crate::handlers::order_items::router())"));
}

#[test]
fn generate_service_creates_container_module_once() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "myproj");
    let project = tmp.path().join("myproj");

    generate(&project, "service", "billing").success();
    generate(&project, "service", "shipping").success();

    let services = fs::read_to_string(project.join("src/services/mod.rs")).unwrap();
    assert!(services.contains("pub mod billing;"));
    assert!(services.contains("pub mod shipping;"));
    let lib = fs::read_to_string(project.join("src/lib.rs")).unwrap();
    assert_eq!(lib.matches("pub mod services;").count(), 1);
}

#[test]
fn generate_refuses_duplicates_and_reserved_names() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "myproj");
    let project = tmp.path().join("myproj");

    generate(&project, "dto", "user").success();
    generate(&project, "dto", "user").failure();
    generate(&project, "module", "config").failure();
    generate(&project, "handler", "fn").failure();
    generate(&project, "handler", "../escape").failure();
}

#[test]
fn generate_outside_a_project_fails() {
    let tmp = TempDir::new().unwrap();
    generate(tmp.path(), "module", "user").failure();
    assert!(!tmp.path().join("src").exists());
}
