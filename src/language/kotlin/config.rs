use std::env;
use std::sync::OnceLock;

fn kotlin_mirror() -> &'static str {
    static MIRROR: OnceLock<String> = OnceLock::new();
    MIRROR.get_or_init(|| {
        env::var("LVM_KOTLIN_MIRROR")
            .unwrap_or_else(|_| "https://github.com/JetBrains/kotlin/releases/download".to_string())
    })
}

pub(crate) fn download_url(version: &str) -> String {
    format!(
        "{}/v{}/kotlin-compiler-{}.zip",
        kotlin_mirror(),
        version,
        version,
    )
}

pub(crate) fn tarball_filename(version: &str) -> String {
    format!("kotlin-compiler-{}.zip", version)
}

pub(crate) fn kotlin_versions_cache_filename() -> &'static str {
    "kotlin-versions.json"
}
