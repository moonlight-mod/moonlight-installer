use crate::installer::types::MoonlightBranch;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub branch: MoonlightBranch,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            branch: MoonlightBranch::Stable,
        }
    }
}
