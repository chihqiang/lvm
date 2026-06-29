use std::env;
use std::sync::OnceLock;

static JAVA_MIRROR: OnceLock<String> = OnceLock::new();

pub(crate) fn java_mirror() -> &'static str {
    JAVA_MIRROR.get_or_init(|| {
        env::var("LVM_JAVA_MIRROR")
            .unwrap_or_else(|_| "https://api.adoptium.net/v3".to_string())
    })
}

pub(crate) fn target_os() -> &'static str {
    match env::consts::OS {
        "macos" => "mac",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

pub(crate) fn target_arch() -> &'static str {
    match env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x64",
        "x86" => "x86",
        other => other,
    }
}

pub(crate) fn java_versions_cache_filename() -> &'static str {
    "java-versions.txt"
}
