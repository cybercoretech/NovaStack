use crate::cli::{DbChoice, PluginChoice};
use crate::config::ProjectConfig;
use crate::error::{NovaError, Result};
use crate::plugins::get_plugin;
use crate::scaffold::hooks::HookRunner;
use console::style;
use heck::{ToKebabCase, ToPascalCase, ToSnakeCase};
use minijinja::Environment;
use rust_embed::RustEmbed;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;

#[derive(Debug, Clone)]
pub struct ScaffoldConfig {
    pub project_name: String,
    pub target_dir: PathBuf,
    pub db: DbChoice,
    pub plugins: Vec<PluginChoice>,
    pub dry_run: bool,
    pub skip_hooks: bool,
}

pub struct ScaffoldEngine;

impl ScaffoldEngine {
    pub fn execute(config: &ScaffoldConfig) -> Result<()> {
        println!(
            "{}",
            style(format!(
                "🚀 Scaffolding '{}' with NovaStack...",
                config.project_name
            ))
            .bold()
            .cyan()
        );

        if config.dry_run {
            println!(
                "{}",
                style("[DRY RUN MODE] Simulating file generation without writing to disk.\n")
                    .yellow()
            );
        }

        // 1. Prepare Jinja template environment
        let mut env = Environment::new();
        env.add_filter("kebab_case", |s: String| s.to_kebab_case());
        env.add_filter("snake_case", |s: String| s.to_snake_case());
        env.add_filter("pascal_case", |s: String| s.to_pascal_case());

        // Context object passed into all templates
        let ctx = json!({
            "project_name": config.project_name,
            "project_name_kebab": config.project_name.to_kebab_case(),
            "project_name_snake": config.project_name.to_snake_case(),
            "project_name_pascal": config.project_name.to_pascal_case(),
            "db": match config.db {
                DbChoice::Postgres => "postgres",
                DbChoice::Sqlite => "sqlite",
                DbChoice::None => "none",
            },
            "plugins": config.plugins.iter().map(|p| format!("{:?}", p).to_lowercase()).collect::<Vec<_>>(),
            "has_auth": config.plugins.contains(&PluginChoice::Auth),
            "has_otel": config.plugins.contains(&PluginChoice::Otel),
            "has_redis": config.plugins.contains(&PluginChoice::Redis),
            "has_workers": config.plugins.contains(&PluginChoice::Workers),
        });

        // 2. Render base template files
        let mut files: HashMap<String, String> = HashMap::new();

        for file in Templates::iter() {
            let path_str = file.as_ref();
            if path_str.starts_with("base/") {
                let target_path = path_str
                    .strip_prefix("base/")
                    .unwrap_or(path_str)
                    .strip_suffix(".jinja")
                    .unwrap_or(path_str);

                let raw = Templates::get(path_str).ok_or_else(|| {
                    NovaError::Other(format!("Embedded template not found: {}", path_str))
                })?;

                let template_text = std::str::from_utf8(raw.data.as_ref()).map_err(|e| {
                    NovaError::Other(format!("Invalid UTF-8 in template '{}': {}", path_str, e))
                })?;

                let rendered =
                    env.render_str(template_text, &ctx)
                        .map_err(|e| NovaError::TemplateRender {
                            template: path_str.to_string(),
                            source: e,
                        })?;

                files.insert(target_path.to_string(), rendered);
            }
        }

        // 3. Render plugin template files
        for choice in &config.plugins {
            let plugin = get_plugin(*choice);
            for (template_rel, target_rel) in plugin.template_files() {
                if let Some(raw) = Templates::get(template_rel) {
                    let template_text = std::str::from_utf8(raw.data.as_ref()).map_err(|e| {
                        NovaError::Other(format!(
                            "Invalid UTF-8 in template '{}': {}",
                            template_rel, e
                        ))
                    })?;

                    let rendered = env.render_str(template_text, &ctx).map_err(|e| {
                        NovaError::TemplateRender {
                            template: template_rel.to_string(),
                            source: e,
                        }
                    })?;

                    files.insert(target_rel.to_string(), rendered);
                }
            }
        }

        // 4. Apply plugin patch modifications (Cargo.toml deps, router merges, AppState)
        for choice in &config.plugins {
            let plugin = get_plugin(*choice);
            plugin.apply_patches(&mut files)?;
        }

        // 5. Write to target directory (or print dry run)
        if config.dry_run {
            println!("{}", style("Files to be created:").bold());
            let mut sorted_keys: Vec<_> = files.keys().collect();
            sorted_keys.sort();
            for file in sorted_keys {
                println!("  {} {}", style("+").green(), file);
            }
            println!(
                "\n{}",
                style("Dry run complete. No changes were made.").green()
            );
            return Ok(());
        }

        // Validate directory is empty or doesn't exist
        if config.target_dir.exists() {
            let is_empty = fs::read_dir(&config.target_dir)
                .map_err(|e| NovaError::Io {
                    path: config.target_dir.clone(),
                    source: e,
                })?
                .next()
                .is_none();

            if !is_empty {
                return Err(NovaError::DirectoryNotEmpty {
                    path: config.target_dir.clone(),
                });
            }
        } else {
            fs::create_dir_all(&config.target_dir).map_err(|e| NovaError::Io {
                path: config.target_dir.clone(),
                source: e,
            })?;
        }

        // Write files to disk
        for (rel_path, content) in &files {
            let full_path = config.target_dir.join(rel_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|e| NovaError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }

            fs::write(&full_path, content).map_err(|e| NovaError::Io {
                path: full_path.clone(),
                source: e,
            })?;

            println!("  {} {}", style("CREATED").green().bold(), rel_path);
        }

        // Save project metadata config (novastack.toml)
        let project_cfg = ProjectConfig::new(
            config.project_name.clone(),
            config.db,
            config.plugins.clone(),
        );
        project_cfg.save(&config.target_dir)?;
        println!("  {} novastack.toml", style("CREATED").green().bold());

        // 6. Run post-scaffold hooks
        if !config.skip_hooks {
            println!();
            if let Err(e) = HookRunner::run_git_init(&config.target_dir) {
                eprintln!("  {} Warning: git init failed: {}", style("!").yellow(), e);
            } else {
                println!("  {} Initialized git repository", style("✔").green());
            }

            if let Err(e) = HookRunner::run_cargo_check(&config.target_dir) {
                eprintln!(
                    "  {} Warning: cargo check reported issues:\n{}",
                    style("!").yellow(),
                    e
                );
            } else {
                println!(
                    "  {} Verified compilation with `cargo check`",
                    style("✔").green()
                );
            }
        }

        print_success_summary(config);
        Ok(())
    }

    pub fn add_plugin(project_dir: &Path, plugin_choice: PluginChoice) -> Result<()> {
        let mut cfg = ProjectConfig::load(project_dir)?;
        cfg.add_plugin(plugin_choice)?;

        let plugin = get_plugin(plugin_choice);
        println!(
            "{}",
            style(format!(
                "⚡ Installing plugin '{}' to {}",
                plugin.name(),
                cfg.name
            ))
            .cyan()
            .bold()
        );

        // Load existing files into map for patcher
        let mut files = HashMap::new();
        for rel in &["Cargo.toml", "src/lib.rs", "src/router.rs", "src/state.rs"] {
            let p = project_dir.join(rel);
            if p.exists() {
                let content = fs::read_to_string(&p).map_err(|e| NovaError::Io {
                    path: p.clone(),
                    source: e,
                })?;
                files.insert(rel.to_string(), content);
            }
        }

        // Set up Jinja context for rendering new plugin files
        let mut env = Environment::new();
        env.add_filter("kebab_case", |s: String| s.to_kebab_case());
        env.add_filter("snake_case", |s: String| s.to_snake_case());
        env.add_filter("pascal_case", |s: String| s.to_pascal_case());

        let ctx = json!({
            "project_name": cfg.name,
            "project_name_kebab": cfg.name.to_kebab_case(),
            "project_name_snake": cfg.name.to_snake_case(),
            "project_name_pascal": cfg.name.to_pascal_case(),
            "db": match cfg.db {
                DbChoice::Postgres => "postgres",
                DbChoice::Sqlite => "sqlite",
                DbChoice::None => "none",
            },
        });

        // Write new plugin template files
        for (tmpl_path, target_rel) in plugin.template_files() {
            if let Some(raw) = Templates::get(tmpl_path) {
                let tmpl_str = std::str::from_utf8(raw.data.as_ref()).map_err(|e| {
                    NovaError::Other(format!("Invalid UTF-8 in template '{}': {}", tmpl_path, e))
                })?;
                let rendered =
                    env.render_str(tmpl_str, &ctx)
                        .map_err(|e| NovaError::TemplateRender {
                            template: tmpl_path.to_string(),
                            source: e,
                        })?;

                let dest = project_dir.join(target_rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| NovaError::Io {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
                }
                fs::write(&dest, rendered).map_err(|e| NovaError::Io {
                    path: dest.clone(),
                    source: e,
                })?;
                println!("  {} {}", style("CREATED").green().bold(), target_rel);
            }
        }

        // Apply patches to existing project files
        plugin.apply_patches(&mut files)?;

        // Write updated patched files back to disk
        for (rel, content) in files {
            let p = project_dir.join(&rel);
            fs::write(&p, content).map_err(|e| NovaError::Io {
                path: p.clone(),
                source: e,
            })?;
            println!("  {} {}", style("PATCHED").yellow().bold(), rel);
        }

        // Save updated novastack.toml
        cfg.save(project_dir)?;
        println!("  {} Updated novastack.toml", style("✔").green());
        println!(
            "\n{}",
            style("✨ Plugin installed successfully! Run `cargo check` to verify.")
                .green()
                .bold()
        );

        Ok(())
    }
}

fn print_success_summary(config: &ScaffoldConfig) {
    println!(
        "\n{}",
        style("✨ Project scaffolded successfully!").bold().green()
    );
    println!("{}", style("Next steps:").bold());
    println!("  1. cd {}", style(config.target_dir.display()).cyan());
    println!("  2. cp .env.example .env");
    if config.db == DbChoice::Postgres {
        println!(
            "  3. docker compose up -d db   {}",
            style("# Start local Postgres").dim()
        );
        println!(
            "  4. cargo run                 {}",
            style("# Run your new Axum server").dim()
        );
    } else {
        println!(
            "  3. cargo run                 {}",
            style("# Run your new Axum server").dim()
        );
    }
    println!(
        "\nTry your health check: {}",
        style("curl http://localhost:8080/health").cyan()
    );
}
