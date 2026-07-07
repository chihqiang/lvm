use std::env;
use std::sync::OnceLock;

// python-build-standalone default tag (latest as of 2026-06-23)
// Override via LVM_PYTHON_TAG env var.
static PYTHON_TAG: OnceLock<String> = OnceLock::new();

pub(crate) fn python_tag() -> &'static str {
    PYTHON_TAG.get_or_init(|| env::var("LVM_PYTHON_TAG").unwrap_or_else(|_| "20260623".to_string()))
}

// Base download URL. Override via LVM_PYTHON_MIRROR env var.
// Default: https://github.com/astral-sh/python-build-standalone
pub(crate) fn download_base() -> &'static str {
    static BASE: OnceLock<String> = OnceLock::new();
    BASE.get_or_init(|| {
        env::var("LVM_PYTHON_MIRROR").unwrap_or_else(|_| {
            "https://github.com/astral-sh/python-build-standalone/releases/download".to_string()
        })
    })
}

pub(crate) fn download_url(version: &str, os: &str, arch: &str, ext: &str) -> String {
    let tag = python_tag();
    format!(
        "{}/{}/cpython-{version}+{tag}-{arch}-{os}-install_only.{ext}",
        download_base(),
        tag,
    )
}

pub(crate) fn tarball_filename(version: &str, os: &str, arch: &str, ext: &str) -> String {
    let tag = python_tag();
    format!("cpython-{version}+{tag}-{arch}-{os}-install_only.{ext}")
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        "x86" => "i686",
        other => other,
    }
}

pub(crate) fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

pub(crate) fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

pub(crate) fn python_versions_cache_filename() -> &'static str {
    "python-versions.txt"
}
