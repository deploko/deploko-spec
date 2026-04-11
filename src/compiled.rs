//! Compiled deployment specifications.
//!
//! This module provides functionality to compile a parsed `DeploySpec` with
//! environment-specific overrides and defaults into a final `CompiledSpec`.

use crate::error::{Error, Result};
use crate::schema::{
    AuthConfig, BackendConfig, DatabaseConfig, DeploySpec, EnvValue, EnvironmentOverride,
    FrontendConfig, ObservabilityConfig, ProjectConfig, StorageConfig,
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
    /// Final environment variables (resolved from EnvValue)
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
    let frontend = spec.frontend.clone();
    let backend = compile_backend(spec.backend.as_ref(), env_overrides);
    let database = compile_database(spec.database.as_ref(), env_overrides);
    let auth = spec.auth.clone();
    let storage = spec.storage.clone();
    let observability = compile_observability(spec.observability.as_ref(), env_overrides);

    // Compile environment variables
    let mut env = HashMap::new();

    // Add base environment variables (resolving EnvValue to String)
    if let Some(base_env) = &spec.env {
        for (key, value) in base_env {
            let resolved = match value {
                EnvValue::Literal(s) => s.clone(),
                EnvValue::Secret(secret_ref) => format!("[secret:{}]", secret_ref.key()),
            };
            env.insert(key.clone(), resolved);
        }
    }

    // Add environment-specific overrides
    if let Some(overrides) = env_overrides
        && let Some(env_overrides) = &overrides.env
    {
        for (key, value) in env_overrides {
            let resolved = match value {
                EnvValue::Literal(s) => s.clone(),
                EnvValue::Secret(secret_ref) => format!("[secret:{}]", secret_ref.key()),
            };
            env.insert(key.clone(), resolved);
        }
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
fn compile_project(base: &ProjectConfig, overrides: Option<&EnvironmentOverride>) -> ProjectConfig {
    if let Some(overrides) = overrides {
        ProjectConfig {
            name: base.name.clone(),
            region: overrides.region.unwrap_or(base.region),
            environment: base.environment.clone(),
        }
    } else {
        base.clone()
    }
}

/// Compile backend configuration with overrides.
fn compile_backend(
    base: Option<&BackendConfig>,
    overrides: Option<&EnvironmentOverride>,
) -> Option<BackendConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let mut result = base.clone();
            // Apply scale override if present
            if let Some(override_scale) = &overrides.scale {
                result.scale = Some(override_scale.clone());
            }
            Some(result)
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(_)) | (None, None) => None,
    }
}

/// Compile database configuration with overrides.
fn compile_database(
    base: Option<&DatabaseConfig>,
    overrides: Option<&EnvironmentOverride>,
) -> Option<DatabaseConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let mut result = base.clone();
            if let Some(override_db) = &overrides.database {
                result.engine = override_db.engine;
                // Only overwrite if override value is non-empty
                if !override_db.version.is_empty() {
                    result.version = override_db.version.clone();
                }
                if !override_db.instance_size.is_empty() {
                    result.instance_size = override_db.instance_size.clone();
                }
                if override_db.pooler.is_some() {
                    result.pooler = override_db.pooler;
                }
                if override_db.extensions.is_some() {
                    result.extensions = override_db.extensions.clone();
                }
                if override_db.backups.is_some() {
                    result.backups = override_db.backups.clone();
                }
            }
            Some(result)
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.database.clone(),
        (None, None) => None,
    }
}

/// Compile observability configuration with overrides.
fn compile_observability(
    base: Option<&ObservabilityConfig>,
    overrides: Option<&EnvironmentOverride>,
) -> Option<ObservabilityConfig> {
    match (base, overrides) {
        (Some(base), Some(overrides)) => {
            let mut result = base.clone();
            if let Some(override_obs) = &overrides.observability {
                if override_obs.logs.is_some() {
                    result.logs = override_obs.logs.clone();
                }
                if override_obs.metrics.is_some() {
                    result.metrics = override_obs.metrics.clone();
                }
                if override_obs.uptime.is_some() {
                    result.uptime = override_obs.uptime.clone();
                }
                if override_obs.alerts.is_some() {
                    result.alerts = override_obs.alerts.clone();
                }
            }
            Some(result)
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(overrides)) => overrides.observability.clone(),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Region;

    #[test]
    fn test_compile_no_overrides() {
        let spec = DeploySpec::default();

        let compiled = compile(&spec, None).unwrap();

        assert_eq!(compiled.project.name, "default");
        assert_eq!(compiled.project.region, Region::UsEast1);
        assert_eq!(compiled.environment, "development");
    }

    #[test]
    fn test_compile_with_region_override() {
        let spec = DeploySpec::default();

        let mut environments = HashMap::new();
        let prod_env = EnvironmentOverride {
            region: Some(Region::EuCentral1),
            scale: None,
            database: None,
            env: None,
            observability: None,
        };

        environments.insert("production".to_string(), prod_env);
        let mut spec = spec;
        spec.environments = Some(environments);

        let compiled = compile(&spec, Some("production")).unwrap();

        assert_eq!(compiled.project.region, Region::EuCentral1);
        assert_eq!(compiled.environment, "production");
    }

    #[test]
    fn test_compile_env_vars_merge() {
        let mut spec = DeploySpec::default();

        let mut base_env = HashMap::new();
        base_env.insert(
            "BASE_VAR".to_string(),
            EnvValue::Literal("base_value".to_string()),
        );
        spec.env = Some(base_env);

        let mut environments = HashMap::new();
        let mut prod_env_vars = HashMap::new();
        prod_env_vars.insert(
            "PROD_VAR".to_string(),
            EnvValue::Literal("prod_value".to_string()),
        );
        prod_env_vars.insert(
            "BASE_VAR".to_string(),
            EnvValue::Literal("overridden".to_string()),
        );

        let prod_env = EnvironmentOverride {
            region: None,
            scale: None,
            database: None,
            env: Some(prod_env_vars),
            observability: None,
        };

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
    fn test_compile_with_secret_env_value() {
        use crate::schema::SecretRef;

        let mut spec = DeploySpec::default();

        let mut base_env = HashMap::new();
        base_env.insert(
            "API_KEY".to_string(),
            EnvValue::Secret(SecretRef::new("PROD_API_KEY").unwrap()),
        );
        spec.env = Some(base_env);

        let compiled = compile(&spec, None).unwrap();

        assert_eq!(
            compiled.env.get("API_KEY"),
            Some(&"[secret:PROD_API_KEY]".to_string())
        );
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
