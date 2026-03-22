#[path = "support/fixture_root.rs"]
mod fixture_root;
#[path = "support/scratch.rs"]
mod scratch;

use std::{ffi::OsStr, fs, path::Path};

use anyhow::Result;
use assert_cmd::Command as AssertCommand;
use predicates::prelude::predicate;

use cw_optimizoor::{Error, RunOptions};

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

    let runtime = runtime()?;
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
            "Usage: cargo cw-optimizoor [OPTIONS] [WORKSPACE_PATH]",
        ))
        .stdout(predicate::str::contains("[WORKSPACE_PATH]"))
        .stdout(predicate::str::contains("-F, --features <FEATURES>"))
        .stdout(predicate::str::contains("--all-features"))
        .stdout(predicate::str::contains("--no-default-features"))
        .stdout(predicate::str::contains(
            "Space or comma separated list of features to activate",
        ));

    Ok(())
}

#[test]
fn forwards_features_to_workspace_and_ephemeral_contract_builds() -> Result<()> {
    let dir = scratch::create_feature_forwarding_workspace("feature-forwarding")?;
    let runtime = runtime()?;

    let err = runtime.block_on(cw_optimizoor::run(&dir)).unwrap_err();
    assert!(matches!(err, Error::CompileWorkspace { .. }));

    runtime.block_on(cw_optimizoor::run_with_options(RunOptions {
        workspace_path: dir.clone(),
        features: vec![String::from("needs_flag")],
        all_features: false,
        no_default_features: false,
    }))?;
    assert_optimized_wasm_exists(&dir)?;

    Ok(())
}

#[test]
fn forwards_all_features_to_contract_builds() -> Result<()> {
    let dir = scratch::create_all_features_workspace("all-features")?;
    let runtime = runtime()?;

    let err = runtime.block_on(cw_optimizoor::run(&dir)).unwrap_err();
    assert!(matches!(err, Error::CompileWorkspace { .. }));

    runtime.block_on(cw_optimizoor::run_with_options(RunOptions {
        workspace_path: dir.clone(),
        features: Vec::new(),
        all_features: true,
        no_default_features: false,
    }))?;
    assert_optimized_wasm_exists(&dir)?;

    Ok(())
}

#[test]
fn forwards_no_default_features_to_contract_builds() -> Result<()> {
    let dir = scratch::create_no_default_features_workspace("no-default-features")?;
    let runtime = runtime()?;

    let err = runtime.block_on(cw_optimizoor::run(&dir)).unwrap_err();
    assert!(matches!(err, Error::CompileWorkspace { .. }));

    runtime.block_on(cw_optimizoor::run_with_options(RunOptions {
        workspace_path: dir.clone(),
        features: Vec::new(),
        all_features: false,
        no_default_features: true,
    }))?;
    assert_optimized_wasm_exists(&dir)?;

    Ok(())
}

#[test]
fn reports_invalid_feature_selection() -> Result<()> {
    let dir = scratch::create_feature_forwarding_workspace("invalid-feature-selection")?;
    let runtime = runtime()?;

    let err = runtime
        .block_on(cw_optimizoor::run_with_options(RunOptions {
            workspace_path: dir,
            features: vec![String::from("dep:helper")],
            all_features: false,
            no_default_features: false,
        }))
        .unwrap_err();
    match err {
        Error::FeatureSelection { source } => {
            assert!(source.to_string().contains("explicit `dep:` syntax"));
        }
        other => panic!("expected FeatureSelection, got {other:?}"),
    }

    Ok(())
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Into::into)
}

fn assert_optimized_wasm_exists(workspace_dir: &Path) -> Result<()> {
    let artifacts_dir = workspace_dir.join("artifacts");
    assert!(artifacts_dir.join("checksums.txt").exists());
    assert!(
        fs::read_dir(&artifacts_dir)?
            .flatten()
            .any(|entry| { entry.path().extension() == Some(OsStr::new("wasm")) })
    );

    Ok(())
}
