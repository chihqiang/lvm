use semver::Version;

use anyhow::{Context, Result, anyhow, bail};

/// Parse a GitHub releases API JSON response and extract stable versions.
/// Filters out draft and prerelease entries. Expects `tag_name` to contain
/// a semver version string (with or without a leading `v`).
pub fn parse_github_releases(json: &str) -> Result<Vec<String>> {
    let root: Vec<serde_json::Value> =
        serde_json::from_str(json).context("Failed to parse GitHub releases JSON")?;

    let mut versions: Vec<Version> = Vec::new();
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
        let ver_str = tag.trim_start_matches('v');
        if let Ok(ver) = Version::parse(ver_str)
            && ver.pre.is_empty()
        {
            versions.push(ver);
        }
    }

    versions.sort();
    versions.dedup();
    Ok(versions.into_iter().map(|v| v.to_string()).collect())
}

/// Fetch all stable versions from a GitHub releases API endpoint with pagination.
/// `base_url` should already include `?per_page=100`. Pages are fetched until
/// fewer than 100 results are returned. Results are cached via `cache_file`.
pub fn fetch_github_releases_paginated(
    base_url: &str,
    cache_file: &std::path::Path,
) -> Result<Vec<String>> {
    use crate::core::http::{fetch_with_cache, get_url};

    let text = fetch_with_cache(cache_file, || {
        let mut all_versions = Vec::new();
        let separator = if base_url.contains('?') { "&" } else { "?" };

        for page in 1..=10 {
            let url = format!("{base_url}{separator}page={page}");
            let response = get_url(&url)
                .call()
                .context("Failed to fetch GitHub releases")?;
            let page_text = response
                .into_string()
                .context("Failed to read GitHub releases response")?;
            let versions = parse_github_releases(&page_text)?;
            let count = versions.len();
            all_versions.extend(versions);
            if count < 100 {
                break;
            }
        }

        Ok(all_versions.join("\n"))
    })?;

    Ok(text.lines().map(String::from).collect())
}

/// Resolve a version string for a language.
///
/// If `version` is `None`, calls `fetch_latest`.
/// Otherwise trims, strips `v` prefix, tries exact semver parse,
/// then falls back to partial match against `fetch_all` results.
///
/// The caller should validate language-specific constraints (e.g. system
/// keyword, LTS prefix) before calling this function.
pub fn resolve_version(
    lang_name: &str,
    version: Option<&str>,
    fetch_latest: &dyn Fn() -> Result<String>,
    fetch_all: &dyn Fn() -> Result<Vec<String>>,
) -> Result<String> {
    match version {
        None => fetch_latest(),
        Some(v) => {
            let v = v.trim();
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<Version> = fetch_all()?
                .iter()
                .filter_map(|s| Version::parse(s).ok())
                .collect();
            resolve_partial_version(candidate, &avail, lang_name)
        }
    }
}

pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_ver = Version::parse(a).ok();
    let b_ver = Version::parse(b).ok();
    match (a_ver, b_ver) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        _ => a.cmp(b),
    }
}

pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_versions(a, b));
}

pub fn resolve_partial_version(candidate: &str, avail: &[Version], lang: &str) -> Result<String> {
    let parts: Vec<&str> = candidate.split('.').collect();
    let want_major = parts.first().and_then(|s| s.parse::<u64>().ok());
    if want_major.is_none() {
        bail!("Invalid version '{candidate}' for {lang}");
    }
    let want_minor = parts.get(1).and_then(|s| s.parse::<u64>().ok());

    let best = avail
        .iter()
        .filter(|av| {
            want_major.is_none_or(|maj| av.major == maj)
                && want_minor.is_none_or(|min| av.minor == min)
        })
        .max()
        .cloned();
    match best {
        Some(v) => Ok(v.to_string()),
        None => {
            // Sort a copy so the suggestion is always newest-first regardless
            // of whether the caller pre-sorted `avail`.
            let mut sorted: Vec<&Version> = avail.iter().collect();
            sorted.sort();
            let mut similar = sorted
                .iter()
                .rev()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();
            similar.truncate(5);
            if similar.is_empty() {
                Err(anyhow!(
                    "Could not find {lang} version matching: {candidate}"
                ))
            } else {
                Err(anyhow!(
                    "Could not find {lang} version matching: {candidate}, available versions: {}",
                    similar.join(", ")
                ))
            }
        }
    }
}
