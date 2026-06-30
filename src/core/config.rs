//! LVM 配置模块
//! 集中管理 lvm 的基础路径、超时、通用常量等配置项
//! 与 language 无关，语言专用配置在 language/ 目录下各语言文件中

use anyhow::{Result, bail};
use std::env;
use std::path::PathBuf;

// ─── 目录名常量 ───

/// 二进制文件目录名 (~/.lvm/bin/)
pub const BIN_DIR: &str = "bin";

/// 当前版本软链接目录名 (~/.lvm/current/)
pub const CURRENT_DIR: &str = "current";

/// 别名配置目录名 (~/.lvm/aliases/)
pub const ALIASES_DIR: &str = "aliases";

const DOWNLOADS_DIR: &str = "downloads";

const CACHE_DIR_NAME: &str = "cache";

// ─── 通用字符串常量 ───

/// .lvmrc 文件名
pub const LVM_FILENAME: &str = ".lvmrc";

/// "system" 版本关键字
pub const SYSTEM_VERSION_KEYWORD: &str = "system";

/// LTS 版本前缀
pub const LTS_PREFIX: &str = "lts/";

/// 列表分隔符（用于人类可读的列表）
pub const LIST_SEPARATOR: &str = ", ";

/// .lvmrc 向上遍历最大层数
pub const MAX_LVM_DEPTH: u32 = 100;

// ─── 路径配置 ───

/// 返回 lvm 根目录
/// 遵循 XDG Base Directory 规范，优先使用 $XDG_DATA_HOME/lvm
/// 回退到 ~/.lvm（Unix）或 %USERPROFILE%\.lvm（Windows）
pub fn lvm_home() -> Result<PathBuf> {
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
pub fn downloads_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(DOWNLOADS_DIR))
}

/// `downloads_dir()`, 但如果失败则回退至默认路径
pub fn downloads_dir_or_default() -> PathBuf {
    downloads_dir().unwrap_or_else(|_| default_downloads_dir())
}

/// 通用缓存目录
pub fn cache_dir() -> Result<PathBuf> {
    Ok(lvm_home()?.join(CACHE_DIR_NAME))
}

pub fn default_cache_dir() -> PathBuf {
    PathBuf::from(".lvm/cache")
}

/// 缓存中某个文件的完整路径，自动处理 `cache_dir()` 失败回退
pub fn cache_path(filename: &str) -> PathBuf {
    cache_dir()
        .unwrap_or_else(|_| default_cache_dir())
        .join(filename)
}

pub fn default_downloads_dir() -> PathBuf {
    PathBuf::from(".lvm/downloads")
}
