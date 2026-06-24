use std::process::Command;

use crate::plugin;
use crate::plugin::PluginRegistry;

use super::{binary_dir, get_plugin, output};
use anyhow::{Context, Result, bail};

/// 从指定版本迁移全局 npm 包到当前版本
pub(crate) fn reinstall_packages(
    registry: &PluginRegistry,
    language: &str,
    from_version: &str,
) -> Result<()> {
    let from_bin_dir = binary_dir(registry, language, from_version)?;
    let npm_path = from_bin_dir.join(format!("npm{}", plugin::exe_suffix()));
    if !npm_path.exists() {
        output::info(format!(
            "No npm found in {from_version}, skipping package reinstallation"
        ));
        return Ok(());
    }

    let output = Command::new(npm_path)
        .args(["list", "-g", "--depth=0", "--parseable"])
        .output()
        .context("Failed to list global packages")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        output::warn(format!(
            "Failed to list global packages from {from_version}: {stderr}"
        ));
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let node_modules_dir = crate::plugin::node::node_modules_dir();
    let npm_name = crate::plugin::node::npm_binary_name();
    let needle = format!("/{node_modules_dir}/");
    let packages: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            // npm --parseable 在不同平台输出不同路径分隔符，统一转为 / 再解析
            let normalized = line.trim().replace('\\', "/");
            if normalized.is_empty() || !normalized.contains(&needle) {
                return None;
            }
            let name = normalized.rsplit(&needle).next()?;
            if name == npm_name || name == "corepack" {
                return None;
            }
            Some(name.to_string())
        })
        .collect();

    if packages.is_empty() {
        output::info("No global packages to reinstall");
        return Ok(());
    }

    // 用当前（新安装的）版本的 npm 来安装
    let plugin = get_plugin(registry, language)?;
    let to_bin_dir = if let Some(cur) = plugin.current_version()? {
        let bin_path = plugin.binary_path(&cur)?;
        std::path::Path::new(&bin_path)
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or(from_bin_dir.clone())
    } else {
        // 没有当前版本，退而用 from 版本的 npm
        from_bin_dir.clone()
    };

    output::info(format!(
        "Reinstalling {} global package(s)...",
        packages.len()
    ));
    let status = Command::new(to_bin_dir.join(crate::plugin::node::npm_binary_name()))
        .args(["install", "-g", "--quiet"])
        .args(&packages)
        .status()
        .context("Failed to install packages")?;

    if !status.success() {
        bail!("Failed to reinstall some packages");
    }

    output::info("Global packages reinstalled");
    Ok(())
}
