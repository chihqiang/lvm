//! Node.js 语言插件
//! 实现 Plugin trait，处理 Node.js 版本的安装、列出现有版本、切换版本等操作

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};

use semver::Version;

use super as plugin;
use super::Plugin;
use crate::config;

// ─── Node.js 专用配置 ───

static NODE_MIRROR: OnceLock<String> = OnceLock::new();

fn node_mirror() -> &'static str {
    NODE_MIRROR.get_or_init(|| {
        env::var("LVM_NODE_MIRROR").unwrap_or_else(|_| "https://nodejs.org/dist".to_string())
    })
}

fn version_prefix() -> &'static str {
    "v"
}

fn subdir() -> &'static str {
    "node"
}

fn current_subdir() -> &'static str {
    "current/node"
}

fn tarball_filename(version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("node-v{version}-{os}-{arch}.{ext}")
}

fn download_url(mirror: &str, version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("{mirror}/v{version}/node-v{version}-{os}-{arch}.{ext}")
}

fn latest_version_path() -> &'static str {
    "latest/SHASUMS256.txt"
}

fn index_tab_path() -> &'static str {
    "index.tab"
}

fn tarball_prefix() -> &'static str {
    "node-v"
}

fn default_packages_filename() -> &'static str {
    "default-packages"
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "win",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "x86" => "x86",
        other => other,
    }
}

// ─── Node 二进制/缓存常量 ───

pub(crate) fn node_binary_name() -> &'static str {
    "node"
}

pub(crate) fn npm_binary_name() -> &'static str {
    "npm"
}

fn node_versions_cache_filename() -> &'static str {
    "node-versions.tab"
}

pub(crate) fn node_modules_dir() -> &'static str {
    "node_modules"
}

// ─── Node 提供者 ───

struct LtsInfo {
    latest: String,
    name_to_ver: HashMap<String, String>,
    ordered: Vec<(String, Option<String>)>,
}

fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

/// Node.js 语言插件
pub struct NodePlugin;

impl NodePlugin {
    /// Node 版本的存储目录 (~/.lvm/node/)
    fn lvm_dir() -> PathBuf {
        config::lvm_home().join(subdir())
    }

    /// 当前使用版本的符号链接路径 (~/.lvm/current/node)
    fn current_link() -> PathBuf {
        config::lvm_home().join(current_subdir())
    }

    /// 当前版本的可执行文件符号链接 (~/.lvm/bin/node → current/node/bin/node)
    fn bin_link() -> PathBuf {
        config::lvm_home()
            .join(config::bin_dir_name())
            .join(format!("node{}", plugin::exe_suffix()))
    }

    /// 指定版本的安装目录 (~/.lvm/node/v{version}/)
    fn version_dir(version: &str) -> PathBuf {
        let prefix = version_prefix();
        Self::lvm_dir().join(format!("{prefix}{}", version.trim_start_matches(prefix)))
    }

    /// 构造 Node.js 下载 URL
    fn download_url(version: &str) -> String {
        download_url(
            node_mirror(),
            version,
            target_os(),
            target_arch(),
            plugin::archive_ext(),
        )
    }

    /// 下载文件缓存路径
    fn cached_tar(version: &str) -> PathBuf {
        config::downloads_dir().join(Self::tarball_filename(version))
    }

    /// 构造下载文件名（用于临时缓存）
    fn tarball_filename(version: &str) -> String {
        tarball_filename(version, target_os(), target_arch(), plugin::archive_ext())
    }

    fn fetch_text(url_path: &str) -> Result<String> {
        plugin::fetch_from_mirror(node_mirror(), url_path)
            .context("Failed to fetch text from mirror")
    }

    /// 从镜像源获取最新稳定版版本号
    /// 通过解析 SHASUMS256.txt 中的 node-v*.*.*-*.tar.gz 文件名提取版本号
    fn fetch_latest_version() -> Result<String> {
        let text = Self::fetch_text(latest_version_path())?;

        for line in text.lines() {
            if let Some(filename) = line.split_whitespace().nth(1)
                && let Some(version) = filename
                    .strip_prefix(tarball_prefix())
                    .and_then(|s| s.split('-').next())
            {
                return Ok(version.to_string());
            }
        }

        bail!("Could not find latest version from mirror")
    }

    /// 获取镜像源中所有可用的版本列表（不带前缀），用于解析部分版本号
    /// 结果缓存到本地 ~/.lvm/cache/node-versions.tab
    fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir().join(node_versions_cache_filename());

        if let Ok(meta) = fs::metadata(&cache_file)
            && let Ok(modified) = meta.modified()
            && let Ok(elapsed) = modified.elapsed()
            && elapsed < config::cache_ttl()
        {
            let text = fs::read_to_string(&cache_file).context("Failed to read response")?;
            return Ok(Self::parse_index_tab(&text));
        }

        let text = Self::fetch_text(index_tab_path())?;

        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cache_file, &text);

        Ok(Self::parse_index_tab(&text))
    }

    /// 解析 index.tab 内容，提取版本号列表
    fn parse_index_tab(text: &str) -> Vec<String> {
        let prefix = version_prefix();
        text.lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.is_empty() {
                    return None;
                }
                parts[0].strip_prefix(prefix).map(str::to_string)
            })
            .collect()
    }

    /// 解析 index.tab 中的 LTS 信息，返回 (version, lts_codename) 列表
    /// LTS 列为空表示非 LTS 版本
    fn parse_lts_info(text: &str) -> Vec<(String, Option<String>)> {
        let prefix = version_prefix();
        text.lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 8 {
                    return None;
                }
                let version = parts[0].strip_prefix(prefix)?;
                let lts = parts.get(7).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                Some((version.to_string(), lts))
            })
            .collect()
    }

    fn get_lts_info() -> Result<LtsInfo> {
        let text = Self::fetch_text(index_tab_path())?;
        let ordered = Self::parse_lts_info(&text);

        let mut name_to_ver: HashMap<String, String> = HashMap::new();

        for (ver, lts) in &ordered {
            if let Some(codename) = lts {
                let lower = codename.to_lowercase();
                name_to_ver.insert(lower, ver.clone());
            }
        }

        let latest = ordered
            .iter()
            .rev()
            .find(|(_, lts)| lts.is_some())
            .map(|(v, _)| v.clone())
            .unwrap_or_default();

        Ok(LtsInfo {
            latest,
            name_to_ver,
            ordered,
        })
    }

    /// 解析 LTS 版本描述符，如 "lts/*", "lts/argon", "lts/-1"
    fn resolve_lts(desc: &str) -> Result<String> {
        let info = Self::get_lts_info()?;

        if desc == "*" || desc.is_empty() {
            if info.latest.is_empty() {
                bail!("No LTS version found");
            }
            return Ok(info.latest);
        }

        if let Some(offset_str) = desc.strip_prefix('-')
            && let Ok(n) = offset_str.parse::<usize>()
        {
            let mut lts_versions: Vec<&str> = info
                .ordered
                .iter()
                .rev()
                .filter(|(_, lts)| lts.is_some())
                .map(|(v, _)| v.as_str())
                .collect();
            let mut seen = std::collections::HashSet::new();
            lts_versions.retain(|v| {
                let major = v.split('.').next().unwrap_or("");
                seen.insert(major.to_string())
            });
            if n < lts_versions.len() {
                return Ok(lts_versions[n].to_string());
            }
            bail!("LTS offset {n} is out of range");
        }

        let lower = desc.to_lowercase();
        if let Some(ver) = info.name_to_ver.get(&lower) {
            return Ok(ver.clone());
        }

        bail!("Unknown LTS release: {desc}")
    }

    /// 检查版本目录是否安装完整
    fn is_installed(version_dir: &Path) -> bool {
        version_dir
            .join(config::bin_dir_name())
            .join(format!("node{}", plugin::exe_suffix()))
            .exists()
    }

    /// 读取 ~/.lvm/current/node 符号链接，获取当前使用的版本号
    fn current_version() -> Option<String> {
        plugin::current_version_from_link(&Self::current_link(), version_prefix())
    }

    fn version_from_url(url: &str) -> Result<String> {
        let filename = url
            .rsplit('/')
            .next()
            .context(format!("Invalid Node tarball URL: {url}"))?;
        let version = filename
            .strip_prefix(tarball_prefix())
            .and_then(|s| s.split('-').next())
            .context(format!("Invalid Node tarball URL: {url}"))?;
        let prefix = version_prefix();
        let version = version.trim_start_matches(prefix);
        if Version::parse(version).is_err() {
            bail!("Invalid Node version in URL: {version}");
        }
        Ok(version.to_string())
    }
}

impl Plugin for NodePlugin {
    fn name(&self) -> &str {
        subdir()
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let prefix = version_prefix();

        let (download_url, resolved_version, source_is_url) = if let Some(v) = version {
            let v = v.trim();
            if v.starts_with("http://") || v.starts_with("https://") {
                let version = Self::version_from_url(v)?;
                (v.to_string(), version, true)
            } else if v.starts_with(config::lts_prefix()) {
                let desc = v.strip_prefix(config::lts_prefix()).unwrap_or("");
                let resolved = Self::resolve_lts(desc)?;
                let url = Self::download_url(&resolved);
                (url, resolved, false)
            } else {
                let candidate = v.trim_start_matches(prefix);
                let resolved = if Version::parse(candidate).is_ok() {
                    candidate.to_string()
                } else {
                    let avail = Self::fetch_all_versions()?;
                    plugin::resolve_partial_version(candidate, &avail, "Node")?
                };
                let url = Self::download_url(&resolved);
                (url, resolved, false)
            }
        } else {
            let latest = Self::fetch_latest_version()?;
            let url = Self::download_url(&latest);
            (url, latest, false)
        };
        let version_dir = Self::version_dir(&resolved_version);
        if Self::is_installed(&version_dir) {
            plugin::report(format!("Node {resolved_version} is already installed"));
            return Ok(resolved_version);
        }

        fs::create_dir_all(&version_dir).context("Failed to create version directory")?;

        let tar_path = if source_is_url {
            let filename = download_url
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("download.tar.gz");
            config::downloads_dir().join(filename)
        } else {
            Self::cached_tar(&resolved_version)
        };

        let verify_node_checksum = |tar_path: &Path| -> Result<()> {
            let tarball_filename = tar_path
                .file_name()
                .context(format!("Invalid tar path: {}", tar_path.display()))?
                .to_string_lossy();
            let checksums = plugin::fetch_checksums(node_mirror(), &resolved_version)?;
            if let Some(expected) = checksums.get(tarball_filename.as_ref()) {
                plugin::report("Verifying checksum...");
                plugin::verify_sha256(tar_path, expected)?;
                plugin::report("Checksum verified");
            }
            Ok(())
        };

        plugin::download_and_install(
            &download_url,
            &tar_path,
            &resolved_version,
            &version_dir,
            "Node",
            verify_node_checksum,
        )?;
        Ok(resolved_version)
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
        .context("Failed to uninstall Node version")
    }

    fn list_installed(&self) -> Result<Vec<String>> {
        plugin::list_installed_versions(&Self::lvm_dir(), version_prefix(), Self::is_installed)
            .context("Failed to list installed Node versions")
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
            bail!("Node {version} is not installed");
        }

        plugin::use_version_symlinks(
            &version_dir,
            &Self::current_link(),
            &Self::bin_link(),
            node_binary_name(),
        )?;

        if set_default {
            config::alias::set_default_version(self.name(), version)
                .context("Failed to write default config")?;
        }

        plugin::report(format!("Switched to Node {version}!"));
        Ok(())
    }

    fn binary_path(&self, version: &str) -> Result<String> {
        let prefix = version_prefix();
        let version = version.trim_start_matches(prefix);
        plugin::binary_path_in_dir(&Self::version_dir(version), "node", version)
            .context("Failed to get binary path")
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        let mut versions = Self::fetch_all_versions()?;
        plugin::sort_versions(&mut versions);

        let text = Self::fetch_text(index_tab_path())?;
        let lts_info: HashMap<String, Option<String>> =
            Self::parse_lts_info(&text).into_iter().collect();

        let result: Vec<String> = versions
            .iter()
            .map(|v| {
                if let Some(Some(codename)) = lts_info.get(v) {
                    format!("{v} (LTS: {codename})")
                } else {
                    v.clone()
                }
            })
            .collect();

        Ok(result)
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }

    fn post_install(&self, version: &str) -> Result<()> {
        let prefix = version_prefix();
        let version_dir = Self::version_dir(version.trim_start_matches(prefix));
        let npm_path = version_dir.join(config::bin_dir_name()).join(format!(
            "{}{}",
            npm_binary_name(),
            plugin::exe_suffix()
        ));
        install_default_packages(&npm_path)
    }
}

/// 读取 ~/.lvm/default-packages 并安装其中的全局包
fn install_default_packages(npm_path: &std::path::Path) -> Result<()> {
    let packages_file = config::lvm_home().join(default_packages_filename());
    if !packages_file.exists() {
        return Ok(());
    }

    let content =
        std::fs::read_to_string(&packages_file).context("Failed to read default-packages")?;

    let packages: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if packages.is_empty() {
        return Ok(());
    }

    plugin::report("Installing default packages...");
    let status = std::process::Command::new(npm_path)
        .args(["install", "-g", "--quiet"])
        .args(&packages)
        .status()
        .context("Failed to install default packages")?;

    if !status.success() {
        bail!("Failed to install some default packages");
    }

    plugin::report("Default packages installed");
    Ok(())
}
