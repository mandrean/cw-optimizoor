use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Error};
use cargo::{core::Workspace, ops, util::interning::InternedString};
use path_absolutize::Absolutize;

use crate::{compilation::*, ext::*, hashing::*, optimization::*};

pub mod compilation;
pub mod ext;
pub mod hashing;
pub mod optimization;
pub mod self_updater;

const CONTRACTS: &str = "contracts";
const LIBRARY: &str = "library";
const ARTIFACTS: &str = "artifacts";

/// Runs cw-optimizoor against the workspace path.
pub async fn run<P: AsRef<Path> + TakeExt<PathBuf>>(
    workspace_path: P,
) -> anyhow::Result<(), Error> {
    let manifest_path = find_manifest(&workspace_path)?;
    let cfg = config()?;
    let ws = Workspace::new(manifest_path.as_path(), &cfg).expect("couldn't create workspace");
    let output_dir = create_artifacts_dir(&ws)?;

    // Find all ws members that are contracts. If the workspace is virtual, only consider members
    // that are located in the 'contracts' directory. Otherwise, consider all members.
    let all_contracts = if ws.is_virtual() {
        ws.members()
            .filter(|&p| p.manifest_path().starts_with(ws.root().join(CONTRACTS)))
            .collect::<Vec<_>>()
    } else {
        ws.members().collect::<Vec<_>>()
    };

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
    let mut special_intermediate_wasm_paths = compile_ephemerally(&cfg, individual_contracts)?;
    intermediate_wasm_paths.append(&mut special_intermediate_wasm_paths);

    println!("🤓  Intermediate checksums:");
    let mut prev_intermediate_checksums = String::new();

    let checksums_intermediate_path = output_dir.join("checksums_intermediate.txt");
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&checksums_intermediate_path)
        .and_then(|mut file| file.read_to_string(&mut prev_intermediate_checksums))
        .context(format!(
            "Failed read from {path}",
            path = checksums_intermediate_path.display()
        ))?;
    write_checksums(&intermediate_wasm_paths, &checksums_intermediate_path).context(format!(
        "Failed write into {path}",
        path = checksums_intermediate_path.display()
    ))?;

    println!("🥸  Ahh I'm optimiziing");
    let final_wasm_paths = incremental_optimizations(
        &output_dir,
        intermediate_wasm_paths,
        prev_intermediate_checksums,
    )?;

    println!("🤓  Final checksums:");
    let checksums_path = output_dir.join("checksums.txt");
    write_checksums(&final_wasm_paths, &checksums_path).context(format!(
        "Failed write into {path}",
        path = checksums_path.display()
    ))?;

    println!(
        "🫡  Done. Saved optimized artifacts to:\n   {}",
        ws.root().join(ARTIFACTS).display()
    );

    Ok(())
}

/// Find the Cargo.toml if a directory path is passed in
pub fn find_manifest<P: AsRef<Path>>(workspace_path: P) -> anyhow::Result<PathBuf> {
    let manifest_path = match workspace_path.as_ref().absolutize()?.to_path_buf() {
        absolute_path if absolute_path.ends_with("Cargo.toml") => absolute_path,
        absolute_path if absolute_path.is_dir() => absolute_path.join("Cargo.toml"),
        absolute_path => {
            return Err(anyhow!(
                "Invalid workspace path: {}",
                absolute_path.display()
            ))
        }
    };

    if !manifest_path.exists() {
        return Err(anyhow!(
            "Couldn't locate manifest {}",
            manifest_path.display()
        ));
    }

    Ok(manifest_path)
}

/// Creates the artifacts dir if it doesn't exist.
fn create_artifacts_dir(ws: &Workspace) -> anyhow::Result<PathBuf> {
    let output_dir = ws.root().absolutize()?.to_path_buf().join(ARTIFACTS);
    fs::create_dir_all(&output_dir)?;

    Ok(output_dir)
}
