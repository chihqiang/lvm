use std::env;

crate::define_env_once!(python_tag, "LVM_PYTHON_TAG", "20260623");
crate::define_env_once!(
    download_base,
    "LVM_PYTHON_MIRROR",
    "https://github.com/astral-sh/python-build-standalone/releases/download"
);

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
