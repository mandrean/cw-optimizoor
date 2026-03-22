use std::path::PathBuf;

use cargo::{
    GlobalContext,
    core::{
        Package, Workspace,
        compiler::{CompileKind, CompileMode, CompileTarget},
        resolver::CliFeatures,
    },
    ops::{self, CompileFilter, CompileOptions},
    util::interning::InternedString,
};
use lazy_static::lazy_static;

const PROFILE_RELEASE: &str = "release";
const TARGET_WASM32: &str = "wasm32-unknown-unknown";
lazy_static! {
    static ref KIND_WASM32: CompileKind =
        CompileKind::Target(CompileTarget::new(TARGET_WASM32).expect("couldn't create target"));
}

use crate::error::{Error, Result};

/// Compiles the workspace packages and returns the paths to the created WASM artifacts.
pub fn compile(
    cfg: &GlobalContext,
    ws: &Workspace,
    packages: ops::Packages,
) -> Result<Vec<PathBuf>> {
    let wasm_paths = ops::compile(ws, &compile_opts(cfg, packages)?)
        .map_err(|source| Error::CompileWorkspace { source })?
        .cdylibs
        .into_iter()
        .filter(|o| o.unit.kind.eq(&KIND_WASM32))
        .map(|o| o.path)
        .collect::<Vec<PathBuf>>();

    Ok(wasm_paths)
}

/// Variant of [`compile()`](fn@compile) which compiles each package individually by using ephemeral workspaces.
pub fn compile_ephemerally(cfg: &GlobalContext, packages: Vec<Package>) -> Result<Vec<PathBuf>> {
    packages.into_iter().try_fold(vec![], |mut acc, package| {
        let package_name = package.package_id().name().to_string();
        let ws = Workspace::ephemeral(package, cfg, None, false).map_err(|source| {
            Error::EphemeralWorkspace {
                package: package_name.clone(),
                source,
            }
        })?;
        let mut res = compile(cfg, &ws, ops::Packages::Packages(vec![package_name]))?;
        acc.append(&mut res);
        Ok(acc)
    })
}

/// Sets up the high-level compilation options.
pub fn compile_opts(config: &GlobalContext, spec: ops::Packages) -> Result<CompileOptions> {
    let mut options = CompileOptions::new(config, CompileMode::Build)
        .map_err(|source| Error::BuildConfiguration { source })?;
    options.build_config.requested_profile = InternedString::from(PROFILE_RELEASE);
    options.build_config.requested_kinds =
        CompileKind::from_requested_targets(config, &[String::from(TARGET_WASM32)])
            .map_err(|source| Error::BuildConfiguration { source })?;
    options.cli_features = CliFeatures::new_all(false);
    options.spec = spec;
    options.filter = CompileFilter::lib_only();
    options.honor_rust_version = Some(true);
    Ok(options)
}

/// Creates the cargo config.
pub fn config() -> Result<GlobalContext> {
    let mut config = GlobalContext::default().map_err(|source| Error::CargoConfig { source })?;
    let cli_config = [String::from("profile.release.strip=\"symbols\"")];
    config
        .configure(0, false, None, false, false, false, &None, &[], &cli_config)
        .map_err(|source| Error::CargoConfig { source })?;

    Ok(config)
}
