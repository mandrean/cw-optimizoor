use glob::glob;
use std::env;
use std::process::{Command, Stdio};

#[test]
fn it_can_compile_repo() {
    let cwd = env::current_dir().unwrap();
    let manifest_path = &cwd.join("tests/cw-plus/Cargo.toml");
    let res = cw_optimizoor::run(manifest_path);

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
