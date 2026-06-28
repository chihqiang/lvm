//! LVM 配置模块
//! 集中管理 lvm 的基础路径、超时、通用常量等配置项
//! 与 language 无关，语言专用配置在 language/ 目录下各语言文件中

use anyhow::{Context, Result, bail};
use semver::Version;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

// ─── 目录名常量 ───

/// 二进制文件目录名 (~/.lvm/bin/)
pub(crate) fn bin_dir_name() -> &'static str {
    "bin"
}

/// 别名配置目录名 (~/.lvm/aliases/)
pub(crate) fn aliases_dir_name() -> &'static str {
    "aliases"
}

// ─── 超时与缓冲区配置 ───

/// 版本列表缓存 TTL
pub(crate) fn cache_ttl() -> Duration {
    Duration::from_mins(5)
}

/// HTTP 连接超时
pub(crate) fn http_connect_timeout() -> Duration {
    Duration::from_secs(30)
}

/// HTTP 读取超时
pub(crate) fn http_read_timeout() -> Duration {
    Duration::from_mins(2)
}

/// HTTP 总超时
pub(crate) fn http_total_timeout() -> Duration {
    Duration::from_mins(5)
}

// ─── 通用字符串常量 ───

/// .lvmrc 文件名
pub(crate) fn lvmrc_filename() -> &'static str {
    ".lvmrc"
}

/// "system" 版本关键字
pub(crate) fn system_version_keyword() -> &'static str {
    "system"
}

/// LTS 版本前缀
pub(crate) fn lts_prefix() -> &'static str {
    "lts/"
}

/// 列表分隔符（用于人类可读的列表）
pub(crate) fn list_separator() -> &'static str {
    ", "
}

/// .lvmrc 向上遍历最大层数
pub(crate) fn max_lvmrc_depth() -> u32 {
    100
}

// ─── 进度条配置 ───

/// 进度条模板
pub(crate) fn progress_bar_template() -> &'static str {
    "[{bar:20}] {percent:3}%  {bytes} / {total_bytes}"
}

/// 进度条字符
pub(crate) fn progress_bar_chars() -> &'static str {
    "=>-"
}

// ─── 路径配置 ───

fn downloads_dir_name() -> &'static str {
    "downloads"
}

fn cache_dir_name() -> &'static str {
    "cache"
}

/// 返回 lvm 根目录
/// 遵循 XDG Base Directory 规范，优先使用 $`XDG_DATA_HOME/lvm`
/// 回退到 ~/.lvm（Unix）或 %USERPROFILE%\.lvm（Windows）
pub(crate) fn lvm_home() -> Result<PathBuf> {
    if let Ok(data_home) = env::var("XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        return Ok(PathBuf::from(data_home).join("lvm"));
    }
    for var in &["HOME", "USERPROFILE"] {
        if let Ok(val) = env::var(var)
            && !val.is_empty()
        {
            return Ok(PathBuf::from(val).join(".lvm"));
        }
    }
    bail!("Cannot determine home directory (set $HOME or $XDG_DATA_HOME)")
}

/// 下载缓存目录
pub(crate) fn downloads_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(downloads_dir_name()))
}

/// 通用缓存目录
pub(crate) fn cache_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(cache_dir_name()))
}

// ─── 别名配置 ───

/// 语言别名目录: ~/.lvm/aliases/{lang}/
pub(crate) fn aliases_dir(language: &str) -> Result<PathBuf> {
    validate_path_component("language", language)?;
    Ok(lvm_home()?.join(aliases_dir_name()).join(language))
}

fn validate_path_component(kind: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        bail!("Invalid {kind} '{value}' (allowed: ASCII letters, numbers, '.', '_' and '-')");
    }
    Ok(())
}

/// 获取语言的别名
pub(crate) fn get_alias(language: &str, name: &str) -> Result<Option<String>> {
    validate_path_component("alias name", name)?;
    let path = aliases_dir(language)?.join(name);
    match fs::read_to_string(&path) {
        Ok(text) => {
            let v = text.trim().to_string();
            Ok(if v.is_empty() { None } else { Some(v) })
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("Failed to read alias '{name}'")),
    }
}

/// 设置语言的别名
pub(crate) fn set_alias(language: &str, name: &str, version: &str) -> Result<()> {
    validate_path_component("alias name", name)?;
    let version = version.trim_start_matches('v');
    if version != system_version_keyword()
        && !version.starts_with(lts_prefix())
        && !version.chars().all(|c| c.is_ascii_digit() || c == '.')
        && Version::parse(version).is_err()
    {
        bail!("Invalid version '{version}'");
    }
    let dir = aliases_dir(language)?;
    fs::create_dir_all(&dir).context("Failed to create alias directory")?;
    fs::write(dir.join(name), version)
        .with_context(|| format!("Failed to write alias '{name}'"))?;
    Ok(())
}

/// 列出语言的所有别名名
pub(crate) fn list_alias_names(language: &str) -> Result<Vec<String>> {
    let dir = aliases_dir(language)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).context("Failed to read aliases")? {
        let entry = entry.context("Failed to read entry")?;
        if entry.file_type().is_ok_and(|t| t.is_file())
            && let Some(name) = entry.file_name().to_str()
            && !name.starts_with('.')
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// 获取语言默认版本（读取 default 别名）
pub(crate) fn get_default_version(language: &str) -> Result<Option<String>> {
    get_alias(language, "default")
}

/// 设置语言默认版本
pub(crate) fn set_default_version(language: &str, version: &str) -> Result<()> {
    set_alias(language, "default", version)
}

/// 删除指定别名
pub(crate) fn remove_alias(language: &str, name: &str) -> Result<()> {
    validate_path_component("alias name", name)?;
    let path = aliases_dir(language)?.join(name);
    if !path.exists() {
        bail!("Alias '{name}' not found for {language}");
    }
    fs::remove_file(&path).with_context(|| format!("Failed to remove alias '{name}'"))
}

// ─── .lvmrc 配置 ───

/// 从当前目录向上遍历查找 .lvmrc 文件
pub(crate) fn find_lvmrc() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..max_lvmrc_depth() {
        let candidate = dir.join(lvmrc_filename());
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
    None
}

/// 解析 .lvmrc 内容，返回 language → version 映射
/// 格式：
///   node=20.14.0
///   go=1.22.3
///   # 注释行
///   空行忽略
pub(crate) fn parse_lvmrc(path: &Path) -> Result<HashMap<String, String>> {
    let text = fs::read_to_string(path).context("Failed to read .lvmrc")?;
    let mut map = HashMap::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            bail!(".lvmrc:{}: invalid format (expected key=value)", lineno + 1);
        };
        let key = key.trim().to_string();
        let value = value.trim().to_string();
        if key.is_empty() || value.is_empty() {
            bail!(".lvmrc:{}: empty key or value", lineno + 1);
        }
        map.insert(key, value);
    }
    Ok(map)
}

/// 读取 .lvmrc 中指定语言的版本
pub(crate) fn read_lvmrc_version(language: &str) -> Result<Option<String>> {
    let Some(path) = find_lvmrc() else {
        return Ok(None);
    };
    let map = parse_lvmrc(&path)?;
    Ok(map.get(language).cloned())
}

/// 写入或更新 .lvmrc 中指定语言的版本
/// 保留注释、空行和已有条目顺序
/// 如果 .lvmrc 不存在，则在当前目录创建
pub(crate) fn write_lvmrc(language: &str, version: &str) -> Result<()> {
    let path = match find_lvmrc() {
        Some(p) => p,
        None => std::env::current_dir()
            .context("Cannot determine current directory")?
            .join(lvmrc_filename()),
    };
    let mut updated = false;

    let content = if path.exists() {
        let text = fs::read_to_string(&path).context("Failed to read .lvmrc")?;
        let mut new_lines = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                new_lines.push(line.to_string());
                continue;
            }
            if let Some((key, _)) = trimmed.split_once('=') {
                let key = key.trim();
                if key == language {
                    if !updated {
                        new_lines.push(format!("{language}={version}"));
                        updated = true;
                    }
                    continue;
                }
            }
            new_lines.push(line.to_string());
        }
        if !updated {
            new_lines.push(format!("{language}={version}"));
        }
        new_lines.join("\n") + "\n"
    } else {
        format!("{language}={version}\n")
    };

    fs::write(&path, &content).context("Failed to write .lvmrc")?;
    Ok(())
}

// ─── 显示和格式化常量配置 ───

/// 检测 stdout 是否支持颜色输出
pub(crate) fn use_color() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// 绿色加粗 ANSI 代码（用于强调当前版本）
pub(crate) fn color_green_bold() -> &'static str {
    "\x1b[1;32m"
}

/// 黄色 ANSI 代码（用于备选版本）
pub(crate) fn color_yellow() -> &'static str {
    "\x1b[33m"
}

/// 绿色 ANSI 代码（用于已安装版本）
pub(crate) fn color_green() -> &'static str {
    "\x1b[32m"
}

/// 青色 ANSI 代码（用于 LTS 版本）
pub(crate) fn color_cyan() -> &'static str {
    "\x1b[36m"
}

/// 粗体 ANSI 代码
fn color_bold() -> &'static str {
    "\x1b[1m"
}

/// 重置 ANSI 代码
pub(crate) fn color_reset() -> &'static str {
    "\x1b[0m"
}

/// LTS 版本标记（在版本列表中）
pub(crate) fn lts_marker() -> &'static str {
    "(LTS:"
}

/// 勾号符号 ✓（表示已安装）
pub(crate) fn installed_check_mark() -> &'static str {
    "\u{2713}"
}

/// 带颜色的勾号（粗体）
pub(crate) fn colored_check_mark() -> String {
    format!(
        "{}{}{}",
        color_bold(),
        installed_check_mark(),
        color_reset()
    )
}

/// 系统版本关键字
pub(crate) fn system_keyword() -> &'static str {
    "system"
}
