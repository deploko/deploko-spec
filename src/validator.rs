//! Validation for Deploko deployment specifications.
//!
//! This module provides functionality to validate parsed deployment specifications
//! against the Deploko specification rules and constraints.

use crate::schema::{
    BackendConfig, DatabaseConfig, DeploySpec, EnvValue, FrontendConfig, ProjectConfig,
};
use std::collections::HashMap;

/// A validation report containing all found issues.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Create a new validation report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the report.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a warning to the report.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Check if the specification is valid (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the total number of issues.
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Field path (e.g., "project.name")
    pub field: String,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// A validation warning.
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Field path
    pub field: String,
}

/// Error severity levels.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Critical error that prevents deployment
    Critical,
    /// Error that may cause issues
    Error,
    /// Minor issue
    Warning,
}

/// Validate a complete deployment specification.
pub fn validate(spec: &DeploySpec) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Validate project configuration
    validate_project(&spec.project, &mut report);

    // Validate frontend configuration if present
    if let Some(frontend) = &spec.frontend {
        validate_frontend(frontend, &mut report);
    }

    // Validate backend configuration if present
    if let Some(backend) = &spec.backend {
        validate_backend(backend, &mut report);
    }

    // Validate database configuration if present
    if let Some(database) = &spec.database {
        validate_database(database, &mut report);
    }

    // Validate environment configurations
    if let Some(environments) = &spec.environments {
        validate_environments(environments, &mut report);
    }

    // Validate environment variables
    if let Some(env_vars) = &spec.env {
        validate_env_values(env_vars, &mut report);
    }

    report
}

/// Validate project configuration.
fn validate_project(project: &ProjectConfig, report: &mut ValidationReport) {
    // Validate project name
    if project.name.is_empty() {
        report.add_error(ValidationError {
            message: "Project name cannot be empty".to_string(),
            field: "project.name".to_string(),
            severity: ErrorSeverity::Critical,
        });
    } else if !is_valid_name(&project.name) {
        report.add_error(ValidationError {
            message: "Project name contains invalid characters (allowed: alphanumeric, '-', '_')"
                .to_string(),
            field: "project.name".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    // Region is now an enum, so it's always valid
    // (no additional validation needed)

    // Validate default environment
    if let Some(env) = &project.environment
        && !is_valid_name(env)
    {
        report.add_error(ValidationError {
            message:
                "Environment name contains invalid characters (allowed: alphanumeric, '-', '_')"
                    .to_string(),
            field: "project.environment".to_string(),
            severity: ErrorSeverity::Error,
        });
    }
}

/// Validate frontend configuration.
fn validate_frontend(frontend: &FrontendConfig, report: &mut ValidationReport) {
    // Framework is now an enum, so it's always valid
    // (no additional validation needed)

    // Validate repository URL
    if frontend.repo.is_empty() {
        report.add_error(ValidationError {
            message: "Frontend repository cannot be empty".to_string(),
            field: "frontend.repo".to_string(),
            severity: ErrorSeverity::Critical,
        });
    } else if !is_valid_git_url(&frontend.repo) {
        report.add_error(ValidationError {
            message: "Invalid Git repository URL".to_string(),
            field: "frontend.repo".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    // Validate branch
    if frontend.branch.is_empty() {
        report.add_error(ValidationError {
            message: "Frontend branch cannot be empty".to_string(),
            field: "frontend.branch".to_string(),
            severity: ErrorSeverity::Critical,
        });
    }

    // Validate build command
    if frontend.build_cmd.is_empty() {
        report.add_error(ValidationError {
            message: "Build command cannot be empty".to_string(),
            field: "frontend.build_cmd".to_string(),
            severity: ErrorSeverity::Critical,
        });
    }
}

/// Validate backend configuration.
fn validate_backend(backend: &BackendConfig, report: &mut ValidationReport) {
    // Runtime is now an enum, so it's always valid
    // (no additional validation needed)

    // Validate scale configuration if present
    if let Some(scale) = &backend.scale {
        validate_scale_config(scale, report);
    }

    // Validate health check configuration if present
    if let Some(health_check) = &backend.health_check {
        validate_health_check_config(health_check, report);
    }
}

/// Validate scale configuration.
fn validate_scale_config(scale: &crate::schema::ScaleConfig, report: &mut ValidationReport) {
    if scale.min == 0 {
        report.add_error(ValidationError {
            message: "Minimum instances cannot be zero".to_string(),
            field: "scale.min".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    if scale.max == 0 {
        report.add_error(ValidationError {
            message: "Maximum instances cannot be zero".to_string(),
            field: "scale.max".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    if scale.max < scale.min {
        report.add_error(ValidationError {
            message: "Maximum instances cannot be less than minimum".to_string(),
            field: "scale.max".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    if let Some(target_cpu) = scale.target_cpu
        && target_cpu > 100
    {
        report.add_error(ValidationError {
            message: "Target CPU cannot exceed 100%".to_string(),
            field: "scale.target_cpu".to_string(),
            severity: ErrorSeverity::Error,
        });
    }
}

/// Validate health check configuration.
fn validate_health_check_config(
    health_check: &crate::schema::HealthCheckConfig,
    report: &mut ValidationReport,
) {
    if health_check.path.is_empty() {
        report.add_error(ValidationError {
            message: "Health check path cannot be empty".to_string(),
            field: "health_check.path".to_string(),
            severity: ErrorSeverity::Error,
        });
    } else if !health_check.path.starts_with('/') {
        report.add_error(ValidationError {
            message: "Health check path must start with '/'".to_string(),
            field: "health_check.path".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    if health_check.interval_secs == 0 {
        report.add_error(ValidationError {
            message: "Health check interval cannot be zero".to_string(),
            field: "health_check.interval_secs".to_string(),
            severity: ErrorSeverity::Error,
        });
    }

    if health_check.timeout_secs >= health_check.interval_secs {
        report.add_error(ValidationError {
            message: "Health check timeout must be less than interval".to_string(),
            field: "health_check.timeout_secs".to_string(),
            severity: ErrorSeverity::Error,
        });
    }
}

/// Validate database configuration.
fn validate_database(database: &DatabaseConfig, report: &mut ValidationReport) {
    // Validate engine is supported
    if !is_supported_database_engine(&database.engine) {
        report.add_error(ValidationError {
            message: format!(
                "Database engine {:?} is not supported (only Postgres is supported)",
                database.engine
            ),
            field: "database.engine".to_string(),
            severity: ErrorSeverity::Critical,
        });
    }

    // Validate version
    if database.version.is_empty() {
        report.add_error(ValidationError {
            message: "Database version cannot be empty".to_string(),
            field: "database.version".to_string(),
            severity: ErrorSeverity::Critical,
        });
    }

    // Validate instance size
    if database.instance_size.is_empty() {
        report.add_error(ValidationError {
            message: "Database instance size cannot be empty".to_string(),
            field: "database.instance_size".to_string(),
            severity: ErrorSeverity::Critical,
        });
    }

    // Validate backup configuration
    if let Some(backups) = &database.backups {
        // Validate retention duration is not zero
        if backups.retain.seconds() == 0 {
            report.add_error(ValidationError {
                message: "Backup retention duration cannot be zero".to_string(),
                field: "database.backups.retain".to_string(),
                severity: ErrorSeverity::Error,
            });
        }
    }
}

/// Validate environment configurations.
fn validate_environments(
    environments: &HashMap<String, crate::schema::EnvironmentOverride>,
    report: &mut ValidationReport,
) {
    for name in environments.keys() {
        if !is_valid_name(name) {
            report.add_error(ValidationError {
                message:
                    "Environment name contains invalid characters (allowed: alphanumeric, '-', '_')"
                        .to_string(),
                field: format!("environments.{}", name),
                severity: ErrorSeverity::Error,
            });
        }

        // EnvironmentOverride doesn't have a name field - validation not needed
        // (the HashMap key serves as the environment name)
    }
}

/// Validate environment variables (EnvValue version).
fn validate_env_values(
    env_vars: &HashMap<String, crate::schema::EnvValue>,
    report: &mut ValidationReport,
) {
    for (key, value) in env_vars {
        if !is_valid_env_key(key) {
            report.add_error(ValidationError {
                message: "Environment variable key contains invalid characters (allowed: alphabetic start, then alphanumeric or '_')".to_string(),
                field: format!("env.{}", key),
                severity: ErrorSeverity::Error,
            });
        }

        // Check if the value is effectively empty
        let is_empty = match value {
            EnvValue::Literal(s) => s.is_empty(),
            EnvValue::Secret(_) => false, // Secrets are never "empty"
        };
        if is_empty {
            report.add_warning(ValidationWarning {
                message: "Environment variable value is empty".to_string(),
                field: format!("env.{}", key),
            });
        }
    }
}

/// Check if a name contains only valid characters.
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Check if a database engine is supported.
fn is_supported_database_engine(engine: &crate::schema::DatabaseEngine) -> bool {
    matches!(engine, crate::schema::DatabaseEngine::Postgres)
}

/// Check if a URL is a valid Git repository URL.
fn is_valid_git_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("git@") || url.starts_with("git://")
}

/// Check if an environment variable key is valid.
fn is_valid_env_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .next()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
        && key.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{BackendConfig, FrontendConfig, ProjectConfig};

    #[test]
    fn test_valid_project() {
        let project = ProjectConfig {
            name: "test-app".to_string(),
            region: crate::schema::Region::UsEast1,
            environment: Some("production".to_string()),
        };

        let mut report = ValidationReport::new();
        validate_project(&project, &mut report);

        assert!(report.is_valid());
        assert_eq!(report.total_issues(), 0);
    }

    #[test]
    fn test_invalid_project_name() {
        let project = ProjectConfig {
            name: "".to_string(),
            region: crate::schema::Region::UsEast1,
            environment: None,
        };

        let mut report = ValidationReport::new();
        validate_project(&project, &mut report);

        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_valid_frontend() {
        let frontend = FrontendConfig {
            framework: crate::schema::Framework::Vite,
            repo: "https://github.com/example/app.git".to_string(),
            branch: "main".to_string(),
            build_cmd: "npm run build".to_string(),
            output_dir: Some("build".to_string()),
            env: None,
            node_version: None,
        };

        let mut report = ValidationReport::new();
        validate_frontend(&frontend, &mut report);

        assert!(report.is_valid());
        assert_eq!(report.total_issues(), 0);
    }

    #[test]
    fn test_invalid_scale_config() {
        use crate::schema::ScaleConfig;

        let scale = ScaleConfig {
            min: 5,
            max: 2, // Invalid: max < min
            target_cpu: Some(80),
        };

        let backend = BackendConfig {
            runtime: crate::schema::Runtime::Node,
            dockerfile: None,
            scale: Some(scale),
            health_check: None,
            env: None,
            port: None,
        };

        let mut report = ValidationReport::new();
        validate_backend(&backend, &mut report);

        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
    }
}
