use novastack_core::cli::{DbChoice, PluginChoice, TemplateChoice};
use novastack_core::config::ProjectConfig;
use novastack_core::scaffold::{ScaffoldConfig, ScaffoldEngine};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_scaffold_creates_files() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("my-test-api");

    let config = ScaffoldConfig {
        project_name: "my-test-api".to_string(),
        target_dir: target_dir.clone(),
        db: DbChoice::Postgres,
        plugins: vec![PluginChoice::Auth],
        dry_run: false,
        skip_hooks: true,
        template: TemplateChoice::Rest,
    };

    ScaffoldEngine::execute(&config).expect("Scaffolding should succeed");

    assert!(target_dir.join("Cargo.toml").exists());
    assert!(target_dir.join("Dockerfile").exists());
    assert!(target_dir.join("docker-compose.yml").exists());
    assert!(target_dir.join("Justfile").exists());
    assert!(target_dir.join("novastack.toml").exists());
    assert!(target_dir.join("src/main.rs").exists());
    assert!(target_dir.join("src/lib.rs").exists());
    assert!(target_dir.join("src/error.rs").exists());
    assert!(target_dir.join("src/router.rs").exists());
    assert!(target_dir.join("src/auth/jwt.rs").exists());

    let cargo_toml = std::fs::read_to_string(target_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("jsonwebtoken"));
    assert!(cargo_toml.contains("sqlx"));

    // Verify metadata
    let project_cfg = ProjectConfig::load(&target_dir).unwrap();
    assert_eq!(project_cfg.name, "my-test-api");
    assert_eq!(project_cfg.db, DbChoice::Postgres);
    assert!(project_cfg.plugins.contains(&PluginChoice::Auth));

    // Test adding another plugin (redis)
    ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Redis)
        .expect("Adding plugin should succeed");
    assert!(target_dir.join("src/cache.rs").exists());

    let updated_cfg = ProjectConfig::load(&target_dir).unwrap();
    assert!(updated_cfg.plugins.contains(&PluginChoice::Redis));
}

#[test]
fn test_scaffold_stateless_structure() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("stateless-api");

    let config = ScaffoldConfig {
        project_name: "stateless-api".to_string(),
        target_dir: target_dir.clone(),
        db: DbChoice::None,
        plugins: vec![],
        dry_run: false,
        skip_hooks: true,
        template: TemplateChoice::Rest,
    };

    ScaffoldEngine::execute(&config).expect("Scaffolding should succeed");

    assert!(target_dir.join("Cargo.toml").exists());
    assert!(target_dir.join("src/main.rs").exists());
    assert!(target_dir.join("src/lib.rs").exists());
    assert!(target_dir.join("src/router.rs").exists());

    // Verify offline syntax check if cargo can run offline
    let _ = Command::new("cargo")
        .args(["check", "--offline"])
        .current_dir(&target_dir)
        .status();
}
