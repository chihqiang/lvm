crate::define_env_once!(
    kotlin_mirror,
    "LVM_KOTLIN_MIRROR",
    "https://github.com/JetBrains/kotlin/releases/download"
);

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
