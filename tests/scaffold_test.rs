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
    // No database: accounts live in memory and no users migration is written
    assert!(read("src/state.rs").contains("let users = crate::auth::UserStore::default();"));
    assert!(!target_dir.join("src/db").exists());

    // Installing twice is rejected
    assert!(ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Auth).is_err());
}

#[test]
fn test_auth_with_database_gets_users_table_and_persistent_examples() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("db-api");

    ScaffoldEngine::execute(&ScaffoldConfig {
        project_name: "db-api".to_string(),
        target_dir: target_dir.clone(),
        db: DbChoice::Sqlite,
        plugins: vec![],
        dry_run: false,
        skip_hooks: true,
        template: TemplateChoice::Rest,
    })
    .unwrap();
    let read = |rel: &str| std::fs::read_to_string(target_dir.join(rel)).unwrap();

    // The example resource is a table, not an in-memory map
    assert!(read("src/db/migrations/20240101000001_items.sql").contains("CREATE TABLE"));
    assert!(read("src/api/items.rs").contains("INSERT INTO items"));
    assert!(!read("src/api/items.rs").contains("BTreeMap"));

    // Adding auth later writes the users migration and backs the store with the pool
    ScaffoldEngine::add_plugin(&target_dir, PluginChoice::Auth).unwrap();
    assert!(read("src/db/migrations/20240101000002_users.sql").contains("username"));
    assert!(read("src/state.rs").contains("let users = crate::auth::UserStore::new(db.clone());"));
    assert!(read("src/handlers/auth.rs").contains("/register"));
}

#[test]
fn test_archetype_is_recorded_and_raw_files_are_copied_verbatim() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("ui-app");

    ScaffoldEngine::execute(&ScaffoldConfig {
        project_name: "ui-app".to_string(),
        target_dir: target_dir.clone(),
        db: DbChoice::None,
        plugins: vec![],
        dry_run: false,
        skip_hooks: true,
        template: TemplateChoice::Htmx,
    })
    .unwrap();

    let cfg = ProjectConfig::load(&target_dir).unwrap();
    assert_eq!(cfg.template, TemplateChoice::Htmx);

    // Askama syntax must survive untouched (not rendered by the scaffolder)
    let page = std::fs::read_to_string(target_dir.join("templates/index.html")).unwrap();
    assert!(page.contains("{% extends \"base.html\" %}"));
    assert!(page.contains("{{ title }}"));
}

#[test]
fn test_config_without_template_defaults_to_rest() {
    let temp_dir = tempdir().unwrap();
    std::fs::write(
        temp_dir.path().join("novastack.toml"),
        "name = \"old\"\ndb = \"none\"\nplugins = []\ncreated_at = \"x\"\nnovastack_version = \"0.1.0\"\n",
    )
    .unwrap();
    let cfg = ProjectConfig::load(temp_dir.path()).unwrap();
    assert_eq!(cfg.template, TemplateChoice::Rest);
}
