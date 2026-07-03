pub(crate) mod config;
mod version;

use anyhow::{Result, bail};

use super::Language;
use crate::core::extract;
use crate::language;

pub struct FlutterLanguage;

impl Language for FlutterLanguage {
    fn name(&self) -> &'static str {
        "flutter"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Flutter", &resolved);
            return Ok(resolved);
        }

        // Flutter SDK is per-OS (not per-arch); skip arch fallback.
        let os = config::target_os();
        let arch = config::target_arch();
        let url = config::download_url(&resolved, os, arch);
        let tar_path = crate::config::downloads_dir_or_default()
            .join(config::tarball_filename(&resolved, os, arch));

        language::download_and_install(
            &url,
            &tar_path,
            &resolved,
            &version_dir,
            "Flutter",
            extract::verify_zip_archive,
        )?;

        Ok(resolved)
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(crate::config::BIN_DIR)]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![
            ("FLUTTER_HOME", self.current_link()),
            ("PUB_CACHE", self.current_link().join("pub-cache")),
        ]
    }

    fn package_manager_binary(&self) -> Option<&'static str> {
        Some("dart")
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    if version.is_some_and(|v| v.trim() == crate::config::SYSTEM_VERSION_KEYWORD) {
        bail!("Use 'lvm use system' instead of 'lvm install system'");
    }
    language::resolve_version(
        "Flutter",
        version,
        &|| FlutterLanguage::fetch_latest_version(),
        &|| FlutterLanguage::fetch_all_versions(),
    )
}
