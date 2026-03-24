use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use petname::petname;

pub fn create_scratch_dir(name: &str) -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tests/scratch")
        .join(format!("{name}-{}", petname(2, "-")));
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

pub fn create_no_contract_workspace(name: &str) -> Result<PathBuf> {
    let dir = create_scratch_dir(name)?;
    fs::create_dir_all(dir.join("src"))
        .with_context(|| format!("failed to create {}", dir.join("src").display()))?;
    write_file(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )?;
    write_file(dir.join("src/lib.rs"), "pub fn noop() {}\n")?;

    Ok(dir)
}

pub fn create_feature_forwarding_workspace(name: &str) -> Result<PathBuf> {
    let dir = create_scratch_dir(name)?;
    write_file(
        dir.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = [
    "contracts/common-contract",
    "contracts/ephemeral-contract",
    "packages/helper",
]
"#,
    )?;
    write_contract_crate(
        &dir,
        "common-contract",
        r#"[features]
needs_flag = []
"#,
        r#"#[cfg(not(feature = "needs_flag"))]
compile_error!("enable needs_flag");

#[unsafe(no_mangle)]
pub extern "C" fn common_contract() -> u32 {
    1
}
"#,
    )?;
    write_contract_crate(
        &dir,
        "ephemeral-contract",
        r#"[features]
needs_flag = []

[dependencies]
helper = { path = "../../packages/helper", features = ["library"] }
"#,
        r#"#[cfg(not(feature = "needs_flag"))]
compile_error!("enable needs_flag");

#[unsafe(no_mangle)]
pub extern "C" fn ephemeral_contract() -> u32 {
    helper::helper_value()
}
"#,
    )?;
    write_file(
        dir.join("packages/helper/Cargo.toml"),
        r#"[package]
name = "helper"
version = "0.1.0"
edition = "2024"

[features]
library = []
"#,
    )?;
    write_file(
        dir.join("packages/helper/src/lib.rs"),
        r#"pub fn helper_value() -> u32 {
    7
}
"#,
    )?;

    Ok(dir)
}

pub fn create_all_features_workspace(name: &str) -> Result<PathBuf> {
    let dir = create_scratch_dir(name)?;
    write_file(
        dir.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = ["contracts/all-features-contract"]
"#,
    )?;
    write_contract_crate(
        &dir,
        "all-features-contract",
        r#"[features]
alpha = []
beta = []
"#,
        r#"#[cfg(not(all(feature = "alpha", feature = "beta")))]
compile_error!("enable every feature");

#[unsafe(no_mangle)]
pub extern "C" fn all_features_contract() -> u32 {
    2
}
"#,
    )?;

    Ok(dir)
}

pub fn create_no_default_features_workspace(name: &str) -> Result<PathBuf> {
    let dir = create_scratch_dir(name)?;
    write_file(
        dir.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = ["contracts/no-default-contract"]
"#,
    )?;
    write_contract_crate(
        &dir,
        "no-default-contract",
        r#"[features]
default = ["default_only"]
default_only = []
"#,
        r#"#[cfg(feature = "default_only")]
compile_error!("disable default features");

#[unsafe(no_mangle)]
pub extern "C" fn no_default_contract() -> u32 {
    3
}
"#,
    )?;

    Ok(dir)
}

fn write_contract_crate(
    workspace_dir: &Path,
    package_name: &str,
    manifest_extras: &str,
    lib_rs: &str,
) -> Result<()> {
    let crate_dir = workspace_dir.join("contracts").join(package_name);
    write_file(
        crate_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n{manifest_extras}"
        ),
    )?;
    write_file(crate_dir.join("src/lib.rs"), lib_rs)
}

fn write_file<P: AsRef<Path>, S: AsRef<[u8]>>(path: P, contents: S) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}
