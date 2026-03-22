use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use petname::petname;

const CW_PLUS_FIXTURE: &str = "tests/cw-plus";

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn prepare_workspace_copy(name: &str) -> Result<PathBuf> {
    let source = workspace_source(name)?;
    let destination = repo_root()
        .join("target/tests/workspaces")
        .join(unique_dir_name(name));

    copy_tree(&source, &destination)?;

    Ok(destination)
}

fn workspace_source(name: &str) -> Result<PathBuf> {
    let fixture_root = repo_root().join(CW_PLUS_FIXTURE);
    let source = match name {
        "cw-plus" => fixture_root,
        other => fixture_root.join(other),
    };

    if !source.join("Cargo.toml").exists() {
        bail!(
            "workspace fixture {} is unavailable at {}. Run `git submodule update --init --recursive` first.",
            name,
            source.display()
        );
    }

    Ok(source)
}

fn unique_dir_name(prefix: &str) -> String {
    format!("{prefix}-{}", petname(2, "-"))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", source.display()))?;
        let file_name = entry.file_name();
        if should_skip(&file_name) {
            continue;
        }

        let source_path = entry.path();
        let destination_path = destination.join(&file_name);
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", source_path.display()))?;

        if file_type.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn should_skip(file_name: &OsStr) -> bool {
    matches!(
        file_name,
        name if name == OsStr::new(".git")
            || name == OsStr::new("target")
            || name == OsStr::new("artifacts")
    )
}
