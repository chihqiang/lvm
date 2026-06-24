use crate::plugin::PluginRegistry;

use super::get_plugin;
use super::output;
use anyhow::Result;

/// 安装指定语言版本，可选择是否设为默认
pub(crate) fn install(
    registry: &PluginRegistry,
    language: &str,
    version: Option<&str>,
    no_default: bool,
) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    let installed_version = plugin.install(version)?;
    output::flush_plugin();
    plugin.use_version(&installed_version, !no_default)?;
    output::flush_plugin();
    plugin.post_install(&installed_version)?;
    output::flush_plugin();
    Ok(())
}
