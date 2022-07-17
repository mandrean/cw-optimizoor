use core::{
    convert::AsRef,
    result::Result::{Err, Ok},
};
use std::{
    env::consts::ARCH,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use binaryen::Module;

/// Optimizes the WASM artifact using binaryen/wasm-opt.
pub fn optimize<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let cfg = binaryen::CodegenConfig {
        optimization_level: 2,
        shrink_level: 2,
        debug_info: false,
    };

    let mut wasm = read_module(input_path.as_ref())?;
    wasm.optimize(&cfg);

    write_module(&output_path, &wasm)
}

/// Reads & deserializes the WASM artifact into a binaryen IR module.
pub fn read_module<P: AsRef<Path>>(wasm_path: P) -> Result<binaryen::Module> {
    let mut f = File::open(wasm_path).map_err(|_| anyhow!("WASM file not found"))?;
    let mut contents = Vec::new();
    f.read_to_end(&mut contents)
        .map_err(|_| anyhow!("error reading WASM file"))?;

    binaryen::Module::read(&contents).map_err(|_| anyhow!("error parsing WASM file"))
}

/// Serializes & writes the binaryen IR module to a WASM artifact.
pub fn write_module<P: AsRef<Path>>(output_path: P, wasm: &Module) -> Result<()> {
    let mut f = File::create(output_path).map_err(|_| anyhow!("error creating WASM file"))?;
    f.write_all(wasm.write().as_slice())
        .map_err(|_| anyhow!("error writing WASM file"))
}

/// Returns the optimized WASM output path.
/// Suffixes the filename (before extension) with the host's CPU arch.
pub fn optimized_output_path<P: AsRef<Path>>(wasm_path: P, output_dir: P) -> Result<PathBuf> {
    let filename = PathBuf::from(
        wasm_path
            .as_ref()
            .file_name()
            .ok_or_else(|| anyhow!("missing filename"))?,
    );
    let filename = match (
        filename.file_stem().and_then(OsStr::to_str),
        filename.extension().and_then(OsStr::to_str),
    ) {
        (Some(stem), Some(ext)) => Ok(format!("{}-{}.{}", stem, ARCH, ext)),
        _ => Err(anyhow!("couldn't parse filename")),
    }?;

    let mut output_path = output_dir.as_ref().to_path_buf();
    output_path.push(filename);

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use std::{env::consts::ARCH, path::PathBuf};

    use crate::optimization::optimized_output_path;

    #[test]
    fn suffixes_filename_with_arch() {
        let input_path = PathBuf::from("some/path/to/artifact.wasm");
        let output_dir = PathBuf::from("some/output/dir");

        assert_eq!(
            format!("some/output/dir/artifact-{}.wasm", ARCH),
            format!(
                "{}",
                optimized_output_path(&input_path, &output_dir)
                    .unwrap()
                    .display()
            )
        )
    }
}
