//! Go 语言插件
//! 实现 Plugin trait，处理 Go 版本的安装、版本切换、校验等操作

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};

use crate::config;
use super as plugin;
use super::Plugin;

// ─── Go 专用配置 ───

static GO_MIRROR: OnceLock<String> = OnceLock::new();

fn go_mirror() -> &'static str {
    GO_MIRROR.get_or_init(|| {
        env::var("LVM_GO_MIRROR").unwrap_or_else(|_| "https://go.dev/dl".to_string())
    })
}

fn version_prefix() -> &'static str {
    "v"
}

pub(crate) fn subdir() -> &'static str {
    "go"
}

fn current_subdir() -> &'static str {
    "current/go"
}

fn tarball_filename(version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("go{version}.{os}-{arch}.{ext}")
}

fn download_url(mirror: &str, version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("{mirror}/go{version}.{os}-{arch}.{ext}")
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        "x86" => "386",
        other => other,
    }
}

/// Go per-version 包目录路径组件
pub(crate) fn go_packages_bin_path() -> (&'static str, &'static str) {
    ("packages", "bin")
}

// ─── Go 二进制/缓存常量 ───

fn go_versions_cache_filename() -> &'static str {
    "go-versions.json"
}

fn go_versions_query_suffix() -> &'static str {
    "/?mode=json&include=all"
}

// ─── Go 提供者 ───

fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

pub struct GoPlugin;

impl GoPlugin {
    fn lvm_dir() -> PathBuf {
        config::lvm_home().join(subdir())
    }

    fn current_link() -> PathBuf {
        config::lvm_home().join(current_subdir())
    }

    fn bin_link() -> PathBuf {
        config::lvm_home()
            .join(config::bin_dir_name())
            .join(format!("go{}", plugin::exe_suffix()))
    }

    fn version_dir(version: &str) -> PathBuf {
        let prefix = version_prefix();
        Self::lvm_dir().join(format!("{prefix}{}", version.trim_start_matches(prefix)))
    }

    fn download_url(version: &str) -> String {
        download_url(
            go_mirror(),
            version,
            target_os(),
            target_arch(),
            plugin::archive_ext(),
        )
    }

    fn cached_tar(version: &str) -> PathBuf {
        config::downloads_dir().join(tarball_filename(
            version,
            target_os(),
            target_arch(),
            plugin::archive_ext(),
        ))
    }

    fn is_installed(version_dir: &Path) -> bool {
        version_dir
            .join(config::bin_dir_name())
            .join(format!("go{}", plugin::exe_suffix()))
            .exists()
    }

    fn current_version() -> Option<String> {
        plugin::current_version_from_link(&Self::current_link(), version_prefix())
    }

    fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions
            .last()
            .cloned()
            .context("No stable Go versions found")
    }

    fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir().join(go_versions_cache_filename());

        if let Ok(meta) = fs::metadata(&cache_file)
            && let Ok(modified) = meta.modified()
            && let Ok(elapsed) = modified.elapsed()
            && elapsed < config::cache_ttl()
        {
            let text = fs::read_to_string(&cache_file).context("Failed to read cache")?;
            return Self::parse_versions_json(&text);
        }

        let url = format!("{}{}", go_mirror(), go_versions_query_suffix());
        let response = plugin::get_url(&url)
            .call()
            .context("Failed to fetch Go versions")?;
        let text = response.into_string().context("Failed to read response")?;

        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cache_file, &text);

        Self::parse_versions_json(&text)
    }

    fn parse_versions_json(text: &str) -> Result<Vec<String>> {
        let versions: Vec<serde_json::Value> =
            serde_json::from_str(text).context("Failed to parse Go versions")?;

        let mut result: Vec<String> = versions
            .iter()
            .filter(|entry| {
                entry
                    .get("stable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .filter_map(|entry| {
                let v = entry.get("version")?.as_str()?;
                v.strip_prefix("go").map(String::from)
            })
            .collect();

        result.sort_by(|a, b| plugin::compare_versions(a, b));
        Ok(result)
    }
}

impl Plugin for GoPlugin {
    fn name(&self) -> &str {
        subdir()
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = if let Some(v) = version {
            let v = v.trim();
            if let Ok(ver) = semver::Version::parse(v) {
                ver.to_string()
            } else {
                let avail = Self::fetch_all_versions()?;
                plugin::resolve_partial_version(v, &avail, "Go")?
            }
        } else {
            Self::fetch_latest_version()?
        };

        let version_dir = Self::version_dir(&resolved);
        if Self::is_installed(&version_dir) {
            plugin::report(format!("Go {resolved} is already installed"));
            return Ok(resolved);
        }

        fs::create_dir_all(&version_dir).context("Failed to create version directory")?;

        let tar_path = Self::cached_tar(&resolved);

        let verify_go_checksum = |tar_path: &Path| -> Result<()> {
            let dl_url = Self::download_url(&resolved);
            let checksum_url = format!("{dl_url}.sha256");
            let resp = plugin::get_url(&checksum_url)
                .call()
                .context("Failed to fetch checksum")?;
            let expected = resp.into_string().context("Failed to read checksum")?;
            plugin::report("Verifying checksum...");
            plugin::verify_sha256(tar_path, expected.trim())?;
            plugin::report("Checksum verified");
            Ok(())
        };

        plugin::download_and_install(
            &Self::download_url(&resolved),
            &tar_path,
            &resolved,
            &version_dir,
            "Go",
            verify_go_checksum,
        )?;
        Ok(resolved)
    }

    fn uninstall(&self, version: &str) -> Result<()> {
        let prefix = version_prefix();
        let version = version.trim_start_matches(prefix);
        plugin::uninstall_version(
            &Self::version_dir(version),
            &Self::current_link(),
            &Self::bin_link(),
            Self::current_version(),
            version,
        )
        .context("Failed to uninstall Go version")
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        plugin::list_installed_versions(&Self::lvm_dir(), version_prefix(), Self::is_installed)
            .context("Failed to list installed Go versions")
    }

    fn format_installed(&self, versions: &[String]) -> Result<Vec<String>> {
        Ok(plugin::format_installed_versions(
            version_prefix(),
            Self::current_version(),
            config::alias::get_default_version(self.name())
                .ok()
                .flatten(),
            versions,
        ))
    }

    fn current_version(&self) -> Result<Option<String>> {
        Ok(Self::current_version())
    }

    fn use_version(&self, version: &str, set_default: bool) -> Result<()> {
        let prefix = version_prefix();
        let version = version.trim_start_matches(prefix);
        let version_dir = Self::version_dir(version);

        if !Self::is_installed(&version_dir) {
            bail!("Go {version} is not installed");
        }

        plugin::use_version_symlinks(
            &version_dir,
            &Self::current_link(),
            &Self::bin_link(),
            "go",
        )?;

        // 创建 per-version 包目录用于数据隔离
        let (packages_dir, bin_dir_name) = go_packages_bin_path();
        let packages_bin = version_dir.join(packages_dir).join(bin_dir_name);
        fs::create_dir_all(&packages_bin).context("Failed to create Go packages directory")?;

        if set_default {
            config::alias::set_default_version(self.name(), version)
                .context("Failed to write default config")?;
        }

        plugin::report(format!("Switched to Go {version}!"));
        Ok(())
    }

    fn binary_path(&self, version: &str) -> Result<String> {
        let prefix = version_prefix();
        let version = version.trim_start_matches(prefix);
        plugin::binary_path_in_dir(&Self::version_dir(version), "go", version)
            .context("Failed to get binary path")
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }
}
