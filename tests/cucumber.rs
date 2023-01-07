use std::{
    env, fmt, fs,
    io::{Error as IoError, Read, Write},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use assert_cmd::{assert::OutputAssertExt, cargo::CargoError, Command as AssertCommand};
use cucumber::{given, then, when, writer, World};
use glob::glob;
use itertools::Itertools;
use path_absolutize::Absolutize;
use petname::petname;
use predicates::prelude::predicate;
use regex::Regex;
use thiserror::Error;

const CARGO_CW_OPTIMIZOOR: &str = "cargo-cw-optimizoor";
const CW_OPTIMIZOOR: &str = "cw-optimizoor";

#[derive(Debug, Clone, World)]
#[world(init = Self::new)]
pub struct CwWorld {
    ws_root: PathBuf,
    cmd_output: Option<Output>,
    artifacts: Vec<PathBuf>,
}

impl CwWorld {
    async fn new() -> Result<CwWorld, CwWorldError> {
        Ok(Self {
            ws_root: env::current_dir()?,
            cmd_output: None,
            artifacts: vec![],
        })
    }
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

const MIGRATE_REGEX: &str = r"pub fn migrate\(\s*(?:mut\s+)?deps: DepsMut, (?:_env:|env:)\s*Env, (?:_msg:|msg:)\s*(?:Empty|MigrateMsg)\s*\) -> Result<Response, ContractError> \{";

#[given(expr = "the user makes a change in the {string} contract")]
async fn makes_a_change_in_contract(world: &mut CwWorld, name: String) -> anyhow::Result<()> {
    let filename = world
        .ws_root
        .join("contracts")
        .join(name)
        .join("src/contract.rs");
    let mut file = fs::OpenOptions::new().read(true).open(&filename)?;
    let mut contract = String::new();
    file.read_to_string(&mut contract)?;

    let petname = petname(1, "");
    let change = format!(
        "\n    if \"{}\" == \"{}\" {{ panic!() }}",
        petname,
        petname
    );
    let replaced = Regex::new(MIGRATE_REGEX)?
        .replace_all(&contract, format!("$0{}", change))
        .to_string();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&filename)?;
    file.write(replaced.as_bytes())?;

    Ok(())
}

#[given(expr = "the user deletes the artifact {string}")]
async fn deletes_str_artifact(world: &mut CwWorld, name: String) -> anyhow::Result<()> {
    let wasm_pattern = world
        .ws_root
        .as_path()
        .join(format!("artifacts/{}*.wasm", &name))
        .to_str()
        .unwrap_or_default()
        .to_string();
    let matches: Vec<PathBuf> = glob(&wasm_pattern)?.try_collect()?;
    match matches.first() {
        Some(path) => fs::remove_file(path)?,
        None => panic!("couldn't find any artifact matching \"{}\"", name),
    }
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

#[then(expr = "{int} contracts are optimized")]
async fn n_optimizations(world: &mut CwWorld, n: usize) -> anyhow::Result<()> {
    // TODO: verify that the checksum(s) didn't change
    world
        .cmd_output
        .as_ref()
        .expect("missing cmd output")
        .clone()
        .assert()
        .stdout(predicate::str::contains("was optimized").count(n));

    Ok(())
}

#[then(expr = "{int} contracts are unchanged and skipped")]
async fn n_optimizations_cached(world: &mut CwWorld, n: usize) -> anyhow::Result<()> {
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

#[then(expr = "{string} is reoptimized")]
async fn str_is_reoptimized(world: &mut CwWorld, name: String) -> anyhow::Result<()> {
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

#[tokio::main]
async fn main() {
    CwWorld::cucumber()
        .with_writer(writer::Libtest::or_basic())
        .max_concurrent_scenarios(1)
        .run("tests/features")
        .await;
}
