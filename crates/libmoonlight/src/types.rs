use crate::get_moonlight_dir;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize, Deserialize, clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonlightBranch {
    Stable,
    Nightly,
}

impl Display for MoonlightBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Nightly => write!(f, "nightly"),
        }
    }
}

impl MoonlightBranch {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Nightly => "Nightly",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Stable => {
                "Periodic updates and fixes when they're ready. Suggested for most users."
            }
            Self::Nightly => {
                "In-progress development snapshots while it's being worked on. May contain issues."
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Branch {
    Stable,
    PTB,
    Canary,
    Development,
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::PTB => write!(f, "PTB"),
            Self::Canary => write!(f, "Canary"),
            Self::Development => write!(f, "Development"),
        }
    }
}

impl Branch {
    #[must_use]
    pub fn config(&self) -> PathBuf {
        get_moonlight_dir().join(format!("{}.json", self.to_string().to_lowercase()))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Stable => "Discord",
            Self::PTB => "DiscordPTB",
            Self::Canary => "DiscordCanary",
            Self::Development => "DiscordDevelopment",
        }
    }

    pub fn dashed_name(&self) -> &'static str {
        match self {
            Self::Stable => "discord",
            Self::PTB => "discord-ptb",
            Self::Canary => "discord-canary",
            Self::Development => "discord-development",
        }
    }

    pub fn kill_discord(&self) {
        let name = self.name();

        match std::env::consts::OS {
            "windows" => {
                std::process::Command::new("taskkill")
                    .args(["/F", "/IM", &format!("{name}.exe")])
                    .output()
                    .ok();
            }

            "macos" | "linux" => {
                std::process::Command::new("killall")
                    .args([name])
                    .output()
                    .ok();
            }

            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DetectedInstall {
    pub branch: Branch,
    pub path: PathBuf,
    pub flatpak_id: Option<String>,
}

// Just DetectedInstall but tracking patched for the UI
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstallInfo {
    pub install: DetectedInstall,
    pub patched: bool,
    pub has_config: bool,
}

// Lot more in here but idc
#[derive(Deserialize, Debug)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Deserialize, Debug)]
pub struct GitHubRelease {
    pub name: String,
    pub assets: Vec<GitHubReleaseAsset>,
}

// we only care about filesystem so
#[derive(Serialize, Deserialize, Debug)]
pub struct FlatpakOverrides {
    #[serde(rename = "Context")]
    pub context: Option<FlatpakOverridesContext>,
    #[serde(flatten)]
    other: serde_value::Value,
}

impl Default for FlatpakOverrides {
    fn default() -> Self {
        Self {
            context: Some(Default::default()),
            other: serde_value::Value::Map(BTreeMap::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlatpakOverridesContext {
    pub filesystems: Option<FlatpakArray<FlatpakFilesystemOverride>>,
    #[serde(flatten)]
    other: serde_value::Value,
}

impl Default for FlatpakOverridesContext {
    fn default() -> Self {
        Self {
            filesystems: Some(Default::default()),
            other: serde_value::Value::Map(BTreeMap::new()),
        }
    }
}

#[derive(Debug)]
pub struct FlatpakArray<T> {
    inner: Vec<T>,
}

impl<T> Default for FlatpakArray<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T> std::ops::Deref for FlatpakArray<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for FlatpakArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> From<FlatpakArray<T>> for Vec<T> {
    fn from(value: FlatpakArray<T>) -> Self {
        value.inner
    }
}

impl<T> From<Vec<T>> for FlatpakArray<T> {
    fn from(value: Vec<T>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum FlatpakFilesystemOverridePermission {
    #[default]
    ReadWrite,
    ReadOnly,
    Create,
    Off,
}

#[derive(Debug)]
pub struct FlatpakFilesystemOverride {
    pub path: String,
    pub permission: FlatpakFilesystemOverridePermission,
}

#[derive(Debug)]
pub struct FlatpakFilesystemOverrideParseError(String);

impl Display for FlatpakFilesystemOverrideParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid permission: {:?}", self.0)
    }
}

impl std::error::Error for FlatpakFilesystemOverrideParseError {}

impl FromStr for FlatpakFilesystemOverride {
    type Err = FlatpakFilesystemOverrideParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use FlatpakFilesystemOverridePermission::*;

        if let Some(p) = s.strip_prefix('!') {
            return Ok(Self {
                path: String::from(p),
                permission: Off,
            });
        }

        match s.split_once(':') {
            Some((lhs, rhs)) => {
                let permission = match rhs {
                    "rw" => ReadWrite,
                    "ro" => ReadOnly,
                    "create" => Create,
                    _ => return Err(FlatpakFilesystemOverrideParseError(rhs.into())),
                };

                Ok(Self {
                    path: String::from(lhs),
                    permission,
                })
            }
            None => Ok(Self {
                path: String::from(s),
                permission: Default::default(),
            }),
        }
    }
}

impl Display for FlatpakFilesystemOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FlatpakFilesystemOverridePermission::*;

        let p = match self.permission {
            ReadWrite => "rw",
            ReadOnly => "ro",
            Create => "create",
            Off => return write!(f, "!{}", self.path),
        };

        write!(f, "{}:{p}", self.path)
    }
}

// no one has implemented the flatpak filesystem overrides serialization
// format correctly except for flatpak itself, so we wont try too hard
impl<T> Serialize for FlatpakArray<T>
where
    T: ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_strings: Vec<String> = self.iter().map(|x| x.to_string()).collect();
        let mut v = as_strings.join(";");
        v += ";";
        serializer.serialize_str(&v)
    }
}

impl<'de, T> Deserialize<'de> for FlatpakArray<T>
where
    T: FromStr,
    T::Err: std::error::Error,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FlatpakArrayVisitor<T>(PhantomData<T>);
        impl<T> FlatpakArrayVisitor<T> {
            pub fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl<T> Visitor<'_> for FlatpakArrayVisitor<T>
        where
            T: FromStr,
            T::Err: std::error::Error,
        {
            type Value = FlatpakArray<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an array of strings separated by semicolons")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let parts: Vec<String> = v
                    .strip_suffix(";")
                    .unwrap_or(v)
                    .split(';')
                    .map(String::from)
                    .collect();
                let mut vec = Vec::with_capacity(parts.len());

                for part in parts {
                    vec.push(T::from_str(&part).map_err(E::custom)?);
                }

                Ok(vec.into())
            }
        }

        deserializer.deserialize_str(FlatpakArrayVisitor::<T>::new())
    }
}
