use anyhow::{Context, Result};

use crate::config;
use crate::language;

use super::config::{
    flutter_latest_version_cache_filename, flutter_versions_cache_filename, releases_url,
};

impl super::FlutterLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| config::default_cache_dir())
            .join(flutter_latest_version_cache_filename());

        let text = language::fetch_with_cache(&cache_file, || {
            let response = language::get_url(&releases_url())
                .call()
                .context("Failed to fetch Flutter releases")?;
            response.into_string().context("Failed to read response")
        })?;

        let root: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse Flutter releases JSON")?;

        root.get("current_release")
            .and_then(|cr| cr.get("stable"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .context("Missing 'current_release.stable' in Flutter releases JSON")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| config::default_cache_dir())
            .join(flutter_versions_cache_filename());

        let text = language::fetch_with_cache(&cache_file, || {
            let response = language::get_url(&releases_url())
                .call()
                .context("Failed to fetch Flutter releases")?;
            response.into_string().context("Failed to read response")
        })?;

        let root: serde_json::Value =
            serde_json::from_str(&text).context("Failed to parse Flutter releases JSON")?;

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
        versions.dedup();
        Ok(versions.into_iter().map(|v| v.to_string()).collect())
    }
}
