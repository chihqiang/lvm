use crate::plugin::PluginRegistry;
use std::io::IsTerminal;

use super::get_plugin;
use super::output;
use crate::config;
use anyhow::Result;

/// 列出本地已安装的语言版本
pub(crate) fn list(registry: &PluginRegistry, language: &str) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    let versions = plugin.list_installed()?;
    if versions.is_empty() {
        output::info("No versions installed");
    } else {
        let formatted = plugin.format_installed(&versions)?;
        let use_color = std::io::stdout().is_terminal();
        for v in &formatted {
            if use_color {
                let colored = if v.contains("(current") {
                    format!(
                        "{}{}{}",
                        config::display::color_green_bold(),
                        v,
                        config::display::color_reset()
                    )
                } else if v.contains("(default") {
                    format!(
                        "{}{}{}",
                        config::display::color_yellow(),
                        v,
                        config::display::color_reset()
                    )
                } else {
                    v.to_string()
                };
                println!("{colored}");
            } else {
                println!("{v}");
            }
        }
    }
    Ok(())
}
