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

use anyhow::{Result, bail};

pub use crate::core::{
    CURRENT_DEFAULT_MARKER, CURRENT_MARKER, DEFAULT_MARKER, archive_ext, download_and_install,
    exe_suffix, fetch_from_mirror, fetch_with_cache, flush_reports_to_stdout, get_url,
    parse_github_releases, path_separator, remove_symlink, report, report_already_installed,
    report_checksum_verified, report_fallback, report_non_native_arch, report_verifying_checksum,
    resolve_partial_version, resolve_version, set_offline, sort_versions, verify_sha256,
};
pub use language_trait::Language;
pub use registry::LanguageRegistry;

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
