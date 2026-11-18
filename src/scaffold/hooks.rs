use crate::error::{NovaError, Result};
use console::style;
use std::path::Path;
use std::process::Command;

pub struct HookRunner;

impl HookRunner {
    pub fn run_git_init(project_dir: &Path) -> Result<()> {
        if which::which("git").is_err() {
            return Ok(());
        }

        // Only git init if .git doesn't exist
        if project_dir.join(".git").exists() {
            return Ok(());
        }

        let output = Command::new("git")
            .arg("init")
            .current_dir(project_dir)
            .output()
            .map_err(|e| NovaError::Io {
                path: project_dir.to_path_buf(),
                source: e,
            })?;

        if !output.status.success() {
            let msg = String::from_utf8_lossy(&output.stderr);
            return Err(NovaError::HookFailed {
                command: "git init".to_string(),
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    pub fn run_cargo_check(project_dir: &Path) -> Result<()> {
        if which::which("cargo").is_err() {
            return Ok(());
        }

        println!(
            "  {}",
            style("Verifying compilation via `cargo check`...").dim()
        );

        let output = Command::new("cargo")
            .arg("check")
            .current_dir(project_dir)
            .output()
            .map_err(|e| NovaError::Io {
                path: project_dir.to_path_buf(),
                source: e,
            })?;

        if !output.status.success() {
            let msg = String::from_utf8_lossy(&output.stderr);
            return Err(NovaError::HookFailed {
                command: "cargo check".to_string(),
                message: msg.to_string(),
            });
        }

        Ok(())
    }
}
