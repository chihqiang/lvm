use crate::config;
use crate::language::{self, LanguageRegistry};

use crate::commands::{flush, get_language, output};
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
            if let Some(v) = config::read_lvmrc_version(language)? {
                v
            } else if let Some(v) = config::get_default_version(language)? {
                v
            } else {
                let latest = p.latest_version()?;
                output::info(format!("Using latest {language} version {latest}"));
                latest
            }
        }
    };

    if version == config::system_keyword() {
        language::remove_symlink(&p.current_link())
            .with_context(|| format!("Failed to remove {}", p.current_link().display()))?;
        language::remove_symlink(&p.bin_link())
            .with_context(|| format!("Failed to remove {}", p.bin_link().display()))?;
        output::info(format!("Using system {language}"));
        return Ok(());
    }

    let installed = p.install(Some(&version))?;
    flush();
    p.use_version(&installed, set_default)?;
    flush();
    Ok(())
}
