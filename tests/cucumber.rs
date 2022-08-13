use std::{
    env, fs, io,
    path::PathBuf,
    process::{Command, Stdio},
};

use assert_cmd::Command as AssertCommand;
use async_trait::async_trait;
use cucumber::{given, then, when, World, WorldInit};
use glob::glob;
use itertools::Itertools;
use path_absolutize::Absolutize;

const CARGO_CW_OPTIMIZOOR: &str = "cargo-cw-optimizoor";
const CW_OPTIMIZOOR: &str = "cw-optimizoor";

#[derive(Debug, WorldInit)]
pub struct CwWorld {
    ws_root: PathBuf,
    artifacts: Vec<PathBuf>,
}

#[async_trait(? Send)]
impl World for CwWorld {
    type Error = io::Error;

    async fn new() -> io::Result<CwWorld> {
        Ok(Self {
            ws_root: env::current_dir()?,
            artifacts: vec![],
        })
    }
}

#[given(expr = "the user is in the workspace {string}")]
async fn is_in_workspace(world: &mut CwWorld, ws: String) -> anyhow::Result<()> {
    world.ws_root = PathBuf::from("tests").join(PathBuf::from(ws));
    Ok(())
}

#[when(regex = r"the user runs cw-optimizoor\s?(for the first time|again)?")]
async fn runs_cw_optimizoor(world: &mut CwWorld, cond: String) -> anyhow::Result<()> {
    if !cond.is_empty() && cond.ne("again") {
        let artifacts = world.ws_root.join("artifacts");
        if artifacts.is_dir() {
            fs::remove_dir_all(artifacts)?;
        }

        let target = world.ws_root.join("target");
        if target.is_dir() {
            fs::remove_dir_all(target)?;
        }
    }

    let mut cmd = AssertCommand::cargo_bin(CARGO_CW_OPTIMIZOOR)?;
    cmd.current_dir(world.ws_root.as_path());
    cmd.arg(CW_OPTIMIZOOR);
    cmd.assert().success();

    Ok(())
}

#[then(expr = "{int} wasm files exist in the artifacts dir")]
async fn n_wasm_artficats(world: &mut CwWorld, n: usize) -> anyhow::Result<()> {
    let wasm_pattern = world
        .ws_root
        .as_path()
        .join("artifacts/*.wasm")
        .to_str()
        .unwrap_or_default()
        .to_string();
    world.artifacts = glob(&wasm_pattern)?.try_collect()?;

    assert_eq!(n, world.artifacts.len());
    Ok(())
}

#[then(expr = "each artifact contains a function named {string}")]
async fn each_artifact_contains_str_function(
    world: &mut CwWorld,
    name: String,
) -> anyhow::Result<()> {
    for entry in &world.artifacts {
        let path = format!("{}", entry.absolutize()?.to_path_buf().display());
        let decomp = Command::new("wasm-decompile")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .expect("missing decompilation output");

        let grep = Command::new("grep")
            .arg(format!("function {}", name))
            .stdin(decomp)
            .output()?;

        assert!(
            grep.status.success(),
            "couldn't find \"execute\" WASM function for {}",
            entry.display()
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    CwWorld::run("tests/features").await;
}
