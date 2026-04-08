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
pub use error::{Error, Result};
pub use parser::parse_str;
pub use schema::DeploySpec;
pub use validator::ValidationReport;

/// Parse a deploko.toml file from the given path.
pub fn parse_auto(path: &std::path::Path) -> Result<schema::DeploySpec> {
    parser::parse_file(path)
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
