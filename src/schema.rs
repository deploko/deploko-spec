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
}
