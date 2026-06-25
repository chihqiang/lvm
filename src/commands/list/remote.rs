use crate::language::LanguageRegistry;
use std::collections::HashSet;

use crate::commands::{get_language, output};
use crate::config;
use anyhow::Result;

fn extract_version(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

/// 列出远程可用版本，支持仅显示 LTS
pub(crate) fn list_remote(
    registry: &LanguageRegistry,
    language: &str,
    lts_only: bool,
) -> Result<()> {
    let lang = get_language(registry, language)?;
    let remote_versions = lang.list_remote_versions()?;
    let installed = lang.list_installed()?;
    let installed_versions: HashSet<&str> =
        installed.iter().map(std::string::String::as_str).collect();

    let lts_marker = config::lts_marker();
    let use_color = config::use_color();

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
                config::color_green()
            } else if is_lts {
                config::color_cyan()
            } else {
                ""
            };
            let reset = if is_installed || is_lts {
                config::color_reset()
            } else {
                ""
            };
            let check = if is_installed {
                format!(" {}", config::colored_check_mark())
            } else {
                String::new()
            };
            println!("{color}{version}{check}{reset}");
        } else {
            let check = if is_installed {
                format!(" {}", config::installed_check_mark())
            } else {
                String::new()
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
