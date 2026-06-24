use std::fs;
use std::path::Path;

use crate::config;
use crate::plugin::report::report;
use crate::plugin::version::compare_versions;
use anyhow::{Context, Result, anyhow, bail};

pub(crate) fn create_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    let _ = fs::remove_file(dst);
    #[cfg(unix)]
    return std::os::unix::fs::symlink(src, dst);
    #[cfg(windows)]
    {
        if src.is_dir() {
            std::os::windows::fs::symlink_dir(src, dst)
        } else {
            std::os::windows::fs::symlink_file(src, dst)
        }
    }
    #[cfg(not(any(unix, windows)))]
    compile_error!("unsupported platform");
}

pub(crate) fn remove_symlink(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    return fs::remove_file(path);
    #[cfg(windows)]
    {
        let meta = path.symlink_metadata()?;
        if meta.is_dir() {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    }
    #[cfg(not(any(unix, windows)))]
    compile_error!("unsupported platform");
}

pub(crate) fn exe_suffix() -> &'static str {
    std::env::consts::EXE_SUFFIX
}

pub(crate) fn path_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

pub(crate) fn archive_ext() -> &'static str {
    #[cfg(windows)]
    {
        "zip"
    }
    #[cfg(not(windows))]
    {
        "tar.gz"
    }
}

pub(crate) fn current_version_from_link(link: &Path, prefix: &str) -> Option<String> {
    let target = fs::read_link(link).ok()?;
    if !target.exists() {
        return None;
    }
    let dir_name = target.file_name()?;
    let name = dir_name.to_str()?;
    Some(name.trim_start_matches(prefix).to_string())
}

pub(crate) fn list_installed_versions(
    dir: &Path,
    prefix: &str,
    is_installed: fn(&Path) -> bool,
) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut versions = Vec::new();
    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read entry")?;
        if entry.file_type().is_ok_and(|t| t.is_dir())
            && let Some(name) = entry.file_name().to_str()
            && let Some(ver) = name.strip_prefix(prefix)
            && is_installed(&entry.path())
        {
            versions.push(ver.to_string());
        }
    }
    versions.sort_by(|a, b| compare_versions(a, b));
    Ok(versions)
}

pub(crate) fn format_installed_versions(
    prefix: &str,
    current: Option<String>,
    default_ver: Option<String>,
    versions: &[String],
) -> Vec<String> {
    versions
        .iter()
        .map(|v| {
            let is_current = current.as_deref() == Some(v.as_str());
            let is_default = default_ver.as_deref() == Some(v.as_str());
            let suffix = match (is_current, is_default) {
                (true, true) => " (current, default)",
                (true, false) => " (current)",
                (false, true) => " (default)",
                (false, false) => "",
            };
            format!("{prefix}{v}{suffix}")
        })
        .collect()
}

pub(crate) fn uninstall_version(
    version_dir: &Path,
    current_link: &Path,
    bin_link: &Path,
    current: Option<String>,
    version: &str,
) -> Result<()> {
    if !version_dir.exists() {
        bail!("{version} is not installed")
    }
    if current.as_deref() == Some(version) {
        let _ = remove_symlink(current_link);
        let _ = remove_symlink(bin_link);
    }
    fs::remove_dir_all(version_dir).context("Failed to uninstall")?;
    report(format!("{version} uninstalled"));
    Ok(())
}

pub(crate) fn binary_path_in_dir(
    version_dir: &Path,
    bin_name: &str,
    version: &str,
) -> Result<String> {
    let bin =
        version_dir
            .join(config::bin_dir_name())
            .join(format!("{}{}", bin_name, exe_suffix()));
    if bin.exists() {
        Ok(bin.to_string_lossy().to_string())
    } else {
        bail!("{version} is not installed")
    }
}

pub(crate) fn use_version_symlinks(
    version_dir: &Path,
    current_link: &Path,
    bin_link: &Path,
    bin_name: &str,
) -> Result<()> {
    let link_parent = current_link
        .parent()
        .ok_or_else(|| anyhow!("current_link must have a parent"))?;
    fs::create_dir_all(link_parent).context("Failed to create symlink directory")?;

    create_symlink(version_dir, current_link).context("Failed to create symlink")?;

    let bin_parent = bin_link
        .parent()
        .ok_or_else(|| anyhow!("bin_link must have a parent"))?;
    fs::create_dir_all(bin_parent).context("Failed to create symlink directory")?;
    let bin_target =
        current_link
            .join(config::bin_dir_name())
            .join(format!("{}{}", bin_name, exe_suffix()));
    if let Err(e) = create_symlink(&bin_target, bin_link) {
        let _ = fs::remove_file(current_link);
        bail!("Failed to create symlink: {e}")
    }
    Ok(())
}
