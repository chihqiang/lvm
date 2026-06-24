use std::fs;

use crate::config;

use super::output;
use anyhow::{Context, Result};

fn remove_dir_contents(dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {dir:?}"))? {
        let entry = entry.context("Failed to read entry")?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).context("Failed to remove directory")?;
        } else {
            fs::remove_file(&path).context("Failed to remove file")?;
        }
    }
    Ok(())
}

/// 清空下载缓存和版本列表缓存
pub(crate) fn cache_clear() -> Result<()> {
    remove_dir_contents(&config::downloads_dir())?;
    remove_dir_contents(&config::cache_dir())?;
    output::info("Cache cleared");
    Ok(())
}
