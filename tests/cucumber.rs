#[path = "support/workspace_copy.rs"]
mod workspace_copy;

use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use anyhow::{Context, Result};
use assert_cmd::{Command as AssertCommand, assert::OutputAssertExt};
use cucumber::{World, given, then, when, writer};
use glob::glob;
use path_absolutize::Absolutize;
use petname::petname;
use predicates::prelude::predicate;
use regex::Regex;

const CARGO_CW_OPTIMIZOOR: &str = "cargo-cw-optimizoor";
const CW_OPTIMIZOOR: &str = "cw-optimizoor";

#[derive(Debug, Clone, World)]
#[world(init = Self::new)]
pub struct CwWorld {
    ws_root: PathBuf,
    cmd_output: Option<Output>,
    artifacts: Vec<PathBuf>,
    primed: bool,
}

impl CwWorld {
    async fn new() -> Result<CwWorld> {
        Ok(Self {
            ws_root: workspace_copy::repo_root(),
            cmd_output: None,
            artifacts: vec![],
            primed: false,
        })
    }
}

#[given(expr = "the user is in the workspace {string}")]
async fn is_in_workspace(world: &mut CwWorld, ws: String) -> Result<()> {
    world.ws_root = workspace_copy::prepare_workspace_copy(&ws)?;
    world.cmd_output = None;
    world.artifacts.clear();
    world.primed = false;
    Ok(())
}

#[given(
    regex = r"the user\s?(successfully|unsuccessfully)? runs cw-optimizoor\s?(for the first time|again)?"
)]
#[when(
    regex = r"the user\s?(successfully|unsuccessfully)? runs cw-optimizoor\s?(for the first time|again)?"
)]
async fn runs_cw_optimizoor(world: &mut CwWorld, result: String, cond: String) -> Result<()> {
    if cond == "again" {
        prime_workspace(world).await?;
    }

    let outcome = if result.is_empty() {
        "successfully"
    } else {
        result.as_str()
    };
    world.cmd_output = Some(run_cw_optimizoor(&world.ws_root, outcome)?);
    world.primed = true;

    Ok(())
}

const MIGRATE_REGEX: &str = r"pub fn migrate\(\s*(?:mut\s+)?deps: DepsMut, (?:_env:|env:)\s*Env, (?:_msg:|msg:)\s*(?:Empty|MigrateMsg)\s*\) -> Result<Response, ContractError> \{";

#[given(expr = "the user makes a change in the {string} contract")]
async fn makes_a_change_in_contract(world: &mut CwWorld, name: String) -> Result<()> {
    prime_workspace(world).await?;
    let filename = world
        .ws_root
        .join("contracts")
        .join(name)
        .join("src/contract.rs");
    let mut contract = String::new();
    fs::OpenOptions::new()
        .read(true)
        .open(&filename)?
        .read_to_string(&mut contract)?;

    let petname = petname(1, "");
    let change = format!("\n    if \"{}\" == \"{}\" {{ panic!() }}", petname, petname);
    let replaced = Regex::new(MIGRATE_REGEX)?
        .replace_all(&contract, format!("$0{}", change))
        .to_string();
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&filename)?
        .write_all(replaced.as_bytes())?;

    Ok(())
}

#[given(expr = "the user deletes the artifact {string}")]
async fn deletes_str_artifact(world: &mut CwWorld, name: String) -> Result<()> {
    prime_workspace(world).await?;
    let wasm_pattern = world
        .ws_root
        .as_path()
        .join(format!("artifacts/{}*.wasm", &name))
        .to_str()
        .unwrap_or_default()
        .to_string();
    let matches: Vec<PathBuf> = glob(&wasm_pattern)?.collect::<std::result::Result<_, _>>()?;
    match matches.first() {
        Some(path) => fs::remove_file(path)?,
        None => panic!("couldn't find any artifact matching \"{}\"", name),
    }
    Ok(())
}

#[then(expr = "{int} wasm files exist in the artifacts dir")]
async fn n_wasm_artficats(world: &mut CwWorld, n: usize) -> Result<()> {
    let wasm_pattern = world
        .ws_root
        .as_path()
        .join("artifacts/*.wasm")
        .to_str()
        .unwrap_or_default()
        .to_string();
    world.artifacts = glob(&wasm_pattern)?.collect::<std::result::Result<_, _>>()?;

    assert_eq!(n, world.artifacts.len());
    Ok(())
}

#[then(expr = "each artifact contains a function named {string}")]
async fn each_artifact_contains_str_function(world: &mut CwWorld, name: String) -> Result<()> {
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
async fn n_optimizations(world: &mut CwWorld, n: usize) -> Result<()> {
    // TODO: verify that the checksum(s) didn't change
    world
        .cmd_output
        .as_ref()
        .context("missing command output")?
        .clone()
        .assert()
        .stdout(predicate::str::contains("was optimized").count(n));

    Ok(())
}

#[then(expr = "{int} contracts are unchanged and skipped")]
async fn n_optimizations_cached(world: &mut CwWorld, n: usize) -> Result<()> {
    // TODO: verify that the checksum(s) didn't change
    world
        .cmd_output
        .as_ref()
        .context("missing command output")?
        .clone()
        .assert()
        .stdout(predicate::str::contains("is unchanged. Skipping").count(n));

    Ok(())
}

#[then(expr = "{string} is reoptimized")]
async fn str_is_reoptimized(world: &mut CwWorld, name: String) -> Result<()> {
    // TODO: verify that the checksum changed
    world
        .cmd_output
        .as_ref()
        .context("missing command output")?
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

async fn prime_workspace(world: &mut CwWorld) -> Result<()> {
    if world.primed {
        return Ok(());
    }

    let _ = run_cw_optimizoor(&world.ws_root, "successfully")?;
    world.primed = true;

    Ok(())
}

fn run_cw_optimizoor(workspace_root: &PathBuf, expected_result: &str) -> Result<Output> {
    let mut cmd = AssertCommand::cargo_bin(CARGO_CW_OPTIMIZOOR)?;
    cmd.current_dir(workspace_root);
    cmd.arg(CW_OPTIMIZOOR);

    let assert = match expected_result {
        "successfully" => cmd.assert().success(),
        "unsuccessfully" => cmd.assert().failure(),
        other => unreachable!("unexpected run outcome: {other}"),
    };

    Ok(assert.get_output().clone())
}
