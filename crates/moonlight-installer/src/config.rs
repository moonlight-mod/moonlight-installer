use std::collections::HashMap;
use std::path::PathBuf;

use libmoonlight::types::MoonlightBranch;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub selected_branch: MoonlightBranch,
    pub install_selected_branches: HashMap<PathBuf, MoonlightBranch>,
}
