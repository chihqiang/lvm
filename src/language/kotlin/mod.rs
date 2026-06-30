pub(crate) mod config;
mod version;

use std::path::Path;

use anyhow::{Result, bail};

use super::Language;
use crate::language;

pub struct KotlinLanguage;

impl Language for KotlinLanguage {
    fn name(&self) -> &'static str {
        "kotlin"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Kotlin", &resolved);
            return Ok(resolved);
        }

        let url = config::download_url(&resolved);
        let tar_path =
            crate::config::downloads_dir_or_default().join(config::tarball_filename(&resolved));

        match language::download_and_install(
            &url,
            &tar_path,
            &resolved,
            &version_dir,
            "Kotlin",
            |_| Ok(()),
        ) {
            Ok(()) => Ok(resolved),
            Err(e) => Err(e),
        }
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let bin = crate::config::bin_dir_name();
        version_dir.join(bin).join("kotlinc").exists()
            || version_dir.join(bin).join("kotlinc.bat").exists()
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(crate::config::bin_dir_name())]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![("KOTLIN_HOME", self.current_link())]
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    match version {
        None => KotlinLanguage::fetch_latest_version(),
        Some(v) => {
            let v = v.trim();
            if v == crate::config::system_version_keyword() {
                bail!("'system' is not supported for Kotlin");
            }
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = KotlinLanguage::fetch_all_versions()?
                .iter()
                .filter_map(|s| semver::Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(candidate, &avail, "Kotlin")
        }
    }
}
