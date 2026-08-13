pub(crate) mod config;
mod version;

use std::path::Path;

use anyhow::{Context, Result, bail};

use super::Language;
use crate::config as lvm_config;
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
        if self.skip_if_installed(&resolved)? {
            return Ok(resolved);
        }
        let version_dir = self.version_dir(&resolved);

        let url = config::download_url(&resolved);
        let tar_path =
            lvm_config::downloads_dir_or_default().join(config::tarball_filename(&resolved));

        let verify_kotlin = |tar_path: &Path| -> Result<()> {
            match fetch_sha256(&url) {
                Ok(hex) => {
                    language::report_verifying_checksum();
                    language::verify_sha256(tar_path, &hex)?;
                    language::report_checksum_verified();
                    Ok(())
                }
                Err(e) => {
                    language::report(format!(
                        "Warning: checksum verification skipped for {} ({e})",
                        tar_path.display()
                    ));
                    Ok(())
                }
            }
        };

        language::download_and_install(
            &url,
            &tar_path,
            &resolved,
            &version_dir,
            "Kotlin",
            verify_kotlin,
        )?;
        Ok(resolved)
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let bin = lvm_config::BIN_DIR;
        version_dir.join(bin).join("kotlinc").exists()
            || version_dir.join(bin).join("kotlinc.bat").exists()
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(lvm_config::BIN_DIR)]
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
            language::reject_system_install(Some(v))?;
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

/// Fetch the SHA-256 checksum published alongside a Kotlin compiler zip
/// (e.g. `kotlin-compiler-2.1.0.zip.sha256`).
fn fetch_sha256(download_url: &str) -> Result<String> {
    let sha_url = format!("{download_url}.sha256");
    let text = language::get_url(&sha_url)
        .call()
        .context("Failed to fetch Kotlin checksum")?
        .into_string()
        .context("Failed to read Kotlin checksum")?;
    let hex = text.split_whitespace().next().unwrap_or_default();
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid Kotlin checksum file content");
    }
    Ok(hex.to_string())
}
