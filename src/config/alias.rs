use anyhow::{Context, Result, bail};
use semver::{Version, VersionReq};
use std::fs;
use std::path::PathBuf;

/// 语言别名目录: ~/.lvm/aliases/{lang}/
pub(crate) fn aliases_dir(language: &str) -> PathBuf {
    super::lvm_home()
        .join(super::aliases_dir_name())
        .join(language)
}

/// 获取语言的别名
pub(crate) fn get_alias(language: &str, name: &str) -> Result<Option<String>> {
    let path = aliases_dir(language).join(name);
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
    if version != super::system_version_keyword()
        && !version.starts_with(super::lts_prefix())
        && !version.chars().all(|c| c.is_ascii_digit() || c == '.')
        && Version::parse(version).is_err()
        && VersionReq::parse(version).is_err()
    {
        bail!("Invalid version '{version}'");
    }
    let dir = aliases_dir(language);
    fs::create_dir_all(&dir).context("Failed to create alias directory")?;
    fs::write(dir.join(name), version)
        .with_context(|| format!("Failed to write alias '{name}'"))?;
    Ok(())
}

/// 列出语言的所有别名名
pub(crate) fn list_alias_names(language: &str) -> Result<Vec<String>> {
    let dir = aliases_dir(language);
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
    let path = aliases_dir(language).join(name);
    if !path.exists() {
        bail!("Alias '{name}' not found for {language}");
    }
    fs::remove_file(&path).with_context(|| format!("Failed to remove alias '{name}'"))
}
