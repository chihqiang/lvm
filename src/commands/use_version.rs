use crate::config;
use crate::plugin::PluginRegistry;

use super::get_plugin;
use super::output;
use crate::plugin;
use anyhow::Result;

/// 切换使用指定版本，自动安装缺失版本
pub(crate) fn use_version(
    registry: &PluginRegistry,
    language: &str,
    version: Option<&str>,
    set_default: bool,
) -> Result<()> {
    let plugin = get_plugin(registry, language)?;

    let version = match version {
        Some(v) => v.to_string(),
        None => {
            if let Some(v) = config::lvmrc::read_lvmrc_version(language)? {
                v
            } else if let Some(v) = config::alias::get_default_version(language)? {
                v
            } else {
                let latest = plugin.latest_version()?;
                output::info(format!("Using latest {language} version {latest}"));
                latest
            }
        }
    };

    if version == config::display::system_keyword() {
        let plugin_name = plugin.name();
        let link = config::lvm_home()
            .join(config::current_dir_name())
            .join(plugin_name);
        let _ = plugin::remove_symlink(&link);
        let _ = plugin::remove_symlink(
            &config::lvm_home()
                .join(config::bin_dir_name())
                .join(plugin_name),
        );
        output::info(format!("Using system {language}"));
        return Ok(());
    }

    let installed = plugin.install(Some(&version))?;
    output::flush_plugin();
    plugin.use_version(&installed, set_default)?;
    output::flush_plugin();
    Ok(())
}
