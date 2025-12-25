// package/mod.rs

pub mod bulk;

use color_eyre::Result;
use color_eyre::eyre::bail;
use rand::random_range;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Write;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{debug, error, info};

use crate::NO_CACHE;
use crate::SHLIB_PATH;
use crate::VAT_CACHE;
use crate::VAT_ROOT;
use crate::args::ARGS;
use crate::utils::cmd::cmd;
use crate::utils::float::defloat;
use crate::utils::str::basename;
use crate::utils::ver::Version;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Package {
    pub name: String,
    pub config: PackageConfig,
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PackageConfig {
    pub upstream: String,
    pub chance: f64,
    pub channels: Vec<PackageChannel>,
}

/// Struct to be used when serializing into p/ALL
#[derive(Debug, Serialize, Clone)]
pub struct PackageVersions {
    pub package: String,
    pub versions: Vec<VersionChannel>,
}

#[derive(Hash, PartialEq, Eq, Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PackageChannel {
    pub name: String,
    pub enabled: bool,
    pub upstream: Option<String>,
    pub fetch: String,
    pub expected: Option<String>,
    // TODO: Consider adding per-channel chances
}

impl Default for PackageChannel {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            upstream: None,
            fetch: String::new(),
            expected: None,
        }
    }
}

impl PackageChannel {
    pub fn cmd(&self, package: &Package, command: &[&str]) -> Result<String> {
        let package_root = Package::dir(&package.name);

        let Some(vat_root) = VAT_ROOT.to_str() else {
            bail!("Invalid Unicode in {}", VAT_ROOT.display());
        };

        let Some(vat_cache) = VAT_CACHE.to_str() else {
            bail!("Invalid Unicode in {}", VAT_CACHE.display());
        };

        let Some(shlib_path) = SHLIB_PATH.to_str() else {
            bail!("Invalid Unicode in {}", SHLIB_PATH.display());
        };

        let no_cache = NO_CACHE.to_string();

        let upstream = self.upstream.as_ref().unwrap_or(&package.config.upstream);

        let env = HashMap::from([
            ("GIT_TERMINAL_PROMPT", "false"),
            ("PACKAGE_ROOT", &package_root),
            ("VAT_ROOT", vat_root),
            ("VAT_CACHE", vat_cache),
            ("SHLIB_PATH", shlib_path),
            ("NO_CACHE", &no_cache),
            ("channel", &self.name),
            ("name", basename(&package.name)),
            ("upstream", upstream),
        ]);

        cmd(command, env, &package_root)
    }

    pub fn fetch(&self, package: &Package) -> Result<String> {
        let fetch = format!(". {} && {}", SHLIB_PATH.display(), self.fetch);
        let command = ["bash", "-c", &fetch];

        let ver = match self.cmd(package, &command) {
            Err(e) => bail!("Failed to fetch version: {e}"),
            Ok(v) => v,
        };

        let mut version = Version::new(ver);
        version.trim(package);
        let v = version.fmt;

        if let Some(re) = &self.expected {
            let re = match Regex::from_str(re) {
                Ok(re) => re,
                Err(e) => {
                    error!("Invalid expected regex '{re}': {e}");
                    bail!("Invalid expected regex");
                }
            };

            if !re.is_match(&v) {
                error!("Version '{v}' does not match expected '{re}'");
                bail!("Version does not match expected");
            }
        }

        Ok(v)
    }
}

impl Hash for PackageConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.upstream.hash(state);
        defloat(self.chance).hash(state);
        self.channels.hash(state);
    }
}

impl PartialEq for PackageConfig {
    fn eq(&self, other: &Self) -> bool {
        self.upstream == other.upstream
            && (self.chance - other.chance).abs() < 0.01
            && self.channels == other.channels
    }
}

impl Eq for PackageConfig {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionChannel {
    pub channel: String,
    pub version: String,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            upstream: String::new(),
            chance: 1.0,
            channels: vec![],
        }
    }
}

pub enum UpstreamType {
    Arch,
    Curl,
    Empty,
    Git,
}

impl UpstreamType {
    fn from_str(str: &str) -> Self {
        match str {
            // match arch
            s if s.contains("archlinux.org") => Self::Arch,

            // match (sorted) distfile pages
            s if s.contains("C=M") && s.contains("O=D") => Self::Curl,

            "" => Self::Empty,

            // assume all else is git
            _ => Self::Git,
        }
    }
}

impl Package {
    pub fn from_name<S: Into<String>>(name: S) -> Result<Self> {
        let name = name.into();
        let config_path = Path::new(Self::dir(&name).as_str()).join("config");

        let raw = fs::read_to_string(config_path)?;
        let config: PackageConfig = toml::from_str(&raw)?;

        let mut package = Self { name, config };
        package.set_defaults();

        Ok(package)
    }

    /// Retrieve the directory for a package
    pub fn dir<S: AsRef<str>>(name: S) -> String {
        format!("{}/p/{}", VAT_ROOT.display(), name.as_ref())
    }

    pub fn get_channel(&self, name: &str) -> Option<&PackageChannel> {
        self.config.channels.iter().find(|c| c.name == name)
    }

    pub fn set_defaults(&mut self) {
        if self.config.upstream.is_empty() {
            self.config.upstream = format!("gh:{n}/{n}", n = basename(&self.name));
        }

        for channel in &mut self.config.channels {
            let upstream = channel.upstream.as_ref().unwrap_or(&self.config.upstream);
            let ut = UpstreamType::from_str(upstream);

            if channel.fetch.is_empty() {
                channel.fetch = match (ut, channel.name.as_str()) {
                    (UpstreamType::Arch, "release") => "archver".into(),

                    (UpstreamType::Curl, "release") => "defcurlrelease".into(),
                    (UpstreamType::Curl, "unstable") => "defcurlunstable".into(),
                    (UpstreamType::Curl, "commit") => "defcurlcommit".into(),

                    (UpstreamType::Empty, _) => String::new(),

                    (UpstreamType::Git, "release") => "defgitrelease".into(),
                    (UpstreamType::Git, "unstable") => "defgitunstable".into(),
                    (UpstreamType::Git, "commit") => "defgitcommit".into(),

                    _ => panic!(
                        "Invalid config in {}: Missing fetch for {}",
                        self.name, channel.name
                    ),
                }
            }

            if channel.expected.is_none() {
                channel.expected = match channel.name.as_str() {
                    "release" => Some(r"^[0-9]+(\.[0-9]+)*$".into()),
                    "unstable" => {
                        Some(r"^[0-9]+(\.[0-9]+)*-?(rc|alpha|beta|a|b|pre|dev)?[0-9]*$".into())
                    }
                    "commit" => Some(r"^[0-9a-f]{40}$".into()),
                    n if n.parse::<u64>().is_ok() => Some(format!(r"^{n}(\.[0-9]+)*$")),

                    _ => panic!(
                        "Invalid config in {}: Missing expected for {}",
                        self.name, channel.name
                    ),
                }
            }
        }
    }

    pub fn has_fallback_versions(&self) -> bool {
        let path = Path::new("p").join(&self.name).join("versions.json");
        if !path.exists() {
            return false;
        }

        let Ok(version_channels_str) = fs::read_to_string(path) else {
            return false;
        };
        let Ok(version_channels) =
            serde_json::from_str::<Vec<VersionChannel>>(&version_channels_str)
        else {
            return false;
        };

        for channel in &version_channels {
            if self.get_channel(&channel.channel).is_none() {
                return false;
            }
        }

        true
    }

    /// Used for log output only
    pub fn format_fetched(&self, version_channels: &[VersionChannel]) -> String {
        let mut s = String::new();
        let _ = writeln!(&mut s, "Fetched versions for {}", self.name);
        for vc in version_channels {
            let _ = writeln!(&mut s, "        - {}: {}", vc.channel, vc.version);
        }
        s
    }

    pub fn fetch(&self) -> Result<Vec<VersionChannel>> {
        // if fallback versions don't exist, or --guarantee is passed, guarantee a fetch
        let should_guarantee = ARGS.guarantee || !self.has_fallback_versions();

        if self.config.chance < 1.0
            && !should_guarantee
            && random_range(0.0..=1.0) > self.config.chance
        {
            bail!("Tails!")
        }

        let mut version_channels = vec![];
        for channel in &self.config.channels {
            if channel.enabled {
                version_channels.push(VersionChannel {
                    channel: channel.name.clone(),
                    version: channel.fetch(self)?,
                });
            }
        }

        info!("{}", self.format_fetched(&version_channels));
        debug!(
            "Versions as JSON: {}",
            serde_json::to_string_pretty(&version_channels)?
        );

        Ok(version_channels)
    }

    pub fn get_package_path(&self) -> PathBuf {
        Path::new("p").join(&self.name)
    }

    /// Write version data for all version channels for all APIs
    pub fn write_versions(&self, version_channels: Vec<VersionChannel>) -> Result<()> {
        let path = self.get_package_path();
        fs::write(
            path.join("versions.json"),
            serde_json::to_string_pretty(&version_channels)?,
        )?;

        let channels_dir = path.join("channels");

        if !channels_dir.exists() {
            fs::create_dir(&channels_dir)?;
        }

        let mut versionstxt = String::new();
        for channel in version_channels {
            fs::write(channels_dir.join(&channel.channel), &channel.version)?;
            versionstxt = format!("{versionstxt}{}\t{}\n", channel.channel, channel.version);
        }

        fs::write(path.join("versions.txt"), versionstxt)?;
        Ok(())
    }

    /// Write version data for all version channels (reads from JSON API)
    pub fn read_versions(&self) -> Result<Vec<VersionChannel>> {
        let path = self.get_package_path().join("versions.json");
        let json_str = fs::read_to_string(path)?;

        let version_channels = serde_json::from_str(&json_str)?;
        Ok(version_channels)
    }
}
