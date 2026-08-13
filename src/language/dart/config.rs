use std::env;

crate::define_env_once!(
    dart_mirror,
    "LVM_DART_MIRROR",
    "https://storage.googleapis.com/dart-archive"
);

const DART_EXT: &str = "zip";

pub(crate) fn tarball_filename(version: &str, os: &str, arch: &str) -> String {
    // The upstream tarball name contains no version, so embed it in the local
    // cache filename to avoid different versions sharing one cache entry.
    format!("dartsdk-{version}-{os}-{arch}-release.{DART_EXT}")
}

pub(crate) fn download_url(version: &str, os: &str, arch: &str) -> String {
    format!(
        "{}/channels/stable/release/{version}/sdk/dartsdk-{os}-{arch}-release.{DART_EXT}",
        dart_mirror()
    )
}

fn os_name(system_os: &str) -> &str {
    match system_os {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

fn arch_name(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        other => other,
    }
}

pub(crate) fn target_os() -> &'static str {
    os_name(env::consts::OS)
}

pub(crate) fn target_arch() -> &'static str {
    arch_name(env::consts::ARCH)
}

pub(crate) fn dart_versions_cache_filename() -> &'static str {
    "dart-versions.txt"
}

pub(crate) fn dart_latest_version_cache_filename() -> &'static str {
    "dart-latest-version.json"
}
