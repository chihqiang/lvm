use crate::plugin::PluginRegistry;

use super::get_plugin;
use super::output;
use anyhow::Result;

/// 显示当前使用的版本
pub(crate) fn current(registry: &PluginRegistry, language: &str) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    match plugin.current_version()? {
        Some(ver) => output::info(format!("{language}: v{ver} (current)")),
        None => output::info(format!("No active version for {language}")),
    }
    Ok(())
}

/// 显示所有已注册语言的当前版本，带表头
pub(crate) fn current_all(registry: &PluginRegistry) -> Result<()> {
    let names = registry.list_names();
    if names.is_empty() {
        output::info("No languages registered");
        return Ok(());
    }
    for name in names {
        current(registry, name)?;
    }
    Ok(())
}
