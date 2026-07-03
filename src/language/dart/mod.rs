pub(crate) mod config;
mod version;

use anyhow::{Result, bail};

use super::Language;
use crate::core::extract;
use crate::language;

pub struct DartLanguage;

impl Language for DartLanguage {
    fn name(&self) -> &'static str {
        "dart"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Dart", &resolved);
            return Ok(resolved);
        }

        let os = config::target_os();
        let native_arch = config::target_arch();
        let archs: &[&str] = if native_arch != "x64" {
            &[native_arch, "x64"]
        } else {
            &[native_arch]
        };

        language::install_with_fallback(
            "Dart",
            &resolved,
            os,
            native_arch,
            archs,
            &|| self.is_installed(&version_dir),
            &mut |arch| {
                let url = config::download_url(&resolved, os, arch);
                let tar_path = crate::config::downloads_dir_or_default()
                    .join(config::tarball_filename(&resolved, os, arch));

                language::download_and_install(
                    &url,
                    &tar_path,
                    &resolved,
                    &version_dir,
                    "Dart",
                    extract::verify_zip_archive,
                )
            },
        )
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
            ("DART_HOME", self.current_link()),
            ("PUB_CACHE", self.current_link().join("pub-cache")),
        ]
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    if version.is_some_and(|v| v.trim() == crate::config::SYSTEM_VERSION_KEYWORD) {
        bail!("Use 'lvm use system' instead of 'lvm install system'");
    }
    language::resolve_version(
        "Dart",
        version,
        &|| DartLanguage::fetch_latest_version(),
        &|| DartLanguage::fetch_all_versions(),
    )
}
