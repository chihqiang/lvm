use std::env;

use crate::language;

use super::NodeLanguage;

crate::define_env_once!(node_mirror, "LVM_NODE_MIRROR", "https://nodejs.org/dist");

pub(crate) fn tarball_filename(version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("node-v{version}-{os}-{arch}.{ext}")
}

pub(crate) fn download_url(mirror: &str, version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("{mirror}/v{version}/node-v{version}-{os}-{arch}.{ext}")
}

pub(crate) fn latest_version_path() -> &'static str {
    "latest/SHASUMS256.txt"
}

pub(crate) fn index_tab_path() -> &'static str {
    "index.tab"
}

pub(crate) fn tarball_prefix() -> &'static str {
    "node-v"
}

pub(crate) fn default_packages_filename() -> &'static str {
    "default-packages"
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "win",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "x86" => "x86",
        other => other,
    }
}

pub(crate) fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

pub(crate) fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

pub(crate) fn npm_binary_name() -> &'static str {
    "npm"
}

pub(crate) fn node_versions_cache_filename() -> &'static str {
    "node-versions.tab"
}

pub(crate) fn node_latest_cache_filename() -> &'static str {
    "node-latest.txt"
}

impl NodeLanguage {
    pub(crate) fn download_url(version: &str) -> String {
        download_url(
            node_mirror(),
            version,
            target_os(),
            target_arch(),
            language::archive_ext(),
        )
    }
}
