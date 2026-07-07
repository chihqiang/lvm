pub(crate) mod config;
mod version;

use anyhow::{Result, bail};
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
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Python", &resolved);
            return Ok(resolved);
        }

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
                language::download_and_install(
                    &url,
                    &tar_path,
                    &resolved,
                    &version_dir,
                    "Python",
                    |_| Ok(()),
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
    if version.is_some_and(|v| v.trim() == lvm_config::SYSTEM_VERSION_KEYWORD) {
        bail!("Use 'lvm use system' instead of 'lvm install system'");
    }
    language::resolve_version(
        "Python",
        version,
        &|| PythonLanguage::fetch_latest_version(),
        &|| PythonLanguage::fetch_all_versions(),
    )
}
