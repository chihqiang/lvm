use lvm::core::alias;
use lvm::core::config;
use lvm::core::lvmrc;
use lvm::language::{self, LanguageRegistry};

use crate::commands::{flush, get_language, output, try_resolve_installed_local};
use anyhow::{Context, Result};

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

    output::info(format!("{language} {version} is not installed, installing..."));
    let installed = p.install(Some(&version))?;
    flush();
    p.use_version(&installed, set_default)?;
    flush();
    Ok(())
}
