# deploko-spec

[![crates.io](https://img.shields.io/crates/v/deploko-spec.svg)](https://crates.io/crates/deploko-spec)
[![docs.rs](https://docs.rs/deploko-spec/badge.svg)](https://docs.rs/deploko-spec)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/deploko/deploko-spec/actions/workflows/ci.yml/badge.svg)](https://github.com/deploko/deploko-spec/actions)

Parser and validator for the [Deploko](https://deploko.dev) `deploko.toml` spec.


---

## Usage

```toml
# Cargo.toml
[dependencies]
deploko-spec = "0.1"
```

```rust
use deploko_spec::{parse_auto, validate, compile};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Find and parse deploko.toml in current directory
    let spec = parse_auto(Path::new("."))?;

    // Validate — collect all errors, do not short-circuit
    let report = validate(&spec);
    if !report.is_valid() {
        for error in &report.errors {
            eprintln!("✗ {error}");
        }
        std::process::exit(1);
    }

    // Compile — apply defaults and environment overlay
    let compiled = compile(&spec, Some("production"))?;

    // Serialize to JSON for provisioner transport
    println!("{}", compiled.to_json());

    Ok(())
}
```

---

## What It Does

- **Parses** `deploko.toml` into a typed `DeploySpec` struct
- **Validates** all fields: required fields, enum values, cross-field constraints, secret reference syntax
- **Compiles** to `CompiledSpec`: applies defaults, merges environment overrides, produces deterministic JSON output

---

## Supported `deploko.toml` Sections

| Section | Description |
|---|---|
| `[project]` | Name, region, environment |
| `[frontend]` | Framework, repo, branch, build command |
| `[backend]` | Runtime, Dockerfile, scale config, healthcheck |
| `[database]` | Engine, version, connection pooler, backups |
| `[auth]` | Auth providers (email, OAuth) |
| `[storage]` | File storage with size limit |
| `[observability]` | Logs, metrics, uptime, alerts |
| `[env]` | Environment variables and secret references |
| `[environments.*]` | Staging / production overrides |

Full reference: [docs.deploko.dev/spec](https://docs.deploko.dev/spec)

---

## MSRV

Minimum Supported Rust Version: **1.70**

## License

Apache License 2.0 — see [LICENSE](LICENSE).
