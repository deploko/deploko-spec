#![deny(missing_docs)]

//! Parser and validator for the Deploko `deploko.toml` spec.
//!
//! This crate provides functionality to parse, validate, and compile Deploko
//! configuration files. It supports the full specification including project
//! metadata, service definitions, and environment-specific overrides.
//!
//! # Example
//!
//! ```rust
//! use deploko_spec::{parse_str, validate, compile};
//!
//! # fn main() -> anyhow::Result<()> {
//! let config = r#"
//! [project]
//! name = "my-app"
//! region = "us-east-1"
//! "#;
//! let spec = parse_str(config)?;
//! let report = validate(&spec);
//! let compiled = compile(&spec, Some("production"))?;
//! # Ok(())
//! # }
//! ```

pub mod compiled;
pub mod error;
pub mod parser;
pub mod schema;
pub mod validator;

// Re-export main types for convenience
pub use compiled::CompiledSpec;
pub use error::{Error, ParseError, Result};
pub use parser::{parse_file, parse_str, parse_toml};
pub use schema::DeploySpec;
pub use validator::ValidationReport;

/// Parse a deploko.toml file from the given directory.
///
/// Looks for `deploko.toml` in the given directory.
/// Returns `ParseError::NoSpecFile` if the file does not exist.
/// Returns `ParseError::Io` if the file cannot be read.
/// Returns `ParseError::Toml` if the file content is invalid TOML.
pub fn parse_auto(dir: &std::path::Path) -> std::result::Result<schema::DeploySpec, ParseError> {
    let path = dir.join("deploko.toml");
    // Let parse_file handle the IO error naturally; avoids TOCTOU race
    match parser::parse_file(&path) {
        Err(ParseError::Io { path: _, source })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            Err(ParseError::NoSpecFile {
                searched_dir: dir.to_path_buf(),
            })
        }
        other => other,
    }
}

/// Validate a deployment specification.
pub fn validate(spec: &schema::DeploySpec) -> ValidationReport {
    validator::validate(spec)
}

/// Compile a specification with environment overrides.
pub fn compile(
    spec: &schema::DeploySpec,
    environment: Option<&str>,
) -> Result<compiled::CompiledSpec> {
    compiled::compile(spec, environment)
}

#[cfg(test)]
mod tests {

    #[test]
    fn library_works() {
        // Basic smoke test to ensure the library compiles
        assert_eq!(2 + 2, 4);
    }
}
