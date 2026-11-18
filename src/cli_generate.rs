use crate::cli::GenerateKind;
use crate::config::ProjectConfig;
use crate::error::{NovaError, Result};
use crate::scaffold::engine::{template_env, Templates};
use crate::scaffold::patcher::AnchorPatcher as P;
use console::style;
use heck::{ToKebabCase, ToPascalCase, ToSnakeCase};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

impl GenerateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerateKind::Module => "module",
            GenerateKind::Handler => "handler",
            GenerateKind::Service => "service",
            GenerateKind::Dto => "dto",
            GenerateKind::Middleware => "middleware",
        }
    }

    fn template(&self) -> &'static str {
        match self {
            GenerateKind::Module => "generate/module/mod.rs.jinja",
            GenerateKind::Handler => "generate/handler/mod.rs.jinja",
            GenerateKind::Service => "generate/service/mod.rs.jinja",
            GenerateKind::Dto => "generate/dto/mod.rs.jinja",
            GenerateKind::Middleware => "generate/middleware/mod.rs.jinja",
        }
    }
}

/// Generates a component inside an existing NovaStack project and registers it so the
/// project keeps compiling: the module is declared and, for handlers, routed.
pub fn run_generate(kind: GenerateKind, name: &str, project_dir: &Path) -> Result<()> {
    // Only operate inside a NovaStack project
    ProjectConfig::load(project_dir)?;

    let snake = component_ident(name)?;
    let pascal = name.to_pascal_case();
    let kebab = name.to_kebab_case();

    // Target file plus the registrations (file, anchor, line) that make it reachable
    let (target_rel, registrations): (String, Vec<(String, &str, String)>) = match kind {
        GenerateKind::Module => (
            format!("src/{snake}/mod.rs"),
            vec![(
                "src/lib.rs".into(),
                P::IMPORTS_ANCHOR,
                format!("pub mod {snake};"),
            )],
        ),
        GenerateKind::Handler => (
            format!("src/handlers/{snake}.rs"),
            vec![
                (
                    "src/handlers/mod.rs".into(),
                    P::HANDLERS_ANCHOR,
                    format!("pub mod {snake};"),
                ),
                (
                    "src/router.rs".into(),
                    P::ROUTE_ANCHOR,
                    format!(".nest(\"/{kebab}\", crate::handlers::{snake}::router())"),
                ),
            ],
        ),
        GenerateKind::Middleware => (
            format!("src/middleware/{snake}.rs"),
            vec![(
                "src/middleware/mod.rs".into(),
                P::MIDDLEWARE_ANCHOR,
                format!("pub mod {snake};"),
            )],
        ),
        GenerateKind::Service => (
            format!("src/services/{snake}.rs"),
            vec![(
                "src/services/mod.rs".into(),
                P::MODULES_ANCHOR,
                format!("pub mod {snake};"),
            )],
        ),
        GenerateKind::Dto => (
            format!("src/dtos/{snake}.rs"),
            vec![(
                "src/dtos/mod.rs".into(),
                P::MODULES_ANCHOR,
                format!("pub mod {snake};"),
            )],
        ),
    };

    let target = project_dir.join(&target_rel);
    let module_file_clash =
        kind == GenerateKind::Module && project_dir.join(format!("src/{snake}.rs")).exists();
    if target.exists() || module_file_clash {
        return Err(NovaError::Other(format!(
            "{} '{}' already exists at {}",
            kind.as_str(),
            snake,
            target_rel
        )));
    }

    // Load every file we are going to patch
    let mut files: HashMap<String, String> = HashMap::new();
    let mut created_containers = Vec::new();
    for (rel, _, _) in &registrations {
        let path = project_dir.join(rel);
        let content = if path.exists() {
            read(&path)?
        } else if matches!(kind, GenerateKind::Service | GenerateKind::Dto) {
            // First service/dto: create the container module and declare it in lib.rs
            created_containers.push(rel.clone());
            format!("{}\n", P::MODULES_ANCHOR)
        } else {
            return Err(NovaError::Other(format!(
                "Required file '{}' is missing from the project",
                rel
            )));
        };
        files.insert(rel.clone(), content);
    }
    if !created_containers.is_empty() {
        let container = if kind == GenerateKind::Service {
            "services"
        } else {
            "dtos"
        };
        files.insert("src/lib.rs".into(), read(&project_dir.join("src/lib.rs"))?);
        P::patch(
            &mut files,
            "src/lib.rs",
            P::IMPORTS_ANCHOR,
            &format!("pub mod {container};"),
        )?;
    }

    for (rel, anchor, line) in &registrations {
        P::patch(&mut files, rel, anchor, line)?;
    }

    // Render the component
    let raw = Templates::get(kind.template()).ok_or_else(|| {
        NovaError::Other(format!("Embedded template not found: {}", kind.template()))
    })?;
    let template_text = std::str::from_utf8(raw.data.as_ref()).map_err(|e| {
        NovaError::Other(format!(
            "Invalid UTF-8 in template '{}': {}",
            kind.template(),
            e
        ))
    })?;
    let ctx = json!({
        "name": name,
        "name_snake": snake,
        "name_pascal": pascal,
        "name_kebab": kebab,
    });
    let rendered = template_env()
        .render_str(template_text, &ctx)
        .map_err(|e| NovaError::TemplateRender {
            template: kind.template().to_string(),
            source: e,
        })?;

    // Everything rendered and patched in memory; now write to disk
    write(&target, &rendered)?;
    println!("  {} {}", style("CREATED").green().bold(), target_rel);
    for (rel, content) in &files {
        write(&project_dir.join(rel), content)?;
        let label = if created_containers.contains(rel) {
            style("CREATED").green().bold()
        } else {
            style("PATCHED").yellow().bold()
        };
        println!("  {} {}", label, rel);
    }

    update_readme(project_dir, kind, &snake)?;
    if kind == GenerateKind::Handler {
        println!(
            "\n{} Routed at {}",
            style("✨").green(),
            style(format!("/{kebab}")).cyan()
        );
    }
    Ok(())
}

/// Validates a component name and returns its snake_case Rust identifier.
fn component_ident(name: &str) -> Result<String> {
    let invalid = |reason: &str| NovaError::Other(format!("Invalid name '{name}': {reason}"));

    if name.is_empty() {
        return Err(invalid("name cannot be empty"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(invalid(
            "only letters, numbers, hyphens and underscores are allowed",
        ));
    }

    let snake = name.to_snake_case();
    if snake.is_empty() || !snake.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(invalid("name must start with a letter"));
    }
    if RESERVED.contains(&snake.as_str()) {
        return Err(invalid(
            "it is a Rust keyword or clashes with a built-in project module",
        ));
    }
    Ok(snake)
}

const RESERVED: &[&str] = &[
    // Rust keywords
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "gen",
    "macro",
    "override",
    "priv",
    "try",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    // Modules every NovaStack project already has
    "config",
    "error",
    "handlers",
    "middleware",
    "router",
    "state",
    "db",
    "services",
    "dtos",
    "health",
    "main",
    "lib",
];

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| NovaError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| NovaError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    fs::write(path, content).map_err(|e| NovaError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn update_readme(project_dir: &Path, kind: GenerateKind, name: &str) -> Result<()> {
    let readme_path = project_dir.join("README.md");
    let mut content = fs::read_to_string(&readme_path).unwrap_or_default();
    let heading = "## Generated Components";
    if !content.contains(heading) {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("\n{}\n\n", heading));
    }
    let entry = format!("- {}: {}\n", kind.as_str(), name);
    if !content.contains(&entry) {
        content.push_str(&entry);
    }
    write(&readme_path, &content)
}
