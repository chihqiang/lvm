use anyhow::{Context, Result};

pub fn read_nvmrc() -> Result<Option<String>> {
    let path = match crate::core::lvmrc::find_rc_file(".nvmrc") {
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
