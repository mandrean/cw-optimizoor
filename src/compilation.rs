use std::{cell::RefCell, env, path::PathBuf};

use anyhow::Result;
use cargo::{
    core::{
        compiler::{BuildConfig, CompileKind, CompileMode, CompileTarget, MessageFormat},
        resolver::CliFeatures,
        Workspace,
    },
    ops,
    ops::{CompileFilter, CompileOptions},
    util::interning::InternedString,
    Config,
};
use lazy_static::lazy_static;

const PROFILE_RELEASE: &str = "release";
const TARGET_WASM32: &str = "wasm32-unknown-unknown";
lazy_static! {
    static ref KIND_WASM32: CompileKind =
        CompileKind::Target(CompileTarget::new(TARGET_WASM32).expect("couldn't create target"));
}

/// Compiles the workspace and returns the created WASM artifacts.
pub fn compile(compile_opts: &CompileOptions, ws: &Workspace) -> Result<Vec<PathBuf>> {
    let wasm_paths = ops::compile(ws, compile_opts)?
        .cdylibs
        .into_iter()
        .filter(|o| o.unit.kind.eq(&KIND_WASM32))
        .map(|o| o.path)
        .collect::<Vec<PathBuf>>();

    Ok(wasm_paths)
}

/// Sets up the high-level compilation options.
pub fn compile_opts(config: &Config, spec: Option<ops::Packages>) -> Result<CompileOptions> {
    Ok(CompileOptions {
        build_config: build_cfg(config)?,
        cli_features: CliFeatures::new_all(false),
        spec: spec.unwrap_or(ops::Packages::Default),
        filter: CompileFilter::Default {
            required_features_filterable: false,
        },
        target_rustdoc_args: None,
        target_rustc_args: None,
        target_rustc_crate_types: None,
        local_rustdoc_args: None,
        rustdoc_document_private_items: false,
        honor_rust_version: true,
    })
}

/// Creates the cargo config.
pub fn config() -> Result<Config> {
    // https://github.com/rust-lang/rust/issues/71757
    // https://github.com/rust-lang/cargo/pull/8246
    env::set_var("RUSTFLAGS", "-C strip=symbols");

    let mut cfg = Config::default()?;
    cfg.configure(0, false, None, false, true, false, &None, &[], &[])?;

    Ok(cfg)
}

/// Creates the rustc build config.
pub fn build_cfg(config: &Config) -> Result<BuildConfig> {
    let cfg = config.build_config()?;
    let requested_kinds =
        CompileKind::from_requested_targets(config, &[String::from(TARGET_WASM32)])?;

    let jobs = cfg.jobs.unwrap_or(::num_cpus::get() as u32);
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
