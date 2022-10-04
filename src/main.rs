use std::env;

use anyhow::Result;
use clap::Parser;
use futures::TryFutureExt;
use semver::Version;

use cw_optimizoor::self_updater;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// cw-optimizoor
#[derive(Debug, Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
#[clap(about = "CosmWasm optimizer", long_about = None)]
enum Cargo {
    CwOptimizoor(CwOptimizoor),
}

#[derive(clap::Args, Debug)]
#[clap(author, version, about, long_about = None)]
struct CwOptimizoor {
    /// Path to the workspace dir or Cargo.toml
    #[clap(value_parser)]
    workspace_path: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();

    let workspace_path = args
        .workspace_path
        .unwrap_or_else(|| env::current_dir().expect("couldn't get current directory"));

    let current_version = PKG_VERSION.parse::<Version>()?;
    let (latest_version, run_res) = tokio::join!(
        self_updater::fetch_latest_version(PKG_NAME).unwrap_or_else(|_| current_version.clone()),
        cw_optimizoor::run(workspace_path)
    );

    run_res?;

    self_updater::check_version(PKG_NAME, &current_version, &latest_version);

    Ok(())
}
