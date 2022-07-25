use anyhow::Result;
use clap::Parser;
use std::env;

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

fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();

    let workspace_path = args
        .workspace_path
        .unwrap_or_else(|| env::current_dir().expect("couldn't get current directory"));

    cw_optimizoor::run(workspace_path)
}
