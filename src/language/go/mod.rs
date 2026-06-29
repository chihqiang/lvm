pub(crate) mod config;
mod version;

use anyhow::{Context, Result, bail};
use std::fs;

use super::Language;
use crate::language;

pub(crate) use config::{go_mirror, go_packages_bin_path};

pub struct GoLanguage;

impl Language for GoLanguage {
    fn name(&self) -> &'static str {
        "go"
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = if let Some(v) = version {
            let v = v.trim();
            if v.starts_with(crate::config::lts_prefix()) {
                bail!("Go does not have LTS releases");
            }
            if v == crate::config::system_version_keyword() {
                bail!("'system' is not supported for Go");
            }
            let v = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(v) {
                ver.to_string()
            } else {
                let avail: Vec<semver::Version> = Self::fetch_all_versions()?
                    .iter()
                    .filter_map(|s| semver::Version::parse(s).ok())
                    .collect();
                language::resolve_partial_version(v, &avail, "Go")?
            }
        } else {
            Self::fetch_latest_version()?
        };

        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Go", &resolved);
            return Ok(resolved);
        }

        let os = config::target_os();
        let ext = language::archive_ext();
        let archs: &[&str] = if config::target_arch() != "amd64" {
            &[config::target_arch(), "amd64"]
        } else {
            &[config::target_arch()]
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved);
            }

            let url = config::download_url(go_mirror(), &resolved, os, arch, ext);
            let tar_path = crate::config::downloads_dir_or_default()
                .join(config::tarball_filename(&resolved, os, arch, ext));

            let verify_go_checksum = |tar_path: &std::path::Path| -> Result<()> {
                let filename = tar_path
                    .file_name()
                    .context(format!("Invalid tar path: {}", tar_path.display()))?
                    .to_string_lossy();
                let expected = Self::fetch_file_sha256(&resolved, filename.as_ref())?;
                language::report_verifying_checksum();
                language::verify_sha256(tar_path, &expected)?;
                language::report_checksum_verified();
                Ok(())
            };

            if arch != config::target_arch() {
                language::report_non_native_arch(os, arch);
            }

            match language::download_and_install(
                &url,
                &tar_path,
                &resolved,
                &version_dir,
                "Go",
                verify_go_checksum,
            ) {
                Ok(()) => return Ok(resolved),
                Err(_e) if i + 1 < archs.len() => {
                    language::report_fallback(arch, archs[i + 1]);
                }
                Err(e) => return Err(e),
            }
        }

        bail!("Failed to install Go {resolved}")
    }

    fn post_switch(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(version);
        let (packages_dir, bin_dir_name) = go_packages_bin_path();
        let packages_bin = version_dir.join(packages_dir).join(bin_dir_name);
        fs::create_dir_all(&packages_bin).context("Failed to create Go packages directory")
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        let (pkgs, bin) = go_packages_bin_path();
        vec![self.current_link().join(pkgs).join(bin)]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        let (pkgs, _) = go_packages_bin_path();
        vec![("GOPATH", self.current_link().join(pkgs))]
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }
}
