use std::{fs, path::PathBuf};

use anyhow::Result;
use cargo::{core::Workspace, ops};
use clap::Parser;
use path_absolutize::Absolutize;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use cw_optimizoor::{compilation::*, ext::*, optimization::*};

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

    let cfg = config()?;
    let ws = Workspace::new(manifest_path.as_ref(), &cfg).expect("couldn't create workspace");
    let compile_opts = compile_opts(&cfg, Some(ops::Packages::Default))?;
    let output_dir = create_artifacts_dir(&ws)?;

    println!("🧐️  Compiling ...{}", manifest_path.display());
    let wasm_paths = compile(&compile_opts, &ws)?;

    println!("🥸  Ahh I'm optimiziing");
    wasm_paths.into_par_iter().try_for_each(|wasm_path| {
        println!(
            "    ...{}",
            wasm_path.to_string_lossy().to_string().rtake(70)
        );
        let output_path = optimized_output_path(&wasm_path, &output_dir)?;
        optimize(&wasm_path, &output_path)?;
        anyhow::Ok(())
    })?;
    println!(
        "🫡  Done. Saved optimized artifacts to {}/artifacts",
        ws.root().display()
    );

    Ok(())
}

/// Creates the artifacts dir if it doesn't exist.
fn create_artifacts_dir(ws: &Workspace) -> Result<PathBuf> {
    let mut output_dir = ws.root().absolutize()?.to_path_buf();
    output_dir.push("artifacts");
    fs::create_dir_all(&output_dir)?;
    Ok(output_dir)
}
