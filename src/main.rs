use std::env;

use anyhow::{Context, Result};
use clap::{Args, Parser};
use semver::Version;

use cw_optimizoor::self_updater;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// cw-optimizoor
#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(about = "CosmWasm optimizer", long_about = None)]
enum Cargo {
    CwOptimizoor(CwOptimizoor),
}

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
struct CwOptimizoor {
    /// Path to the workspace dir or Cargo.toml
    #[arg(value_parser)]
    workspace_path: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();

    let workspace_path = match args.workspace_path {
        Some(path) => path,
        None => env::current_dir().context("failed to resolve the current directory")?,
    };

    let current_version = Version::parse(PKG_VERSION).context("failed to parse package version")?;
    let update_check =
        tokio::spawn(async { self_updater::fetch_latest_version(PKG_NAME).await.ok() });

    cw_optimizoor::run(workspace_path).await?;

    if update_check.is_finished() {
        if let Ok(Some(latest_version)) = update_check.await {
            self_updater::check_version(PKG_NAME, &current_version, &latest_version);
        }
    }

    Ok(())
}
