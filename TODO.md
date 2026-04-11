# deploko-spec — TODO

Parser and validator crate for the Deploko `deploko.toml` spec.

**SemVer target:** `v0.1.0` first stable publish to crates.io
**MSRV:** 1.94 (declared in `.clippy.toml`, tested in CI)
**Commit convention:** [Conventional Commits](https://www.conventionalcommits.org)

---

## Phase 0 — Crate Bootstrap
**Milestone:** `v0.1.0-alpha`

### 0.1 Cargo Setup
- [x] Create `Cargo.toml`
  - [x] Set `name = "deploko-spec"`
  - [x] Set `description = "Parser and validator for the Deploko deploko.toml spec"`
  - [x] Set `keywords = ["deployment", "infrastructure", "paas", "config"]`
  - [x] Set `categories = ["config", "development-tools"]`
  - [x] Dev-dependencies: `insta`, `tempfile`, `anyhow`

### 0.2 Repo Files
- [x] Create `README.md` (crate-specific install and usage)
- [x] Create `ARCHITECTURE.md` (module structure, data flow)
- [x] Create `LICENSE` (Apache-2.0)
- [x] Create `CHANGELOG.md` ([Keep a Changelog](https://keepachangelog.com) format)
- [x] Create `.gitignore` (`/target`, `**/*.rs.bk`, `fuzz/corpus/`, `fuzz/artifacts/`)

### 0.3 CI
- [x] Create `.github/workflows/ci.yml`
  - [x] Job `fmt`: `cargo fmt --all --check`
  - [x] Job `clippy`: `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] Job `test`: matrix `[ubuntu-latest, macos-latest, windows-latest]` × `[stable, 1.94]`
    - [x] Steps: checkout → install toolchain → cache `~/.cargo` and `target/` → `cargo test --all-features`
    - [x] Coverage upload on ubuntu/stable via `cargo-llvm-cov` → Codecov
  - [x] Job `deny`: `cargo deny check`
  - [x] Job `docs`: `cargo doc --no-deps` — fail on broken intra-doc links
  - [x] Job `msrv`: install `1.94` toolchain → `cargo check`
  - [x] Concurrency group: cancel stale runs on same branch

- [x] Create `.github/workflows/release.yml`
  - [x] Trigger: push tags `v[0-9]+.[0-9]+.[0-9]+*`
  - [x] Job `validate-tag`: verify tag matches `Cargo.toml` version; verify `CHANGELOG.md` has entry
  - [x] Job `publish` (`needs: validate-tag`): `cargo publish -p deploko-spec`
  - [x] Job `generate-sbom` (`needs: validate-tag`): `cargo cyclonedx` → upload artifact
  - [x] Job `create-release` (`needs: [publish, generate-sbom]`): create GitHub Release with SBOM and CHANGELOG entry

- [x] Create `.github/workflows/security.yml`
  - [x] Trigger: weekly cron Monday 03:00 UTC + `workflow_dispatch`
  - [x] Job `audit`: `cargo audit --json`; on failure open GitHub issue
  - [x] Job `deny`: `cargo deny check advisories`; on failure open GitHub issue

- [x] Create `.github/dependabot.yml`
  - [x] Cargo: weekly, Monday, max 5 PRs, label `dependencies`
  - [x] GitHub Actions: monthly, label `ci`

### 0.4 DevSecOps
- [x] Create `.pre-commit-config.yaml` with Rust hooks (fmt, clippy, test)
- [ ] Enable secret scanning on repo
- [ ] Enable push protection on repo
- [ ] Enable private vulnerability reporting
- [ ] Enable Dependabot security updates
- [ ] Branch protection `main`: require PR, require CI pass, require signed commits, no force push
- [ ] Branch protection `develop`: require PR, require CI pass
- [ ] Install DCO app, add DCO as required status check on `main`
- [x] Pin all GitHub Actions workflow steps to full commit SHA
- [x] Add `permissions: read-all` default to all workflow files; scope `contents: write` to release job only
- [x] Create `.github/workflows/scorecard.yml`: OpenSSF Scorecard, weekly + push to `main`

---

## Phase 1 — Schema Definitions
**Milestone:** `v0.1.0`

### 1.1 Module Skeleton
- [x] Create `src/lib.rs`
  - [x] Declare modules: `schema`, `parser`, `validator`, `compiled`, `error`
  - [x] Add `#![deny(missing_docs)]`
- [x] Create `src/schema.rs`
- [x] Create `src/parser.rs`
- [x] Create `src/validator.rs`
- [x] Create `src/compiled.rs`
- [x] Create `src/error.rs`

### 1.2 Custom Scalar Types (`src/schema.rs`)
- [x] Implement `ByteSize` newtype wrapping `u64` (bytes)
  - [x] `Deserialize`: parse `"10gb"`, `"500mb"`, `"1tb"`, `"100kb"` → `u64` bytes
  - [x] Error on unrecognized unit
  - [x] Error on zero value
  - [x] `Display`: render as human-readable (e.g. `10 GB`)
  - [x] `Serialize`: round-trip as string
- [x] Implement `Duration` newtype wrapping `u64` (seconds)
  - [x] `Deserialize`: parse `"30d"`, `"7d"`, `"1h"`, `"15m"`, `"60s"` → `u64` seconds
  - [x] Error on unrecognized unit
  - [x] Error on zero value
  - [x] `Display`: render as human-readable (e.g. `30 days`)
  - [x] `Serialize`: round-trip as string

### 1.3 Secret Types (`src/schema.rs`)
- [x] Implement `SecretRef(String)` newtype
  - [x] `Deserialize`: parse `${secrets.KEY}` → extract `KEY`
  - [x] Error if pattern does not match `^\$\{secrets\.[A-Z][A-Z0-9_]{0,63}\}$`
  - [x] `Display`: render as `${secrets.KEY}` (key name visible, never the value)
- [x] Implement `EnvValue` enum
  - [x] Variant `Literal(String)`
  - [x] Variant `Secret(SecretRef)`
  - [x] `Deserialize`: detect `${secrets.*}` pattern → `Secret`, else `Literal`
  - [x] `Display`: `Literal` → raw value; `Secret` → `[secret:KEY]` (never expose value)

### 1.4 Enums (`src/schema.rs`)
- [x] `Region` enum
  - [x] `ApSoutheast1` → `"ap-southeast-1"`
  - [x] `EuCentral1` → `"eu-central-1"`
  - [x] `UsEast1` → `"us-east-1"`
  - [x] `Display` with full city name
  - [x] `FromStr` for CLI arg parsing
- [x] `Framework` enum
  - [x] Variants: `Nextjs`, `Sveltekit`, `Nuxt`, `Astro`, `Remix`, `Vite`, `Static`
  - [x] `default_output_dir(&self) -> &str` per variant
  - [x] `default_build_cmd(&self) -> &str` per variant
- [x] `Runtime` enum
  - [x] Variants: `Rust`, `Node`, `Python`, `Go`, `Java`, `Ruby`, `Php`, `Docker`
  - [x] `default_dockerfile_hint(&self) -> &str` (base image suggestion)
- [x] `DatabaseEngine` enum
  - [x] `Postgres` → `"postgres"`
  - [x] `Mysql`, `Redis` → parse but emit unsupported error in validator
- [x] `BackupSchedule` enum: `Hourly`, `Daily`, `Weekly`
- [x] `AuthProviderKind` enum: `Email`, `Google`, `Github`, `Apple`, `Discord`, `Slack`

### 1.5 Config Structs (`src/schema.rs`)
- [ ] `ProjectConfig`: `name: String`, `region: Region`, `environment: Option<String>`
- [ ] `ScaleConfig`: `min: u32` (default 1), `max: u32` (default 1) + `Default` impl
- [ ] `HealthcheckConfig`: `path` (default `"/health"`), `interval_secs` (30), `timeout_secs` (5), `retries` (3)
- [ ] `BackupConfig`: `schedule: BackupSchedule`, `retain: Duration` + `Default` impl
- [ ] `FrontendConfig`: `framework`, `repo`, `branch`, `build_cmd`, `output_dir`, `env`, `node_version`
- [ ] `BackendConfig`: `runtime`, `dockerfile`, `scale`, `healthcheck`, `env`, `port`
- [ ] `DatabaseConfig`: `engine`, `version`, `pooler`, `extensions`, `backups`
- [ ] `AuthConfig`: `enabled`, `providers`, `jwt_expiry`, `refresh_token_expiry`
- [ ] `StorageConfig`: `enabled`, `limit`, `public_buckets`
- [ ] `AlertConfig`: `email`, `slack_webhook`, `pagerduty_key`
- [ ] `ObservabilityConfig`: `logs`, `metrics`, `uptime`, `alerts` + `Default` impl
- [ ] `EnvironmentOverride`: `region`, `scale`, `database`, `env`, `observability`
- [ ] `DeploySpec` (top-level): all sections as `Option<*Config>` except `project`

---

## Phase 2 — Parser
**Milestone:** `v0.1.0`

### 2.1 TOML Parser (`src/parser.rs`)
- [ ] `pub fn parse_toml(input: &str) -> Result<DeploySpec, ParseError>`
  - [ ] `toml::from_str::<DeploySpec>(input)`
  - [ ] Map `toml::de::Error` → `ParseError::Toml { line, col, message }`


### 2.3 File Helpers (`src/parser.rs`)
- [ ] `pub fn parse_file(path: &Path) -> Result<DeploySpec, ParseError>`
  - [ ] Read file → map IO error to `ParseError::Io`
  - [ ] Error `ParseError::UnknownFormat` if extension unrecognized
- [ ] `pub fn parse_auto(dir: &Path) -> Result<DeploySpec, ParseError>`
  - [ ] Look for `deploko.toml` in the given directory
  - [ ] Error `ParseError::NoSpecFile { searched_dir }` if not found

### 2.4 Error Types (`src/error.rs`)
- [ ] `ParseError` enum with `thiserror::Error`
  - [ ] `Toml { line: Option<usize>, col: Option<usize>, message: String }`
  - [ ] `Io { path: PathBuf, #[source] source: std::io::Error }`
  - [ ] `UnknownFormat { extension: String }`
  - [ ] `NoSpecFile { searched_dir: PathBuf }`

---

## Phase 3 — Validation Engine
**Milestone:** `v0.1.0`

### 3.1 Report and Error Types (`src/error.rs`)
- [ ] `ValidationError` enum with `thiserror::Error`
  - [ ] `RequiredField { field: String }`
  - [ ] `InvalidValue { field: String, value: String, reason: String }`
  - [ ] `InvalidFormat { field: String, expected: String, got: String }`
  - [ ] `CrossFieldConflict { fields: Vec<String>, reason: String }`
  - [ ] `UnsupportedValue { field: String, value: String, supported: Vec<String> }`
  - [ ] `InvalidSecretRef { key: String, reason: String }`
- [ ] `ValidationWarning` enum
  - [ ] `DefaultValueUsed { field: String, default: String }`
  - [ ] `NoHealthcheck`
  - [ ] `ScaleMinIsZero`
  - [ ] `ObservabilityDisabled`
  - [ ] `LowNodeVersion { version: String }`
- [ ] `ValidationReport` struct
  - [ ] `errors: Vec<ValidationError>`
  - [ ] `warnings: Vec<ValidationWarning>`
  - [ ] `fn is_valid(&self) -> bool` → `self.errors.is_empty()`

### 3.2 Validation Rules (`src/validator.rs`)
- [ ] `pub fn validate(spec: &DeploySpec) -> ValidationReport` — collect ALL errors, no short-circuit

**`[project]` rules:**
- [ ] `project.name` not empty
- [ ] `project.name` length 3–63
- [ ] `project.name` matches `^[a-z0-9][a-z0-9-]*[a-z0-9]$`
- [ ] `project.region` is known `Region` variant

**`[frontend]` rules:**
- [ ] `frontend.repo` matches `^(github|gitlab)\.com/[^/]+/[^/]+$`
- [ ] `frontend.branch` not empty
- [ ] `frontend.branch` valid git ref (no whitespace, no `..`, no `@{`, no `\`)
- [ ] `frontend.build_cmd` not empty
- [ ] Warn if `frontend.node_version` < `"18"`

**`[backend]` rules:**
- [ ] `backend.scale.min` ≤ `backend.scale.max`
- [ ] `backend.scale.max` ≤ `50`
- [ ] `backend.port` in range `1024`–`65535`
- [ ] `backend.healthcheck.timeout_secs` < `backend.healthcheck.interval_secs`
- [ ] Warn if `backend.scale.min = 0`
- [ ] Warn if `backend.healthcheck` is absent

**`[database]` rules:**
- [ ] `database.version` is `14`, `15`, or `16` when `engine = "postgres"`
- [ ] `database.extensions` entries non-empty
- [ ] `database.backups.retain` ≥ `"1d"`
- [ ] Warn if `engine = "mysql"` or `"redis"` (planned, not yet supported)

**`[auth]` rules:**
- [ ] `auth.enabled = true` → `auth.providers` non-empty
- [ ] `auth.enabled = true` → `[database]` block must exist
- [ ] `auth.providers` no duplicates

**`[storage]` rules:**
- [ ] `storage.limit` ≥ `"1mb"` if set
- [ ] `storage.public_buckets` entries match `^[a-z0-9][a-z0-9-]*[a-z0-9]$`

**`[observability]` rules:**
- [ ] `alerts.email` entries match basic email pattern
- [ ] Warn if all of `logs`, `metrics`, `uptime` are `false`

**`[env]` rules:**
- [ ] Each key matches `^[A-Z][A-Z0-9_]*$`

**Cross-field rules:**
- [ ] At least one of `[frontend]` or `[backend]` must be present
- [ ] `[environments.*]` names match `^[a-z][a-z0-9-]*$`
- [ ] `[environments.*]` names not `"base"` or `"default"` (reserved)

---

## Phase 4 — CompiledSpec
**Milestone:** `v0.1.0`

### 4.1 Struct and Compilation (`src/compiled.rs`)
- [ ] Define `CompiledSpec`
  - [ ] Fields: `spec_version: String` (`"1"`), all config fields with defaults applied, `env: IndexMap<String, EnvValue>`, `resolved_environment: String`
  - [ ] `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]`
- [ ] `CompileError` enum (`src/error.rs`)
  - [ ] `UnknownEnvironment { name: String }`
- [ ] `pub fn compile(spec: &DeploySpec, environment: Option<&str>) -> Result<CompiledSpec, CompileError>`
  - [ ] Apply `Default` for all absent `Option` fields
  - [ ] Apply environment overlay if provided and present in `spec.environments`
  - [ ] Error `UnknownEnvironment` if env name requested but not found
  - [ ] Merge `env` maps: overlay wins, keys sorted
  - [ ] Verify internal consistency post-merge
- [ ] `impl CompiledSpec`
  - [ ] `fn to_json(&self) -> String` — `serde_json::to_string_pretty`, keys sorted
  - [ ] `fn to_json_compact(&self) -> String` — `serde_json::to_string`

---

## Phase 5 — Testing
**Milestone:** `v0.1.0`

### 5.1 Fixtures
- [ ] Create `tests/fixtures/valid/minimal.toml` (project + frontend only)
- [ ] Create `tests/fixtures/valid/full.toml` (all sections)
- [ ] Create `tests/fixtures/valid/with_backend.toml`
- [ ] Create `tests/fixtures/valid/with_database.toml`
- [ ] Create `tests/fixtures/valid/with_environments.toml`
- [ ] Create `tests/fixtures/valid/with_secrets.toml`
- [ ] Create `tests/fixtures/invalid/missing_project_name.toml`
- [ ] Create `tests/fixtures/invalid/invalid_region.toml`
- [ ] Create `tests/fixtures/invalid/scale_min_exceeds_max.toml`
- [ ] Create `tests/fixtures/invalid/auth_without_database.toml`
- [ ] Create `tests/fixtures/invalid/empty_providers.toml`
- [ ] Create `tests/fixtures/invalid/invalid_repo_format.toml`
- [ ] Create `tests/fixtures/invalid/invalid_secret_ref.toml`
- [ ] Create `tests/fixtures/invalid/invalid_env_key.toml`
- [ ] Create `tests/fixtures/invalid/nothing_to_deploko.toml`

### 5.2 Parser Tests
- [ ] `parse_toml` succeeds on all 7 valid fixtures
- [ ] `parse_toml` fails with `ParseError::Toml` on malformed TOML
- [ ] `parse_auto` finds `deploko.toml` in directory
- [ ] `parse_auto` returns `NoSpecFile` for empty directory
- [ ] `ByteSize::deserialize("10gb")` == `10_737_418_240`
- [ ] `ByteSize::deserialize("500mb")` == `524_288_000`
- [ ] `ByteSize::deserialize("0gb")` returns error
- [ ] `ByteSize::deserialize("10 gb")` (space) returns error
- [ ] `Duration::deserialize("30d")` == `2_592_000`
- [ ] `Duration::deserialize("1h")` == `3_600`
- [ ] `Duration::deserialize("0d")` returns error
- [ ] `EnvValue::deserialize("${secrets.DB_URL}")` is `Secret("DB_URL")`
- [ ] `EnvValue::deserialize("plain-value")` is `Literal("plain-value")`
- [ ] `SecretRef` rejects `"${secrets.}"` (empty key)
- [ ] `SecretRef` rejects `"${secrets.lower}"` (lowercase key)

### 5.3 Validation Tests
- [ ] Empty errors for all 7 valid fixtures
- [ ] `RequiredField { field: "project.name" }` for missing name
- [ ] `InvalidValue` for `name = "a"` (too short)
- [ ] `InvalidValue` for `name = "My App"` (uppercase + space)
- [ ] `InvalidValue` for `region = "us-west-99"` (unknown)
- [ ] `CrossFieldConflict` for `scale.min=5, max=2`
- [ ] `CrossFieldConflict` for auth enabled without `[database]`
- [ ] `RequiredField` for `auth.enabled = true` with empty `providers`
- [ ] `ScaleMinIsZero` warning for `scale.min = 0`
- [ ] `CrossFieldConflict` for no `[frontend]` and no `[backend]`
- [ ] All 9 invalid fixtures produce ≥ 1 `ValidationError`

### 5.4 CompiledSpec Tests
- [ ] `compile(full.toml, None)` matches insta snapshot
- [ ] `compile(minimal.toml, None)` applies all defaults correctly
- [ ] `compile(with_environments.toml, Some("staging"))` applies staging overlay
- [ ] `compile` is deterministic: two calls produce byte-identical JSON
- [ ] `compile` env merge: overlay key wins over base key
- [ ] `compile` returns `UnknownEnvironment` for unknown env name

### 5.5 Fuzz Targets
- [ ] Create `fuzz/fuzz_targets/fuzz_parse_toml.rs` — arbitrary bytes → `parse_toml`, no panics
- [ ] Create `fuzz/fuzz_targets/fuzz_validate.rs` — structured fuzzing of `DeploySpec`, no panics

---

## Phase 6 — Documentation
**Milestone:** `v0.1.0`

- [ ] Write module-level doc comment for each module (`schema`, `parser`, `validator`, `compiled`, `error`)
- [ ] Write doc comment + `# Examples` for every public function
- [ ] Write doc comment for every public struct and enum
- [ ] Verify `cargo doc --no-deps` produces zero warnings
- [ ] Verify no broken intra-doc links

---

## Phase 7 — Publish `v0.1.0`

- [ ] `cargo publish --dry-run -p deploko-spec` — zero errors
- [ ] `cargo package -p deploko-spec` — inspect included files
- [ ] Verify `README.md` renders correctly on crates.io preview
- [ ] Update `CHANGELOG.md`: `[Unreleased]` → `[0.1.0] - YYYY-MM-DD`
- [ ] Bump version to `0.1.0` in `Cargo.toml`
- [ ] Commit: `chore(release): v0.1.0`
- [ ] Annotated tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] Push tag → verify `release.yml` completes
- [ ] Verify `deploko-spec` on `crates.io/crates/deploko-spec`
- [ ] Verify GitHub Release with SBOM attached

---

## Ongoing

### SemVer Release Checklist
- [ ] All CI jobs green
- [ ] `cargo test --all-features` passes locally
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `cargo deny check` passes
- [ ] `cargo audit` clean
- [ ] `CHANGELOG.md` updated
- [ ] Version bumped in `Cargo.toml`
- [ ] Annotated tag pushed → release workflow runs
- [ ] crates.io publish verified
- [ ] GitHub Release with SBOM verified

### Dependency Management (weekly)
- [ ] Review `cargo audit` output
- [ ] Review Dependabot PRs: merge or close with justification
- [ ] `cargo deny check` still passes after any merges

### Icebox
- [ ] JSON Schema generation from `DeploySpec` (export `deploko.schema.json`)
- [ ] `#[non_exhaustive]` on all public enums at `v1.0.0` stabilization
- [ ] LSP-friendly error spans with byte offsets (for IDE integration)
