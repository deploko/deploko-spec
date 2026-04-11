//! Schema definitions for Deploko deployment specifications.
//!
//! This module contains the data structures that represent the parsed
//! `deploko.toml` configuration file.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// A newtype wrapper for byte sizes with human-readable parsing.
///
/// Parses strings like "10gb", "500mb", "1tb", "100kb" into bytes.
/// Case-insensitive and supports spaces between number and unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSize(u64);

impl ByteSize {
    /// Get the raw byte value.
    pub fn bytes(self) -> u64 {
        self.0
    }

    /// Create a new ByteSize from bytes.
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0;

        if bytes >= 1_000_000_000_000 {
            write!(f, "{:.2} TB", bytes as f64 / 1_000_000_000_000.0)
        } else if bytes >= 1_000_000_000 {
            write!(f, "{:.2} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            write!(f, "{:.2} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            write!(f, "{:.2} KB", bytes as f64 / 1_000.0)
        } else {
            write!(f, "{} B", bytes)
        }
    }
}

impl FromStr for ByteSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();

        // Find where the unit starts (handle scientific notation)
        let unit_start = trimmed
            .chars()
            .position(|c| !c.is_ascii_digit() && c != ' ' && c != '.' && c != 'e' && c != 'E')
            .ok_or("No unit found in byte size string")?;

        let (number_str, unit_str) = trimmed.split_at(unit_start);
        let unit_str = unit_str.trim();

        let number: f64 = number_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid number: {}", number_str))?;

        if number <= 0.0 {
            return Err("Byte size must be greater than zero".to_string());
        }

        // Add reasonable upper bounds to prevent overflow
        if number > 1_000_000_000_000_000.0 {
            return Err("Byte size value too large (max 1 quadrillion units)".to_string());
        }

        let multiplier = match unit_str {
            "b" | "byte" | "bytes" => 1.0,
            "kb" | "kilobyte" | "kilobytes" => 1_000.0,
            "mb" | "megabyte" | "megabytes" => 1_000_000.0,
            "gb" | "gigabyte" | "gigabytes" => 1_000_000_000.0,
            "tb" | "terabyte" | "terabytes" => 1_000_000_000_000.0,
            _ => return Err(format!("Unknown byte size unit: {}", unit_str)),
        };

        // Use safe multiplication to prevent overflow
        let bytes_f64 = number * multiplier;
        if bytes_f64 > u64::MAX as f64 {
            return Err("Byte size calculation would overflow".to_string());
        }

        let bytes = bytes_f64.round() as u64;
        Ok(ByteSize(bytes))
    }
}

impl<'de> Deserialize<'de> for ByteSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ByteSize::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ByteSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// A newtype wrapper for durations with human-readable parsing.
///
/// Parses strings like "30d", "7d", "1h", "15m", "60s" into seconds.
/// Case-insensitive and supports spaces between number and unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(u64);

impl Duration {
    /// Get the raw duration in seconds.
    pub fn seconds(self) -> u64 {
        self.0
    }

    /// Create a new Duration from seconds.
    pub fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.0;

        // Use the most precise unit that doesn't lose information
        if seconds.is_multiple_of(86400) && seconds >= 86400 {
            let days = seconds / 86400;
            write!(f, "{} day{}", days, if days == 1 { "" } else { "s" })
        } else if seconds.is_multiple_of(3600) && seconds >= 3600 {
            let hours = seconds / 3600;
            write!(f, "{} hour{}", hours, if hours == 1 { "" } else { "s" })
        } else if seconds.is_multiple_of(60) && seconds >= 60 {
            let minutes = seconds / 60;
            write!(
                f,
                "{} minute{}",
                minutes,
                if minutes == 1 { "" } else { "s" }
            )
        } else {
            write!(
                f,
                "{} second{}",
                seconds,
                if seconds == 1 { "" } else { "s" }
            )
        }
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();

        // Find where the unit starts (handle scientific notation)
        let unit_start = trimmed
            .chars()
            .position(|c| !c.is_ascii_digit() && c != ' ' && c != '.' && c != 'e' && c != 'E')
            .ok_or("No unit found in duration string")?;

        let (number_str, unit_str) = trimmed.split_at(unit_start);
        let unit_str = unit_str.trim();

        let number: f64 = number_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid number: {}", number_str))?;

        if number <= 0.0 {
            return Err("Duration must be greater than zero".to_string());
        }

        // Add reasonable upper bounds to prevent overflow
        if number > 1_000_000_000_000_000.0 {
            return Err("Duration value too large (max 1 quadrillion units)".to_string());
        }

        let multiplier = match unit_str {
            "s" | "sec" | "second" | "seconds" => 1.0,
            "m" | "min" | "minute" | "minutes" => 60.0,
            "h" | "hr" | "hour" | "hours" => 3600.0,
            "d" | "day" | "days" => 86400.0,
            _ => return Err(format!("Unknown duration unit: {}", unit_str)),
        };

        // Use safe multiplication to prevent overflow
        let seconds_f64 = number * multiplier;
        if seconds_f64 > u64::MAX as f64 {
            return Err("Duration calculation would overflow".to_string());
        }

        let seconds = seconds_f64.round() as u64;
        Ok(Duration(seconds))
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Duration::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// A newtype wrapper for secret references.
///
/// Parses strings like `${secrets.KEY}` where KEY must match the pattern
/// `^[A-Z][A-Z0-9_]{0,63}$` (starts with uppercase letter, followed by
/// uppercase letters, digits, or underscores, max 64 characters).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretRef(String);

impl SecretRef {
    /// Get the secret key name.
    pub fn key(&self) -> &str {
        &self.0
    }

    /// Create a new SecretRef from a key string.
    ///
    /// Returns an error if the key doesn't match the required pattern.
    pub fn new(key: impl Into<String>) -> Result<Self, String> {
        let key = key.into();
        if Self::is_valid_key(&key) {
            Ok(Self(key))
        } else {
            Err(format!(
                "Invalid secret key '{}'. Must match pattern: ^[A-Z][A-Z0-9_]{{0,63}}$",
                key
            ))
        }
    }

    /// Validate that a key matches the required pattern.
    fn is_valid_key(key: &str) -> bool {
        if key.is_empty() || key.len() > 64 {
            return false;
        }
        let mut chars = key.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_uppercase() {
            return false;
        }
        chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${{secrets.{}}}", self.0)
    }
}

impl FromStr for SecretRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        // Check if it starts with ${secrets. and ends with }
        let prefix = "${secrets.";
        let suffix = "}";

        if !trimmed.starts_with(prefix) || !trimmed.ends_with(suffix) {
            return Err(format!(
                "Invalid secret reference format '{}'. Expected format: ${{secrets.KEY}}",
                s
            ));
        }

        // Extract the key between prefix and suffix
        let key_start = prefix.len();
        let key_end = trimmed.len() - suffix.len();
        let key = &trimmed[key_start..key_end];

        if key.is_empty() {
            return Err("Secret key cannot be empty".to_string());
        }

        Self::new(key)
    }
}

impl<'de> Deserialize<'de> for SecretRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SecretRef::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for SecretRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// An environment variable value that can be either a literal string or a secret reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvValue {
    /// A literal string value.
    Literal(String),
    /// A reference to a secret.
    Secret(SecretRef),
}

impl EnvValue {
    /// Get the literal value if it exists.
    pub fn as_literal(&self) -> Option<&str> {
        match self {
            EnvValue::Literal(s) => Some(s),
            _ => None,
        }
    }

    /// Get the secret reference if it exists.
    pub fn as_secret(&self) -> Option<&SecretRef> {
        match self {
            EnvValue::Secret(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this is a secret reference.
    pub fn is_secret(&self) -> bool {
        matches!(self, EnvValue::Secret(_))
    }

    /// Check if this is a literal value.
    pub fn is_literal(&self) -> bool {
        matches!(self, EnvValue::Literal(_))
    }
}

impl fmt::Display for EnvValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvValue::Literal(s) => write!(f, "{}", s),
            EnvValue::Secret(secret_ref) => write!(f, "[secret:{}]", secret_ref.key()),
        }
    }
}

impl<'de> Deserialize<'de> for EnvValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let trimmed = s.trim();

        // Check if it looks like a secret reference
        if trimmed.starts_with("${secrets.") && trimmed.ends_with("}") {
            match SecretRef::from_str(trimmed) {
                Ok(secret_ref) => Ok(EnvValue::Secret(secret_ref)),
                Err(e) => Err(serde::de::Error::custom(e)),
            }
        } else {
            Ok(EnvValue::Literal(s))
        }
    }
}

impl Serialize for EnvValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EnvValue::Literal(s) => serializer.serialize_str(s),
            EnvValue::Secret(secret_ref) => serializer.serialize_str(&secret_ref.to_string()),
        }
    }
}

/// A complete deployment specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploySpec {
    /// Project metadata
    pub project: ProjectConfig,
    /// Frontend configuration
    pub frontend: Option<FrontendConfig>,
    /// Backend configuration
    pub backend: Option<BackendConfig>,
    /// Database configuration
    pub database: Option<DatabaseConfig>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Storage configuration
    pub storage: Option<StorageConfig>,
    /// Observability configuration
    pub observability: Option<ObservabilityConfig>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Environment-specific overrides
    pub environments: Option<HashMap<String, EnvironmentConfig>>,
}

/// Project metadata and basic configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Target deployment region
    pub region: String,
    /// Default environment
    pub environment: Option<String>,
}

/// Frontend service configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrontendConfig {
    /// Frontend framework
    pub framework: String,
    /// Git repository URL
    pub repo: String,
    /// Git branch to deploy
    pub branch: String,
    /// Build command
    pub build_command: String,
    /// Output directory
    pub output_dir: Option<String>,
}

/// Backend service configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackendConfig {
    /// Runtime environment
    pub runtime: String,
    /// Dockerfile path
    pub dockerfile: Option<String>,
    /// Scaling configuration
    pub scale: Option<ScaleConfig>,
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
}

/// Scaling configuration for services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleConfig {
    /// Minimum number of instances
    pub min: u32,
    /// Maximum number of instances
    pub max: u32,
    /// Target CPU utilization percentage
    pub target_cpu: Option<u32>,
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub path: String,
    /// Check interval in seconds
    pub interval: u32,
    /// Timeout in seconds
    pub timeout: u32,
    /// Required successful checks
    pub healthy_threshold: u32,
    /// Allowed failed checks
    pub unhealthy_threshold: u32,
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    /// Database engine
    pub engine: String,
    /// Engine version
    pub version: String,
    /// Instance size
    pub instance_size: String,
    /// Connection pooler configuration
    pub connection_pooler: Option<bool>,
    /// Backup configuration
    pub backups: Option<BackupConfig>,
}

/// Backup configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupConfig {
    /// Backup retention period in days
    pub retention_days: u32,
    /// Backup window (cron format)
    pub backup_window: String,
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConfig {
    /// Authentication providers
    pub providers: Vec<AuthProvider>,
}

/// Authentication provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthProvider {
    /// Provider type (email, oauth, etc.)
    pub provider: String,
    /// Provider-specific configuration
    pub config: HashMap<String, String>,
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    /// Storage size limit
    pub size_limit: Option<String>,
    /// Storage type
    pub storage_type: Option<String>,
}

/// Observability configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityConfig {
    /// Logging configuration
    pub logs: Option<LogsConfig>,
    /// Metrics configuration
    pub metrics: Option<MetricsConfig>,
    /// Uptime monitoring
    pub uptime: Option<UptimeConfig>,
    /// Alerting configuration
    pub alerts: Option<AlertsConfig>,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogsConfig {
    /// Log level
    pub level: String,
    /// Log retention in days
    pub retention_days: Option<u32>,
}

/// Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsConfig {
    /// Metrics collection enabled
    pub enabled: bool,
    /// Metrics retention in days
    pub retention_days: Option<u32>,
}

/// Uptime monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UptimeConfig {
    /// Monitoring enabled
    pub enabled: bool,
    /// Check interval in seconds
    pub interval: Option<u32>,
}

/// Alerting configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertsConfig {
    /// Alerting enabled
    pub enabled: bool,
    /// Alert channels
    pub channels: Vec<String>,
}

/// Environment-specific configuration overrides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Project overrides
    pub project: Option<ProjectConfig>,
    /// Frontend overrides
    pub frontend: Option<FrontendConfig>,
    /// Backend overrides
    pub backend: Option<BackendConfig>,
    /// Database overrides
    pub database: Option<DatabaseConfig>,
    /// Auth overrides
    pub auth: Option<AuthConfig>,
    /// Storage overrides
    pub storage: Option<StorageConfig>,
    /// Observability overrides
    pub observability: Option<ObservabilityConfig>,
    /// Environment variable overrides
    pub env: Option<HashMap<String, String>>,
}

impl Default for DeploySpec {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "default".to_string(),
                region: "us-east-1".to_string(),
                environment: Some("development".to_string()),
            },
            frontend: None,
            backend: None,
            database: None,
            auth: None,
            storage: None,
            observability: None,
            env: None,
            environments: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_bytesize_parse_valid() {
        let test_cases = vec![
            ("100b", 100),
            ("1kb", 1000),
            ("500mb", 500_000_000),
            ("2gb", 2_000_000_000),
            ("1tb", 1_000_000_000_000),
            ("10 KB", 10_000),
            ("2.5 GB", 2_500_000_000),
            ("1.5mb", 1_500_000),
        ];

        for (input, expected) in test_cases {
            let result = ByteSize::from_str(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(result.unwrap().bytes(), expected, "Mismatch for: {}", input);
        }
    }

    #[test]
    fn test_bytesize_parse_invalid() {
        let invalid_cases = vec!["0b", "-10mb", "abc", "10xb", "10", "", "10.5.2gb"];

        for input in invalid_cases {
            let result = ByteSize::from_str(input);
            assert!(result.is_err(), "Should have failed to parse: {}", input);
        }
    }

    #[test]
    fn test_bytesize_display() {
        let test_cases = vec![
            (500, "500 B"),
            (1500, "1.50 KB"),
            (2_500_000, "2.50 MB"),
            (3_500_000_000, "3.50 GB"),
            (1_500_000_000_000, "1.50 TB"),
        ];

        for (bytes, expected) in test_cases {
            let size = ByteSize(bytes);
            let displayed = size.to_string();
            assert!(
                displayed.contains(expected.split_whitespace().next().unwrap()),
                "Expected {} to contain {}, got {}",
                bytes,
                expected,
                displayed
            );
        }
    }

    #[test]
    fn test_bytesize_serialize_deserialize() {
        let original = ByteSize::from_str("2.5gb").unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ByteSize = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(original.bytes(), 2_500_000_000);
    }

    #[test]
    fn test_duration_parse_valid() {
        let test_cases = vec![
            ("30s", 30),
            ("15m", 900),
            ("2h", 7200),
            ("3d", 259200),
            ("60 S", 60),
            ("1.5 h", 5400),
            ("2.5d", 216000),
        ];

        for (input, expected) in test_cases {
            let result = Duration::from_str(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(
                result.unwrap().seconds(),
                expected,
                "Mismatch for: {}",
                input
            );
        }
    }

    #[test]
    fn test_duration_parse_invalid() {
        let invalid_cases = vec!["0s", "-10m", "abc", "10x", "10", "", "10.5.2h"];

        for input in invalid_cases {
            let result = Duration::from_str(input);
            assert!(result.is_err(), "Should have failed to parse: {}", input);
        }
    }

    #[test]
    fn test_duration_display() {
        let test_cases = vec![
            (30, "30 seconds"),
            (60, "1 minute"),
            (7200, "2 hours"),
            (172800, "2 days"),
            (2592000, "30 days"),
            (86400, "1 day"),
            (3600, "1 hour"),
            (1, "1 second"),
        ];

        for (seconds, expected) in test_cases {
            let duration = Duration(seconds);
            let displayed = duration.to_string();
            assert!(
                displayed.contains(expected.split_whitespace().next().unwrap()),
                "Expected {} to contain {}, got {}",
                seconds,
                expected,
                displayed
            );
        }
    }

    #[test]
    fn test_duration_serialize_deserialize() {
        let original = Duration::from_str("2.5h").unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Duration = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(original.seconds(), 9000);
    }

    #[test]
    fn test_bytesize_case_insensitive() {
        let cases = vec!["1GB", "1gb", "1Gb", "1gB"];
        for case in cases {
            let result = ByteSize::from_str(case);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().bytes(), 1_000_000_000);
        }
    }

    #[test]
    fn test_duration_case_insensitive() {
        let cases = vec!["1H", "1h", "1Hr", "1hr"];
        for case in cases {
            let result = Duration::from_str(case);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().seconds(), 3600);
        }
    }

    #[test]
    fn test_bytesize_zero_error() {
        let result = ByteSize::from_str("0b");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than zero"));
    }

    #[test]
    fn test_duration_zero_error() {
        let result = Duration::from_str("0s");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than zero"));
    }

    #[test]
    fn test_bytesize_boundary_values() {
        // Test exact boundary values
        let test_cases = vec![
            ("999b", 999),
            ("1000b", 1000),
            ("999kb", 999_000),
            ("1000kb", 1_000_000),
            ("999mb", 999_000_000),
            ("1000mb", 1_000_000_000),
            ("999gb", 999_000_000_000),
            ("1000gb", 1_000_000_000_000),
        ];

        for (input, expected) in test_cases {
            let result = ByteSize::from_str(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(result.unwrap().bytes(), expected, "Mismatch for: {}", input);
        }
    }

    #[test]
    fn test_bytesize_overflow_protection() {
        let overflow_cases = vec![
            "999999999999999999999tb",
            "1e308gb",
            "999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999tb",
        ];

        for input in overflow_cases {
            let result = ByteSize::from_str(input);
            assert!(
                result.is_err(),
                "Should have failed to parse overflow case: {}",
                input
            );
            let error = result.unwrap_err();
            assert!(
                error.contains("too large") || error.contains("overflow"),
                "Expected overflow error for: {}, got: {}",
                input,
                error
            );
        }
    }

    #[test]
    fn test_duration_overflow_protection() {
        let overflow_cases = vec![
            "999999999999999999999d",
            "1e308h",
            "999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999d",
        ];

        for input in overflow_cases {
            let result = Duration::from_str(input);
            assert!(
                result.is_err(),
                "Should have failed to parse overflow case: {}",
                input
            );
            let error = result.unwrap_err();
            assert!(
                error.contains("too large") || error.contains("overflow"),
                "Expected overflow error for: {}, got: {}",
                input,
                error
            );
        }
    }

    #[test]
    fn test_bytesize_small_decimal_values() {
        let test_cases = vec![
            ("0.001gb", 1_000_000), // 0.001 * 1,000,000,000
            ("0.001mb", 1_000),     // 0.001 * 1,000,000
            ("0.001kb", 1),         // 0.001 * 1,000
        ];

        for (input, expected) in test_cases {
            let result = ByteSize::from_str(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(result.unwrap().bytes(), expected, "Mismatch for: {}", input);
        }
    }

    #[test]
    fn test_negative_zero_handling() {
        // Test negative zero cases - should fail
        let negative_zero_cases = vec!["-0b", "-0kb", "-0s", "-0m"];

        for input in negative_zero_cases {
            let bytesize_result = ByteSize::from_str(input);
            assert!(
                bytesize_result.is_err(),
                "ByteSize should fail for negative zero: {}",
                input
            );

            let duration_result = Duration::from_str(input);
            assert!(
                duration_result.is_err(),
                "Duration should fail for negative zero: {}",
                input
            );
        }
    }

    #[test]
    fn test_scientific_notation_parsing() {
        // Test that scientific notation is handled properly
        let test_cases = vec![("1e3b", 1000), ("1e6b", 1_000_000), ("1e9b", 1_000_000_000)];

        for (input, expected) in test_cases {
            let result = ByteSize::from_str(input);
            assert!(
                result.is_ok(),
                "Failed to parse scientific notation: {}",
                input
            );
            assert_eq!(result.unwrap().bytes(), expected, "Mismatch for: {}", input);
        }
    }

    #[test]
    fn test_secret_ref_parse_valid() {
        let test_cases = vec![
            ("${secrets.API_KEY}", "API_KEY"),
            ("${secrets.DB_PASSWORD}", "DB_PASSWORD"),
            ("${secrets.MY_SECRET_123}", "MY_SECRET_123"),
            ("  ${secrets.TRIMMED}  ", "TRIMMED"), // whitespace is trimmed
        ];

        for (input, expected_key) in test_cases {
            let result = SecretRef::from_str(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(
                result.unwrap().key(),
                expected_key,
                "Mismatch for: {}",
                input
            );
        }
    }

    #[test]
    fn test_secret_ref_parse_invalid() {
        let invalid_cases = vec![
            "${secrets.}",          // empty key
            "${secrets.lowercase}", // lowercase
            "${secrets.123KEY}",    // starts with digit
            "${secrets.KEY!}",      // invalid character
            "${secrets.A0123456789012345678901234567890123456789012345678901234567890123}", // too long (65 chars)
            "secrets.KEY",   // missing ${ and }
            "${secrets.KEY", // missing closing }
            "secrets.KEY}",  // missing opening ${
            "${other.KEY}",  // wrong prefix
            "${secret.KEY}", // singular secret
            "",              // empty string
            "plain_value",   // not a reference
        ];

        for input in invalid_cases {
            let result = SecretRef::from_str(input);
            assert!(result.is_err(), "Should have failed to parse: {}", input);
        }
    }

    #[test]
    fn test_secret_ref_display() {
        let secret_ref = SecretRef::new("MY_KEY").unwrap();
        assert_eq!(secret_ref.to_string(), "${secrets.MY_KEY}");
    }

    #[test]
    fn test_secret_ref_serialize_deserialize() {
        let original = SecretRef::new("API_KEY").unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        assert_eq!(serialized, "\"${secrets.API_KEY}\"");

        let deserialized: SecretRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.key(), "API_KEY");
    }

    #[test]
    fn test_secret_ref_key_length_boundary() {
        // 1 character key (valid)
        assert!(SecretRef::new("A").is_ok());

        // 64 character key (valid - max length)
        let max_key = "A".to_string() + &"B".repeat(63);
        assert_eq!(max_key.len(), 64);
        assert!(SecretRef::new(&max_key).is_ok());

        // 65 character key (invalid - too long)
        let too_long = "A".to_string() + &"B".repeat(64);
        assert_eq!(too_long.len(), 65);
        assert!(SecretRef::new(&too_long).is_err());
    }

    #[test]
    fn test_env_value_literal() {
        // Test deserializing a literal value
        let json = "\"plain_value\"";
        let env_value: EnvValue = serde_json::from_str(json).unwrap();
        assert!(env_value.is_literal());
        assert!(!env_value.is_secret());
        assert_eq!(env_value.as_literal(), Some("plain_value"));
        assert_eq!(env_value.as_secret(), None);
        assert_eq!(env_value.to_string(), "plain_value");
    }

    #[test]
    fn test_env_value_secret() {
        // Test deserializing a secret reference
        let json = "\"${secrets.DB_PASSWORD}\"";
        let env_value: EnvValue = serde_json::from_str(json).unwrap();
        assert!(!env_value.is_literal());
        assert!(env_value.is_secret());
        assert_eq!(env_value.as_literal(), None);
        assert!(env_value.as_secret().is_some());
        assert_eq!(env_value.to_string(), "[secret:DB_PASSWORD]");
    }

    #[test]
    fn test_env_value_serialize_roundtrip() {
        // Literal roundtrip
        let literal = EnvValue::Literal("my_value".to_string());
        let serialized = serde_json::to_string(&literal).unwrap();
        assert_eq!(serialized, "\"my_value\"");
        let deserialized: EnvValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(literal, deserialized);

        // Secret roundtrip
        let secret = EnvValue::Secret(SecretRef::new("API_KEY").unwrap());
        let serialized = serde_json::to_string(&secret).unwrap();
        assert_eq!(serialized, "\"${secrets.API_KEY}\"");
        let deserialized: EnvValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(secret, deserialized);
    }

    #[test]
    fn test_env_value_display() {
        let literal = EnvValue::Literal("visible_value".to_string());
        assert_eq!(literal.to_string(), "visible_value");

        let secret = EnvValue::Secret(SecretRef::new("HIDDEN_KEY").unwrap());
        assert_eq!(secret.to_string(), "[secret:HIDDEN_KEY]");
    }

    #[test]
    fn test_env_value_invalid_secret() {
        // Invalid secret pattern should fail during deserialization
        let json = "\"${secrets.invalid_key}\"";
        let result: Result<EnvValue, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
