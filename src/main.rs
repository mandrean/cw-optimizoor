use anyhow::Result;

use clap::Parser;
use path_absolutize::Absolutize;

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
    /// Path to the Cargo.toml
    #[clap(value_parser)]
    manifest_path: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();

    let manifest_path = args
        .manifest_path
        .expect("missing manifest path")
        .absolutize()?
        .to_path_buf();

    cw_optimizoor::run(&manifest_path)
}
