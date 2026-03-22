use std::{
    fs,
    fs::File,
    io,
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use hex::ToHex;
use sha2::{Digest, Sha256};

use crate::{
    error::{Error, Result},
    ext::TakeExt,
};

/// Calculates the SHA-256 digest of a buffer.
pub fn sha256_digest<R: Read>(mut reader: R) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let _ = io::copy(&mut reader, &mut hasher)?;
    let digest = hasher.finalize().encode_hex::<String>();
    Ok(digest)
}

/// Reads a checksum file if it exists.
pub fn read_checksums<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(Error::ReadChecksums {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Calculates the SHA-256 checksums of the provided WASM artifacts, and outputs them to a file.
pub fn write_checksums<P: AsRef<Path>>(wasm_paths: &[PathBuf], output_file: P) -> Result<()> {
    let output_file = output_file.as_ref();
    let mut checksums = BufWriter::new(
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_file)
            .map_err(|source| Error::WriteChecksums {
                path: output_file.to_path_buf(),
                source,
            })?,
    );
    wasm_paths.iter().try_for_each(|wasm_path| {
        let checksum = checksum(wasm_path)?;
        checksums
            .write_all(checksum.as_bytes())
            .map_err(|source| Error::WriteChecksums {
                path: output_file.to_path_buf(),
                source,
            })?;

        print!("    ...{}", &checksum);
        Ok(())
    })?;

    checksums.flush().map_err(|source| Error::WriteChecksums {
        path: output_file.to_path_buf(),
        source,
    })
}

/// Calculates the checksum of a provided artifact.
pub fn checksum<P: AsRef<Path>>(wasm_path: P) -> Result<String> {
    let wasm_path = wasm_path.as_ref();
    let input = File::open(wasm_path).map_err(|source| Error::ArtifactChecksum {
        path: wasm_path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(input);
    let checksum = format!(
        "{}  {}\n",
        sha256_digest(reader).map_err(|source| Error::ArtifactChecksum {
            path: wasm_path.to_path_buf(),
            source,
        })?,
        wasm_path.to_path_buf().rtake(1).display(),
    );

    Ok(checksum)
}
