use std::env;
use std::sync::OnceLock;

fn rust_mirror() -> &'static str {
    static MIRROR: OnceLock<String> = OnceLock::new();
    MIRROR.get_or_init(|| {
        env::var("LVM_RUST_MIRROR")
            .unwrap_or_else(|_| "https://static.rust-lang.org/dist".to_string())
    })
}

pub(crate) fn download_url(version: &str, target: &str) -> String {
    format!("{}/rust-{version}-{target}.tar.gz", rust_mirror(),)
}

pub(crate) fn tarball_filename(version: &str, target: &str) -> String {
    format!("rust-{version}-{target}.tar.gz")
}

fn os_target(system_os: &str) -> &str {
    match system_os {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        other => other,
    }
}

fn arch_target(system_arch: &str) -> &str {
    match system_arch {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        "x86" => "i686",
        other => other,
    }
}

pub(crate) fn target_triple(os: &str, arch: &str) -> String {
    format!("{}-{}", arch_target(arch), os_target(os))
}

pub(crate) fn target_os() -> &'static str {
    env::consts::OS
}

pub(crate) fn target_arch() -> &'static str {
    env::consts::ARCH
}

pub(crate) fn rust_versions_cache_filename() -> &'static str {
    "rust-versions.json"
}
