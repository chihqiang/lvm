use crate::language::LanguageRegistry;

use crate::commands::get_language;
use anyhow::{Context, Result};

pub(crate) fn which(registry: &LanguageRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_language(registry, language)?;
    let ver = match version {
        "current" => p
            .current_version()?
            .with_context(|| format!("No active version for {language}"))?,
        v => v.to_string(),
    };
    let path = p.binary_path(&ver)?;
    println!("{path}");
    Ok(())
}
