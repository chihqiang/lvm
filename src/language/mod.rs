pub mod dart;
pub mod flutter;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod node;
pub mod python;
pub mod rust;

mod language_trait;
mod registry;

use crate::core::config;
use anyhow::{Result, bail};

pub use crate::core::{
    CURRENT_DEFAULT_MARKER, CURRENT_MARKER, DEFAULT_MARKER, archive_ext, download_and_install,
    exe_suffix, fetch_from_mirror, fetch_github_releases_paginated, fetch_with_cache,
    fetch_with_cache_ttl, flush_reports_to_stdout, get_url, parse_github_releases, path_separator,
    remove_symlink, report, report_already_installed, report_checksum_verified, report_fallback,
    report_non_native_arch, report_verifying_checksum, resolve_partial_version, resolve_version,
    set_offline, set_parallel_downloads, sort_versions, verify_sha256,
};
pub use language_trait::Language;
pub use registry::LanguageRegistry;

/// Macro to define an environment-variable-overridable configuration value.
/// Creates a `pub(crate)` function backed by a function-local `OnceLock`.
///
/// # Example
/// ```ignore
/// crate::define_env_once!(node_mirror, "LVM_NODE_MIRROR", "https://nodejs.org/dist");
/// ```
#[macro_export]
macro_rules! define_env_once {
    ($name:ident, $env_var:expr, $default:expr) => {
        pub(crate) fn $name() -> &'static str {
            use std::sync::OnceLock;
            static VALUE: OnceLock<String> = OnceLock::new();
            VALUE.get_or_init(|| std::env::var($env_var).unwrap_or_else(|_| $default.to_string()))
        }
    };
}

pub fn reject_system_install(version: Option<&str>) -> Result<()> {
    if version.is_some_and(|v| v.trim() == config::SYSTEM_VERSION_KEYWORD) {
        bail!("Use 'lvm use system' instead of 'lvm install system'");
    }
    Ok(())
}

pub fn reject_lts_install(lang_name: &str, version: Option<&str>) -> Result<()> {
    if version.is_some_and(|v| v.trim().starts_with(config::LTS_PREFIX)) {
        bail!("{lang_name} does not have LTS releases");
    }
    Ok(())
}

/// Install a version with architecture fallback.
///
/// Iterates `archs`, calling `attempt(arch)` for each. On success returns the
/// resolved version string. On failure with remaining archs, logs a fallback
/// warning and tries the next arch. On final failure, bails with a descriptive
/// message.
pub(crate) fn install_with_fallback(
    lang_name: &str,
    resolved: &str,
    os: &str,
    native_arch: &str,
    archs: &[&str],
    is_installed: &dyn Fn() -> bool,
    attempt: &mut dyn FnMut(&str) -> Result<()>,
) -> Result<String> {
    for (i, &arch) in archs.iter().enumerate() {
        if i > 0 && is_installed() {
            return Ok(resolved.to_string());
        }

        if arch != native_arch {
            report_non_native_arch(os, arch);
        }

        match attempt(arch) {
            Ok(()) => return Ok(resolved.to_string()),
            Err(_e) if i + 1 < archs.len() => report_fallback(arch, archs[i + 1]),
            Err(e) => return Err(e),
        }
    }

    bail!("Failed to install {lang_name} {resolved}")
}
