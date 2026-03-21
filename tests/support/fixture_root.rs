use std::path::PathBuf;

use anyhow::{Result, bail};

const CW_PLUS_FIXTURE: &str = "tests/cw-plus";

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixture_source_root() -> Result<PathBuf> {
    let root = repo_root().join(CW_PLUS_FIXTURE);
    let manifest = root.join("Cargo.toml");
    if !manifest.exists() {
        bail!(
            "cw-plus fixture is unavailable at {}. Run `git submodule update --init --recursive` first.",
            manifest.display()
        );
    }

    Ok(root)
}
