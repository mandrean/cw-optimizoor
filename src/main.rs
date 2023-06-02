use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use futures::TryFutureExt;
use semver::Version;

use cw_optimizoor::{self_updater, RunContext};

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
    workspace_path: Option<PathBuf>,

    /// Space or comma separated list of features to activate
    #[arg(short, long)]
    features: Option<String>,

    /// Activate all available features
    #[arg(long)]
    all_features: bool,

    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
}

trait RunContextExt {
    fn from_args(args: CwOptimizoor) -> RunContext;
}

impl RunContextExt for RunContext {
    fn from_args(args: CwOptimizoor) -> RunContext {
        RunContext {
            workspace_path: args
                .workspace_path
                .unwrap_or_else(|| env::current_dir().expect("couldn't get current directory")),
            features: args
                .features
                .map(|fs| {
                    fs.split([',', ' '])
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            all_features: args.all_features,
            no_default_features: args.no_default_features,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();
    let ctx = RunContext::from_args(args);

    let current_version = PKG_VERSION.parse::<Version>()?;
    let (latest_version, run_res) = tokio::join!(
        self_updater::fetch_latest_version(PKG_NAME).unwrap_or_else(|_| current_version.clone()),
        cw_optimizoor::run(ctx)
    );

    run_res?;

    self_updater::check_version(PKG_NAME, &current_version, &latest_version);

    Ok(())
}

#[cfg(test)]
mod tests {
    use cw_optimizoor::RunContext;
    use rstest::rstest;

    use crate::{CwOptimizoor, RunContextExt};

    #[rstest]
    #[case("test", vec![format!("test")])]
    #[case("test,test2,test3", vec![format!("test"), format!("test2"), format!("test3")])]
    #[case("test test2 test3", vec![format!("test"), format!("test2"), format!("test3")])]
    #[case("test,test2 test3", vec![format!("test"), format!("test2"), format!("test3")])]
    fn parses_features(#[case] given: String, #[case] expected: Vec<String>) {
        let args = CwOptimizoor {
            workspace_path: None,
            features: Some(given),
            all_features: false,
            no_default_features: false,
        };
        let ctx = RunContext::from_args(args);

        assert_eq!(expected, ctx.features)
    }
}
