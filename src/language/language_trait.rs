use std::fs;
use std::path::{Path, PathBuf};

use crate::core::alias;
use crate::core::config;
use crate::core::fslink;
use crate::core::report::report;
use crate::core::version::compare_versions;
use anyhow::{Context, Result, bail};

pub trait Language: Send + Sync {
    fn name(&self) -> &str;

    fn install(&self, version: Option<&str>) -> Result<String>;

    fn latest_version(&self) -> Result<String>;

    // ─── Language constants (can override) ───

    fn version_prefix(&self) -> &'static str {
        "v"
    }

    fn strip_version_prefix<'a>(&self, version: &'a str) -> &'a str {
        version.trim_start_matches(self.version_prefix())
    }

    fn subdir_name(&self) -> &str {
        self.name()
    }

    fn binary_name(&self) -> &str {
        self.name()
    }

    // ─── Path helpers ───

    fn lvm_dir(&self) -> PathBuf {
        config::lvm_home()
            .unwrap_or_else(|_| std::path::PathBuf::from(".lvm"))
            .join(self.subdir_name())
    }

    fn current_link(&self) -> PathBuf {
        config::lvm_home()
            .unwrap_or_else(|_| std::path::PathBuf::from(".lvm"))
            .join(config::CURRENT_DIR)
            .join(self.subdir_name())
    }

    fn bin_link(&self) -> PathBuf {
        config::lvm_home()
            .unwrap_or_else(|_| std::path::PathBuf::from(".lvm"))
            .join(config::BIN_DIR)
            .join(format!(
                "{}{}",
                self.binary_name(),
                std::env::consts::EXE_SUFFIX
            ))
    }

    fn version_dir(&self, version: &str) -> PathBuf {
        let prefix = self.version_prefix();
        self.lvm_dir()
            .join(format!("{prefix}{}", self.strip_version_prefix(version)))
    }

    fn is_installed(&self, version_dir: &Path) -> bool {
        version_dir
            .join(config::BIN_DIR)
            .join(format!(
                "{}{}",
                self.binary_name(),
                std::env::consts::EXE_SUFFIX
            ))
            .exists()
    }

    // ─── Optional overrides ───

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        bail!("Remote version listing is not supported for this language")
    }

    fn post_install(&self, _version: &str) -> Result<()> {
        Ok(())
    }

    fn post_switch(&self, _version: &str) -> Result<()> {
        Ok(())
    }

    fn env_extra_paths(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn env_extra_vars(&self) -> Vec<(&'static str, PathBuf)> {
        vec![]
    }

    fn package_manager_binary(&self) -> Option<&'static str> {
        None
    }

    fn packages_dir_name(&self) -> Option<&'static str> {
        None
    }

    fn rc_version(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn use_version(&self, version: &str, set_default: bool) -> Result<()> {
        let version_dir = self.version_dir(version);

        if !self.is_installed(&version_dir) {
            bail!("{} {version} is not installed", self.name());
        }

        fslink::use_version_symlinks(
            &version_dir,
            &self.current_link(),
            &self.bin_link(),
            self.binary_name(),
        )?;

        self.post_switch(version)?;

        if set_default {
            alias::set_default_version(self.name(), version)
                .context("Failed to write default config")?;
        }

        report(format!("Switched to {} {version}!", self.name()));
        Ok(())
    }

    // ─── Default implementations (identical for Node/Go) ───

    fn list_installed(&self) -> Result<Vec<String>> {
        let dir = self.lvm_dir();
        let prefix = self.version_prefix();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let read_dir = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) => {
                report(format!("Warning: failed to read {}: {e}", dir.display()));
                return Ok(Vec::new());
            }
        };
        let mut versions = Vec::new();
        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report(format!("Warning: skipping entry: {e}"));
                    continue;
                }
            };
            if entry.file_type().is_ok_and(|t| t.is_dir())
                && let Some(name) = entry.file_name().to_str()
                && let Some(ver) = name.strip_prefix(prefix)
                && self.is_installed(&entry.path())
            {
                versions.push(ver.to_string());
            }
        }
        versions.sort_by(|a, b| compare_versions(a, b));
        Ok(versions)
    }

    fn format_installed(&self, versions: &[String]) -> Result<Vec<String>> {
        Ok(fslink::format_installed_versions(
            self.version_prefix(),
            self.current_version()?.as_deref(),
            alias::get_default_version(self.name())?.as_deref(),
            versions,
        ))
    }

    fn current_version(&self) -> Result<Option<String>> {
        let link = self.current_link();
        match fs::read_link(&link) {
            Ok(target) if target.exists() => {
                let dir_name = target
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| self.strip_version_prefix(s).to_string());
                Ok(dir_name)
            }
            _ => Ok(None),
        }
    }

    fn uninstall(&self, version: &str) -> Result<()> {
        let version = self.strip_version_prefix(version);
        fslink::uninstall_version(
            &self.version_dir(version),
            &self.current_link(),
            &self.bin_link(),
            self.current_version()?.as_deref(),
            version,
        )
        .with_context(|| format!("Failed to uninstall {} version", self.name()))
    }

    fn binary_path(&self, version: &str) -> Result<String> {
        let version = self.strip_version_prefix(version);
        let version_dir = self.version_dir(version);
        let bin = version_dir.join(config::BIN_DIR).join(format!(
            "{}{}",
            self.binary_name(),
            std::env::consts::EXE_SUFFIX
        ));
        if bin.exists() {
            Ok(bin.to_string_lossy().to_string())
        } else {
            bail!("{version} is not installed")
        }
    }
}
