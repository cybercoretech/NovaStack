//! Compile matrix for generated projects: scaffolds real projects and runs
//! `cargo clippy --all-targets -- -D warnings` plus `cargo test` on them.
//!
//! These tests need network access (crates.io) and take a few minutes on a cold cache,
//! so they are ignored by default. Run them with:
//!
//! ```bash
//! cargo test --test generated_compile -- --ignored
//! ```
//!
//! Generated projects share one target directory under `CARGO_TARGET_TMPDIR`, so
//! dependencies are only compiled once across the whole matrix.

use novastack_core::cli::{DbChoice, GenerateKind, PluginChoice, TemplateChoice};
use novastack_core::cli_generate::run_generate;
use novastack_core::scaffold::{ScaffoldConfig, ScaffoldEngine};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const ALL_PLUGINS: [PluginChoice; 4] = [
    PluginChoice::Auth,
    PluginChoice::Otel,
    PluginChoice::Redis,
    PluginChoice::Workers,
];

fn shared_target_dir() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("generated-check")
}

fn scaffold(tmp: &TempDir, name: &str, db: DbChoice, plugins: &[PluginChoice]) -> PathBuf {
    scaffold_template(tmp, name, TemplateChoice::Rest, db, plugins)
}

fn scaffold_template(
    tmp: &TempDir,
    name: &str,
    template: TemplateChoice,
    db: DbChoice,
    plugins: &[PluginChoice],
) -> PathBuf {
    let target_dir = tmp.path().join(name);
    ScaffoldEngine::execute(&ScaffoldConfig {
        project_name: name.to_string(),
        target_dir: target_dir.clone(),
        db,
        plugins: plugins.to_vec(),
        dry_run: false,
        skip_hooks: true,
        template,
    })
    .expect("scaffolding should succeed");
    target_dir
}

fn cargo(project: &Path, args: &[&str]) {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(project)
        .env("CARGO_TARGET_DIR", shared_target_dir())
        .output()
        .expect("failed to run cargo");
    assert!(
        output.status.success(),
        "`cargo {}` failed for {}:\n{}",
        args.join(" "),
        project.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_builds_clean(project: &Path) {
    cargo(
        project,
        &["clippy", "--all-targets", "--", "-D", "warnings"],
    );
    cargo(project, &["test"]);
}

fn check_new(name: &str, db: DbChoice, plugins: &[PluginChoice]) {
    let tmp = TempDir::new().unwrap();
    assert_builds_clean(&scaffold(&tmp, name, db, plugins));
}

#[test]
#[ignore = "needs network; slow"]
fn gen_no_db_no_plugins() {
    check_new("gen-none", DbChoice::None, &[]);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_postgres_no_plugins() {
    check_new("gen-pg", DbChoice::Postgres, &[]);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_sqlite_no_plugins() {
    check_new("gen-sqlite", DbChoice::Sqlite, &[]);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_each_plugin_alone() {
    for plugin in ALL_PLUGINS {
        let name = format!("gen-only-{:?}", plugin).to_lowercase();
        check_new(&name, DbChoice::None, &[plugin]);
    }
}

#[test]
#[ignore = "needs network; slow"]
fn gen_postgres_all_plugins() {
    check_new("gen-pg-all", DbChoice::Postgres, &ALL_PLUGINS);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_sqlite_all_plugins() {
    check_new("gen-sqlite-all", DbChoice::Sqlite, &ALL_PLUGINS);
}

/// Each non-default archetype on its own and with sqlite + every plugin.
fn check_archetype(template: TemplateChoice) {
    let tmp = TempDir::new().unwrap();
    let base = format!("gen-{}", template.as_str());
    assert_builds_clean(&scaffold_template(
        &tmp,
        &base,
        template,
        DbChoice::None,
        &[],
    ));
    let full = format!("{base}-all");
    assert_builds_clean(&scaffold_template(
        &tmp,
        &full,
        template,
        DbChoice::Sqlite,
        &ALL_PLUGINS,
    ));
}

#[test]
#[ignore = "needs network; slow"]
fn gen_htmx_archetype() {
    check_archetype(TemplateChoice::Htmx);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_grpc_archetype() {
    check_archetype(TemplateChoice::Grpc);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_realtime_archetype() {
    check_archetype(TemplateChoice::Realtime);
}

#[test]
#[ignore = "needs network; slow"]
fn gen_minimal_archetype() {
    check_archetype(TemplateChoice::Minimal);
}

/// The minimal archetype has a trimmed dependency set: plugins and every generator must
/// still compile on it when added later.
#[test]
#[ignore = "needs network; slow"]
fn gen_minimal_add_plugins_and_generate_components() {
    let tmp = TempDir::new().unwrap();
    let project = scaffold_template(
        &tmp,
        "gen-min-grow",
        TemplateChoice::Minimal,
        DbChoice::None,
        &[],
    );
    for plugin in ALL_PLUGINS {
        ScaffoldEngine::add_plugin(&project, plugin).expect("add plugin should succeed");
    }
    for kind in [
        GenerateKind::Module,
        GenerateKind::Handler,
        GenerateKind::Service,
        GenerateKind::Dto,
        GenerateKind::Middleware,
    ] {
        run_generate(kind, "UserProfile", &project).expect("generate should succeed");
    }
    assert_builds_clean(&project);
}

/// `novastack add` for every plugin, then `generate` for every component kind, on one project.
#[test]
#[ignore = "needs network; slow"]
fn gen_add_plugins_and_generate_components() {
    let tmp = TempDir::new().unwrap();
    let project = scaffold(&tmp, "gen-grow", DbChoice::Sqlite, &[]);

    for plugin in ALL_PLUGINS {
        ScaffoldEngine::add_plugin(&project, plugin).expect("add plugin should succeed");
    }
    for kind in [
        GenerateKind::Module,
        GenerateKind::Handler,
        GenerateKind::Service,
        GenerateKind::Dto,
        GenerateKind::Middleware,
    ] {
        run_generate(kind, "UserProfile", &project).expect("generate should succeed");
    }
    // A second component of the same kind reuses the existing container modules
    run_generate(GenerateKind::Service, "billing", &project).expect("second service");
    run_generate(GenerateKind::Handler, "orders", &project).expect("second handler");

    assert_builds_clean(&project);
}
