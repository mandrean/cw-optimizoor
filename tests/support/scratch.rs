use std::{fs, path::PathBuf};

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
    fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )
    .with_context(|| format!("failed to write {}", dir.join("Cargo.toml").display()))?;
    fs::write(dir.join("src/lib.rs"), "pub fn noop() {}\n")
        .with_context(|| format!("failed to write {}", dir.join("src/lib.rs").display()))?;

    Ok(dir)
}
