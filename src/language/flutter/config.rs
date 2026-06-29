use std::env;
use std::sync::OnceLock;

static FLUTTER_MIRROR: OnceLock<String> = OnceLock::new();

pub(crate) fn flutter_mirror() -> &'static str {
    FLUTTER_MIRROR.get_or_init(|| {
        env::var("LVM_FLUTTER_MIRROR").unwrap_or_else(|_| {
            "https://storage.googleapis.com/flutter_infra_release/releases".to_string()
        })
    })
}

const FLUTTER_EXT: &str = "zip";

pub(crate) fn tarball_filename(version: &str, os: &str, _arch: &str) -> String {
    format!("flutter_{os}_{version}-stable.{FLUTTER_EXT}")
}

pub(crate) fn download_url(version: &str, os: &str, _arch: &str) -> String {
    format!(
        "{}/stable/{}/flutter_{}_{}-stable.{FLUTTER_EXT}",
        flutter_mirror(),
        os,
        os,
        version,
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

pub(crate) fn releases_url() -> String {
    format!("{}/releases_{}.json", flutter_mirror(), target_os())
}

pub(crate) fn flutter_versions_cache_filename() -> &'static str {
    "flutter-versions.json"
}

pub(crate) fn flutter_latest_version_cache_filename() -> &'static str {
    "flutter-latest-version.json"
}
