use std::path::PathBuf;

pub fn get_nightly_version() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://moonlight-mod.github.io/moonlight/ref";
    let resp = reqwest::blocking::get(url)?.text()?;
    let first_line = resp.lines().next().unwrap();
    Ok(first_line.to_string())
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
