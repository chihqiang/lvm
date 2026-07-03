use lvm::core::alias;
use lvm::core::config;
use lvm::core::lvmrc;
use lvm::core::version::resolve_partial_version;
use lvm::language::{self, Language, LanguageRegistry};

use crate::commands::{flush, get_language, output};
use anyhow::{Context, Result};
use semver::Version;

/// Resolve a version against locally installed releases when possible,
/// avoiding network calls during `lvm use`.
fn try_resolve_installed_local(p: &dyn Language, version: &str) -> Result<Option<String>> {
    let candidate = version.trim().trim_start_matches('v');

    if let Ok(ver) = Version::parse(candidate) {
        let resolved = ver.to_string();
        if p.is_installed(&p.version_dir(&resolved)) {
            return Ok(Some(resolved));
        }
        return Ok(None);
    }

    let installed = p.list_installed()?;
    if installed.is_empty() {
        return Ok(None);
    }

    let avail: Vec<Version> = installed
        .iter()
        .filter_map(|s| Version::parse(s).ok())
        .collect();
    if avail.is_empty() {
        return Ok(None);
    }

    match resolve_partial_version(candidate, &avail, p.name()) {
        Ok(resolved) if p.is_installed(&p.version_dir(&resolved)) => Ok(Some(resolved)),
        Ok(_) | Err(_) => Ok(None),
    }
}

pub(crate) fn use_version(
    registry: &LanguageRegistry,
    language: &str,
    version: Option<&str>,
    set_default: bool,
) -> Result<()> {
    let p = get_language(registry, language)?;

    let version = match version {
        Some(v) => v.to_string(),
        None => {
            if let Some(v) = lvmrc::read_lvmrc_version(language)? {
                v
            } else if let Some(v) = p.rc_version()? {
                v
            } else if let Some(v) = alias::get_default_version(language)? {
                v
            } else {
                let latest = p.latest_version()?;
                output::info(format!("Using latest {language} version {latest}"));
                latest
            }
        }
    };

    if version == config::SYSTEM_VERSION_KEYWORD {
        language::remove_symlink(&p.current_link())
            .with_context(|| format!("Failed to remove {}", p.current_link().display()))?;
        language::remove_symlink(&p.bin_link())
            .with_context(|| format!("Failed to remove {}", p.bin_link().display()))?;
        output::info(format!("Using system {language}"));
        return Ok(());
    }

    if let Some(resolved) = try_resolve_installed_local(p, &version)? {
        p.use_version(&resolved, set_default)?;
        flush();
        return Ok(());
    }

    let installed = p.install(Some(&version))?;
    flush();
    p.use_version(&installed, set_default)?;
    flush();
    Ok(())
}
