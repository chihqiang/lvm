use crate::plugin::PluginRegistry;

use super::get_plugin;
use anyhow::Result;

/// 显示指定版本可执行文件的路径
pub(crate) fn which(registry: &PluginRegistry, language: &str, version: &str) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    let path = plugin.binary_path(version)?;
    println!("{path}");
    Ok(())
}
