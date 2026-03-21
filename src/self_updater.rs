use colour::{green, green_ln, red, yellow, yellow_ln};
use crates_io_api::AsyncClient;
use semver::Version;

use crate::error::{Error, Result};

/// Fetches the latest version of the crate on Crates.io
pub async fn fetch_latest_version(crate_name: &str) -> Result<Version> {
    let client = AsyncClient::new(
        "SelfUpdater/CheckLatestVersion",
        std::time::Duration::from_millis(1000),
    )
    .map_err(|source| Error::UpdateCheckClient {
        source: anyhow::Error::new(source),
    })?;

    let crate_name = crate_name.to_owned();
    let max_version = client
        .get_crate(&crate_name)
        .await
        .map_err(|source| Error::UpdateCheckRequest {
            crate_name: crate_name.clone(),
            source,
        })?
        .crate_data
        .max_version;
    let latest_version =
        max_version
            .parse::<Version>()
            .map_err(|source| Error::UpdateCheckVersion {
                crate_name: crate_name.clone(),
                version: max_version.clone(),
                source,
            })?;

    Ok(latest_version)
}

/// Checks if the current version of cw-optimizoor is up to date.
/// If not, it prints a helpful upgrade message to the user.
pub fn check_version(crate_name: &str, current_version: &Version, latest_version: &Version) {
    if latest_version.gt(current_version) {
        yellow!("\nThere is a newer version (");
        green!("v{}", latest_version);
        yellow_ln!(") of {} available!", crate_name);
        print!("Current version is ");
        red!("v{}", current_version);
        print!(". To update, run: ");
        green_ln!("cargo install --locked {}", crate_name);
    }
}
