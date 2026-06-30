//! .lvmrc 配置文件读写

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::config;

/// 从当前目录向上遍历查找指定名称的 rc 文件
pub fn find_rc_file(filename: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..config::MAX_LVM_DEPTH {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
    None
}

/// 从当前目录向上遍历查找 .lvmrc 文件
pub fn find_lvmrc() -> Option<PathBuf> {
    find_rc_file(config::LVM_FILENAME)
}

/// 解析 .lvmrc 内容，返回 language → version 映射
/// 格式：
///   node=20.14.0
///   go=1.22.3
///   # 注释行
///   空行忽略
pub fn parse_lvmrc(path: &Path) -> Result<HashMap<String, String>> {
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
pub fn read_lvmrc_version(language: &str) -> Result<Option<String>> {
    let Some(path) = find_lvmrc() else {
        return Ok(None);
    };
    let map = parse_lvmrc(&path)?;
    Ok(map.get(language).cloned())
}

/// 写入或更新 .lvmrc 中指定语言的版本
/// 保留注释、空行和已有条目顺序
/// 如果 .lvmrc 不存在，则在当前目录创建
pub fn write_lvmrc(language: &str, version: &str) -> Result<()> {
    let path = match find_lvmrc() {
        Some(p) => p,
        None => std::env::current_dir()
            .context("Cannot determine current directory")?
            .join(config::LVM_FILENAME),
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
