use std::{fs, fs::File, io::Read, path::PathBuf};

use anyhow::Error;
use cargo::{core::Workspace, ops};
use path_absolutize::Absolutize;

use crate::{compilation::*, ext::*, hashing::*, optimization::*};

pub mod compilation;
pub mod ext;
pub mod hashing;
pub mod optimization;

pub fn run(manifest_path: &PathBuf) -> anyhow::Result<(), Error> {
    let cfg = config()?;
    let ws = Workspace::new(manifest_path.as_ref(), &cfg).expect("couldn't create workspace");
    let contracts = ws
        .members()
        .filter(|&p| p.manifest_path().starts_with(&ws.root().join("contracts")))
        .map(|p| p.package_id().name().to_string())
        .collect::<Vec<String>>();
    let compile_opts = compile_opts(&cfg, ops::Packages::Packages(contracts))?;
    let output_dir = create_artifacts_dir(&ws)?;

    println!("🧐️  Compiling .../{}", &manifest_path.rtake(2).display());
    let intermediate_wasm_paths = compile(&compile_opts, &ws)?;
    println!("🤓  Intermediate checksums:");
    let mut prev_intermediate_checksums = String::new();
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&output_dir.join("checksums_intermediate.txt"))?
        .read_to_string(&mut prev_intermediate_checksums)?;
    write_checksums(
        &intermediate_wasm_paths,
        &output_dir.join("checksums_intermediate.txt"),
    )?;

    println!("🥸  Ahh I'm optimiziing");
    let final_wasm_paths = incremental_optimizations(
        &output_dir,
        intermediate_wasm_paths,
        prev_intermediate_checksums,
    )?;

    println!("🤓  Final checksums:");
    write_checksums(&final_wasm_paths, &output_dir.join("checksums.txt"))?;

    println!(
        "🫡  Done. Saved optimized artifacts to:\n   {}/artifacts",
        ws.root().display()
    );

    Ok(())
}

/// Creates the artifacts dir if it doesn't exist.
fn create_artifacts_dir(ws: &Workspace) -> anyhow::Result<PathBuf> {
    let output_dir = ws.root().absolutize()?.to_path_buf().join("artifacts");
    fs::create_dir_all(&output_dir)?;

    Ok(output_dir)
}
