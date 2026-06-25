use crate::config;
use crate::language::{self, LanguageRegistry};

use crate::commands::{flush, get_language, output};
use anyhow::Result;

/// 清理旧版本，保留最新 N 个（不含当前使用和 default 版本）
pub(crate) fn prune(registry: &LanguageRegistry, language: &str, keep: usize) -> Result<()> {
    let p = get_language(registry, language)?;
    let mut versions = p.list_installed()?;
    if versions.is_empty() {
        output::info("No versions installed");
        return Ok(());
    }

    language::sort_versions(&mut versions);
    versions.reverse();

    let current = p.current_version()?;
    let default = config::get_default_version(language)?;

    let to_remove: Vec<&str> = versions
        .iter()
        .skip(keep)
        .map(String::as_str)
        .filter(|v| current.as_deref() != Some(*v) && default.as_deref() != Some(*v))
        .collect();

    if to_remove.is_empty() {
        output::info(format!(
            "Only {}+ versions installed, nothing to prune",
            versions.len().min(keep)
        ));
        return Ok(());
    }

    output::info(format!(
        "Removing {} old {} version(s)...",
        to_remove.len(),
        language
    ));
    for v in to_remove {
        p.uninstall(v)?;
        flush();
    }
    Ok(())
}
