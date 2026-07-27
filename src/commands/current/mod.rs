use lvm::language::LanguageRegistry;

use crate::commands::{get_language, output};
use anyhow::Result;

/// Show the currently active version
pub(crate) fn current(registry: &LanguageRegistry, language: &str) -> Result<()> {
    let lang = get_language(registry, language)?;
    let prefix = lang.version_prefix();
    match lang.current_version()? {
        Some(ver) => output::info(format!("{language}: {prefix}{ver} (current)")),
        None => output::info(format!("No active version for {language}")),
    }
    Ok(())
}

/// Show current versions for all registered languages
pub(crate) fn current_all(registry: &LanguageRegistry) -> Result<()> {
    let names = registry.list_names();
    if names.is_empty() {
        output::info("No languages registered");
        return Ok(());
    }
    let mut errors = Vec::new();
    for name in names {
        if let Err(e) = current(registry, name) {
            errors.push(format!("{}: {e}", name));
        }
    }
    if !errors.is_empty() {
        anyhow::bail!("Some languages failed:\n  {}", errors.join("\n  "));
    }
    Ok(())
}
