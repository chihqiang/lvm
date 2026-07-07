use anyhow::{Context, Result};

use crate::config as lvm_config;
use crate::language;

use super::config::{flutter_versions_cache_filename, releases_url};

fn parse_stable_versions(text: &str) -> Vec<semver::Version> {
    let root: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut versions: Vec<semver::Version> = Vec::new();
    if let Some(releases) = root.get("releases").and_then(|r| r.as_array()) {
        for release in releases {
            if let (Some(ver_str), Some(channel)) = (
                release.get("version").and_then(|v| v.as_str()),
                release.get("channel").and_then(|c| c.as_str()),
            ) && channel == "stable"
                && let Ok(ver) = semver::Version::parse(ver_str)
                && ver.pre.is_empty()
            {
                versions.push(ver);
            }
        }
    }

    versions.sort();
    versions
}

fn fetch_releases_json() -> Result<String> {
    let cache_file = lvm_config::cache_path(flutter_versions_cache_filename());
    language::fetch_with_cache(&cache_file, || {
        let response = language::get_url(&releases_url())
            .call()
            .context("Failed to fetch Flutter releases")?;
        response.into_string().context("Failed to read response")
    })
}

impl super::FlutterLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let text = fetch_releases_json()?;
        let versions = parse_stable_versions(&text);
        versions
            .last()
            .map(|v| v.to_string())
            .context("No stable Flutter release found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let text = fetch_releases_json()?;
        let versions = parse_stable_versions(&text);
        let mut deduped = versions;
        deduped.dedup();
        Ok(deduped.into_iter().map(|v| v.to_string()).collect())
    }
}
