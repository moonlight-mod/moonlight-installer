use std::path::PathBuf;

use crate::types::GitHubRelease;

const USER_AGENT: &str =
    "moonlight-installer (https://github.com/moonlight-mod/moonlight-installer)";

pub fn get_stable_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/moonlight-mod/moonlight/releases/latest";
    let resp = reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()?
        .json()?;
    Ok(resp)
}

pub fn get_nightly_version() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://moonlight-mod.github.io/moonlight/ref";
    let resp = reqwest::blocking::get(url)?.text()?;
    let first_line = resp.lines().next().unwrap();
    Ok(first_line.to_string())
}

pub fn get_stable_version() -> Result<String, Box<dyn std::error::Error>> {
    let release = get_stable_release()?;
    Ok(release.name)
}

pub fn download_nightly(
    version_txt: PathBuf,
    dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://moonlight-mod.github.io/moonlight/dist.tar.gz";
    let resp = reqwest::blocking::get(url)?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(resp));
    archive.unpack(dir)?;
    std::fs::write(version_txt, get_nightly_version()?)?;
    Ok(())
}

pub fn download_stable(
    version_txt: PathBuf,
    dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let release = get_stable_release()?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == "dist.tar.gz")
        .unwrap();

    let resp = reqwest::blocking::Client::new()
        .get(&asset.browser_download_url)
        .header("User-Agent", USER_AGENT)
        .send()?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(resp));

    archive.unpack(dir)?;
    std::fs::write(version_txt, get_stable_version()?)?;

    Ok(())
}
