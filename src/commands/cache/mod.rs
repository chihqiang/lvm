use std::fs;

use crate::config;

use crate::commands::output;
use anyhow::{Context, Result};

fn remove_dir_contents(dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut had_error = false;
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                output::warn(format!("Failed to read entry: {e}"));
                had_error = true;
                continue;
            }
        };
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                output::warn(format!(
                    "Failed to read metadata for {}: {e}",
                    path.display()
                ));
                had_error = true;
                continue;
            }
        };
        let result = if meta.is_symlink() || meta.is_file() {
            fs::remove_file(&path)
        } else {
            fs::remove_dir_all(&path)
        };
        if let Err(e) = result {
            output::warn(format!("Failed to remove {}: {e}", path.display()));
            had_error = true;
        }
    }
    if had_error {
        anyhow::bail!("Some cache entries could not be removed");
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
