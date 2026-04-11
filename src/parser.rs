//! TOML parser for Deploko deployment specifications.
//!
//! This module provides functionality to parse `deploko.toml` files into
//! structured `DeploySpec` objects.

use crate::error::{Error, Result};
use crate::schema::DeploySpec;
use std::path::Path;

/// Parse a deploko.toml file from the given path.
pub fn parse_file(path: &Path) -> Result<DeploySpec> {
    if !path.exists() {
        return Err(Error::ParseError(format!(
            "Configuration file not found: {}",
            path.display()
        )));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::ParseError(format!("Failed to read file: {}", e)))?;

    parse_str(&content)
}

/// Parse a deploko.toml configuration from a string.
pub fn parse_str(content: &str) -> Result<DeploySpec> {
    let spec: DeploySpec = toml::from_str(content)
        .map_err(|e| Error::ParseError(format!("Failed to parse TOML: {}", e)))?;

    Ok(spec)
}

/// Find and parse a deploko.toml file in the given directory or its parents.
pub fn find_and_parse(start_dir: &Path) -> Result<DeploySpec> {
    let mut current_dir = start_dir;

    loop {
        let config_path = current_dir.join("deploko.toml");
        if config_path.exists() {
            return parse_file(&config_path);
        }

        // Move to parent directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }

    Err(Error::ParseError(
        "deploko.toml not found in current directory or any parent directory".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_valid_config() {
        let config = r#"
[project]
name = "test-app"
region = "us-east-1"

[frontend]
framework = "static"
repo = "https://github.com/example/app.git"
branch = "main"
build_command = "npm run build"
"#;

        use crate::schema::{Framework, Region};

        let spec = parse_str(config).unwrap();
        assert_eq!(spec.project.name, "test-app");
        assert_eq!(spec.project.region, Region::UsEast1);
        assert!(spec.frontend.is_some());

        let frontend = spec.frontend.unwrap();
        assert_eq!(frontend.framework, Framework::Static);
        assert_eq!(frontend.repo, "https://github.com/example/app.git");
    }

    #[test]
    fn test_parse_minimal_config() {
        let config = r#"
[project]
name = "minimal-app"
region = "us-east-1"
"#;

        use crate::schema::Region;

        let spec = parse_str(config).unwrap();
        assert_eq!(spec.project.name, "minimal-app");
        assert_eq!(spec.project.region, Region::UsEast1);
        assert!(spec.frontend.is_none());
        assert!(spec.backend.is_none());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let config = r#"
[project
name = "invalid-toml"
"#;

        let result = parse_str(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_and_parse() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("deploko.toml");

        let config = r#"
[project]
name = "found-app"
region = "eu-central-1"
"#;

        fs::write(&config_path, config).unwrap();

        let spec = find_and_parse(temp_dir.path()).unwrap();
        assert_eq!(spec.project.name, "found-app");
    }

    #[test]
    fn test_find_and_parse_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_and_parse(temp_dir.path());
        assert!(result.is_err());
    }
}
