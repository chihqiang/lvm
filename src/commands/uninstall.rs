use crate::plugin::PluginRegistry;

use super::get_plugin;
use super::output;
use anyhow::Result;

/// 卸载指定语言版本
pub(crate) fn uninstall(registry: &PluginRegistry, language: &str, version: &str) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    let result = plugin.uninstall(version);
    output::flush_plugin();
    result
}
