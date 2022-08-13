use path_absolutize::Absolutize;
use std::{env, path::PathBuf};

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
