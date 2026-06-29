pub(crate) mod config;
mod version;

use anyhow::{Result, bail};
use std::path::Path;

use super::Language;
use crate::language;

pub struct JavaLanguage;

impl Language for JavaLanguage {
    fn name(&self) -> &'static str {
        "java"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Java", &resolved);
            return Ok(resolved);
        }

        let (download_url, tarball_name, checksum_hex) = version::fetch_download_info(&resolved)?;
        let tar_path = crate::config::downloads_dir_or_default().join(&tarball_name);

        let verify_java_checksum = |tar_path: &Path| -> Result<()> {
            language::report_verifying_checksum();
            language::verify_sha256(tar_path, &checksum_hex)?;
            language::report_checksum_verified();
            Ok(())
        };

        language::download_and_install(
            &download_url,
            &tar_path,
            &resolved,
            &version_dir,
            "Java",
            verify_java_checksum,
        )?;
        Ok(resolved)
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        version::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        version::fetch_latest_lts_version()
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, std::path::PathBuf)> {
        vec![("JAVA_HOME", self.current_link())]
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    match version {
        None => version::fetch_latest_lts_version(),
        Some(v) => {
            let v = v.trim().trim_start_matches('v');
            if v == crate::config::system_version_keyword() {
                bail!("Use 'lvm use system' instead of 'lvm install system'");
            }
            if let Ok(major) = v.parse::<i32>()
                && v.split('.').count() == 1
            {
                return version::fetch_latest_major_version(major);
            }
            if let Ok(ver) = semver::Version::parse(v) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = version::fetch_all_versions()?
                .iter()
                .filter_map(|s| semver::Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(v, &avail, "Java")
        }
    }
}
