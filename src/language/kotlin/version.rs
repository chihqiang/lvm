use anyhow::{Context, Result};

use crate::config as lvm_config;
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
        let cache_file = lvm_config::cache_path(kotlin_versions_cache_filename());
        language::fetch_github_releases_paginated(KOTLIN_VERSIONS_URL, &cache_file)
    }
}
