//! Error types for the deploko-spec crate.
//!
//! This module defines the error types used throughout the crate for
//! parsing, validation, and compilation operations.

use std::fmt;

/// Result type alias for the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the deploko-spec crate.
#[derive(Debug, Clone)]
pub enum Error {
    /// Error during parsing of configuration files
    ParseError(String),

    /// Error during validation of configuration
    ValidationError(String),

    /// Error during compilation of configuration
    CompilationError(String),

    /// Error during serialization/deserialization
    SerializationError(String),

    /// I/O error
    IoError(String),

    /// Error with environment variable references
    EnvironmentError(String),

    /// Error with secret references
    SecretError(String),

    /// Generic error with custom message
    Generic(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Error::IoError(msg) => write!(f, "I/O error: {}", msg),
            Error::EnvironmentError(msg) => write!(f, "Environment error: {}", msg),
            Error::SecretError(msg) => write!(f, "Secret error: {}", msg),
            Error::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::ParseError(format!("TOML parsing error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(format!("JSON serialization error: {}", err))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::SerializationError(format!("YAML serialization error: {}", err))
    }
}

/// Error context for better error reporting.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The original error
    pub error: Error,
    /// Context information
    pub context: String,
    /// File path where the error occurred
    pub file_path: Option<String>,
    /// Line number where the error occurred
    pub line: Option<usize>,
    /// Column number where the error occurred
    pub column: Option<usize>,
}

impl ErrorContext {
    /// Create a new error context.
    pub fn new(error: Error, context: String) -> Self {
        Self {
            error,
            context,
            file_path: None,
            line: None,
            column: None,
        }
    }

    /// Add file path to the context.
    pub fn with_file(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    /// Add line number to the context.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Add column number to the context.
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context)?;

        if let Some(file_path) = &self.file_path {
            write!(f, " (file: {})", file_path)?;
        }

        if let Some(line) = self.line {
            write!(f, " (line: {})", line)?;
        }

        if let Some(column) = self.column {
            write!(f, " (column: {})", column)?;
        }

        write!(f, ": {}", self.error)
    }
}

impl std::error::Error for ErrorContext {}

/// Trait for adding context to errors.
pub trait ErrorExt<T> {
    /// Add context to a result.
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| Error::Generic(format!("{}: {}", context, e.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = Error::ParseError("Invalid TOML".to_string());
        assert_eq!(error.to_string(), "Parse error: Invalid TOML");
    }

    #[test]
    fn test_error_context() {
        let error = Error::ValidationError("Missing required field".to_string());
        let context = ErrorContext::new(error, "Failed to validate configuration".to_string())
            .with_file("deploko.toml".to_string())
            .with_line(10)
            .with_column(5);

        let display = context.to_string();
        assert!(display.contains("Failed to validate configuration"));
        assert!(display.contains("file: deploko.toml"));
        assert!(display.contains("line: 10"));
        assert!(display.contains("column: 5"));
        assert!(display.contains("Validation error: Missing required field"));
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();
        match error {
            Error::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_ext() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let result: Result<()> = result.with_context("Failed to read configuration");
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::Generic(msg) => {
                assert!(msg.contains("Failed to read configuration"));
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected Generic error"),
        }
    }
}
