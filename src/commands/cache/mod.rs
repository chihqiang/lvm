use std::fs;

use lvm::core::config;

use crate::commands::output;
use anyhow::{Context, Result};

fn remove_dir_contents(dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut errors: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                output::warn(format!("Failed to read entry: {e}"));
                continue;
            }
        };
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                errors.push(format!(
                    "failed to read metadata for {}: {e}",
                    path.display()
                ));
                continue;
            }
        };
        let result = if meta.is_symlink() || meta.is_file() {
            fs::remove_file(&path)
        } else {
            fs::remove_dir_all(&path)
        };
        if let Err(e) = result {
            errors.push(format!("failed to remove {}: {e}", path.display()));
        }
    }
    if !errors.is_empty() {
        let detail = errors.join("; ");
        anyhow::bail!("Some cache entries could not be removed:\n  {detail}");
    }
    Ok(())
}

/// 清空下载缓存和版本列表缓存
pub(crate) fn cache_clear() -> Result<()> {
    remove_dir_contents(&config::downloads_dir()?)?;
    remove_dir_contents(&config::cache_dir()?)?;
    output::info("Cache cleared");
    Ok(())
}
