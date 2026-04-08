//! Compiled deployment specifications.
//!
//! This module provides functionality to compile a parsed `DeploySpec` with
//! environment-specific overrides and defaults into a final `CompiledSpec`.

use crate::error::{Error, Result};
use crate::schema::{
    AuthConfig, BackendConfig, DatabaseConfig, DeploySpec, EnvironmentConfig, FrontendConfig,
    ObservabilityConfig, ProjectConfig, StorageConfig,
};
use std::collections::HashMap;

/// A compiled deployment specification with all overrides applied.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompiledSpec {
    /// Final project configuration
    pub project: ProjectConfig,
    /// Final frontend configuration
    pub frontend: Option<FrontendConfig>,
    /// Final backend configuration
    pub backend: Option<BackendConfig>,
    /// Final database configuration
    pub database: Option<DatabaseConfig>,
    /// Final auth configuration
    pub auth: Option<AuthConfig>,
    /// Final storage configuration
    pub storage: Option<StorageConfig>,
    /// Final observability configuration
    pub observability: Option<ObservabilityConfig>,
    /// Final environment variables
    pub env: HashMap<String, String>,
    /// Target environment
    pub environment: String,
}

impl CompiledSpec {
    /// Serialize the compiled specification to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize to JSON: {}", e)))
    }

    /// Serialize the compiled specification to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize to YAML: {}", e)))
    }
}

/// Compile a deployment specification with environment overrides.
pub fn compile(spec: &DeploySpec, environment: Option<&str>) -> Result<CompiledSpec> {
    let target_env = environment
        .or(spec.project.environment.as_deref())
        .unwrap_or("development");

    // Find environment overrides if they exist
    let env_overrides = spec
        .environments
        .as_ref()
        .and_then(|envs| envs.get(target_env));

    // Compile project configuration
    let project = compile_project(&spec.project, env_overrides);

    // Compile other configurations
    let frontend = compile_frontend(spec.frontend.as_ref(), env_overrides);
    let backend = compile_backend(spec.backend.as_ref(), env_overrides);
    let database = compile_database(spec.database.as_ref(), env_overrides);
    let auth = compile_auth(spec.auth.as_ref(), env_overrides);
    let storage = compile_storage(spec.storage.as_ref(), env_overrides);
    let observability = compile_observability(spec.observability.as_ref(), env_overrides);

    // Compile environment variables
    let mut env = HashMap::new();

    // Add base environment variables
    if let Some(base_env) = &spec.env {
        env.extend(base_env.clone());
    }

    // Add environment-specific overrides
    if let Some(overrides) = env_overrides
        && let Some(env_overrides) = &overrides.env
    {
        env.extend(env_overrides.clone());
    }

    Ok(CompiledSpec {
        project,
        frontend,
        backend,
        database,
        auth,
        storage,
        observability,
        env,
        environment: target_env.to_string(),
    })
}

/// Compile project configuration with overrides.
fn compile_project(base: &ProjectConfig, overrides: Option<&EnvironmentConfig>) -> ProjectConfig {
    if let Some(overrides) = overrides {
        ProjectConfig {
            name: overrides
                .project
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| base.name.clone()),
            region: overrides
                .project
                .as_ref()
                .map(|p| p.region.clone())
                .unwrap_or_else(|| base.region.clone()),
            environment: overrides
                .project
                .as_ref()
                .and_then(|p| p.environment.clone())
                .or_else(|| base.environment.clone()),
        }
    } else {
        base.clone()
    }
}

/// Compile frontend configuration with overrides.
fn compile_frontend(
    base: Option<&FrontendConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<FrontendConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_frontend = overrides.frontend.as_ref();
            Some(FrontendConfig {
                framework: override_frontend
                    .map(|f| f.framework.clone())
                    .unwrap_or_else(|| base.framework.clone()),
                repo: override_frontend
                    .map(|f| f.repo.clone())
                    .unwrap_or_else(|| base.repo.clone()),
                branch: override_frontend
                    .map(|f| f.branch.clone())
                    .unwrap_or_else(|| base.branch.clone()),
                build_command: override_frontend
                    .map(|f| f.build_command.clone())
                    .unwrap_or_else(|| base.build_command.clone()),
                output_dir: override_frontend
                    .and_then(|f| f.output_dir.clone())
                    .or_else(|| base.output_dir.clone()),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.frontend.clone(),
        (None, None) => None,
    }
}

/// Compile backend configuration with overrides.
fn compile_backend(
    base: Option<&BackendConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<BackendConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_backend = overrides.backend.as_ref();
            Some(BackendConfig {
                runtime: override_backend
                    .map(|b| b.runtime.clone())
                    .unwrap_or_else(|| base.runtime.clone()),
                dockerfile: override_backend
                    .and_then(|b| b.dockerfile.clone())
                    .or_else(|| base.dockerfile.clone()),
                scale: compile_scale_config(
                    base.scale.as_ref(),
                    override_backend.and_then(|b| b.scale.as_ref()),
                ),
                health_check: compile_health_check_config(
                    base.health_check.as_ref(),
                    override_backend.and_then(|b| b.health_check.as_ref()),
                ),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.backend.clone(),
        (None, None) => None,
    }
}

/// Compile scale configuration with overrides.
fn compile_scale_config(
    base: Option<&crate::schema::ScaleConfig>,
    overrides: Option<&crate::schema::ScaleConfig>,
) -> Option<crate::schema::ScaleConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => Some(crate::schema::ScaleConfig {
            min: overrides.min,
            max: overrides.max,
            target_cpu: overrides.target_cpu.or(base.target_cpu),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile health check configuration with overrides.
fn compile_health_check_config(
    base: Option<&crate::schema::HealthCheckConfig>,
    overrides: Option<&crate::schema::HealthCheckConfig>,
) -> Option<crate::schema::HealthCheckConfig> {
    match (base, overrides) {
        (Some(_base), Some(overrides)) => Some(crate::schema::HealthCheckConfig {
            path: overrides.path.clone(),
            interval: overrides.interval,
            timeout: overrides.timeout,
            healthy_threshold: overrides.healthy_threshold,
            unhealthy_threshold: overrides.unhealthy_threshold,
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile database configuration with overrides.
fn compile_database(
    base: Option<&DatabaseConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<DatabaseConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_database = overrides.database.as_ref();
            Some(DatabaseConfig {
                engine: override_database
                    .map(|d| d.engine.clone())
                    .unwrap_or_else(|| base.engine.clone()),
                version: override_database
                    .map(|d| d.version.clone())
                    .unwrap_or_else(|| base.version.clone()),
                instance_size: override_database
                    .map(|d| d.instance_size.clone())
                    .unwrap_or_else(|| base.instance_size.clone()),
                connection_pooler: override_database
                    .and_then(|d| d.connection_pooler)
                    .or(base.connection_pooler),
                backups: compile_backup_config(
                    base.backups.as_ref(),
                    override_database.and_then(|d| d.backups.as_ref()),
                ),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.database.clone(),
        (None, None) => None,
    }
}

/// Compile backup configuration with overrides.
fn compile_backup_config(
    base: Option<&crate::schema::BackupConfig>,
    overrides: Option<&crate::schema::BackupConfig>,
) -> Option<crate::schema::BackupConfig> {
    match (base, overrides) {
        (Some(_base), Some(overrides)) => Some(crate::schema::BackupConfig {
            retention_days: overrides.retention_days,
            backup_window: overrides.backup_window.clone(),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile auth configuration with overrides.
fn compile_auth(
    base: Option<&AuthConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<AuthConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_auth = overrides.auth.as_ref();
            Some(AuthConfig {
                providers: override_auth
                    .map(|a| a.providers.clone())
                    .unwrap_or_else(|| base.providers.clone()),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.auth.clone(),
        (None, None) => None,
    }
}

/// Compile storage configuration with overrides.
fn compile_storage(
    base: Option<&StorageConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<StorageConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_storage = overrides.storage.as_ref();
            Some(StorageConfig {
                size_limit: override_storage
                    .and_then(|s| s.size_limit.clone())
                    .or_else(|| base.size_limit.clone()),
                storage_type: override_storage
                    .and_then(|s| s.storage_type.clone())
                    .or_else(|| base.storage_type.clone()),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.storage.clone(),
        (None, None) => None,
    }
}

/// Compile observability configuration with overrides.
fn compile_observability(
    base: Option<&ObservabilityConfig>,
    overrides: Option<&EnvironmentConfig>,
) -> Option<ObservabilityConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let override_observability = overrides.observability.as_ref();
            Some(ObservabilityConfig {
                logs: compile_logs_config(
                    base.logs.as_ref(),
                    override_observability.and_then(|o| o.logs.as_ref()),
                ),
                metrics: compile_metrics_config(
                    base.metrics.as_ref(),
                    override_observability.and_then(|o| o.metrics.as_ref()),
                ),
                uptime: compile_uptime_config(
                    base.uptime.as_ref(),
                    override_observability.and_then(|o| o.uptime.as_ref()),
                ),
                alerts: compile_alerts_config(
                    base.alerts.as_ref(),
                    override_observability.and_then(|o| o.alerts.as_ref()),
                ),
            })
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.observability.clone(),
        (None, None) => None,
    }
}

/// Compile logs configuration with overrides.
fn compile_logs_config(
    base: Option<&crate::schema::LogsConfig>,
    overrides: Option<&crate::schema::LogsConfig>,
) -> Option<crate::schema::LogsConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => Some(crate::schema::LogsConfig {
            level: overrides.level.clone(),
            retention_days: overrides.retention_days.or(base.retention_days),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile metrics configuration with overrides.
fn compile_metrics_config(
    base: Option<&crate::schema::MetricsConfig>,
    overrides: Option<&crate::schema::MetricsConfig>,
) -> Option<crate::schema::MetricsConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => Some(crate::schema::MetricsConfig {
            enabled: overrides.enabled,
            retention_days: overrides.retention_days.or(base.retention_days),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile uptime configuration with overrides.
fn compile_uptime_config(
    base: Option<&crate::schema::UptimeConfig>,
    overrides: Option<&crate::schema::UptimeConfig>,
) -> Option<crate::schema::UptimeConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => Some(crate::schema::UptimeConfig {
            enabled: overrides.enabled,
            interval: overrides.interval.or(base.interval),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

/// Compile alerts configuration with overrides.
fn compile_alerts_config(
    base: Option<&crate::schema::AlertsConfig>,
    overrides: Option<&crate::schema::AlertsConfig>,
) -> Option<crate::schema::AlertsConfig> {
    match (base, overrides) {
        (Some(_base), Some(overrides)) => Some(crate::schema::AlertsConfig {
            enabled: overrides.enabled,
            channels: overrides.channels.clone(),
        }),
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => Some(overrides.clone()),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{EnvironmentConfig, ProjectConfig};

    #[test]
    fn test_compile_no_overrides() {
        let mut spec = DeploySpec::default();
        spec.project.name = "test-app".to_string();
        spec.project.region = "us-east-1".to_string();

        let compiled = compile(&spec, None).unwrap();

        assert_eq!(compiled.project.name, "test-app");
        assert_eq!(compiled.project.region, "us-east-1");
        assert_eq!(compiled.environment, "development");
    }

    #[test]
    fn test_compile_with_environment_override() {
        let mut spec = DeploySpec::default();
        spec.project.name = "test-app".to_string();
        spec.project.region = "us-east-1".to_string();

        let mut environments = HashMap::new();
        let mut prod_env = EnvironmentConfig {
            name: "production".to_string(),
            project: None,
            frontend: None,
            backend: None,
            database: None,
            auth: None,
            storage: None,
            observability: None,
            env: None,
        };

        let prod_project = ProjectConfig {
            name: "test-app".to_string(),
            region: "us-west-2".to_string(), // Override region
            environment: Some("production".to_string()),
        };
        prod_env.project = Some(prod_project);

        environments.insert("production".to_string(), prod_env);
        spec.environments = Some(environments);

        let compiled = compile(&spec, Some("production")).unwrap();

        assert_eq!(compiled.project.name, "test-app");
        assert_eq!(compiled.project.region, "us-west-2"); // Should be overridden
        assert_eq!(compiled.environment, "production");
    }

    #[test]
    fn test_compile_env_vars_merge() {
        let mut spec = DeploySpec::default();
        spec.project.name = "test-app".to_string();

        let mut base_env = HashMap::new();
        base_env.insert("BASE_VAR".to_string(), "base_value".to_string());
        spec.env = Some(base_env);

        let mut environments = HashMap::new();
        let mut prod_env = EnvironmentConfig {
            name: "production".to_string(),
            project: None,
            frontend: None,
            backend: None,
            database: None,
            auth: None,
            storage: None,
            observability: None,
            env: None,
        };

        let mut prod_env_vars = HashMap::new();
        prod_env_vars.insert("PROD_VAR".to_string(), "prod_value".to_string());
        prod_env_vars.insert("BASE_VAR".to_string(), "overridden".to_string()); // Override base
        prod_env.env = Some(prod_env_vars);

        environments.insert("production".to_string(), prod_env);
        spec.environments = Some(environments);

        let compiled = compile(&spec, Some("production")).unwrap();

        assert_eq!(
            compiled.env.get("BASE_VAR"),
            Some(&"overridden".to_string())
        );
        assert_eq!(
            compiled.env.get("PROD_VAR"),
            Some(&"prod_value".to_string())
        );
        assert_eq!(compiled.env.len(), 2);
    }

    #[test]
    fn test_compile_to_json() {
        let mut spec = DeploySpec::default();
        spec.project.name = "test-app".to_string();

        let compiled = compile(&spec, None).unwrap();
        let json = compiled.to_json().unwrap();

        assert!(json.contains("\"test-app\""));
        assert!(json.contains("\"development\""));
    }
}
