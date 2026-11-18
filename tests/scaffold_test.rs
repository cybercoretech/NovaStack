use novastack_core::cli::{DbChoice, PluginChoice, TemplateChoice};
use novastack_core::config::ProjectConfig;
use novastack_core::scaffold::{ScaffoldConfig, ScaffoldEngine};
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
    assert!(target_dir.join(".env.example").exists());
    assert!(target_dir.join(".gitignore").exists());

    // No database: no db module, migrations or sqlx dependency
    assert!(!target_dir.join("src/db").exists());
    let cargo_toml = std::fs::read_to_string(target_dir.join("Cargo.toml")).unwrap();
    assert!(!cargo_toml.contains("sqlx"));
    // Compilation of generated projects is covered by tests/generated_compile.rs
}

#[test]
fn test_add_plugin_patches_config_and_state() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("grow-api");

    ScaffoldEngine::execute(&ScaffoldConfig {
        project_name: "grow-api".to_string(),
        target_dir: target_dir.clone(),
        db: DbChoice::None,
        plugins: vec![],
        dry_run: false,
        skip_hooks: true,
        template: TemplateChoice::Rest,
    })
    .unwrap();

    ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Auth).unwrap();
    ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Redis).unwrap();

    let read = |rel: &str| std::fs::read_to_string(target_dir.join(rel)).unwrap();
    assert!(read("src/config.rs").contains("pub jwt_secret: String,"));
    assert!(read("src/config.rs").contains("redis_url,"));
    assert!(read("src/state.rs").contains("pub redis: deadpool_redis::Pool,"));
    assert!(read("src/handlers/mod.rs").contains("pub mod auth;"));
    assert!(read(".env.example").contains("JWT_SECRET="));

    // Installing twice is rejected
    assert!(ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Auth).is_err());
}
