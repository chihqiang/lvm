pub(crate) mod config;
mod version;

use anyhow::{Result, bail};
use std::path::Path;

use super::Language;
use crate::language;

pub struct PythonLanguage;

impl Language for PythonLanguage {
    fn name(&self) -> &'static str {
        "python"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report(format!("Python {resolved} is already installed"));
            return Ok(resolved);
        }

        let os = config::target_os();
        let ext = language::archive_ext();
        let archs: &[&str] = if config::target_arch() != "x86_64" {
            &[config::target_arch(), "x86_64"]
        } else {
            &[config::target_arch()]
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved);
            }

            let url = config::download_url(&resolved, os, arch, ext);
            let tar_path = crate::config::downloads_dir_or_default()
                .join(config::tarball_filename(&resolved, os, arch, ext));

            if arch != config::target_arch() {
                language::report(format!("Using {os}-{arch} (non-native arch)"));
            }

            match language::download_and_install(
                &url,
                &tar_path,
                &resolved,
                &version_dir,
                "Python",
                |_| Ok(()),
            ) {
                Ok(()) => return Ok(resolved),
                Err(_e) if i + 1 < archs.len() => {
                    language::report(format!(
                        "Download failed for {arch}, falling back to {next}",
                        next = archs[i + 1]
                    ));
                }
                Err(e) => return Err(e),
            }
        }

        bail!("Failed to install Python {resolved}")
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let exe = std::env::consts::EXE_SUFFIX;
        version_dir
            .join("bin")
            .join(format!("python3{exe}"))
            .exists()
            || version_dir
                .join("bin")
                .join(format!("python{exe}"))
                .exists()
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![("PYTHON_HOME", self.current_link())]
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    match version {
        None => PythonLanguage::fetch_latest_version(),
        Some(v) => {
            let v = v.trim();
            if v == "system" {
                bail!("Use 'lvm use system' instead of 'lvm install system'");
            }
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = PythonLanguage::fetch_all_versions()?
                .iter()
                .filter_map(|s| semver::Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(candidate, &avail, "Python")
        }
    }
}
