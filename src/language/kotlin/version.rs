use anyhow::{Context, Result};

use crate::config;
use crate::language;

use super::config::kotlin_versions_cache_filename;

const KOTLIN_VERSIONS_URL: &str =
    "https://api.github.com/repos/JetBrains/kotlin/releases?per_page=100";

impl super::KotlinLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions.last().cloned().context("No Kotlin versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = config::cache_path(kotlin_versions_cache_filename());
        let text = language::fetch_with_cache(&cache_file, || {
            let response = language::get_url(KOTLIN_VERSIONS_URL)
                .call()
                .context("Failed to fetch Kotlin versions")?;
            response.into_string().context("Failed to read response")
        })?;

        language::parse_github_releases(&text)
    }
}
