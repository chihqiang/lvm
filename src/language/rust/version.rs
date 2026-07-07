use anyhow::{Context, Result};

use crate::config as lvm_config;
use crate::language;

use super::config::rust_versions_cache_filename;

const RUST_VERSIONS_URL: &str =
    "https://api.github.com/repos/rust-lang/rust/releases?per_page=100";

impl super::RustLanguage {
    pub(crate) fn fetch_latest_version() -> Result<String> {
        let versions = Self::fetch_all_versions()?;
        versions.last().cloned().context("No Rust versions found")
    }

    pub(crate) fn fetch_all_versions() -> Result<Vec<String>> {
        let cache_file = lvm_config::cache_path(rust_versions_cache_filename());
        language::fetch_github_releases_paginated(RUST_VERSIONS_URL, &cache_file)
    }
}
