pub(crate) mod config;
mod version;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::Language;
use crate::language;

pub struct RustLanguage;

impl Language for RustLanguage {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn version_prefix(&self) -> &'static str {
        ""
    }

    fn binary_name(&self) -> &str {
        "rustc"
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let resolved = resolve_version(version)?;
        let version_dir = self.version_dir(&resolved);
        if self.is_installed(&version_dir) {
            language::report_already_installed("Rust", &resolved);
            return Ok(resolved);
        }

        let os = config::target_os();
        let archs: &[&str] = if config::target_arch() != "x86_64" {
            &[config::target_arch(), "x86_64"]
        } else {
            &[config::target_arch()]
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved);
            }

            let target = config::target_triple(os, arch);
            let url = config::download_url(&resolved, &target);
            let tar_path = crate::config::downloads_dir_or_default()
                .join(config::tarball_filename(&resolved, &target));

            if arch != config::target_arch() {
                language::report_non_native_arch(os, arch);
            }

            match language::download_and_install(
                &url,
                &tar_path,
                &resolved,
                &version_dir,
                "Rust",
                |_| Ok(()),
            ) {
                Ok(()) => {
                    self.post_install(&resolved)?;
                    return Ok(resolved);
                }
                Err(_e) if i + 1 < archs.len() => {
                    language::report_fallback(arch, archs[i + 1]);
                }
                Err(e) => return Err(e),
            }
        }

        bail!("Failed to install Rust {resolved}")
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let bin = crate::config::bin_dir_name();
        version_dir.join(bin).join("rustc").exists()
            || version_dir.join("rustc").join(bin).join("rustc").exists()
    }

    fn post_install(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(version);
        let bin_dir = version_dir.join(crate::config::bin_dir_name());
        fs::create_dir_all(&bin_dir).context("Failed to create bin directory")?;

        for sub_dir in &["rustc", "cargo"] {
            let src_bin = version_dir
                .join(sub_dir)
                .join(crate::config::bin_dir_name());
            if !src_bin.exists() {
                continue;
            }
            for entry in fs::read_dir(&src_bin).context("Failed to read binary directory")? {
                let entry = entry.context("Failed to read directory entry")?;
                let name = entry.file_name();
                let target = bin_dir.join(&name);
                if !target.exists() {
                    lvm::core::fslink::create_symlink(&entry.path(), &target)
                        .with_context(|| format!("Failed to symlink {:?}", name))?;
                }
            }
        }

        Ok(())
    }

    fn post_switch(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(version);
        let exe = language::exe_suffix();
        let bin_home = crate::config::lvm_home()
            .expect("LVM home directory is required")
            .join(crate::config::bin_dir_name());

        let cargo_target = version_dir
            .join("cargo")
            .join(crate::config::bin_dir_name())
            .join(format!("cargo{exe}"));
        let cargo_link = bin_home.join(format!("cargo{exe}"));
        lvm::core::fslink::replace_symlink(&cargo_target, &cargo_link)
            .context("Failed to create cargo symlink")?;

        Ok(())
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(crate::config::bin_dir_name())]
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
        None => RustLanguage::fetch_latest_version(),
        Some(v) => {
            let v = v.trim();
            if v == crate::config::system_version_keyword() {
                bail!("'system' is not supported for Rust");
            }
            let candidate = v.trim_start_matches('v');
            if let Ok(ver) = semver::Version::parse(candidate) {
                return Ok(ver.to_string());
            }
            let avail: Vec<semver::Version> = RustLanguage::fetch_all_versions()?
                .iter()
                .filter_map(|s| semver::Version::parse(s).ok())
                .collect();
            language::resolve_partial_version(candidate, &avail, "Rust")
        }
    }
}
