use lvm::language::LanguageRegistry;

use crate::commands::{flush, get_language, try_resolve_installed_local};
use anyhow::Result;

pub(crate) fn uninstall(registry: &LanguageRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_language(registry, language)?;
    // Resolve partial versions (e.g. "1.80" → "1.80.1") against installed
    // versions, so uninstalling by the same shorthand used to install works.
    // Falls back to the literal version if it isn't a known installed version
    // (uninstall_version will then report it as not installed).
    let resolved = try_resolve_installed_local(p, version)?.unwrap_or_else(|| version.to_string());
    let result = p.uninstall(&resolved);
    flush();
    result
}
