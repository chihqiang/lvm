use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use lvm::language::Language;
use lvm::language::node::NodeLanguage;

/// Language whose executables live at the root of the version directory.
struct RootBinLanguage;

impl Language for RootBinLanguage {
    fn name(&self) -> &'static str {
        "rootbin"
    }

    fn install(&self, _version: Option<&str>) -> Result<String> {
        unimplemented!()
    }

    fn latest_version(&self) -> Result<String> {
        unimplemented!()
    }

    fn bin_subdir(&self) -> &'static str {
        ""
    }
}

fn node_exe() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

/// Official Node.js archives unpack with `bin/node` on Unix but with
/// `node.exe` at the archive root on Windows.
fn expected_bin_dir(version_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        version_dir.to_path_buf()
    } else {
        version_dir.join("bin")
    }
}

fn other_bin_dir(version_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        version_dir.join("bin")
    } else {
        version_dir.to_path_buf()
    }
}

#[test]
fn bin_subdir_matches_archive_layout() {
    if cfg!(windows) {
        assert_eq!(NodeLanguage.bin_subdir(), "");
    } else {
        assert_eq!(NodeLanguage.bin_subdir(), "bin");
    }
}

#[test]
fn is_installed_accepts_platform_layout() {
    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v24.22.1");
    let bin_dir = expected_bin_dir(&version_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join(node_exe()), b"").unwrap();

    assert!(NodeLanguage.is_installed(&version_dir));
}

#[test]
fn is_installed_rejects_other_platform_layout() {
    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v24.22.1");
    let bin_dir = other_bin_dir(&version_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join(node_exe()), b"").unwrap();

    assert!(!NodeLanguage.is_installed(&version_dir));
}

#[test]
fn empty_bin_subdir_resolves_to_version_root() {
    let version_dir = Path::new("/fake/lvm/rootbin/v1.0.0");
    assert_eq!(RootBinLanguage.version_bin_dir(version_dir), version_dir);

    let dir = tempfile::tempdir().unwrap();
    let version_dir = dir.path().join("v1.0.0");
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(version_dir.join(RootBinLanguage.bin_file_name()), b"").unwrap();

    assert!(RootBinLanguage.is_installed(&version_dir));
}

#[test]
fn npm_executable_matches_archive_layout() {
    let npm = NodeLanguage
        .package_manager_executable()
        .expect("node provides a package manager");

    if cfg!(windows) {
        assert_eq!(npm, "npm.cmd");
    } else {
        assert_eq!(npm, "npm");
    }
}
