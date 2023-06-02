use colour::{green, green_ln, red, yellow, yellow_ln};
use crates_io_api::AsyncClient as CratesIoClient;
use semver::Version;

/// Fetches the latest version of the crate on Crates.io
pub async fn fetch_latest_version(crate_name: &str) -> anyhow::Result<Version> {
    let client = CratesIoClient::new(
        "SelfUpdater/CheckLatestVersion",
        std::time::Duration::from_millis(1000),
    )?;

    let latest_version = client
        .get_crate(crate_name)
        .await?
        .crate_data
        .max_version
        .parse::<Version>()?;

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
        green_ln!("cargo install {}", crate_name);
    }
}
