//! LVM configuration module
//! Centralized lvm base paths, timeouts, and general constants
//! Language-agnostic; language-specific configs live in language/

use anyhow::{Result, bail};
use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;

// ─── Directory name constants ───

/// Binary file directory name (~/.lvm/bin/)
pub const BIN_DIR: &str = "bin";

/// Current version symlink directory (~/.lvm/current/)
pub const CURRENT_DIR: &str = "current";

/// Alias config directory name (~/.lvm/aliases/)
pub const ALIASES_DIR: &str = "aliases";

const DOWNLOADS_DIR: &str = "downloads";

const CACHE_DIR_NAME: &str = "cache";

// ─── General string constants ───

/// .lvmrc filename
pub const LVM_FILENAME: &str = ".lvmrc";

/// "system" version keyword
pub const SYSTEM_VERSION_KEYWORD: &str = "system";

/// LTS version prefix
pub const LTS_PREFIX: &str = "lts/";

/// List separator (for human-readable lists)
pub const LIST_SEPARATOR: &str = ", ";

/// Maximum .lvmrc walk depth
pub const MAX_LVM_DEPTH: u32 = 100;

/// Candidate home env var names (Unix prefers HOME, Windows uses USERPROFILE)
const HOME_CANDIDATES: [&str; 2] = ["HOME", "USERPROFILE"];

// ─── Path configuration ───

/// Returns the lvm root directory
/// Follows XDG Base Directory spec, prefers $XDG_DATA_HOME/lvm
/// Falls back to ~/.lvm (Unix) or %USERPROFILE%\.lvm (Windows)
pub fn lvm_home() -> Result<PathBuf> {
    if let Ok(data_home) = env::var("XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        return Ok(PathBuf::from(data_home).join("lvm"));
    }
    for var in &HOME_CANDIDATES {
        if let Ok(val) = env::var(var)
            && !val.is_empty()
        {
            return Ok(PathBuf::from(val).join(".lvm"));
        }
    }
    bail!("Cannot determine home directory (set $HOME or $XDG_DATA_HOME)")
}

/// Cached version of [`lvm_home`]. Returns a `'static` reference.
/// Avoids repeated environment variable reads and error handling.
pub fn lvm_home_cached() -> &'static PathBuf {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| lvm_home().unwrap_or_else(|_| PathBuf::from(".lvm")))
}

/// Download cache directory
pub fn downloads_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(DOWNLOADS_DIR))
}

/// downloads_dir(), but falls back to a default path on failure
pub fn downloads_dir_or_default() -> PathBuf {
    downloads_dir().unwrap_or_else(|_| default_downloads_dir())
}

/// General cache directory
pub fn cache_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(CACHE_DIR_NAME))
}

pub fn default_cache_dir() -> PathBuf {
    PathBuf::from(".lvm/cache")
}

/// Full cache path for a file, with automatic fallback if cache_dir() fails
pub fn cache_path(filename: &str) -> PathBuf {
    cache_dir()
        .unwrap_or_else(|_| default_cache_dir())
        .join(filename)
}

pub fn default_downloads_dir() -> PathBuf {
    PathBuf::from(".lvm/downloads")
}
