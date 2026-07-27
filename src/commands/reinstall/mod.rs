use std::process::Command;

use lvm::language;
use lvm::language::LanguageRegistry;

use crate::commands::{binary_dir, get_language, output};
use anyhow::{Context, Result, bail};

pub(crate) fn reinstall_packages(
    registry: &LanguageRegistry,
    language: &str,
    from_version: &str,
) -> Result<()> {
    let from_version = from_version.trim_start_matches('v');
    if from_version.is_empty() || !from_version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        bail!("Invalid version '{from_version}' for --reinstall-packages-from");
    }
    let lang = get_language(registry, language)?;

    let Some(pkg_manager) = lang.package_manager_binary() else {
        output::info(format!(
            "--reinstall-packages-from is not supported for {language}"
        ));
        return Ok(());
    };
    let Some(modules_dir) = lang.packages_dir_name() else {
        return Ok(());
    };

    let from_bin_dir = binary_dir(registry, language, from_version)?;
    let pkg_mgr_path = from_bin_dir.join(format!("{}{}", pkg_manager, language::exe_suffix()));
    if !pkg_mgr_path.exists() {
        output::info(format!(
            "No {pkg_manager} found in {from_version}, skipping package reinstallation"
        ));
        return Ok(());
    }

    let output = Command::new(pkg_mgr_path)
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
    let packages: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let normalized = line.trim().replace('\\', "/");
            let path = std::path::Path::new(&normalized);
            if !path.components().any(|c| c.as_os_str() == modules_dir) {
                return None;
            }
            let rev_parts: Vec<_> = path
                .components()
                .rev()
                .take_while(|c| c.as_os_str() != modules_dir)
                .collect();
            if rev_parts.is_empty() {
                return None;
            }
            let name = rev_parts
                .into_iter()
                .rev()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .join("/");
            if name == pkg_manager || name == "corepack" {
                return None;
            }
            Some(name.to_string())
        })
        .collect();

    if packages.is_empty() {
        output::info("No global packages to reinstall");
        return Ok(());
    }

    let to_bin_dir = if let Some(cur) = lang.current_version()? {
        let bin_path = lang.binary_path(&cur)?;
        std::path::Path::new(&bin_path)
            .parent()
            .map_or_else(|| from_bin_dir.clone(), std::path::Path::to_path_buf)
    } else {
        from_bin_dir
    };

    output::info(format!(
        "Reinstalling {} global package(s)...",
        packages.len()
    ));
    let status =
        Command::new(to_bin_dir.join(format!("{}{}", pkg_manager, language::exe_suffix())))
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
