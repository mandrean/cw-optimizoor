use std::{
    env, fmt, fs,
    io::{Error as IoError, Write},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use assert_cmd::{assert::OutputAssertExt, cargo::CargoError, Command as AssertCommand};
use async_trait::async_trait;
use cucumber::{given, then, when, World, WorldInit};
use glob::glob;
use itertools::Itertools;
use path_absolutize::Absolutize;
use petname::petname;
use predicates::prelude::predicate;
use thiserror::Error;

const CARGO_CW_OPTIMIZOOR: &str = "cargo-cw-optimizoor";
const CW_OPTIMIZOOR: &str = "cw-optimizoor";

#[derive(Debug, Clone, WorldInit)]
pub struct CwWorld {
    ws_root: PathBuf,
    cmd_output: Option<Output>,
    artifacts: Vec<PathBuf>,
}

#[derive(Error, Debug)]
pub enum CwWorldError {
    Io(IoError),
    Cargo(CargoError),
}

impl From<IoError> for CwWorldError {
    fn from(source: IoError) -> Self {
        CwWorldError::Io(source)
    }
}

impl From<CargoError> for CwWorldError {
    fn from(source: CargoError) -> Self {
        CwWorldError::Cargo(source)
    }
}

impl fmt::Display for CwWorldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait(? Send)]
impl World for CwWorld {
    type Error = CwWorldError;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self {
            ws_root: env::current_dir()?,
            cmd_output: None,
            artifacts: vec![],
        })
    }
}

#[given(expr = "the user is in the workspace {string}")]
async fn is_in_workspace(world: &mut CwWorld, ws: String) -> anyhow::Result<()> {
    world.ws_root = PathBuf::from("tests").join(PathBuf::from(ws));
    Ok(())
}

#[when(
    regex = r"the user\s?(successfully|unsuccessfully)? runs cw-optimizoor\s?(for the first time|again)?"
)]
async fn runs_cw_optimizoor(
    world: &mut CwWorld,
    result: String,
    cond: String,
) -> anyhow::Result<()> {
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

    let assert = match result.as_str() {
        "successfully" => cmd.assert().success(),
        "unsuccessfully" => cmd.assert().failure(),
        _ => unreachable!(),
    };
    world.cmd_output = Some(assert.get_output().clone());

    Ok(())
}

#[given(expr = "the user makes a change in the {string} contract")]
async fn makes_a_change_in_contract(world: &mut CwWorld, name: String) -> anyhow::Result<()> {
    let filename = world
        .ws_root
        .join("contracts")
        .join(name)
        .join("src/lib.rs");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)?;

    let change = format!("pub fn {}() {{ assert!(true) }}", petname(1, ""));
    writeln!(file, "{}", change)?;

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

#[then(expr = "only {string} is reoptimized")]
async fn only_str_is_reoptimized(world: &mut CwWorld, name: String) -> anyhow::Result<()> {
    // TODO: verify that the checksum changed
    world
        .cmd_output
        .as_ref()
        .expect("missing cmd output")
        .clone()
        .assert()
        .stdout(predicate::str::contains(format!("{} was optimized", name)));

    Ok(())
}

#[then(expr = "the other {int} are skipped")]
async fn other_n_skipped(world: &mut CwWorld, n: usize) -> anyhow::Result<()> {
    // TODO: verify that the checksum(s) didn't change
    world
        .cmd_output
        .as_ref()
        .expect("missing cmd output")
        .clone()
        .assert()
        .stdout(predicate::str::contains("is unchanged. Skipping").count(n));

    Ok(())
}

#[tokio::main]
async fn main() {
    CwWorld::run("tests/features").await;
}
