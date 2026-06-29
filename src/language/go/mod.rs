pub(crate) mod config;
mod version;

use anyhow::{Context, Result, bail};
use std::fs;

use super::Language;
use crate::language;

pub(crate) use config::go_packages_bin_path;

pub struct GoLanguage;

impl Language for GoLanguage {
    fn name(&self) -> &'static str {
        "go"
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = if let Some(v) = version {
            let v = v.trim();
            if v.starts_with("lts/") {
                bail!("Go does not have LTS releases");
            }
            if v == "system" {
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
            language::report(format!("Go {resolved} is already installed"));
            return Ok(resolved);
        }

        let tar_path = Self::cached_tar(&resolved);

        let verify_go_checksum = |tar_path: &std::path::Path| -> Result<()> {
            let filename = tar_path
                .file_name()
                .context(format!("Invalid tar path: {}", tar_path.display()))?
                .to_string_lossy();
            let expected = Self::fetch_file_sha256(&resolved, filename.as_ref())?;
            language::report("Verifying checksum...");
            language::verify_sha256(tar_path, &expected)?;
            language::report("Checksum verified");
            Ok(())
        };

        language::download_and_install(
            &Self::download_url(&resolved),
            &tar_path,
            &resolved,
            &version_dir,
            "Go",
            verify_go_checksum,
        )?;
        Ok(resolved)
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
