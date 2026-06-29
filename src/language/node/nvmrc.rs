use std::path::PathBuf;

use anyhow::{Context, Result};

const NVMRC_FILENAME: &str = ".nvmrc";

fn find_nvmrc() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..crate::config::max_lvmrc_depth() {
        let candidate = dir.join(NVMRC_FILENAME);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
    None
}

pub(crate) fn read_nvmrc() -> Result<Option<String>> {
    let path = match find_nvmrc() {
        Some(p) => p,
        None => return Ok(None),
    };
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let version = text.lines().find_map(|l| {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') {
            return None;
        }
        Some(l.to_string())
    });
    Ok(version)
}
