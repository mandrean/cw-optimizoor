#[path = "support/fixture_root.rs"]
mod fixture_root;
#[path = "support/scratch.rs"]
mod scratch;

use std::fs;

use anyhow::Result;
use assert_cmd::Command as AssertCommand;
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
    let dir = scratch::create_no_contract_workspace("no-contracts-lib")?;

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
fn cli_uses_current_dir_when_workspace_path_is_omitted() -> Result<()> {
    let dir = scratch::create_no_contract_workspace("no-contracts-cli")?;

    let mut cmd = AssertCommand::cargo_bin("cargo-cw-optimizoor")?;
    cmd.current_dir(&dir);
    cmd.arg("cw-optimizoor");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "no CosmWasm contracts found under",
        ))
        .stderr(predicate::str::contains(
            dir.join("contracts").display().to_string(),
        ));

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
