use std::{
    fs,
    path::{Path, PathBuf},
};

use cargo::{core::Workspace, ops, util::interning::InternedString};
use path_absolutize::Absolutize;

use crate::{
    compilation::*,
    ext::TakeExt,
    hashing::{read_checksums, write_checksums},
    optimization::*,
};

pub mod compilation;
pub mod error;
pub mod ext;
pub mod hashing;
pub mod optimization;
pub mod self_updater;

pub use error::{Error, Result};

const CONTRACTS: &str = "contracts";
const LIBRARY: &str = "library";
const ARTIFACTS: &str = "artifacts";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOptions {
    pub workspace_path: PathBuf,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
}

impl RunOptions {
    pub fn new<P: AsRef<Path>>(workspace_path: P) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
            features: Vec::new(),
            all_features: false,
            no_default_features: false,
        }
    }
}

/// Runs cw-optimizoor against the workspace path.
///
/// ```rust,no_run
/// use cw_optimizoor::run;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<(), cw_optimizoor::Error> {
///     run(".").await?;
///     Ok(())
/// }
/// ```
pub async fn run<P: AsRef<Path>>(workspace_path: P) -> Result<()> {
    run_with_options(RunOptions::new(workspace_path)).await
}

/// Runs cw-optimizoor against the provided workspace path and Cargo feature options.
pub async fn run_with_options(run_options: RunOptions) -> Result<()> {
    let manifest_path = find_manifest(&run_options.workspace_path)?;
    let cfg = config()?;
    let ws =
        Workspace::new(manifest_path.as_path(), &cfg).map_err(|source| Error::WorkspaceInit {
            manifest_path: manifest_path.clone(),
            source,
        })?;
    let output_dir = create_artifacts_dir(&ws)?;

    // all ws members that are contracts
    let all_contracts = ws
        .members()
        .filter(|&p| p.manifest_path().starts_with(ws.root().join(CONTRACTS)))
        .collect::<Vec<_>>();

    if all_contracts.is_empty() {
        return Err(Error::NoContracts {
            contracts_dir: ws.root().join(CONTRACTS),
        });
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
    let mut intermediate_wasm_paths = if common_names.is_empty() {
        Vec::new()
    } else {
        compile(
            &run_options,
            &cfg,
            &ws,
            ops::Packages::Packages(common_names),
        )?
    };
    let mut special_intermediate_wasm_paths =
        compile_ephemerally(&run_options, &cfg, individual_contracts)?;
    intermediate_wasm_paths.append(&mut special_intermediate_wasm_paths);

    println!("🤓  Intermediate checksums:");
    let checksums_intermediate_path = output_dir.join("checksums_intermediate.txt");
    let prev_intermediate_checksums = read_checksums(&checksums_intermediate_path)?;
    write_checksums(&intermediate_wasm_paths, &checksums_intermediate_path)?;

    println!("🥸  Ahh I'm optimiziing");
    let final_wasm_paths = incremental_optimizations(
        &output_dir,
        intermediate_wasm_paths,
        &prev_intermediate_checksums,
    )?;

    println!("🤓  Final checksums:");
    let checksums_path = output_dir.join("checksums.txt");
    write_checksums(&final_wasm_paths, &checksums_path)?;

    println!(
        "🫡  Done. Saved optimized artifacts to:\n   {}",
        ws.root().join(ARTIFACTS).display()
    );

    Ok(())
}

/// Find the Cargo.toml if a directory path is passed in
pub fn find_manifest<P: AsRef<Path>>(workspace_path: P) -> Result<PathBuf> {
    let requested_path = workspace_path.as_ref().to_path_buf();
    let manifest_path = match workspace_path
        .as_ref()
        .absolutize()
        .map_err(|source| Error::ResolveWorkspacePath {
            path: requested_path,
            source,
        })?
        .to_path_buf()
    {
        absolute_path if absolute_path.ends_with("Cargo.toml") => absolute_path,
        absolute_path if absolute_path.is_dir() => absolute_path.join("Cargo.toml"),
        absolute_path => {
            return Err(Error::InvalidWorkspacePath {
                path: absolute_path,
            });
        }
    };

    if !manifest_path.exists() {
        return Err(Error::MissingManifest {
            path: manifest_path,
        });
    }

    Ok(manifest_path)
}

/// Creates the artifacts dir if it doesn't exist.
fn create_artifacts_dir(ws: &Workspace) -> Result<PathBuf> {
    let workspace_root = ws.root().to_path_buf();
    let output_dir = ws
        .root()
        .absolutize()
        .map_err(|source| Error::ResolveWorkspacePath {
            path: workspace_root,
            source,
        })?
        .to_path_buf()
        .join(ARTIFACTS);
    fs::create_dir_all(&output_dir).map_err(|source| Error::CreateArtifactsDir {
        path: output_dir.clone(),
        source,
    })?;

    Ok(output_dir)
}
