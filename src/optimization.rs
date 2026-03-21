use std::{
    env::consts::ARCH,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use binaryen::Module;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    error::{Error, Result},
    hashing::{checksum, read_checksums},
};

pub fn incremental_optimizations(
    output_dir: &Path,
    intermediate_wasm_paths: Vec<PathBuf>,
    prev_intermediate_checksums: &str,
) -> Result<Vec<PathBuf>> {
    let checksums_path = output_dir.join("checksums.txt");
    let checksums = read_checksums(&checksums_path)?;
    let final_wasm_paths = intermediate_wasm_paths
        .par_iter()
        .map(|wasm_path| {
            let output_path = optimized_output_path(wasm_path, output_dir)?;
            let wasm_name = artifact_name(wasm_path)?;
            let input_checksum = checksum(wasm_path)?;

            // if optimized artifact exists,
            // and both its and prev intermediate artifact checksums match,
            // then skip optimizing it again
            if output_path.exists()
                && prev_intermediate_checksums.contains(&input_checksum)
                && checksums.contains(&checksum(&output_path)?)
            {
                println!("    ...⏭️  {} is unchanged. Skipping.", wasm_name);
            } else {
                optimize(wasm_path, &output_path)?;
                println!("    ...✅ {} was optimized.", wasm_name);
            }

            Ok(output_path)
        })
        .collect::<Result<Vec<PathBuf>>>()?;

    Ok(final_wasm_paths)
}

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
    let wasm_path = wasm_path.as_ref();
    let mut f = File::open(wasm_path).map_err(|source| Error::ReadWasm {
        path: wasm_path.to_path_buf(),
        source,
    })?;
    let mut contents = Vec::new();
    f.read_to_end(&mut contents)
        .map_err(|source| Error::ReadWasm {
            path: wasm_path.to_path_buf(),
            source,
        })?;

    binaryen::Module::read(&contents).map_err(|source| Error::ParseWasm {
        path: wasm_path.to_path_buf(),
        reason: format!("{source:?}"),
    })
}

/// Serializes & writes the binaryen IR module to a WASM artifact.
pub fn write_module<P: AsRef<Path>>(output_path: P, wasm: &Module) -> Result<()> {
    let output_path = output_path.as_ref();
    let mut f = File::create(output_path).map_err(|source| Error::CreateWasm {
        path: output_path.to_path_buf(),
        source,
    })?;
    f.write_all(wasm.write().as_slice())
        .map_err(|source| Error::WriteWasm {
            path: output_path.to_path_buf(),
            source,
        })
}

/// Returns the optimized WASM output path.
/// Suffixes the filename (before extension) with the host's CPU arch.
pub fn optimized_output_path<P: AsRef<Path>, Q: AsRef<Path>>(
    wasm_path: P,
    output_dir: Q,
) -> Result<PathBuf> {
    let wasm_path = wasm_path.as_ref();
    let filename =
        PathBuf::from(
            wasm_path
                .file_name()
                .ok_or_else(|| Error::MissingArtifactFileName {
                    path: wasm_path.to_path_buf(),
                })?,
        );
    let filename = match (
        filename.file_stem().and_then(OsStr::to_str),
        filename.extension().and_then(OsStr::to_str),
    ) {
        (Some(stem), Some(ext)) => Ok(format!("{}-{}.{}", stem, ARCH, ext)),
        _ => Err(Error::InvalidArtifactFileName {
            path: wasm_path.to_path_buf(),
        }),
    }?;

    let mut output_path = output_dir.as_ref().to_path_buf();
    output_path.push(filename);

    Ok(output_path)
}

fn artifact_name(wasm_path: &Path) -> Result<String> {
    wasm_path
        .file_stem()
        .and_then(OsStr::to_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::InvalidArtifactFileName {
            path: wasm_path.to_path_buf(),
        })
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
