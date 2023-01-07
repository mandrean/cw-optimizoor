use std::{cell::RefCell, env, path::PathBuf, thread};

use anyhow::Result;
use cargo::{
    core::{
        compiler::{BuildConfig, CompileKind, CompileMode, CompileTarget, MessageFormat},
        resolver::CliFeatures,
        Package, Workspace,
    },
    ops::{self, CompileFilter, CompileOptions},
    util::interning::InternedString,
    Config,
};
use lazy_static::lazy_static;

use crate::RunContext;

const RUSTFLAGS: &str = "RUSTFLAGS";
const PROFILE_RELEASE: &str = "release";
const TARGET_WASM32: &str = "wasm32-unknown-unknown";
lazy_static! {
    static ref KIND_WASM32: CompileKind =
        CompileKind::Target(CompileTarget::new(TARGET_WASM32).expect("couldn't create target"));
}

/// Compiles the workspace packages and returns the paths to the created WASM artifacts.
pub fn compile(
    ctx: &RunContext,
    cfg: &Config,
    ws: &Workspace,
    packages: ops::Packages,
) -> Result<Vec<PathBuf>> {
    let wasm_paths = ops::compile(ws, &compile_opts(&ctx, cfg, packages)?)?
        .cdylibs
        .into_iter()
        .filter(|o| o.unit.kind.eq(&KIND_WASM32))
        .map(|o| o.path)
        .collect::<Vec<PathBuf>>();

    Ok(wasm_paths)
}

/// Variant of [`compile()`](fn@compile) which compiles each package individually by using ephemeral workspaces.
pub fn compile_ephemerally(
    ctx: &RunContext,
    cfg: &Config,
    packages: Vec<Package>,
) -> Result<Vec<PathBuf>> {
    packages
        .into_iter()
        .map(|p| {
            (
                p.package_id().name().to_string(),
                Workspace::ephemeral(p, cfg, None, false),
            )
        })
        .try_fold(vec![], |mut acc, (package, ws)| {
            let mut res = compile(&ctx, cfg, &ws?, ops::Packages::Packages(vec![package]))?;
            acc.append(&mut res);
            anyhow::Ok(acc)
        })
}

/// Sets up the high-level compilation options.
pub fn compile_opts(
    ctx: &RunContext,
    config: &Config,
    spec: ops::Packages,
) -> Result<CompileOptions> {
    Ok(CompileOptions {
        build_config: build_cfg(config)?,
        cli_features: CliFeatures::from_command_line(
            &ctx.features,
            ctx.all_features,
            !ctx.no_default_features,
        )?,
        spec,
        filter: CompileFilter::lib_only(),
        target_rustdoc_args: None,
        target_rustc_args: None,
        target_rustc_crate_types: None,
        rustdoc_document_private_items: false,
        honor_rust_version: true,
    })
}

/// Creates the cargo config.
pub fn config() -> Result<Config> {
    // https://github.com/rust-lang/rust/issues/71757
    // https://github.com/rust-lang/cargo/pull/8246
    env::set_var(RUSTFLAGS, "-C strip=symbols");

    let cfg = Config::default()?;

    Ok(cfg)
}

/// Creates the rustc build config.
pub fn build_cfg(config: &Config) -> Result<BuildConfig> {
    let cfg = config.build_config()?;
    let requested_kinds =
        CompileKind::from_requested_targets(config, &[String::from(TARGET_WASM32)])?;

    let jobs: u32 = cfg
        .jobs
        .unwrap_or(thread::available_parallelism()?.get() as i32)
        .try_into()?;
    if jobs == 0 {
        anyhow::bail!("jobs may not be 0");
    }

    Ok(BuildConfig {
        requested_kinds,
        jobs,
        keep_going: false,
        requested_profile: InternedString::from(PROFILE_RELEASE),
        message_format: MessageFormat::Human,
        force_rebuild: false,
        build_plan: false,
        unit_graph: false,
        primary_unit_rustc: None,
        rustfix_diagnostic_server: RefCell::new(None),
        export_dir: None,
        future_incompat_report: false,
        timing_outputs: Vec::new(),

        mode: CompileMode::Build,
    })
}
