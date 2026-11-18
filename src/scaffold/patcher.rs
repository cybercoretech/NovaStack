use crate::error::{NovaError, Result};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AnchorPatcher;

impl AnchorPatcher {
    pub const ROUTE_ANCHOR: &'static str = "// NOVASTACK:ROUTES";
    pub const LAYERS_ANCHOR: &'static str = "// NOVASTACK:LAYERS";
    pub const MIDDLEWARE_ANCHOR: &'static str = "// NOVASTACK:MIDDLEWARE";
    pub const HANDLERS_ANCHOR: &'static str = "// NOVASTACK:HANDLERS";
    pub const MODULES_ANCHOR: &'static str = "// NOVASTACK:MODULES";
    pub const STATE_ANCHOR: &'static str = "// NOVASTACK:STATE";
    pub const STATE_SETUP_ANCHOR: &'static str = "// NOVASTACK:STATE_SETUP";
    pub const STATE_INIT_ANCHOR: &'static str = "// NOVASTACK:STATE_INIT";
    pub const CONFIG_FIELDS_ANCHOR: &'static str = "// NOVASTACK:CONFIG_FIELDS";
    pub const CONFIG_LOAD_ANCHOR: &'static str = "// NOVASTACK:CONFIG_LOAD";
    pub const CONFIG_INIT_ANCHOR: &'static str = "// NOVASTACK:CONFIG_INIT";
    pub const STARTUP_ANCHOR: &'static str = "// NOVASTACK:STARTUP";
    pub const DEPS_ANCHOR: &'static str = "# NOVASTACK:DEPENDENCIES";
    pub const DEV_DEPS_ANCHOR: &'static str = "# NOVASTACK:DEV_DEPENDENCIES";
    pub const ENV_ANCHOR: &'static str = "# NOVASTACK:ENV";
    pub const IMPORTS_ANCHOR: &'static str = "// NOVASTACK:IMPORTS";

    /// Injects `insertion` immediately above the anchor comment, preserving indentation.
    pub fn inject_before(source: &str, anchor: &str, insertion: &str) -> String {
        if let Some(pos) = Self::find_anchor(source, anchor) {
            // Find beginning of the anchor line
            let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let indent = &source[line_start..pos];

            let mut result = String::with_capacity(source.len() + insertion.len() + 32);
            result.push_str(&source[..line_start]);

            for line in insertion.lines() {
                if !line.trim().is_empty() {
                    result.push_str(indent);
                    result.push_str(line);
                }
                result.push('\n');
            }

            result.push_str(&source[line_start..]);
            result
        } else {
            source.to_string()
        }
    }

    /// Like [`Self::inject_before`], but fails when the file or the anchor is missing,
    /// so a broken or hand-edited project is reported instead of silently left unpatched.
    pub fn patch(
        files: &mut HashMap<String, String>,
        path: &str,
        anchor: &str,
        insertion: &str,
    ) -> Result<()> {
        let content = files.get_mut(path).ok_or_else(|| {
            NovaError::Other(format!(
                "Required file '{}' is missing from the project",
                path
            ))
        })?;

        if Self::find_anchor(content, anchor).is_none() {
            return Err(NovaError::AnchorNotFound {
                anchor: anchor.to_string(),
                path: PathBuf::from(path),
            });
        }

        *content = Self::inject_before(content, anchor, insertion);
        Ok(())
    }

    /// Patches `path` only when it exists (for optional files such as `.env.example`).
    pub fn patch_optional(
        files: &mut HashMap<String, String>,
        path: &str,
        anchor: &str,
        insertion: &str,
    ) {
        if let Some(content) = files.get_mut(path) {
            *content = Self::inject_before(content, anchor, insertion);
        }
    }

    /// Appends `text` to the end of a required file (e.g. a new `[build-dependencies]` table).
    pub fn append(files: &mut HashMap<String, String>, path: &str, text: &str) -> Result<()> {
        let content = files.get_mut(path).ok_or_else(|| {
            NovaError::Other(format!(
                "Required file '{}' is missing from the project",
                path
            ))
        })?;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(text);
        Ok(())
    }

    /// Appends dependencies or lines to a file if an anchor is found, or appends to end
    pub fn append_to_dependencies(cargo_toml: &str, new_deps: &str) -> String {
        if Self::find_anchor(cargo_toml, Self::DEPS_ANCHOR).is_some() {
            Self::inject_before(cargo_toml, Self::DEPS_ANCHOR, new_deps)
        } else {
            let mut s = cargo_toml.to_string();
            s.push('\n');
            s.push_str(new_deps);
            s
        }
    }

    /// Finds an anchor that stands alone on its line, so that `// NOVASTACK:STATE` does not
    /// match inside `// NOVASTACK:STATE_INIT`.
    fn find_anchor(source: &str, anchor: &str) -> Option<usize> {
        let mut offset = 0;
        for line in source.split_inclusive('\n') {
            if line.trim() == anchor {
                return Some(offset + line.find(anchor).unwrap_or(0));
            }
            offset += line.len();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_before_anchor() {
        let input = "fn main() {\n    // NOVASTACK:ROUTES\n}\n";
        let injection = "let r = route();";
        let output = AnchorPatcher::inject_before(input, AnchorPatcher::ROUTE_ANCHOR, injection);
        assert!(output.contains("    let r = route();\n    // NOVASTACK:ROUTES"));
    }

    #[test]
    fn test_anchor_prefix_does_not_match_longer_anchor() {
        let input = "    // NOVASTACK:STATE_INIT\n    // NOVASTACK:STATE\n";
        let output = AnchorPatcher::inject_before(input, AnchorPatcher::STATE_ANCHOR, "x,");
        assert_eq!(
            output,
            "    // NOVASTACK:STATE_INIT\n    x,\n    // NOVASTACK:STATE\n"
        );
    }

    #[test]
    fn test_patch_reports_missing_anchor() {
        let mut files = HashMap::new();
        files.insert("src/lib.rs".to_string(), "pub mod a;\n".to_string());
        let err = AnchorPatcher::patch(
            &mut files,
            "src/lib.rs",
            AnchorPatcher::IMPORTS_ANCHOR,
            "pub mod b;",
        )
        .unwrap_err();
        assert!(matches!(err, NovaError::AnchorNotFound { .. }));
    }
}
