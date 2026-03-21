#[path = "support/fixture_root.rs"]
mod fixture_root;
#[path = "support/scratch.rs"]
mod scratch;

use std::{env, fs, path::PathBuf};

use anyhow::Result;
use assert_cmd::Command as AssertCommand;
use path_absolutize::Absolutize;
use predicates::prelude::{PredicateBooleanExt, predicate};

use cw_optimizoor::Error;

#[test]
fn cw_plus_fixture_is_initialized() -> Result<()> {
    let fixture = fixture_root::fixture_source_root()?;
    assert!(fixture.join("Cargo.toml").exists());
    Ok(())
}

#[test]
fn finds_manifest() -> Result<()> {
    let path = cw_optimizoor::find_manifest(fixture_root::fixture_source_root()?)?;
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));

    let previous_dir = CurrentDirGuard::change_to(fixture_root::fixture_source_root()?)?;
    let path = cw_optimizoor::find_manifest(PathBuf::from("."))?;
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));
    drop(previous_dir);

    Ok(())
}

#[test]
fn reports_missing_manifest_for_workspace_dir_without_manifest() -> Result<()> {
    let dir = scratch::create_scratch_dir("missing-manifest")?;
    let err = cw_optimizoor::find_manifest(&dir).unwrap_err();
    match err {
        Error::MissingManifest { path } => assert_eq!(path, dir.join("Cargo.toml")),
        other => panic!("expected MissingManifest, got {other:?}"),
    }

    Ok(())
}

#[test]
fn reports_invalid_workspace_path_for_non_manifest_file() -> Result<()> {
    let dir = scratch::create_scratch_dir("invalid-workspace-path")?;
    let invalid_path = dir.join("workspace.txt");
    fs::write(&invalid_path, "not a manifest")?;

    let err = cw_optimizoor::find_manifest(&invalid_path).unwrap_err();
    match err {
        Error::InvalidWorkspacePath { path } => assert!(path.ends_with("workspace.txt")),
        other => panic!("expected InvalidWorkspacePath, got {other:?}"),
    }

    Ok(())
}

#[test]
fn reports_no_contracts_for_non_contract_workspace() -> Result<()> {
    let dir = scratch::create_scratch_dir("no-contracts")?;
    fs::create_dir_all(dir.join("src"))?;
    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "no-contracts"
version = "0.1.0"
edition = "2024"
"#,
    )?;
    fs::write(dir.join("src/lib.rs"), "pub fn noop() {}\n")?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let err = runtime.block_on(cw_optimizoor::run(&dir)).unwrap_err();
    match err {
        Error::NoContracts { contracts_dir } => assert_eq!(contracts_dir, dir.join("contracts")),
        other => panic!("expected NoContracts, got {other:?}"),
    }

    Ok(())
}

#[test]
fn cli_help_matches_supported_interface() -> Result<()> {
    let mut cmd = AssertCommand::cargo_bin("cargo-cw-optimizoor")?;
    cmd.arg("cw-optimizoor").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Usage: cargo cw-optimizoor [WORKSPACE_PATH]",
        ))
        .stdout(predicate::str::contains("[WORKSPACE_PATH]"))
        .stdout(predicate::str::contains("--all-features").not());

    Ok(())
}

struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: PathBuf) -> Result<Self> {
        let previous = env::current_dir()?;
        env::set_current_dir(path.absolutize()?.as_ref())?;
        Ok(Self { previous })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.previous);
    }
}
