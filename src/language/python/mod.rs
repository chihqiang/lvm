pub(crate) mod config;
mod version;

use anyhow::{Context, Result, bail};
use std::path::Path;

use super::Language;
use crate::config as lvm_config;
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
        if self.skip_if_installed(&resolved)? {
            return Ok(resolved);
        }
        let version_dir = self.version_dir(&resolved);

        let os = config::target_os();
        let native_arch = config::target_arch();
        let ext = language::archive_ext();
        let archs: &[&str] = if native_arch != "x86_64" {
            &[native_arch, "x86_64"]
        } else {
            &[native_arch]
        };

        language::install_with_fallback(
            "Python",
            &resolved,
            os,
            native_arch,
            archs,
            &|| self.is_installed(&version_dir),
            &mut |arch| {
                let url = config::download_url(&resolved, os, arch, ext);
                let tar_path = lvm_config::downloads_dir_or_default()
                    .join(config::tarball_filename(&resolved, os, arch, ext));
                let verify_python = |tar_path: &Path| -> Result<()> {
                    match fetch_sha256(tar_path) {
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
                    "Python",
                    verify_python,
                )
            },
        )
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let exe = std::env::consts::EXE_SUFFIX;
        let bin = lvm_config::BIN_DIR;
        version_dir.join(bin).join(format!("python3{exe}")).exists()
            || version_dir.join(bin).join(format!("python{exe}")).exists()
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
    language::reject_system_install(version)?;
    language::resolve_version(
        "Python",
        version,
        &|| PythonLanguage::fetch_latest_version(),
        &|| PythonLanguage::fetch_all_versions(),
    )
}

/// Look up the SHA-256 checksum for a python-build-standalone asset from the
/// `SHA256SUMS` file published with the release tag.
fn fetch_sha256(tar_path: &Path) -> Result<String> {
    let tar_filename = tar_path
        .file_name()
        .context("Invalid tar path")?
        .to_string_lossy();
    let sums_url = format!(
        "{}/{}/SHA256SUMS",
        config::download_base(),
        config::python_tag()
    );
    let text = language::get_url(&sums_url)
        .call()
        .context("Failed to fetch Python SHA256SUMS")?
        .into_string()
        .context("Failed to read Python SHA256SUMS")?;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hex), Some(name)) = (parts.next(), parts.next())
            && name == tar_filename.as_ref()
        {
            return Ok(hex.to_string());
        }
    }
    bail!("No checksum entry for {tar_filename} in SHA256SUMS");
}
