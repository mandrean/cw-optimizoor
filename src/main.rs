use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser};
use semver::Version;

use cw_optimizoor::{RunOptions, self_updater};

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
    workspace_path: Option<PathBuf>,

    /// Space or comma separated list of features to activate
    #[arg(short = 'F', long, value_name = "FEATURES")]
    features: Vec<String>,

    /// Activate all available features
    #[arg(long)]
    all_features: bool,

    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
}

impl CwOptimizoor {
    fn into_run_options(self) -> Result<RunOptions> {
        Ok(RunOptions {
            workspace_path: match self.workspace_path {
                Some(path) => path,
                None => env::current_dir().context("failed to resolve the current directory")?,
            },
            features: parse_features(self.features),
            all_features: self.all_features,
            no_default_features: self.no_default_features,
        })
    }
}

fn parse_features(features: Vec<String>) -> Vec<String> {
    features
        .into_iter()
        .flat_map(|feature_group| {
            feature_group
                .split_whitespace()
                .flat_map(|feature_group| feature_group.split(','))
                .filter(|feature| !feature.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cargo::CwOptimizoor(args) = Cargo::parse();
    let run_options = args.into_run_options()?;

    let current_version = Version::parse(PKG_VERSION).context("failed to parse package version")?;
    std::mem::drop(tokio::spawn({
        let current_version = current_version.clone();
        async move {
            // Best-effort only: this background check must never block the optimizer.
            if let Ok(latest_version) = self_updater::fetch_latest_version(PKG_NAME).await {
                self_updater::check_version(PKG_NAME, &current_version, &latest_version);
            }
        }
    }));

    cw_optimizoor::run_with_options(run_options).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Cargo, parse_features};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn normalizes_feature_lists_from_cli_arguments() {
        assert_eq!(
            parse_features(vec![
                String::from("feature_a,feature_b"),
                String::from("feature_c feature_d"),
                String::from("feature_e"),
            ]),
            vec![
                String::from("feature_a"),
                String::from("feature_b"),
                String::from("feature_c"),
                String::from("feature_d"),
                String::from("feature_e"),
            ]
        );
    }

    #[test]
    fn parses_cargo_style_feature_flags() {
        let Cargo::CwOptimizoor(args) = Cargo::try_parse_from([
            "cargo",
            "cw-optimizoor",
            "-F",
            "feature_a,feature_b",
            "-F",
            "feature_c feature_d",
            "--all-features",
            "--no-default-features",
            ".",
        ])
        .expect("expected clap to parse CLI arguments");
        let run_options = args
            .into_run_options()
            .expect("expected CLI arguments to build run options");

        assert_eq!(
            run_options.features,
            vec![
                String::from("feature_a"),
                String::from("feature_b"),
                String::from("feature_c"),
                String::from("feature_d"),
            ]
        );
        assert!(run_options.all_features);
        assert!(run_options.no_default_features);
        assert_eq!(run_options.workspace_path, PathBuf::from("."));
    }
}
