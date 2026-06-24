use crate::plugin::PluginRegistry;

use super::get_plugin;
use anyhow::{Context, Result};

pub(crate) fn which(registry: &PluginRegistry, language: &str, version: &str) -> Result<()> {
    let p = get_plugin(registry, language)?;
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
