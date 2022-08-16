use std::{
    fs::File,
    io,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use hex::ToHex;
use sha2::{Digest, Sha256};

use crate::ext::TakeExt;

/// Calculates the SHA-256 digest of a buffer.
pub fn sha256_digest<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Sha256::new();
    let _ = io::copy(&mut reader, &mut hasher)?;
    let digest = hasher.finalize().encode_hex::<String>();
    Ok(digest)
}

/// Calculates the SHA-256 checksums of the provided WASM artifacts, and outputs them to a file.
pub fn write_checksums(wasm_paths: &[PathBuf], output_file: &PathBuf) -> Result<()> {
    let mut checksums = BufWriter::new(
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_file)?,
    );
    let _ = wasm_paths.iter().try_for_each(|wasm_path| {
        let checksum = checksum(wasm_path)?;
        checksums.write_all(checksum.as_bytes())?;

        print!("    ...{}", &checksum);
        anyhow::Ok(())
    });

    checksums.flush().map_err(|e| anyhow!(e))
}

/// Calculates the checksum of a provided artifact.
pub fn checksum(wasm_path: &PathBuf) -> Result<String> {
    let input = File::open(wasm_path)?;
    let reader = BufReader::new(input);
    let checksum = format!(
        "{}  {}\n",
        sha256_digest(reader)?,
        wasm_path.rtake(1).display(),
    );

    Ok(checksum)
}
