use std::fs;

use color_eyre::eyre::Context;
use color_eyre::Result;
use serde::Deserialize;
use crate::VAT_ROOT;

#[derive(Debug, Deserialize)]
pub struct Shortform {
    pub short: String,
    pub full: String,
}

impl Shortform {
    fn new<S1: Into<String>, S2: Into<String>>(short: S1, full: S2) -> Self {
        Self { short: short.into(), full: full.into() }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Maximum lifespan for a fetch (in seconds)
    pub fetch_timeout: u64,

    /// Maximum lifespan for .vat-cache (in seconds)
    pub cache_timeout: u64,

    /// Shortforms for upstream URLs
    pub shortforms: Vec<Shortform>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fetch_timeout: 30,
            cache_timeout: 3600,
            shortforms: default_shortforms(),
        }
    }
}

impl Config {
    pub fn parse() -> Result<Self> {
        let path = VAT_ROOT.join("config.toml");

        if !path.exists() {
            warn!("Config at '{}' does not exist! Builtin defaults will be used.", path.display());
            return Ok(Self::default());
        }

        let s = fs::read_to_string(path).wrap_err("Could not read config to string")?;
        toml::from_str(&s).wrap_err("Invalid config")
    }
}

fn default_shortforms() -> Vec<Shortform> {
    vec![
        // GitHub aliases
        Shortform::new("github:", "https://github.com/"),
        Shortform::new("gh:", "https://github.com/"),

        // GitLab aliases
        Shortform::new("gitlab:", "https://gitlab.com/"),
        Shortform::new("gl:", "https://gitlab.com/"),

        // Dotgay aliases
        Shortform::new("dotgay:", "https://git.gay/"),

        // Codeberg aliases
        Shortform::new("codeberg:", "https://codeberg.org/"),
        Shortform::new("cb:", "https://codeberg.org/"),

        // Freedesktop aliases
        Shortform::new("freedesktop:", "https://gitlab.freedesktop.org/"),

        // Inria aliases
        Shortform::new("inria:", "https://gitlab.inria.fr/"),

        // Salsa aliases
        Shortform::new("salsa:", "https://salsa.debian.org/"),

        // Kernel aliases
        Shortform::new("kernel:", "https://git.kernel.org/pub/scm/"),

        // SourceHut aliases
        Shortform::new("sourcehut:", "https://git.sr.ht/~"),
        Shortform::new("srht:", "https://git.sr.ht/~"),
    ]
}
