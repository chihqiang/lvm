use anyhow::{Context, Result};
use semver::Version;

use crate::config as lvm_config;
use crate::language;

use super::config::{download_base, python_tag, python_versions_cache_filename};

/// Derive the GitHub API URL for the configured python-build-standalone release tag.
/// Returns None if the download base is not a GitHub URL.
fn github_release_api_url() -> Option<String> {
    let base = download_base();
    // download_base looks like: https://github.com/{owner}/{repo}/releases/download
    let trimmed = base.strip_suffix("/releases/download")?;
    let repo_path = trimmed.strip_prefix("https://github.com/")?;
    Some(format!(
        "https://api.github.com/repos/{repo_path}/releases/tags/{tag}",
        tag = python_tag(),
    ))
}

/// Parse a cpython version from a python-build-standalone asset name.
/// Asset names look like: cpython-3.12.4+20260623-x86_64-unknown-linux-gnu-install_only.tar.gz
fn version_from_asset_name(name: &str) -> Option<String> {
    let rest = name.strip_prefix("cpython-")?;
    // Split at the first '+' to separate version from tag
    let ver_part = rest.split('+').next()?;
    if Version::parse(ver_part).is_ok() {
        Some(ver_part.to_string())
    } else {
        None
    }
}

/// Fetch available Python versions from the python-build-standalone GitHub release assets.
fn fetch_versions_from_github() -> Result<Vec<String>> {
    let api_url = github_release_api_url()
        .context("Cannot derive GitHub API URL from custom Python mirror")?;

    let response = language::get_url(&api_url)
        .call()
        .context("Failed to fetch python-build-standalone release info")?;
    let text = response.into_string().context("Failed to read response")?;

    let root: serde_json::Value =
        serde_json::from_str(&text).context("Failed to parse release JSON")?;

    let mut versions: Vec<Version> = Vec::new();
    if let Some(assets) = root.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                if let Some(ver) = version_from_asset_name(name) {
                    if let Ok(v) = Version::parse(&ver) {
                        if v.pre.is_empty() {
                            versions.push(v);
                        }
                    }
                }
            }
        }
    }

    versions.sort();
    versions.dedup();
    Ok(versions.into_iter().map(|v| v.to_string()).collect())
}

impl super::PythonLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions.last().cloned().context("No Python versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = lvm_config::cache_path(python_versions_cache_filename());

        let text = language::fetch_with_cache(&cache_file, || {
            let versions = fetch_versions_from_github()?;
            Ok(versions.join("\n"))
        })?;

        Ok(text.lines().map(String::from).collect())
    }
}
