use glob::glob;
use path_absolutize::Absolutize;
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

#[test]
fn it_can_compile_repo() {
    let cwd = env::current_dir().unwrap();
    let manifest_path = PathBuf::from("tests/cw-plus/Cargo.toml");
    let res = cw_optimizoor::run(manifest_path.as_path());

    let wasm_pattern = &*cwd
        .join("tests/cw-plus/artifacts/*.wasm")
        .to_str()
        .unwrap_or_default()
        .to_string();
    for entry in glob(wasm_pattern).expect("Failed to read glob pattern") {
        let path = format!("{}", entry.unwrap().display());
        println!("{}", path);
        let decomp = Command::new("wasm-decompile")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let grep = Command::new("grep")
            .arg("function execute")
            .stdin(decomp.stdout.unwrap())
            .output()
            .unwrap();

        assert!(grep.status.success())
    }

    assert!(res.is_ok());
}

#[test]
fn finds_manifest() {
    // dir path
    let path = cw_optimizoor::find_manifest(PathBuf::from("tests/cw-plus")).unwrap();
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));

    // dot (current dir) path
    env::set_current_dir(PathBuf::from("tests/cw-plus/").absolutize().unwrap()).unwrap();
    let path = cw_optimizoor::find_manifest(PathBuf::from(".")).unwrap();
    assert!(path.ends_with("tests/cw-plus/Cargo.toml"));

    // invalid manifest
    let res = cw_optimizoor::find_manifest(PathBuf::from("tests/cw-plus/wrong.toml"));
    assert!(res.is_err());
}
