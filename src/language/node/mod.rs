pub(crate) mod config;
pub(crate) mod lts;
mod version;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::Language;
use crate::language;
use crate::language::http::get_url;

pub(crate) use config::{default_packages_filename, node_mirror, npm_binary_name};

/// Node.js 语言
pub struct NodeLanguage;

impl Language for NodeLanguage {
    fn name(&self) -> &'static str {
        "node"
    }

    fn install(&self, version: Option<&str>) -> Result<String> {
        let (download_url, resolved_version, source_is_url) =
            NodeLanguage::resolve_install_version(version)?;
        let version_dir = self.version_dir(&resolved_version);
        if self.is_installed(&version_dir) {
            language::report(format!("Node {resolved_version} is already installed"));
            return Ok(resolved_version);
        }

        let archs: &[&str] = if source_is_url {
            &[config::target_arch()]
        } else if config::target_arch() != "x64" {
            &[config::target_arch(), "x64"]
        } else {
            &[config::target_arch()]
        };

        let os = config::target_os();
        let ext = language::archive_ext();

        let checksums: HashMap<String, String> = if !source_is_url {
            fetch_checksums(node_mirror(), &resolved_version)?
        } else {
            HashMap::new()
        };

        for (i, &arch) in archs.iter().enumerate() {
            if i > 0 && self.is_installed(&version_dir) {
                return Ok(resolved_version);
            }

            let url = if source_is_url {
                download_url.clone()
            } else {
                config::download_url(node_mirror(), &resolved_version, os, arch, ext)
            };

            let tar = if source_is_url {
                let filename = url
                    .rsplit('/')
                    .next()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("download.tar.gz");
                crate::config::downloads_dir()?.join(filename)
            } else {
                crate::config::downloads_dir_or_default()
                    .join(config::tarball_filename(&resolved_version, os, arch, ext))
            };

            let verify = |tar_path: &Path| -> Result<()> {
                let tarball_filename = tar_path
                    .file_name()
                    .context(format!("Invalid tar path: {}", tar_path.display()))?
                    .to_string_lossy();
                if source_is_url {
                    bail!(
                        "No checksum entry for '{tarball_filename}'; custom Node URL cannot be verified"
                    );
                }
                if let Some(expected) = checksums.get(tarball_filename.as_ref()) {
                    language::report("Verifying checksum...");
                    language::verify_sha256(tar_path, expected)?;
                    language::report("Checksum verified");
                } else {
                    language::report(format!(
                        "Warning: no checksum entry for '{tarball_filename}', verification skipped"
                    ));
                }
                Ok(())
            };

            if arch != config::target_arch() {
                language::report(format!("Using {os}-{arch} (Rosetta/emulation)"));
            }

            match language::download_and_install(
                &url, &tar, &resolved_version, &version_dir, "Node", verify,
            ) {
                Ok(()) => return Ok(resolved_version),
                Err(_e) if i + 1 < archs.len() => {
                    language::report(format!(
                        "Download failed for {arch}, falling back to {next}",
                        next = archs[i + 1]
                    ));
                }
                Err(e) => return Err(e),
            }
        }

        bail!("Failed to install Node {resolved_version}")
    }

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        let text = version::fetch_index_tab()?;
        let mut versions = NodeLanguage::parse_index_tab(&text);
        language::sort_versions(&mut versions);

        let lts_info: HashMap<String, String> = lts::parse_lts_info(&text)
            .into_iter()
            .filter_map(|(v, lts)| lts.map(|c| (v, c)))
            .collect();

        Ok(versions
            .into_iter()
            .map(|v| {
                if let Some(codename) = lts_info.get(&v) {
                    format!("{v} (LTS: {codename})")
                } else {
                    v
                }
            })
            .collect())
    }

    fn latest_version(&self) -> Result<String> {
        NodeLanguage::fetch_latest_version()
    }

    fn package_manager_binary(&self) -> Option<&'static str> {
        Some("npm")
    }

    fn packages_dir_name(&self) -> Option<&'static str> {
        Some("node_modules")
    }

    fn post_install(&self, version: &str) -> Result<()> {
        let version_dir = self.version_dir(self.strip_version_prefix(version));
        let npm_path = version_dir
            .join(crate::config::bin_dir_name())
            .join(format!("{}{}", npm_binary_name(), language::exe_suffix()));
        install_default_packages(&npm_path)
    }
}

pub(crate) fn fetch_checksums(mirror_url: &str, version: &str) -> Result<HashMap<String, String>> {
    let url = format!("{mirror_url}/v{version}/SHASUMS256.txt");
    let response = get_url(&url).call().context("Failed to fetch checksums")?;
    let text = response.into_string().context("Failed to read checksums")?;

    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((checksum, filename)) = line.split_once("  ") {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        } else if let Some((checksum, filename)) = line.split_once(' ') {
            map.insert(filename.trim().to_string(), checksum.trim().to_string());
        }
    }
    Ok(map)
}

fn install_default_packages(npm_path: &Path) -> Result<()> {
    let packages_file = crate::config::lvm_home()?.join(default_packages_filename());
    if !packages_file.exists() {
        return Ok(());
    }

    let content =
        std::fs::read_to_string(&packages_file).context("Failed to read default-packages")?;

    let packages: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if packages.is_empty() {
        return Ok(());
    }

    language::report("Installing default packages...");
    let output = std::process::Command::new(npm_path)
        .args(["install", "-g", "--quiet"])
        .args(&packages)
        .output()
        .context("Failed to install default packages")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.is_empty() {
            bail!("Failed to install some default packages");
        }
        bail!("Failed to install some default packages:\n{stderr}");
    }

    language::report("Default packages installed");
    Ok(())
}
