use std::{io, path::PathBuf};

use semver::Error as SemverError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to resolve workspace path {path}")]
    ResolveWorkspacePath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid workspace path: {path}")]
    InvalidWorkspacePath { path: PathBuf },

    #[error("couldn't locate manifest {path}")]
    MissingManifest { path: PathBuf },

    #[error("failed to initialize cargo configuration")]
    CargoConfig {
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to prepare cargo build configuration")]
    BuildConfiguration {
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to initialize workspace from {manifest_path}")]
    WorkspaceInit {
        manifest_path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to create artifacts directory {path}")]
    CreateArtifactsDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("no CosmWasm contracts found under {contracts_dir}")]
    NoContracts { contracts_dir: PathBuf },

    #[error("failed to compile workspace packages")]
    CompileWorkspace {
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to create an ephemeral workspace for {package}")]
    EphemeralWorkspace {
        package: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to read checksums from {path}")]
    ReadChecksums {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write checksums to {path}")]
    WriteChecksums {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to calculate a checksum for {path}")]
    ArtifactChecksum {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("artifact path {path} is missing a filename")]
    MissingArtifactFileName { path: PathBuf },

    #[error("artifact path {path} is missing a valid stem or extension")]
    InvalidArtifactFileName { path: PathBuf },

    #[error("failed to read WASM artifact {path}")]
    ReadWasm {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse WASM artifact {path}: {reason}")]
    ParseWasm { path: PathBuf, reason: String },

    #[error("failed to create optimized WASM artifact {path}")]
    CreateWasm {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write optimized WASM artifact {path}")]
    WriteWasm {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to create the crates.io client")]
    UpdateCheckClient {
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to fetch crate metadata for {crate_name}")]
    UpdateCheckRequest {
        crate_name: String,
        #[source]
        source: crates_io_api::Error,
    },

    #[error("failed to parse crates.io version {version} for {crate_name}")]
    UpdateCheckVersion {
        crate_name: String,
        version: String,
        #[source]
        source: SemverError,
    },
}
