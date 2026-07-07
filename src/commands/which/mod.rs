use lvm::language::LanguageRegistry;

use crate::commands::{get_language, try_resolve_installed_local};
use anyhow::{Context, Result};

pub(crate) fn which(registry: &LanguageRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_language(registry, language)?;
    let ver = match version {
        "current" => p
            .current_version()?
            .with_context(|| format!("No active version for {language}"))?,
        v => {
            // Try to resolve partial version against installed versions first
            if let Some(resolved) = try_resolve_installed_local(p, v)? {
                resolved
            } else {
                v.to_string()
            }
        }
    };
    let path = p.binary_path(&ver)?;
    println!("{path}");
    Ok(())
}
