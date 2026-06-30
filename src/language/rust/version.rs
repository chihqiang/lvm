use anyhow::{Context, Result};

use crate::config;
use crate::language;

use super::config::rust_versions_cache_filename;

const RUST_VERSIONS_URL: &str = "https://api.github.com/repos/rust-lang/rust/releases?per_page=100";

impl super::RustLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions.last().cloned().context("No Rust versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| config::default_cache_dir())
            .join(rust_versions_cache_filename());
        let text = language::fetch_with_cache(&cache_file, || {
            let response = language::get_url(RUST_VERSIONS_URL)
                .call()
                .context("Failed to fetch Rust versions")?;
            response.into_string().context("Failed to read response")
        })?;

        Ok(Self::parse_versions_json(&text))
    }

    fn parse_versions_json(json: &str) -> Vec<String> {
        let root: Vec<serde_json::Value> = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let mut versions: Vec<semver::Version> = Vec::new();
        for release in &root {
            let draft = release
                .get("draft")
                .and_then(|d| d.as_bool())
                .unwrap_or(true);
            let prerelease = release
                .get("prerelease")
                .and_then(|d| d.as_bool())
                .unwrap_or(true);
            if draft || prerelease {
                continue;
            }
            let tag = match release.get("tag_name").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => continue,
            };
            if let Ok(ver) = semver::Version::parse(tag)
                && ver.pre.is_empty()
            {
                versions.push(ver);
            }
        }

        versions.sort();
        versions.dedup();
        versions.into_iter().map(|v| v.to_string()).collect()
    }
}
