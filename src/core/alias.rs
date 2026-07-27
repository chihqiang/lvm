//! Alias configuration (~/.lvm/aliases/)

use anyhow::{Context, Result, bail};
use semver::Version;
use std::fs;
use std::path::PathBuf;

use crate::core::config;

/// Language alias directory: ~/.lvm/aliases/{lang}/
pub fn aliases_dir(language: &str) -> Result<PathBuf> {
    validate_path_component("language", language)?;
    Ok(config::lvm_home()?.join(config::ALIASES_DIR).join(language))
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

/// Get an alias for a language
pub fn get_alias(language: &str, name: &str) -> Result<Option<String>> {
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

/// Set an alias for a language
pub fn set_alias(language: &str, name: &str, version: &str) -> Result<()> {
    validate_path_component("alias name", name)?;
    let version = version.trim_start_matches('v');
    if version != config::SYSTEM_VERSION_KEYWORD
        && !version.starts_with(config::LTS_PREFIX)
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

/// List all alias names for a language
pub fn list_alias_names(language: &str) -> Result<Vec<String>> {
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

/// Get the default version for a language
pub fn get_default_version(language: &str) -> Result<Option<String>> {
    get_alias(language, "default")
}

/// Set the default version for a language
pub fn set_default_version(language: &str, version: &str) -> Result<()> {
    set_alias(language, "default", version)
}

/// Remove a specific alias
pub fn remove_alias(language: &str, name: &str) -> Result<()> {
    validate_path_component("alias name", name)?;
    let path = aliases_dir(language)?.join(name);
    if !path.exists() {
        bail!("Alias '{name}' not found for {language}");
    }
    fs::remove_file(&path).with_context(|| format!("Failed to remove alias '{name}'"))
}
