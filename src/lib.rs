use std::{fs, fs::File, io::Read, path::PathBuf};

use anyhow::{anyhow, Error};
use cargo::{
    core::Workspace,
    ops,
    util::{interning::InternedString, Filesystem},
};
use path_absolutize::Absolutize;

use crate::{compilation::*, ext::*, hashing::*, optimization::*};

pub mod compilation;
pub mod ext;
pub mod hashing;
pub mod optimization;

const TARGET: &str = "target";
const CONTRACTS: &str = "contracts";
const LIBRARY: &str = "library";
const ARTIFACTS: &str = "contracts";

/// Runs cw-optimizoor against the manifest path.
pub fn run(manifest_path: &PathBuf) -> anyhow::Result<(), Error> {
    let cfg = config()?;
    let ws = Workspace::new(manifest_path.as_ref(), &cfg).expect("couldn't create workspace");
    let output_dir = create_artifacts_dir(&ws)?;
    let shared_target_dir = Filesystem::new(ws.root().to_path_buf().join(TARGET));

    // all ws members that are contracts
    let all_contracts = ws
        .members()
        .filter(|&p| p.manifest_path().starts_with(&ws.root().join(CONTRACTS)))
        .collect::<Vec<_>>();

    if all_contracts.is_empty() {
        return Err(anyhow!("No CW contracts found. Exiting."));
    }

    // collect ws members with deps with feature = library to be compiled individually
    let individual_contracts = all_contracts
        .iter()
        .filter(|p| {
            p.dependencies()
                .iter()
                .any(|d| d.features().contains(&InternedString::from(LIBRARY)))
        })
        .map(|&p| p.clone())
        .collect::<Vec<_>>();

    // package names of contracts to be compiled individually
    let individual_names = individual_contracts
        .iter()
        .map(|p| p.package_id().name().to_string())
        .collect::<Vec<_>>();

    // package names of contracts to be compiled together
    let common_names = all_contracts
        .iter()
        .map(|p| p.package_id().name().to_string())
        .filter(|name| !individual_names.contains(name))
        .collect::<Vec<_>>();

    println!("🧐️  Compiling .../{}", &manifest_path.rtake(2).display());
    let mut intermediate_wasm_paths = compile(&cfg, &ws, ops::Packages::Packages(common_names))?;
    let mut special_intermediate_wasm_paths =
        compile_ephemerally(&cfg, Some(shared_target_dir), individual_contracts)?;
    intermediate_wasm_paths.append(&mut special_intermediate_wasm_paths);

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
    let output_dir = ws.root().absolutize()?.to_path_buf().join(ARTIFACTS);
    fs::create_dir_all(&output_dir)?;

    Ok(output_dir)
}
