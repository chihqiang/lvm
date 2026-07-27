use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::config as lvm_config;
use crate::language;

use super::config::{java_mirror, java_versions_cache_filename, target_arch, target_os};

pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
    let cache_file = lvm_config::cache_path(java_versions_cache_filename());

    let text = language::fetch_with_cache(&cache_file, || {
        let majors = fetch_available_majors()?;
        let mut all_versions = Vec::new();
        for major in majors {
            let releases = fetch_major_versions(major, 3)?;
            all_versions.extend(releases);
        }
        language::sort_versions(&mut all_versions);
        all_versions.dedup();
        Ok(all_versions.join("\n"))
    })?;

    let versions: Vec<String> = text.lines().map(String::from).collect();
    Ok(versions)
}

pub(crate) fn fetch_latest_lts_version() -> Result<String> {
    let text = adoptium_get("/info/available_releases")?;
    let json: Value = serde_json::from_str(&text)?;
    let lts_major = json["most_recent_lts"]
        .as_i64()
        .context("No LTS release found")? as i32;
    let releases = fetch_major_versions(lts_major, 1)?;
    releases.into_iter().next().context("No LTS version found")
}

pub(crate) fn fetch_latest_major_version(major: i32) -> Result<String> {
    let releases = fetch_major_versions(major, 1)?;
    releases
        .into_iter()
        .next()
        .with_context(|| format!("No release found for JDK {major}"))
}

pub(crate) fn fetch_download_info(version: &str) -> Result<(String, String, String)> {
    let major: i32 = version
        .split('.')
        .next()
        .context("Invalid version")?
        .parse()
        .context("Invalid version")?;

    let path = format!("/assets/feature_releases/{major}/ga?page=0&page_size=10&sort_order=DESC");
    let text = adoptium_get(&path)?;
    let releases: Vec<Value> = serde_json::from_str(&text)?;

    let os = target_os();
    let arch = target_arch();
    let version_prefix = format!("{version}+");

    let arch_tries: &[&str] = if arch != "x64" {
        &[arch, "x64"]
    } else {
        &[arch]
    };

    for &try_arch in arch_tries {
        for release in &releases {
            let semver = release["version_data"]["semver"]
                .as_str()
                .context("Invalid release data")?;
            if (semver == version || semver.starts_with(&version_prefix))
                && let Some(binaries) = release["binaries"].as_array()
            {
                for binary in binaries {
                    if binary["os"].as_str() == Some(os)
                        && binary["architecture"].as_str() == Some(try_arch)
                        && binary["image_type"].as_str() == Some("jdk")
                    {
                        let download_url = binary["package"]["link"]
                            .as_str()
                            .context("No download link")?
                            .to_string();
                        let tarball_name = binary["package"]["name"]
                            .as_str()
                            .context("No package name")?
                            .to_string();
                        let checksum = binary["package"]["checksum"]
                            .as_str()
                            .context("No checksum")?
                            .to_string();
                        let checksum_hex = checksum
                            .strip_prefix("sha256-")
                            .unwrap_or(&checksum)
                            .to_string();
                        return Ok((download_url, tarball_name, checksum_hex));
                    }
                }
            }
        }
    }

    bail!("No download found for Java {version} on {os}-{arch}")
}

fn adoptium_get(path: &str) -> Result<String> {
    let url = format!("{}{}", java_mirror(), path);
    let response = language::get_url(&url)
        .call()
        .context("Failed to fetch Java data")?;
    response.into_string().context("Failed to read response")
}

fn fetch_available_majors() -> Result<Vec<i32>> {
    let text = adoptium_get("/info/available_releases")?;
    let json: Value = serde_json::from_str(&text)?;
    Ok(json["available_releases"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect()
        })
        .unwrap_or_default())
}

fn fetch_major_versions(major: i32, limit: i32) -> Result<Vec<String>> {
    let path =
        format!("/assets/feature_releases/{major}/ga?page=0&page_size={limit}&sort_order=DESC");
    let text = adoptium_get(&path)?;
    let releases: Vec<Value> = serde_json::from_str(&text)?;
    Ok(releases
        .iter()
        .filter_map(|r| {
            r["version_data"]["semver"]
                .as_str()
                .map(|s| s.split('+').next().unwrap_or(s).to_string())
        })
        .collect())
}
