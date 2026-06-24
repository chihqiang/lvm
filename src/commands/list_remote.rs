use crate::plugin::PluginRegistry;
use std::collections::HashSet;
use std::io::IsTerminal;

use super::get_plugin;
use super::output;
use crate::config;
use anyhow::Result;

fn extract_version(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

/// 列出远程可用版本，支持仅显示 LTS
pub(crate) fn list_remote(registry: &PluginRegistry, language: &str, lts_only: bool) -> Result<()> {
    let plugin = get_plugin(registry, language)?;
    let remote_versions = plugin.list_remote_versions()?;
    let installed = plugin.list_installed()?;
    let installed_versions: HashSet<&str> = installed.iter().map(|s| s.as_str()).collect();

    let lts_marker = config::display::lts_marker();
    let use_color = std::io::stdout().is_terminal();
    let colored_check = use_color.then(|| format!(" {}", config::display::colored_check_mark()));
    let plain_check = (!use_color).then(|| format!(" {}", config::display::installed_check_mark()));

    let mut count = 0u32;
    for version in remote_versions
        .iter()
        .filter(|v| !lts_only || v.contains(lts_marker))
    {
        count += 1;
        let plain = extract_version(version);
        let is_installed = installed_versions.contains(plain);
        let is_lts = version.contains(lts_marker);
        if use_color {
            let color = if is_installed {
                config::display::color_green()
            } else if is_lts {
                config::display::color_cyan()
            } else {
                ""
            };
            let reset = if is_installed || is_lts {
                config::display::color_reset()
            } else {
                ""
            };
            let check = if is_installed {
                colored_check.as_deref().unwrap_or("")
            } else {
                ""
            };
            println!("{color}{version}{check}{reset}");
        } else {
            let check = if is_installed {
                plain_check.as_deref().unwrap_or("")
            } else {
                ""
            };
            println!("{version}{check}");
        }
    }

    if count == 0 && lts_only {
        output::info("No LTS versions available");
    } else if count == 0 {
        output::info("No remote versions available");
    }
    Ok(())
}
