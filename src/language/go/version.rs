use anyhow::{Context, Result, bail};

use crate::config;
use crate::language;

use super::GoLanguage;
use super::config::{go_mirror, go_versions_cache_filename, go_versions_query_suffix};

impl GoLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions
            .last()
            .cloned()
            .context("No stable Go versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let text = Self::fetch_versions_json()?;
        Self::parse_versions_json(&text)
    }

    pub(crate) fn fetch_file_sha256(version: &str, filename: &str) -> Result<String> {
        let text = Self::fetch_versions_json()?;
        let versions: Vec<serde_json::Value> =
            serde_json::from_str(&text).context("Failed to parse Go versions")?;
        let go_version = format!("go{version}");

        let Some(checksum) = versions
            .iter()
            .find(|entry| {
                entry.get("version").and_then(serde_json::Value::as_str) == Some(&go_version)
            })
            .and_then(|entry| entry.get("files"))
            .and_then(serde_json::Value::as_array)
            .and_then(|files| {
                files.iter().find_map(|file| {
                    let name = file.get("filename")?.as_str()?;
                    if name == filename {
                        file.get("sha256")?.as_str().map(str::to_string)
                    } else {
                        None
                    }
                })
            })
        else {
            bail!("No checksum found for {filename}");
        };

        Ok(checksum)
    }

    fn fetch_versions_json() -> Result<String> {
        let cache_file = config::cache_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(".lvm/cache"))
            .join(go_versions_cache_filename());
        language::fetch_with_cache(&cache_file, || {
            let url = format!("{}{}", go_mirror(), go_versions_query_suffix());
            let response = language::get_url(&url)
                .call()
                .context("Failed to fetch Go versions")?;
            response.into_string().context("Failed to read response")
        })
    }

    fn parse_versions_json(text: &str) -> Result<Vec<String>> {
        let versions: Vec<serde_json::Value> =
            serde_json::from_str(text).context("Failed to parse Go versions")?;

        let mut result: Vec<String> = versions
            .iter()
            .filter(|entry| {
                entry
                    .get("stable")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|entry| {
                let v = entry.get("version")?.as_str()?;
                v.strip_prefix("go").map(String::from)
            })
            .collect();

        language::sort_versions(&mut result);
        Ok(result)
    }
}
