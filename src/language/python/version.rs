use anyhow::{Context, Result};
use semver::Version;
use std::path::PathBuf;

use crate::config;
use crate::language;

use super::config::python_versions_cache_filename;

const PYTHON_VERSIONS_URL: &str = "https://www.python.org/ftp/python/";

fn version_from_string(s: &str) -> Option<Version> {
    let s = s.trim_end_matches('/');
    if !s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return None;
    }
    let semver_str = if parts.len() == 2 {
        format!("{}.0", s)
    } else {
        s.to_string()
    };
    let ver = Version::parse(&semver_str).ok()?;
    if ver.pre.is_empty() {
        Some(ver)
    } else {
        None
    }
}

impl super::PythonLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions
            .last()
            .cloned()
            .context("No Python versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| PathBuf::from(".lvm/cache"))
            .join(python_versions_cache_filename());
        let text = language::fetch_with_cache(&cache_file, || {
            let response = language::get_url(PYTHON_VERSIONS_URL)
                .call()
                .context("Failed to fetch Python versions")?;
            response.into_string().context("Failed to read response")
        })?;
        Ok(Self::parse_versions_html(&text))
    }

    pub(crate) fn parse_versions_html(html: &str) -> Vec<String> {
        let mut versions: Vec<Version> = Vec::new();
        for line in html.lines() {
            if let Some(href_start) = line.find("href=\"") {
                let rest = &line[href_start + 6..];
                if let Some(href_end) = rest.find('"') {
                    let dir = &rest[..href_end];
                    if let Some(ver) = version_from_string(dir) {
                        versions.push(ver);
                    }
                }
            }
        }
        versions.sort();
        versions.dedup();
        versions.into_iter().map(|v| v.to_string()).collect()
    }
}
