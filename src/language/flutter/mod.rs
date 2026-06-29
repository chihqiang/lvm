pub(crate) mod config;
mod version;

use anyhow::{Context, Result, bail};
use std::path::Path;
use zip::ZipArchive;

use super::Language;
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

        let os = config::target_os();
        let archs: &[&str] = if config::target_arch() != "x64" {
            &[config::target_arch(), "x64"]
        } else {
            &[config::target_arch()]
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved);
            }

            let url = config::download_url(&resolved, os, arch);
            let tar_path = crate::config::downloads_dir_or_default()
                .join(config::tarball_filename(&resolved, os, arch));

            if arch != config::target_arch() {
                language::report_non_native_arch(os, arch);
            }

            let verify_zip = |path: &Path| -> Result<()> {
                let file = std::fs::File::open(path)
                    .with_context(|| format!("Failed to open {}", path.display()))?;
                ZipArchive::new(file).context("Corrupted zip archive")?;
                Ok(())
            };

            match language::download_and_install(
                &url,
                &tar_path,
                &resolved,
                &version_dir,
                "Flutter",
                verify_zip,
            ) {
                Ok(()) => return Ok(resolved),
                Err(_e) if i + 1 < archs.len() => {
                    language::report_fallback(arch, archs[i + 1]);
                }
                Err(e) => return Err(e),
            }
        }

        bail!("Failed to install Flutter {resolved}")
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(crate::config::bin_dir_name())]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![("FLUTTER_HOME", self.current_link())]
    }

    fn package_manager_binary(&self) -> Option<&'static str> {
        Some("dart")
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    match version {
        None => FlutterLanguage::fetch_latest_version(),
        Some(v) => {
            let v = v.trim();
            if v == crate::config::system_version_keyword() {
                bail!("Use 'lvm use system' instead of 'lvm install system'");
            }
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = FlutterLanguage::fetch_all_versions()?
                .iter()
                .filter_map(|s| semver::Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(candidate, &avail, "Flutter")
        }
    }
}
