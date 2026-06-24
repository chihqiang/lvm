//! 语言插件模块
//! 定义 Plugin trait 供所有语言实现，实现插拔式架构
//! 新增语言只需在 plugin/ 下创建目录实现 Plugin trait，然后在 mod.rs 中 pub mod 暴露

pub mod go;
pub mod node;

use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::config;
use anyhow::{Context, Result, anyhow, bail};

/// 消息缓冲区：plugin 层通过 report() 输出信息，由 commands 层统一 flush
static REPORT_BUF: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn report_buf() -> &'static Mutex<Vec<String>> {
    REPORT_BUF.get_or_init(|| Mutex::new(Vec::new()))
}

fn lock_report_buf() -> std::sync::MutexGuard<'static, Vec<String>> {
    report_buf().lock().unwrap_or_else(|e| e.into_inner())
}

/// plugin 层用此函数上报信息，不直接 println
pub(crate) fn report(msg: impl Into<String>) {
    lock_report_buf().push(msg.into());
}

/// 取出所有待打印消息并清空缓冲区
pub(crate) fn drain_reports() -> Vec<String> {
    lock_report_buf().drain(..).collect()
}

/// 立即将缓冲区内容输出到 stdout（供 plugin 层在长时间操作前预刷消息）
pub(crate) fn flush_reports_to_stdout() {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    for msg in drain_reports() {
        let _ = writeln!(out, "{msg}");
    }
    let _ = out.flush();
}

/// 离线模式标志
static OFFLINE: AtomicBool = AtomicBool::new(false);

/// 设置离线模式
pub(crate) fn set_offline(offline: bool) {
    OFFLINE.store(offline, Ordering::SeqCst);
}

/// 检查是否处于离线模式
pub(crate) fn is_offline() -> bool {
    OFFLINE.load(Ordering::SeqCst)
}

/// 全局 HTTP 客户端单例（复用连接池，减少资源开销）
pub(crate) fn http() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(config::http_connect_timeout())
            .timeout_read(config::http_read_timeout())
            .timeout(config::http_total_timeout())
            .build()
    })
}

pub(crate) fn get_url(url: &str) -> ureq::Request {
    let ua = format!("lvm-http-client/{}", env!("CARGO_PKG_VERSION"));
    http().get(url).set("User-Agent", &ua)
}

/// 语义化版本号比较函数
pub(crate) fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_ver = Version::parse(a).ok();
    let b_ver = Version::parse(b).ok();
    match (a_ver, b_ver) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        _ => a.cmp(b),
    }
}

/// 按语义化版本号排序版本列表
pub(crate) fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_versions(a, b));
}

/// 解析部分版本号（如 "20" → 最新 20.x.x, "20.18" → 最新 20.18.x）
pub(crate) fn resolve_partial_version(
    candidate: &str,
    avail: &[String],
    lang: &str,
) -> Result<String> {
    let parts: Vec<&str> = candidate.split('.').collect();
    let want_major = parts.first().and_then(|s| s.parse::<u64>().ok());
    let want_minor = parts.get(1).and_then(|s| s.parse::<u64>().ok());

    let best = avail
        .iter()
        .filter_map(|a| Version::parse(a).ok())
        .filter(|av| {
            want_major.is_none_or(|maj| av.major == maj)
                && want_minor.is_none_or(|min| av.minor == min)
        })
        .max();
    best.map(|v| v.to_string())
        .ok_or_else(|| anyhow!("Could not find {lang} version matching: {candidate}"))
}

/// 下载并验证归档文件的通用流程
/// - 如果目标文件已存在则跳过下载
/// - 下载完成后调用 verify 闭包验证校验和
/// - 解压到版本目录
pub(crate) fn download_and_install(
    dl_url: &str,
    tar_path: &Path,
    version: &str,
    version_dir: &Path,
    display_name: &str,
    verify: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    if !tar_path.exists() {
        fs::create_dir_all(tar_path.parent().context("Invalid tar path")?)
            .context("Failed to create download cache directory")?;
        report(format!("Downloading {display_name} {version}"));
        report(format!("  from: {dl_url}"));
        report(format!("  to:   {}", tar_path.display()));
        // 预刷一次，让 URL 信息在进度条开始前显示
        flush_reports_to_stdout();
        download(dl_url, tar_path, true)?;
        verify(tar_path)?;
    }
    extract_archive(tar_path, version_dir)?;
    report(format!("{display_name} {version} installed successfully!"));
    Ok(())
}

/// 计算文件的 SHA256 校验和
pub(crate) fn sha256_of(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).context("Failed to open file for checksum")?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; config::download_buffer_size()];
    loop {
        let n = file.read(&mut buf).context("Failed to read for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// 从镜像源获取 SHASUMS256.txt 并解析，返回 filename → checksum 映射
pub(crate) fn fetch_checksums(mirror_url: &str, version: &str) -> Result<HashMap<String, String>> {
    let url = format!("{mirror_url}/v{version}/SHASUMS256.txt");
    let response = get_url(&url).call().context("Failed to fetch checksums")?;
    let text = response.into_string().context("Failed to read checksums")?;

    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((checksum, filename)) = line.split_once("  ") {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        } else if let Some((checksum, filename)) = line.split_once(' ') {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        }
    }
    Ok(map)
}

/// 验证下载文件的 SHA256 校验和
/// * `file_path`      - 已下载的文件路径
/// * `expected_hex`   - 期望的 SHA256 十六进制字符串
pub(crate) fn verify_sha256(file_path: &Path, expected_hex: &str) -> Result<()> {
    let actual = sha256_of(file_path)?;
    if actual != expected_hex {
        bail!(
            "Checksum mismatch for {}: expected {expected_hex}, got {actual}",
            file_path.display()
        )
    }
    Ok(())
}

/// 从镜像源获取文本内容
pub(crate) fn fetch_from_mirror(mirror_url: &str, url_path: &str) -> Result<String> {
    let url = format!("{}/{url_path}", mirror_url);
    let response = get_url(&url)
        .call()
        .context("Failed to fetch data from mirror")?;
    response.into_string().context("Failed to read response")
}

/// 通用下载函数，支持断点续传和进度条显示
///
/// 下载流程：
/// 1. 检查本地是否已有部分文件，有则发送 Range 头尝试断点续传
/// 2. 服务器返回 206 表示支持续传，从头开始则覆盖本地文件
/// 3. 用 indicatif 进度条实时显示下载进度
///
/// * `url`          - 下载地址
/// * `dest`         - 本地保存路径
/// * `show_progress`- 是否显示进度条
pub(crate) fn download(url: &str, dest: &Path, show_progress: bool) -> Result<()> {
    // 离线模式：仅使用缓存，跳过网络请求
    if is_offline() {
        if dest.exists() {
            report("Using cached file (offline mode)");
            return Ok(());
        }
        bail!("Offline mode: no cached file at {}", dest.display())
    }

    // 检查本地已有文件大小，用于断点续传
    let existing = fs::metadata(dest).ok().map_or(0, |m| m.len());

    let mut req = get_url(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={existing}-"));
    }

    let resp = req.call().context("Download request failed")?;
    let status = resp.status();
    // 仅接受 200（从头下载）和 206（断点续传），其他状态码视为错误
    if status != 200 && status != 206 {
        bail!("Download failed (HTTP {status})")
    }
    let is_resume = status == 206;

    // 本次下载的 Content-Length
    let content_len = resp
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let total = if is_resume {
        existing + content_len
    } else if content_len > 0 {
        content_len
    } else {
        // 无法确定总大小时仍然开始下载
        0
    };

    // 打开本地文件：续传时追加，否则覆盖
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(is_resume)
        .open(dest)
        .context("Failed to open file")?;

    // 如果服务器不支持续传或返回 200，则从头开始下载
    if !is_resume && existing > 0 {
        file.set_len(0).context("Failed to truncate file")?;
    }

    // 初始化进度条
    let pb = if show_progress {
        let bar = if total > 0 {
            ProgressBar::new(total)
        } else {
            ProgressBar::new_spinner()
        };
        if let Ok(style) = ProgressStyle::default_bar().template(config::progress_bar_template()) {
            bar.set_style(style.progress_chars(config::progress_bar_chars()));
        }
        if is_resume {
            bar.set_position(existing);
        }
        Some(bar)
    } else {
        None
    };

    let mut reader = resp.into_reader();
    let mut buf = vec![0u8; config::download_buffer_size()];
    let mut downloaded = if is_resume { existing } else { 0 };

    loop {
        let n = reader.read(&mut buf).context("Failed to read data")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).context("Failed to write file")?;
        downloaded += n as u64;

        if let Some(ref bar) = pb {
            bar.set_position(downloaded);
        }
    }

    if let Some(bar) = pb {
        bar.finish_and_clear();
    }
    report("Download complete");

    Ok(())
}

// ─── Plugin 共享函数 ───

/// 创建符号链接（跨平台 unix/windows）
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

/// 删除符号链接（跨平台）
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

/// 返回平台对应的可执行文件后缀
pub(crate) fn exe_suffix() -> &'static str {
    std::env::consts::EXE_SUFFIX
}

/// 返回 PATH 分隔符（Unix 返回 ":"，Windows 返回 ";"）
pub(crate) fn path_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

/// 返回当前平台的归档文件扩展名（不含点号）
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

/// 解压 zip 文件到目标目录
pub(crate) fn extract_zip(zip_path: &Path, version_dir: &Path) -> Result<()> {
    report("Extracting...");
    let file = fs::File::open(zip_path).context("Failed to open archive")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("Extraction failed")?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };

        // 安全检测：禁止路径遍历
        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            bail!("Zip entry contains path traversal (..)")
        }

        let mut out_path = version_dir.to_path_buf();
        // 跳过顶层目录（类似 tarball 的 strip first component）
        for component in path.components().skip(1) {
            out_path.push(component);
        }

        if entry.is_dir() {
            fs::create_dir_all(&out_path).context("Extraction failed")?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).context("Extraction failed")?;
            }
            let mut outfile = fs::File::create(&out_path).context("Extraction failed")?;
            std::io::copy(&mut entry, &mut outfile).context("Extraction failed")?;
        }
    }
    Ok(())
}

/// 根据文件扩展名选择解压方式
pub(crate) fn extract_archive(archive_path: &Path, version_dir: &Path) -> Result<()> {
    if archive_path.extension().is_some_and(|e| e == "zip") {
        extract_zip(archive_path, version_dir)
    } else {
        extract_tarball(archive_path, version_dir)
    }
}

/// 从符号链接读取当前版本号
pub(crate) fn current_version_from_link(link: &Path, prefix: &str) -> Option<String> {
    let target = fs::read_link(link).ok()?;
    if !target.exists() {
        return None;
    }
    let dir_name = target.file_name()?;
    let name = dir_name.to_str()?;
    Some(name.trim_start_matches(prefix).to_string())
}

/// 列出目录中的已安装版本
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

/// 格式化版本列表，添加 (current) / (default) 标记
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

/// 卸载指定版本
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

/// 获取版本二进制文件路径
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

/// 解压 tarball 到版本目录
pub(crate) fn extract_tarball(tar_path: &Path, version_dir: &Path) -> Result<()> {
    report("Extracting...");
    let tar_file = fs::File::open(tar_path).context("Failed to open tarball")?;
    let decoder = flate2::read::GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(decoder);
    let result = archive
        .entries()
        .context("Extraction failed")?
        .try_for_each(|entry| {
            let mut entry = entry.context("Extraction failed")?;
            let path = entry.path().context("Extraction failed")?;
            if path.is_absolute() {
                bail!("Extraction failed: absolute path in archive")
            }
            let stripped = path.components().skip(1).collect::<std::path::PathBuf>();
            if stripped.as_os_str().is_empty() {
                return Ok(());
            }
            if stripped
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                bail!("Extraction failed: path traversal in archive");
            }
            let out_path = version_dir.join(&stripped);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).context("Extraction failed")?;
            }
            entry
                .unpack(&out_path)
                .map(|_| ())
                .context("Extraction failed")
        });

    if let Err(err) = result {
        let _ = fs::remove_dir_all(version_dir);
        let _ = fs::remove_file(tar_path);
        bail!("{}", err)
    }
    Ok(())
}

/// 切换版本：创建 current + bin 符号链接，可选标记为 default
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

// ─── Plugin Trait ───

/// 语言插件核心接口
/// 每种语言（Node、Go 等）都需实现此 trait
pub trait Plugin {
    /// 返回语言名称，如 "node"、"go"
    fn name(&self) -> &str;

    /// 安装指定版本，version 为 None 时安装最新版
    /// 返回实际安装的版本号
    fn install(&self, version: Option<&str>) -> Result<String>;

    /// 卸载指定版本
    fn uninstall(&self, version: &str) -> Result<()>;

    /// 列出本地已安装的所有版本
    fn list_installed(&self) -> Result<Vec<String>>;

    /// 切换使用指定版本，set_default 为 true 时同时设为默认
    fn use_version(&self, version: &str, set_default: bool) -> Result<()>;

    /// 返回当前正在使用的版本号（通过符号链接读取）
    fn current_version(&self) -> Result<Option<String>> {
        Ok(None)
    }

    /// 返回指定版本主可执行文件的路径
    fn binary_path(&self, version: &str) -> Result<String>;

    /// 获取远程最新稳定版本号
    fn latest_version(&self) -> Result<String>;

    /// 列出远程可用版本（用于与本地已安装版本比较）
    fn list_remote_versions(&self) -> Result<Vec<String>> {
        bail!("Remote version listing is not supported for this plugin")
    }

    /// 格式化已安装版本列表用于显示
    fn format_installed(&self, versions: &[String]) -> Result<Vec<String>> {
        Ok(versions.to_vec())
    }

    /// 安装后的额外操作，例如 default-packages
    /// version 是刚刚安装的版本号
    fn post_install(&self, _version: &str) -> Result<()> {
        Ok(())
    }
}

/// 语言插件注册表
/// 管理所有注册的 Plugin 实现，按名称查找
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
        }
    }

    /// 注册一个语言插件
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    /// 按名称查找语言插件
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(Box::as_ref)
    }

    /// 获取所有已注册的语言名称列表
    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.plugins.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}
