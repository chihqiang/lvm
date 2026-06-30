use anyhow::{Context, Result, bail};
use semver::Version;

use crate::config;
use crate::language;

use super::NodeLanguage;
use super::config::{
    index_tab_path, latest_version_path, node_mirror, node_versions_cache_filename, tarball_prefix,
};
use super::lts;

fn version_from_tarball_name(filename: &str) -> Option<String> {
    let s = filename.strip_prefix(tarball_prefix())?;
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() < 3 {
        return None;
    }
    Some(parts[..parts.len() - 2].join("-"))
}

impl NodeLanguage {
    pub(crate) fn fetch_text(url_path: &str) -> Result<String> {
        language::fetch_from_mirror(node_mirror(), url_path)
            .context("Failed to fetch text from mirror")
    }

    pub(crate) fn fetch_latest_version() -> Result<String> {
        let text = Self::fetch_text(latest_version_path())?;

        for line in text.lines() {
            if let Some(filename) = line.split_whitespace().nth(1)
                && let Some(version) = version_from_tarball_name(filename)
            {
                return Ok(version);
            }
        }

        bail!("Could not find latest version from mirror")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let text = fetch_index_tab()?;
        Ok(Self::parse_index_tab(&text))
    }

    pub(crate) fn parse_index_tab(text: &str) -> Vec<String> {
        text.lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.is_empty() {
                    return None;
                }
                parts[0].strip_prefix('v').map(str::to_string)
            })
            .collect()
    }

    fn version_from_url(url: &str) -> Result<String> {
        let filename = url
            .rsplit('/')
            .next()
            .with_context(|| format!("Invalid Node tarball URL: {url}"))?;
        let version = version_from_tarball_name(filename)
            .with_context(|| format!("Invalid Node tarball URL: {url}"))?;
        let version = version.trim_start_matches('v');
        if Version::parse(version).is_err() {
            bail!("Invalid Node version in URL: {version}");
        }
        Ok(version.to_string())
    }

    pub(crate) fn resolve_install_version(version: Option<&str>) -> Result<(String, String, bool)> {
        let Some(v) = version else {
            let latest = Self::fetch_latest_version()?;
            return Ok((Self::download_url(&latest), latest, false));
        };
        let v = v.trim();
        if v.starts_with("http://") || v.starts_with("https://") {
            let version = Self::version_from_url(v)?;
            return Ok((v.to_string(), version, true));
        }
        if let Some(desc) = v.strip_prefix(config::LTS_PREFIX) {
            let resolved = lts::resolve_lts(desc)?;
            return Ok((Self::download_url(&resolved), resolved, false));
        }
        if v == config::SYSTEM_VERSION_KEYWORD {
            bail!("Use 'lvm use system' instead of 'lvm install system'");
        }
        let candidate = v.trim_start_matches('v');
        let resolved = if Version::parse(candidate).is_ok() {
            candidate.to_string()
        } else {
            let avail: Vec<Version> = Self::fetch_all_versions()?
                .iter()
                .filter_map(|s| Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(candidate, &avail, "Node")?
        };
        Ok((Self::download_url(&resolved), resolved, false))
    }
}

pub(crate) fn fetch_index_tab() -> Result<String> {
    let cache_file = config::cache_path(node_versions_cache_filename());
    language::fetch_with_cache(&cache_file, || NodeLanguage::fetch_text(index_tab_path()))
}
