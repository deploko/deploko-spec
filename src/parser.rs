//! TOML parser for Deploko deployment specifications.
//!
//! This module provides functionality to parse `deploko.toml` files into
//! structured `DeploySpec` objects.

use crate::error::ParseError;
use crate::schema::DeploySpec;
use std::path::Path;
use std::sync::Arc;

/// Parse TOML content into a `DeploySpec`.
///
/// # Errors
///
/// Returns `ParseError::Toml` if the TOML content is invalid or cannot be
/// deserialized into a `DeploySpec`.
pub fn parse_toml(input: &str) -> Result<DeploySpec, ParseError> {
    toml::from_str::<DeploySpec>(input).map_err(|e| {
        // Calculate line and column from span if available
        let (line, col) = if let Some(span) = e.span() {
            let start = span.start;
            let line = input[..start].chars().filter(|&c| c == '\n').count() + 1;
            let col = input[..start]
                .chars()
                .rev()
                .take_while(|&c| c != '\n')
                .count()
                + 1;
            (Some(line), Some(col))
        } else {
            (None, None)
        };
        ParseError::Toml {
            line,
            col,
            message: e.message().to_string(),
        }
    })
}

/// Parse a deploko.toml file from the given path.
///
/// # Errors
///
/// Returns `ParseError::UnknownFormat` if the file extension is not `.toml`.
/// Returns `ParseError::Io` if the file cannot be read.
/// Returns `ParseError::Toml` if the file content is invalid TOML.
pub fn parse_file(path: &Path) -> Result<DeploySpec, ParseError> {
    // Validate file extension
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension != "toml" {
        return Err(ParseError::UnknownFormat {
            extension: extension.to_string(),
        });
    }

    let content = std::fs::read_to_string(path).map_err(|e| ParseError::Io {
        path: path.to_path_buf(),
        source: Arc::new(e),
    })?;

    parse_toml(&content)
}

/// Parse a deploko.toml configuration from a string.
///
/// This is an alias for `parse_toml` for backward compatibility.
pub fn parse_str(content: &str) -> Result<DeploySpec, ParseError> {
    parse_toml(content)
}

/// Find and parse a deploko.toml file in the given directory or its parents.
///
/// Walks up the directory tree from `start_dir` to the root, looking for
/// a file named `deploko.toml` in each directory.
///
/// # Errors
///
/// Returns `ParseError::NoSpecFile` if no deploko.toml is found in the
/// starting directory or any of its parent directories.
/// Returns `ParseError::Io` if the file cannot be read.
/// Returns `ParseError::Toml` if the file content is invalid TOML.
pub fn find_and_parse(start_dir: &Path) -> Result<DeploySpec, ParseError> {
    let mut current_dir = start_dir;

    loop {
        let config_path = current_dir.join("deploko.toml");
        match parse_file(&config_path) {
            Ok(spec) => return Ok(spec),
            Err(ParseError::Io { path: _, source })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                // File doesn't exist in this directory, try parent
            }
            Err(other) => return Err(other),
        }

        // Move to parent directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }

    Err(ParseError::NoSpecFile {
        searched_dir: start_dir.to_path_buf(),
    })
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

        // Verify error has correct type and line/col info
        match result.unwrap_err() {
            ParseError::Toml { line, col, message } => {
                assert!(line.is_some(), "Expected line number in error");
                assert!(col.is_some(), "Expected column number in error");
                // Error message should not be empty
                assert!(!message.is_empty(), "Error message should not be empty");
            }
            other => panic!("Expected ParseError::Toml, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_toml_deserialization_error() {
        // Invalid region value that doesn't match any enum variant
        let config = r#"
[project]
name = "test"
region = "invalid-region"
"#;

        let result = parse_toml(config);
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::Toml { line, col, message } => {
                assert!(!message.is_empty(), "Error message should not be empty");
                // Line/col may or may not be present for deserialization errors
                let _ = line;
                let _ = col;
            }
            other => panic!("Expected ParseError::Toml, got {:?}", other),
        }
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

    #[test]
    fn test_parse_file_unknown_format() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("deploko.yaml");
        fs::write(&yaml_path, "project:\n  name: test").unwrap();

        let result = parse_file(&yaml_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::UnknownFormat { extension } => {
                assert_eq!(extension, "yaml");
            }
            other => panic!("Expected ParseError::UnknownFormat, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_file_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let no_ext_path = temp_dir.path().join("deploko");
        fs::write(&no_ext_path, "[project]\nname = \"test\"").unwrap();

        let result = parse_file(&no_ext_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::UnknownFormat { extension } => {
                assert_eq!(extension, "");
            }
            other => panic!("Expected ParseError::UnknownFormat, got {:?}", other),
        }
    }
}
