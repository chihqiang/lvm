use std::fs;
use std::path::Path;

use crate::config;
use crate::core::report::report;
use anyhow::{Context, Result, bail};

pub(crate) fn create_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
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

fn temp_symlink_path(dst: &Path) -> Result<std::path::PathBuf> {
    let parent = dst.parent().context("symlink path must have a parent")?;
    let name = dst
        .file_name()
        .context("symlink path must have a file name")?
        .to_string_lossy();
    Ok(parent.join(format!(".{name}.tmp-{}", std::process::id())))
}

pub(crate) fn replace_symlink(src: &Path, dst: &Path) -> Result<()> {
    let tmp = temp_symlink_path(dst)?;
    if tmp.symlink_metadata().is_ok()
        && let Err(e) = remove_symlink(&tmp)
    {
        bail!("Failed to remove stale symlink {}: {e}", tmp.display());
    }

    create_symlink(src, &tmp)
        .with_context(|| format!("Failed to create temporary symlink {}", tmp.display()))?;

    #[cfg(unix)]
    let result = fs::rename(&tmp, dst);

    #[cfg(windows)]
    let result = {
        if dst.symlink_metadata().is_ok() {
            remove_symlink(dst)
        } else {
            Ok(())
        }
        .and_then(|_| fs::rename(&tmp, dst))
    };

    if let Err(e) = result {
        if let Err(cleanup_err) = remove_symlink(&tmp) {
            report(format!(
                "Warning: failed to cleanup symlink {}: {cleanup_err}",
                tmp.display()
            ));
        }
        bail!("Failed to replace symlink {}: {e}", dst.display());
    }

    Ok(())
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

pub(crate) const CURRENT_MARKER: &str = " (current)";
pub(crate) const DEFAULT_MARKER: &str = " (default)";
pub(crate) const CURRENT_DEFAULT_MARKER: &str = " (current, default)";

pub(crate) fn format_installed_versions(
    prefix: &str,
    current: Option<&str>,
    default_ver: Option<&str>,
    versions: &[String],
) -> Vec<String> {
    versions
        .iter()
        .map(|v| {
            let is_current = current == Some(v.as_str());
            let is_default = default_ver == Some(v.as_str());
            let suffix = match (is_current, is_default) {
                (true, true) => CURRENT_DEFAULT_MARKER,
                (true, false) => CURRENT_MARKER,
                (false, true) => DEFAULT_MARKER,
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
    current: Option<&str>,
    version: &str,
) -> Result<()> {
    if !version_dir.exists() {
        bail!("{version} is not installed")
    }
    if current == Some(version) {
        if let Err(e) = remove_symlink(current_link) {
            report(format!(
                "Warning: failed to remove symlink {}: {e}",
                current_link.display()
            ));
        }
        if let Err(e) = remove_symlink(bin_link) {
            report(format!(
                "Warning: failed to remove symlink {}: {e}",
                bin_link.display()
            ));
        }
    }
    fs::remove_dir_all(version_dir).context("Failed to uninstall")?;
    report(format!("{version} uninstalled"));
    Ok(())
}

pub(crate) fn use_version_symlinks(
    version_dir: &Path,
    current_link: &Path,
    bin_link: &Path,
    bin_name: &str,
) -> Result<()> {
    let link_parent = current_link
        .parent()
        .context("current_link must have a parent")?;
    fs::create_dir_all(link_parent).context("Failed to create symlink directory")?;

    let bin_parent = bin_link.parent().context("bin_link must have a parent")?;
    fs::create_dir_all(bin_parent).context("Failed to create symlink directory")?;
    let bin_target =
        current_link
            .join(config::bin_dir_name())
            .join(format!("{}{}", bin_name, exe_suffix()));

    replace_symlink(&bin_target, bin_link).context("Failed to replace binary symlink")?;
    replace_symlink(version_dir, current_link).context("Failed to replace current symlink")?;

    Ok(())
}
