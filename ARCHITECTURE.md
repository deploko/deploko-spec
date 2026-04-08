# deploko-spec — Architecture

Internal architecture of the `deploko-spec` crate. For the system-level architecture see the [deploko org hub](https://github.com/deploko/deploko/blob/main/ARCHITECTURE.md).

---

## Module Structure

```
deploko-spec/
└── src/
    ├── lib.rs          — public API re-exports, #![deny(missing_docs)]
    ├── schema.rs       — all struct and enum definitions (DeploySpec, etc.)
    ├── validator.rs    — validate() → ValidationReport
    ├── compiled.rs     — compile() → CompiledSpec, serialization
    └── error.rs        — ParseError, ValidationError, CompileError types
```

---

## Data Flow

```
Raw string (TOML)
        │
        ▼
   parser.rs
        │  serde deserialization
        ▼
   DeploySpec                    ← schema.rs types
        │
        ├──► validator.rs
        │    validate(&spec)
        │         │
        │         ▼
        │    ValidationReport { errors, warnings }
        │
        └──► compiled.rs
             compile(&spec, env)
                  │  apply defaults
                  │  merge env overlay
                  │  verify consistency
                  ▼
             CompiledSpec         ← deterministic JSON output
```

---

## Key Design Decisions

### Collect-all validation
`validate()` never short-circuits. It runs every validation rule and returns all errors in a single `ValidationReport`. This gives users complete feedback in one pass rather than fix-one-at-a-time.

### Deterministic compilation
`compile()` produces byte-identical JSON for the same input. Keys are sorted alphabetically via `BTreeMap`/`IndexMap`. This property is required for plan diffing (two runs of the same spec must produce identical `CompiledSpec` so the provisioner can detect no-ops).

### Custom scalar types
`ByteSize` (`"10gb"` → `u64` bytes) and `Duration` (`"30d"` → `u64` seconds) are custom newtypes with custom `serde::Deserialize` implementations. They store SI-unit strings in user-visible form and raw numeric values internally.

### Secret references
`EnvValue` is a union type: either a `Literal(String)` or a `Secret(SecretRef)`. The `Secret` variant stores only the key name (`DB_URL`), never the value. The provisioner resolves the actual value from the vault at deploy time.

### `IndexMap` for `env`
`env` fields use `IndexMap` (preserves insertion order) rather than `HashMap`. This keeps `deploko.toml` author intent intact in compiled output and in diffs.

---

## Error Hierarchy

```
ParseError
  ├── Toml { line, col, message }
  ├── Yaml { location, message }
  ├── Io { path, source }
  ├── UnknownFormat { extension }
  └── NoSpecFile { searched_dir }

ValidationError
  ├── RequiredField { field }
  ├── InvalidValue { field, value, reason }
  ├── InvalidFormat { field, expected, got }
  ├── CrossFieldConflict { fields, reason }
  ├── UnsupportedValue { field, value, supported }
  └── InvalidSecretRef { key, reason }

CompileError
  └── UnknownEnvironment { name }
```

---

## Testing Strategy

```
tests/
└── fixtures/
    ├── valid/          — parse + validate + compile must all succeed
    └── invalid/        — validate must produce ≥ 1 ValidationError each

Unit tests:   per-type deserializer correctness (ByteSize, Duration, SecretRef, EnvValue)
Validation:   one test per ValidationError and ValidationWarning variant
Snapshot:     CompiledSpec JSON output via insta (catches silent regressions)
```

---

## Public API Surface

```rust
// Parsing
pub fn parse_toml(input: &str) -> Result<DeploySpec, ParseError>;
pub fn parse_file(path: &Path) -> Result<DeploySpec, ParseError>;
pub fn parse_auto(dir: &Path) -> Result<DeploySpec, ParseError>;

// Validation
pub fn validate(spec: &DeploySpec) -> ValidationReport;

// Compilation
pub fn compile(spec: &DeploySpec, environment: Option<&str>) -> Result<CompiledSpec, CompileError>;

// Key types
pub struct DeploySpec { ... }
pub struct CompiledSpec { ... }  // impl: to_json(), to_json_compact()
pub struct ValidationReport { pub errors: Vec<ValidationError>, pub warnings: Vec<ValidationWarning> }
pub enum ValidationError { ... }
pub enum ValidationWarning { ... }
pub enum ParseError { ... }
pub enum CompileError { ... }
```

All other types (individual config structs, enums) are public for downstream use but considered implementation detail — they may change on MINOR version bumps until `v1.0.0`.
