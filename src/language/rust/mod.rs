pub(crate) mod config;
mod version;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::Language;
use crate::config as lvm_config;
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
        if self.skip_if_installed(&resolved)? {
            return Ok(resolved);
        }
        let version_dir = self.version_dir(&resolved);
        let os = config::target_os();
        let native_arch = config::target_arch();
        let archs: &[&str] = if native_arch != "x86_64" {
            &[native_arch, "x86_64"]
        } else {
            &[native_arch]
        };

        language::install_with_fallback(
            "Rust",
            &resolved,
            os,
            native_arch,
            archs,
            &|| self.is_installed(&version_dir),
            &mut |arch| {
                let target = config::target_triple(os, arch);
                let url = config::download_url(&resolved, &target);
                let tar_path = lvm_config::downloads_dir_or_default()
                    .join(config::tarball_filename(&resolved, &target));
                let verify_rust = |tar_path: &Path| -> Result<()> {
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
                    "Rust",
                    verify_rust,
                )
            },
        )
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        let bin = lvm_config::BIN_DIR;
        version_dir.join(bin).join("rustc").exists()
            || version_dir.join("rustc").join(bin).join("rustc").exists()
    }

    fn post_install(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(version);
        let bin_dir = version_dir.join(lvm_config::BIN_DIR);
        fs::create_dir_all(&bin_dir).context("Failed to create bin directory")?;

        for sub_dir in &["rustc", "cargo"] {
            let src_bin = version_dir.join(sub_dir).join(lvm_config::BIN_DIR);
            if !src_bin.exists() {
                continue;
            }
            for entry in fs::read_dir(&src_bin).context("Failed to read binary directory")? {
                let entry = entry.context("Failed to read directory entry")?;
                let name = entry.file_name();
                let target = bin_dir.join(&name);
                if !target.exists() {
                    crate::core::fslink::create_symlink(&entry.path(), &target)
                        .with_context(|| format!("Failed to symlink {:?}", name))?;
                }
            }
        }

        Ok(())
    }

    fn post_switch(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(version);
        let exe = language::exe_suffix();
        let bin_home = lvm_config::lvm_home()
            .context("LVM home directory is required")?
            .join(lvm_config::BIN_DIR);

        let cargo_target = version_dir
            .join("cargo")
            .join(lvm_config::BIN_DIR)
            .join(format!("cargo{exe}"));
        let cargo_link = bin_home.join(format!("cargo{exe}"));
        crate::core::fslink::replace_symlink(&cargo_target, &cargo_link)
            .context("Failed to create cargo symlink")?;

        Ok(())
    }

    fn extra_bin_links(&self) -> Vec<std::path::PathBuf> {
        let exe = language::exe_suffix();
        let bin_home = lvm_config::lvm_home_cached().join(lvm_config::BIN_DIR);
        vec![bin_home.join(format!("cargo{exe}"))]
    }

    fn env_extra_paths(&self) -> Vec<std::path::PathBuf> {
        vec![self.current_link().join(lvm_config::BIN_DIR)]
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        Self::fetch_all_versions()
    }

    fn latest_version(&self) -> Result<String> {
        Self::fetch_latest_version()
    }
}

fn resolve_version(version: Option<&str>) -> Result<String> {
    language::reject_system_install(version)?;
    language::resolve_version(
        "Rust",
        version,
        &|| RustLanguage::fetch_latest_version(),
        &|| RustLanguage::fetch_all_versions(),
    )
}

/// Fetch the SHA-256 checksum published alongside a Rust dist tarball
/// (e.g. `rust-1.80.0-x86_64-unknown-linux-gnu.tar.gz.sha256`).
fn fetch_sha256(download_url: &str) -> Result<String> {
    let sha_url = format!("{download_url}.sha256");
    let text = language::get_url(&sha_url)
        .call()
        .context("Failed to fetch Rust checksum")?
        .into_string()
        .context("Failed to read Rust checksum")?;
    let hex = text.split_whitespace().next().unwrap_or_default();
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid Rust checksum file content");
    }
    Ok(hex.to_string())
}
