use std::env;
use std::sync::OnceLock;

static GO_MIRROR: OnceLock<String> = OnceLock::new();

pub(crate) fn go_mirror() -> &'static str {
    GO_MIRROR.get_or_init(|| {
        env::var("LVM_GO_MIRROR").unwrap_or_else(|_| "https://go.dev/dl".to_string())
    })
}

pub(crate) fn tarball_filename(version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("go{version}.{os}-{arch}.{ext}")
}

pub(crate) fn download_url(mirror: &str, version: &str, os: &str, arch: &str, ext: &str) -> String {
    format!("{mirror}/go{version}.{os}-{arch}.{ext}")
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        "x86" => "386",
        other => other,
    }
}

pub(crate) fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

pub(crate) fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

pub(crate) fn go_packages_bin_path() -> (&'static str, &'static str) {
    ("packages", "bin")
}

pub(crate) fn go_versions_cache_filename() -> &'static str {
    "go-versions.json"
}

pub(crate) fn go_versions_query_suffix() -> &'static str {
    "/?mode=json&include=all"
}


