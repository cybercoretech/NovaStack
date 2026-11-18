pub struct AnchorPatcher;

impl AnchorPatcher {
    pub const ROUTE_ANCHOR: &'static str = "// NOVASTACK:ROUTES";
    pub const MIDDLEWARE_ANCHOR: &'static str = "// NOVASTACK:MIDDLEWARE";
    pub const STATE_ANCHOR: &'static str = "// NOVASTACK:STATE";
    pub const DEPS_ANCHOR: &'static str = "# NOVASTACK:DEPENDENCIES";
    pub const IMPORTS_ANCHOR: &'static str = "// NOVASTACK:IMPORTS";

    /// Injects `insertion` immediately above the anchor comment, preserving indentation.
    pub fn inject_before(source: &str, anchor: &str, insertion: &str) -> String {
        if let Some(pos) = source.find(anchor) {
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

    /// Appends dependencies or lines to a file if an anchor is found, or appends to end
    pub fn append_to_dependencies(cargo_toml: &str, new_deps: &str) -> String {
        if cargo_toml.contains(Self::DEPS_ANCHOR) {
            Self::inject_before(cargo_toml, Self::DEPS_ANCHOR, new_deps)
        } else {
            let mut s = cargo_toml.to_string();
            s.push('\n');
            s.push_str(new_deps);
            s
        }
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
}
