//! LVM 配置模块
//! 集中管理 lvm 的基础路径、超时、通用常量等配置项
//! 与 plugin 无关，语言专用配置在 plugin/ 目录下各语言文件中

pub mod alias;
pub mod display;
pub mod lvmrc;

use std::env;
use std::path::PathBuf;
use std::time::Duration;

// ─── 目录名常量 ───

/// 二进制文件目录名 (~/.lvm/bin/)
pub(crate) fn bin_dir_name() -> &'static str {
    "bin"
}

/// 当前版本目录名 (~/.lvm/current/)
pub(crate) fn current_dir_name() -> &'static str {
    "current"
}

/// 下载缓存目录名 (~/.lvm/downloads/)
fn downloads_dir_name() -> &'static str {
    "downloads"
}

/// 通用缓存目录名 (~/.lvm/cache/)
fn cache_dir_name() -> &'static str {
    "cache"
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

/// 下载/校验缓冲区大小（64KB）
pub(crate) fn download_buffer_size() -> usize {
    64 * 1024
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
    4096
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

// ─── 路径构造 ───

/// 返回 lvm 根目录
/// 遵循 XDG Base Directory 规范，优先使用 $XDG_DATA_HOME/lvm
/// 回退到 ~/.lvm（Unix）或 %USERPROFILE%\.lvm（Windows）
/// 均不存在时 panic（CLI 工具不应静默使用临时目录导致数据丢失）
pub(crate) fn lvm_home() -> PathBuf {
    if let Ok(data_home) = env::var("XDG_DATA_HOME")
        && !data_home.is_empty()
    {
        return PathBuf::from(data_home).join("lvm");
    }
    for var in &["HOME", "USERPROFILE"] {
        if let Ok(val) = env::var(var)
            && !val.is_empty()
        {
            return PathBuf::from(val).join(".lvm");
        }
    }
    panic!("Cannot determine LVM home directory: set $HOME, $XDG_DATA_HOME, or $USERPROFILE");
}

/// 下载缓存目录
pub(crate) fn downloads_dir() -> PathBuf {
    lvm_home().join(downloads_dir_name())
}

/// 通用缓存目录
pub(crate) fn cache_dir() -> PathBuf {
    lvm_home().join(cache_dir_name())
}
