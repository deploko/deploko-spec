//! Schema definitions for Deploko deployment specifications.
//!
//! This module contains the data structures that represent the parsed
//! `deploko.toml` configuration file.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
