use color_eyre::config::HookBuilder;
use color_eyre::eyre::WrapErr;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use self::args::ARGS;
use self::config::Config;
use self::package::{Package, bulk};
use self::utils::log::log;
use color_eyre::Result;

mod args;
mod config;
mod package;
mod utils;

#[macro_use]
extern crate tracing;

static VAT_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| env::current_dir().expect("Couldn't get working directory"));

static VAT_CACHE: LazyLock<PathBuf> = LazyLock::new(|| VAT_ROOT.join(".vat-cache"));

static SHLIB_PATH: LazyLock<PathBuf> = LazyLock::new(|| VAT_ROOT.join("sh/lib.env"));

static NO_CACHE: LazyLock<bool> = LazyLock::new(|| ARGS.no_cache);

static CONFIG: OnceLock<Config> = OnceLock::new();

#[cfg(test)]
#[ctor::ctor]
fn setup() {
    log();

    CONFIG.set(Config::parse().expect("Invalid config"))
        .expect("Only one thread should be setting the config");
}

fn main() -> color_eyre::Result<()> {
    HookBuilder::default()
        .display_env_section(true)
        .display_location_section(true)
        .add_default_filters()
        .install()?;

    log();
    CONFIG.set(Config::parse()?)
        .expect("Only one thread should be setting the config");

    clean_cache()?;
    let start_timestamp = Instant::now();

    debug!("Determined Vat root to be {}", VAT_ROOT.display());

    let packages = if ARGS.packages.is_empty() {
        bulk::find_all()?
    } else {
        ARGS.packages
            .iter()
            .map(|s| Package::from_name(s.clone()))
            .collect::<Result<Vec<_>>>()?
    };

    debug!("Detected packages: {packages:#?}");
    let map = bulk::fetch_all(&packages)?;

    let elapsed = humantime::format_duration(start_timestamp.elapsed()).to_string();

    if !ARGS.pretend {
        bulk::write_all(&map)?;
        increment_runcount()?;
        debug!("Incremented runcount");
        fs::write(VAT_CACHE.join("elapsed"), &elapsed)?;
    }

    info!("Finished in {elapsed}");
    Ok(())
}

fn increment_runcount() -> Result<()> {
    let path = Path::new("runcount");
    let runcount = fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0u64)
        + 1;
    fs::write(path, runcount.to_string())?;
    Ok(())
}

fn clean_cache() -> Result<()> {
    let cache_path = &*VAT_CACHE;
    if let Ok(m) = cache_path.metadata() {
        #[allow(clippy::cast_sign_loss)]
        let mtime = Duration::from_secs(m.mtime() as u64);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .wrap_err("Time travel detected")?;

        let cache_timeout = Duration::from_secs(
            CONFIG.get().expect("Config should be initialized").cache_timeout
        );

        if now - mtime > cache_timeout {
            debug!("Removing cache");
            fs::remove_dir_all(cache_path).wrap_err("Failed to remove cache")?;
            fs::create_dir(cache_path).wrap_err("Failed to create cache")?;
        }
    } else {
        fs::create_dir(cache_path).wrap_err("Failed to create cache")?;
    }

    Ok(())
}
