use lvm::language::LanguageRegistry;

use crate::commands::{flush, get_language};
use anyhow::Result;

pub(crate) fn install(
    registry: &LanguageRegistry,
    language: &str,
    version: Option<&str>,
    no_default: bool,
) -> Result<()> {
    let p = get_language(registry, language)?;
    let installed_version = p.install(version)?;
    flush();
    p.use_version(&installed_version, !no_default)?;
    flush();
    p.post_install(&installed_version)?;
    flush();
    Ok(())
}
