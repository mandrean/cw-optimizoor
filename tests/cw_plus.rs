use glob::glob;
use itertools::Itertools;
use path_absolutize::Absolutize;
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

#[test]
fn it_can_compile_repo() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let manifest_path = PathBuf::from("tests/cw-plus/Cargo.toml");
    let _ = cw_optimizoor::run(manifest_path.as_path()).expect("optimizoor run failed");

    let wasm_pattern = &*cwd
        .join("tests/cw-plus/artifacts/*.wasm")
        .to_str()
        .unwrap_or_default()
        .to_string();
    let files: Vec<PathBuf> = glob(wasm_pattern)?.try_collect()?;

    assert_eq!(10, files.len());
    for entry in files {
        let path = format!("{}", entry.display());
        println!("Verifying {}...", &path);
        let decomp = Command::new("wasm-decompile")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .expect("missing decompilation output");

        let grep = Command::new("grep")
            .arg("function execute")
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

#[test]
fn finds_manifest() -> anyhow::Result<()> {
    // dir path
    let path = cw_optimizoor::find_manifest(PathBuf::from("tests/cw-plus"))?;
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));

    // dot (current dir) path
    env::set_current_dir(PathBuf::from("tests/cw-plus/").absolutize()?)?;
    let path = cw_optimizoor::find_manifest(PathBuf::from("."))?;
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));

    // invalid manifest
    let res = cw_optimizoor::find_manifest(PathBuf::from("tests/cw-plus/wrong.toml"));
    assert!(res.is_err());

    Ok(())
}
