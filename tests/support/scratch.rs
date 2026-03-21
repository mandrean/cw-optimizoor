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
